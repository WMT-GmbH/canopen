#![allow(unused)]

use core::array::TryFromSliceError;
use core::fmt::Debug;

use embedded_can::{Frame, StandardId};

use crate::slot::{Consumer, Producer, Slot};
use crate::NodeId;

use super::*;

pub type SdoBuffer = Slot<[u8; 8]>;
pub type SdoProducer<'a> = Producer<'a, [u8; 8]>;
pub type SdoConsumer<'a> = Consumer<'a, [u8; 8]>;

pub struct SdoClient<'c> {
    pub rx_cobid: StandardId,
    pub tx_cobid: StandardId,
    sdo_consumer: SdoConsumer<'c>,
}

impl<'c> SdoClient<'c> {
    pub fn new(node_id: NodeId, sdo_consumer: SdoConsumer<'c>) -> Self {
        SdoClient {
            rx_cobid: node_id.sdo_rx_cobid(),
            tx_cobid: node_id.sdo_tx_cobid(),
            sdo_consumer,
        }
    }

    pub fn read<'s, 'b>(&'s mut self, index: u16, subindex: u8) -> Reader<'c, 's> {
        self.sdo_consumer.dequeue();
        Reader {
            sdo_client: self,
            state: ReaderState::Init { index, subindex },
        }
    }
}

pub trait ReadInto {
    fn read_into(&mut self, buf: &[u8]) -> Result<(), ParseError>;
}

pub struct Reader<'c, 's> {
    sdo_client: &'s mut SdoClient<'c>,
    state: ReaderState,
}

enum ReaderState {
    Init { index: u16, subindex: u8 },
    RequestSent { index: u16, subindex: u8 },
    Segmented { toggle_bit: bool },
    Done,
}

impl Reader<'_, '_> {
    pub fn poll<F: Frame, B: ReadInto>(
        &mut self,
        buf: &mut B,
    ) -> Result<ReadResult<F>, ProtocolError> {
        match &self.state {
            ReaderState::Init { index, subindex } => {
                let frame = self.frame(&upload_request(*index, *subindex));
                self.state = ReaderState::RequestSent {
                    index: *index,
                    subindex: *subindex,
                };
                Ok(ReadResult::NextRequest(frame))
            }
            ReaderState::Done => Ok(ReadResult::Done),
            other => {
                let Some(response) = self.sdo_client.sdo_consumer.dequeue() else {
                    return Ok(ReadResult::Waiting);
                };
                match response_css(&response) {
                    RESPONSE_ABORTED => Err(to_abort_code(&response).into()),
                    RESPONSE_UPLOAD => self.on_upload_response(response, buf),
                    RESPONSE_SEGMENT_UPLOAD => self.on_segmented_upload_response(response, buf),
                    _ => Err(ProtocolError::ParseError),
                }
            }
        }
    }

    fn on_upload_response<F: Frame, B: ReadInto>(
        &mut self,
        response: [u8; 8],
        buf: &mut B,
    ) -> Result<ReadResult<F>, ProtocolError> {
        let ReaderState::RequestSent { index, subindex } = &self.state else {
            return Err(ProtocolError::ParseError);
        };
        check_response_index(&response, *index, *subindex)?;
        if response[0] & EXPEDITED > 0 {
            self.state = ReaderState::Done;
            let n = (response[0] >> 2) & 0x3;
            let data = &response[4..8 - n as usize];
            buf.read_into(data)?;
            Ok(ReadResult::Done)
        } else {
            self.state = ReaderState::Segmented { toggle_bit: false };
            // FIXME maybe use size in last 4 bytes
            let frame = self.frame(&segmented_upload_request(false));
            Ok(ReadResult::NextRequest(frame))
        }
    }

