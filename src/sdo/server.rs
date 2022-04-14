use embedded_can::{Id, StandardId};

use crate::objectdictionary::datalink::{WriteData, WriteStream};
use crate::objectdictionary::{CANOpenData, ObjectDictionary, ObjectDictionaryExt, Variable};
use crate::sdo::errors::SDOAbortCode;
use crate::{CanOpenService, NodeId};

use super::*;

type RequestResult = Result<Option<[u8; 8]>, SDOAbortCode>;

enum State<'a> {
    None,
    SegmentedDownload {
        toggle_bit: u8,
        variable: &'a Variable<'a>,
        bytes_downloaded: usize,
    },
    SegmentedUpload {
        toggle_bit: u8,
        variable: &'a Variable<'a>,
        bytes_uploaded: usize,
    },
}

pub struct SdoServer<'a> {
    pub rx_cobid: StandardId,
    pub tx_cobid: StandardId,
    pub(crate) od: ObjectDictionary<'a>,
    last_index: u16,
    last_subindex: u8,
    state: State<'a>,
}

impl<F: embedded_can::Frame> CanOpenService<F> for SdoServer<'_> {
    fn on_message(&mut self, frame: &F) -> Option<F> {
        if frame.id() != Id::Standard(self.rx_cobid) {
            return None;
        }
        if let Ok(data) = frame.data().try_into() {
            self.on_request(data)
        } else {
            None
        }
    }
}

impl<'a> SdoServer<'a> {
    pub fn new(node_id: NodeId, od: ObjectDictionary<'a>) -> Self {
        // SAFETY: Maximum StandardId is 0x7FF, maximum node_id is 0x7F
        let tx_cobid = unsafe { StandardId::new_unchecked(0x580 + node_id.raw() as u16) };
        let rx_cobid = unsafe { StandardId::new_unchecked(0x600 + node_id.raw() as u16) };

        SdoServer {
            rx_cobid,
            tx_cobid,
            od,
            last_index: 0,
            last_subindex: 0,
            state: State::None,
        }
    }

    pub fn on_request<F: embedded_can::Frame>(&mut self, data: &[u8; 8]) -> Option<F> {
        let ccs = data[0] & 0xE0;

        let result = match ccs {
            REQUEST_DOWNLOAD => {
                self.set_index(data);
                self.init_download(data)
            }
            REQUEST_SEGMENT_DOWNLOAD => self.segmented_download(data),
            REQUEST_UPLOAD => {
                self.set_index(data);
                self.init_upload(data)
            }
            REQUEST_SEGMENT_UPLOAD => self.segmented_upload(data[0]),
            REQUEST_ABORTED => Ok(None),
            _ => Err(SDOAbortCode::CommandSpecifierError),
        };
        match result {
            Ok(None) => None,
            Ok(Some(response)) => Some(F::new(self.tx_cobid, &response).unwrap()),
            Err(abort_code) => Some(self.abort(abort_code)),
        }
    }

    fn set_index(&mut self, request: &[u8; 8]) {
        self.last_index = ((request[2] as u16) << 8) + request[1] as u16;
        self.last_subindex = request[3];
    }

    fn init_download(&mut self, request: &[u8; 8]) -> RequestResult {
        let variable = self.od.find(self.last_index, self.last_subindex)?;

        // unpack command
        let mut stream = unpack_init_download_request(request);
        stream.index = self.last_index;
        stream.subindex = self.last_subindex;

        // write data
        match variable.data {
            CANOpenData::B1(_)
            | CANOpenData::B2(_)
            | CANOpenData::B4(_)
            | CANOpenData::Bytes(_) => return Err(SDOAbortCode::ReadOnlyError),
            CANOpenData::ResourceNotAvailable => return Err(SDOAbortCode::ResourceNotAvailable),
            CANOpenData::DataLinkRef(link) => link.write(WriteStream(&stream))?,
            CANOpenData::DataLinkCell(link) => {
                let mut link_ref = link.borrow_mut();
                link_ref.write(WriteStream(&stream))?;
                if !stream.is_last_segment {
                    link_ref.lock();
                }
            }
        }

        // update state
        if !stream.is_last_segment {
            self.state = State::SegmentedDownload {
                toggle_bit: 0,
                bytes_downloaded: 0,
                variable,
            };
        }

        // respond
        let mut response = [RESPONSE_DOWNLOAD, 0, 0, 0, 0, 0, 0, 0];
        response[1..4].copy_from_slice(&request[1..4]);

        Ok(Some(response))
    }

