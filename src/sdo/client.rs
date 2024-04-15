use core::array::TryFromSliceError;
use core::fmt::Debug;

use super::*;

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

pub fn parse_upload_response<T: SdoValue>(
    response: &[u8; 8],
    expected_index: u16,
) -> Result<T, ProtocolError> {
    let ccs = response[0] & 0b1110_0000;
    match ccs {
        RESPONSE_ABORTED => Err(ProtocolError::Abort(to_abort_code(response))),
        RESPONSE_UPLOAD => {
            let index = response[1] as u16 + ((response[2] as u16) << 8);
            if index != expected_index {
                return Err(ProtocolError::IndexMismatch);
            }

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
) -> Result<(), ProtocolError> {
    let ccs = response[0] & 0b1110_0000;
    match ccs {
        RESPONSE_ABORTED => Err(ProtocolError::Abort(to_abort_code(response))),
        RESPONSE_DOWNLOAD => {
            let index = response[1] as u16 + ((response[2] as u16) << 8);
            if index != expected_index {
                return Err(ProtocolError::IndexMismatch);
            }
            Ok(())
        }
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_upload_response() {
        let data = [RESPONSE_UPLOAD, 0, 0, 0, 1, 0, 0, 0];
        assert_eq!(parse_upload_response(&data, 0), Ok(1u32));
        assert_eq!(
            parse_upload_response::<u32>(&data, 1),
            Err(ProtocolError::IndexMismatch)
        );
    }
}