    fn on_segmented_upload_response<F: Frame, B: ReadInto>(
        &mut self,
        response: [u8; 8],
        buf: &mut B,
    ) -> Result<ReadResult<F>, ProtocolError> {
        let ReaderState::Segmented { toggle_bit } = &mut self.state else {
            return Err(ProtocolError::ParseError);
        };
        if (response[0] & TOGGLE_BIT > 0) != *toggle_bit {
            return Err(ProtocolError::ParseError);
        }
        let n = (response[0] >> 1) & 0x7;
        let data = &response[1..8 - n as usize];
        buf.read_into(data)?;
        if response[0] & NO_MORE_DATA > 0 {
            self.state = ReaderState::Done;
            Ok(ReadResult::Done)
        } else {
            *toggle_bit = !*toggle_bit;
            let toggle_bit = *toggle_bit;
            let frame = self.frame(&segmented_upload_request(toggle_bit));
            Ok(ReadResult::NextRequest(frame))
        }
    }

    fn frame<F: Frame>(&self, data: &[u8; 8]) -> F {
        F::new(self.sdo_client.rx_cobid, data).expect("request should fit")
    }
}

pub enum ReadResult<F> {
    NextRequest(F),
    Waiting,
    Done,
}

pub fn upload_request(index: u16, sub_index: u8) -> [u8; 8] {
    let mut request = [0; 8];
    request[0] = REQUEST_UPLOAD;
    request[1] = index as u8;
    request[2] = (index >> 8) as u8;
    request[3] = sub_index;
    request
}

pub fn download_request<T: SdoValue>(index: u16, sub_index: u8, val: T) -> [u8; 8] {
    let bytes = val.to_bytes();
    let data = bytes.as_ref();
    let mut request = [0; 8];
    request[0] = REQUEST_DOWNLOAD | EXPEDITED | SIZE_SPECIFIED | ((4 - data.len()) << 2) as u8;
    request[1] = index as u8;
    request[2] = (index >> 8) as u8;
    request[3] = sub_index;

    request[4..4 + data.len()].copy_from_slice(data);

    request
}

fn response_css(response: &[u8; 8]) -> u8 {
    response[0] & 0b1110_0000
}

fn check_response_index(
    response: &[u8; 8],
    expected_index: u16,
    expected_subindex: u8,
) -> Result<(), ProtocolError> {
    if u16::from_le_bytes([response[1], response[2]]) != expected_index {
        Err(ProtocolError::IndexMismatch)
    } else if response[3] != expected_subindex {
        Err(ProtocolError::SubindexMismatch)
    } else {
        Ok(())
    }
}

pub fn segmented_upload_request(toggle_bit: bool) -> [u8; 8] {
    let mut request = [0; 8];
    request[0] = if toggle_bit {
        REQUEST_SEGMENT_UPLOAD | TOGGLE_BIT
    } else {
        REQUEST_SEGMENT_UPLOAD
    };
    request
}

pub fn parse_upload_response<T: SdoValue>(
    response: &[u8; 8],
    expected_index: u16,
    expected_subindex: u8,
) -> Result<T, ProtocolError> {
    match response_css(response) {
        RESPONSE_ABORTED => Err(ProtocolError::Abort(to_abort_code(response))),
        RESPONSE_UPLOAD => {
            check_response_index(response, expected_index, expected_subindex)?;

            let n = (response[0] >> 2) & 0x3;
            let data = &response[4..8 - n as usize];
            Ok(T::from_bytes(data)?)
        }
        _ => Err(ProtocolError::ParseError),
    }
}

pub fn parse_download_response(
    response: &[u8; 8],
    expected_index: u16,
    expected_subindex: u8,
) -> Result<(), ProtocolError> {
    match response_css(response) {
        RESPONSE_ABORTED => Err(ProtocolError::Abort(to_abort_code(response))),
        RESPONSE_DOWNLOAD => check_response_index(response, expected_index, expected_subindex),
        _ => Err(ProtocolError::ParseError),
    }
}