    fn segmented_download(&mut self, request: &[u8; 8]) -> RequestResult {
        match &mut self.state {
            State::SegmentedDownload {
                toggle_bit,
                variable,
                bytes_downloaded,
            } => {
                // unpack command
                let command = request[0];
                if command & TOGGLE_BIT != *toggle_bit {
                    return Err(SDOAbortCode::ToggleBitNotAlternated);
                }
                let last_byte = (8 - ((command >> 1) & 0x7)) as usize;
                let no_more_data = command & NO_MORE_DATA != 0;

                let stream = WriteData {
                    index: self.last_index,
                    subindex: self.last_subindex,
                    promised_size: None,
                    new_data: &request[1..last_byte],
                    offset: *bytes_downloaded,
                    is_last_segment: no_more_data,
                };

                // write data
                match variable.data {
                    CANOpenData::B1(_)
                    | CANOpenData::B2(_)
                    | CANOpenData::B4(_)
                    | CANOpenData::Bytes(_) => return Err(SDOAbortCode::ReadOnlyError),
                    CANOpenData::ResourceNotAvailable => {
                        return Err(SDOAbortCode::ResourceNotAvailable)
                    }
                    CANOpenData::DataLinkRef(link) => link.write(WriteStream(&stream))?,
                    CANOpenData::DataLinkCell(link) => {
                        if !link.is_locked() {
                            return Err(SDOAbortCode::LocalControlError);
                        }
                        let mut link_ref = link.borrow_mut(); // will automatically unlock
                        link_ref.write(WriteStream(&stream))?;
                        if !no_more_data {
                            link_ref.lock();
                        }
                    }
                }

                // respond
                let response = [RESPONSE_SEGMENT_DOWNLOAD | *toggle_bit, 0, 0, 0, 0, 0, 0, 0];
                *toggle_bit ^= TOGGLE_BIT;
                *bytes_downloaded += last_byte - 1;
                Ok(Some(response))
            }
            _ => Err(SDOAbortCode::CommandSpecifierError),
        }
    }
    fn init_upload(&mut self, request: &[u8; 8]) -> RequestResult {
        let variable = self.od.find(self.last_index, self.last_subindex)?;

        let mut response = [RESPONSE_UPLOAD | SIZE_SPECIFIED, 0, 0, 0, 0, 0, 0, 0];
        response[1..4].copy_from_slice(&request[1..4]);

        let is_expedited = match variable.data {
            CANOpenData::B1(data) => fill_upload_response(&data, &mut response),
            CANOpenData::B2(data) => fill_upload_response(&data, &mut response),
            CANOpenData::B4(data) => fill_upload_response(&data, &mut response),
            CANOpenData::Bytes(data) => fill_upload_response(data, &mut response),
            CANOpenData::DataLinkRef(link) => fill_upload_response(
                link.read(self.last_index, self.last_subindex)?.get(),
                &mut response,
            ),
            CANOpenData::DataLinkCell(link) => {
                let link_ref = link.borrow();
                let data = link_ref.read(self.last_index, self.last_subindex)?;
                let is_expedited = fill_upload_response(data.get(), &mut response);
                if !is_expedited {
                    link_ref.lock();
                }
                is_expedited
            }
            CANOpenData::ResourceNotAvailable => return Err(SDOAbortCode::ResourceNotAvailable),
        };
        if !is_expedited {
            self.state = State::SegmentedUpload {
                toggle_bit: 0,
                bytes_uploaded: 0,
                variable,
            };
        }

        Ok(Some(response))
    }