fn to_abort_code(response: &[u8; 8]) -> SDOAbortCode {
    let abort_code = response[4] as u32
        + ((response[5] as u32) << 8)
        + ((response[6] as u32) << 16)
        + ((response[7] as u32) << 24);
    SDOAbortCode::from(abort_code)
}

/// Error inside Response
#[derive(Eq, PartialEq, Debug)]
pub enum ProtocolError {
    /// Response was invalid
    ParseError,
    /// Response indicated a different index then the one in the request
    IndexMismatch,
    /// Response indicated a different subindex then the one in the request
    SubindexMismatch,
    /// Server sent an AbortCode
    Abort(SDOAbortCode),
}

pub struct ParseError;

impl From<TryFromSliceError> for ParseError {
    fn from(_: TryFromSliceError) -> Self {
        ParseError
    }
}
impl From<ParseError> for ProtocolError {
    fn from(_: ParseError) -> Self {
        ProtocolError::ParseError
    }
}
impl From<SDOAbortCode> for ProtocolError {
    fn from(value: SDOAbortCode) -> Self {
        ProtocolError::Abort(value)
    }
}

/// Conversion trait for integers supported by SDO
pub trait SdoValue: Sized {
    /// `Self` as bytes
    type Bytes: AsRef<[u8]>;
    /// Convert from little endian bytes
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError>;
    /// Convert to little endian bytes
    fn to_bytes(self) -> Self::Bytes;
}

macro_rules! sdo_value {
    ($typ:ty) => {
        impl SdoValue for $typ {
            type Bytes = [u8; core::mem::size_of::<Self>()];
            fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
                let bytes: Self::Bytes = bytes.try_into()?;
                Ok(Self::from_le_bytes(bytes))
            }

            fn to_bytes(self) -> Self::Bytes {
                self.to_le_bytes()
            }
        }
    };
}

sdo_value!(u8);
sdo_value!(u16);
sdo_value!(u32);
sdo_value!(i8);
sdo_value!(i16);
sdo_value!(i32);
sdo_value!(f32);

impl SdoValue for bool {
    type Bytes = [u8; 1];
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let bytes: [u8; 1] = bytes.try_into()?;
        match bytes[0] {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(ParseError),
        }
    }

    fn to_bytes(self) -> Self::Bytes {
        [self as u8]
    }
}

impl<const N: usize> ReadInto for heapless::Vec<u8, N> {
    fn read_into(&mut self, buf: &[u8]) -> Result<(), ParseError> {
        self.extend_from_slice(buf).map_err(|_| ParseError)
    }
}

#[cfg(feature = "std")]
impl ReadInto for Vec<u8> {
    fn read_into(&mut self, buf: &[u8]) -> Result<(), ParseError> {
        self.extend_from_slice(buf);
        Ok(())
    }
}

macro_rules! read_into {
    ($typ:ty) => {
        impl ReadInto for $typ {
            fn read_into(&mut self, buf: &[u8]) -> Result<(), ParseError> {
                let bytes: [u8; core::mem::size_of::<Self>()] = buf.try_into()?;
                *self = Self::from_le_bytes(bytes);
                Ok(())
            }
        }
    };
}

read_into!(u8);
read_into!(u16);
read_into!(u32);
read_into!(i8);
read_into!(i16);
read_into!(i32);
read_into!(f32);

impl ReadInto for bool {
    fn read_into(&mut self, bytes: &[u8]) -> Result<(), ParseError> {
        let bytes: [u8; 1] = bytes.try_into()?;
        match bytes[0] {
            0 => *self = false,
            1 => *self = true,
            _ => return Err(ParseError),
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_upload_response() {
        let data = [RESPONSE_UPLOAD, 0, 0, 0, 1, 0, 0, 0];
        assert_eq!(parse_upload_response(&data, 0, 0), Ok(1u32));
        assert_eq!(
            parse_upload_response::<u32>(&data, 1, 0),
            Err(ProtocolError::IndexMismatch)
        );
    }
}