    fn segmented_upload(&mut self, command: u8) -> RequestResult {
        match &mut self.state {
            State::SegmentedUpload {
                toggle_bit,
                bytes_uploaded,
                variable,
            } => {
                if command & TOGGLE_BIT != *toggle_bit {
                    return Err(SDOAbortCode::ToggleBitNotAlternated);
                }

                let mut response = [RESPONSE_SEGMENT_UPLOAD | *toggle_bit, 0, 0, 0, 0, 0, 0, 0];
                *toggle_bit ^= TOGGLE_BIT;

                match variable.data {
                    CANOpenData::Bytes(data) => {
                        fill_segmented_upload_response(data, &mut response, bytes_uploaded);
                    }
                    CANOpenData::DataLinkRef(link) => {
                        fill_segmented_upload_response(
                            link.read(self.last_index, self.last_subindex)?.get(),
                            &mut response,
                            bytes_uploaded,
                        );
                    }
                    CANOpenData::DataLinkCell(link) => {
                        if !link.is_locked() {
                            return Err(SDOAbortCode::LocalControlError);
                        }
                        let link_ref = link.borrow();

                        let no_more_data = fill_segmented_upload_response(
                            link_ref.read(self.last_index, self.last_subindex)?.get(),
                            &mut response,
                            bytes_uploaded,
                        );
                        if no_more_data {
                            link_ref.unlock();
                        }
                    }
                    CANOpenData::B1(_)
                    | CANOpenData::B2(_)
                    | CANOpenData::B4(_)
                    | CANOpenData::ResourceNotAvailable => unreachable!(), // other datatypes always use expedited transfer
                }

                Ok(Some(response))
            }
            _ => Err(SDOAbortCode::CommandSpecifierError),
        }
    }

    fn abort<F: embedded_can::Frame>(&mut self, abort_error: SDOAbortCode) -> F {
        let [index_lo, index_hi] = self.last_index.to_le_bytes();
        let subindex = self.last_subindex;
        let code: [u8; 4] = (abort_error as u32).to_le_bytes();
        let data: [u8; 8] = [
            RESPONSE_ABORTED,
            index_lo,
            index_hi,
            subindex,
            code[0],
            code[1],
            code[2],
            code[3],
        ];

        F::new(self.tx_cobid, &data).unwrap()
    }
}

fn unpack_init_download_request(request: &[u8; 8]) -> WriteData<'_> {
    let mut stream = WriteData {
        index: 0,
        subindex: 0,
        promised_size: None,
        new_data: &[],
        offset: 0,
        is_last_segment: false,
    };

    let command = request[0];

    if command & EXPEDITED != 0 {
        let size = match command & SIZE_SPECIFIED {
            0 => 4,
            _ => 4 - ((command >> 2) & 0x3) as usize,
        };
        stream.new_data = &request[4..4 + size];
        stream.promised_size = Some(size);
        stream.is_last_segment = true;
    } else if command & SIZE_SPECIFIED != 0 {
        stream.promised_size = Some(u32::from_le_bytes(request[4..8].try_into().unwrap()) as usize);
    }

    stream
}
fn fill_upload_response(data: &[u8], response: &mut [u8]) -> bool {
    if data.len() <= 4 {
        response[0] |= SIZE_SPECIFIED | EXPEDITED | (4 - data.len() as u8) << 2;
        response[4..4 + data.len()].copy_from_slice(data);
        true
    } else {
        response[4..].copy_from_slice(&(data.len() as u32).to_le_bytes());
        false
    }
}
fn fill_segmented_upload_response(
    data: &[u8],
    response: &mut [u8],
    bytes_uploaded: &mut usize,
) -> bool {
    let unread_data = &data[*bytes_uploaded..];

    let size = unread_data.len().min(7);

    response[1..size + 1].copy_from_slice(&unread_data[..size]);
    *bytes_uploaded += size;

    response[0] |= (7 - size as u8) << 1; // add number of bytes not used
    if unread_data.len() <= 7 {
        response[0] |= NO_MORE_DATA; // nothing left in buffer
        true
    } else {
        false
    }
}
