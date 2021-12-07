use core::cmp::Ordering;

use crate::{CanOpenService, NodeId};
use embedded_can::{Id, StandardId};

use crate::objectdictionary::datalink::{ReadStream, ReadStreamData, WriteStream};
use crate::objectdictionary::{ObjectDictionary, ObjectDictionaryExt, Variable};
use crate::sdo::errors::SDOAbortCode;

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

        let command = request[0];
        if command & EXPEDITED != 0 {
            let size = match command & SIZE_SPECIFIED {
                0 => 4,
                _ => 4 - ((command >> 2) & 0x3) as usize,
            };
            if let Some(expected_size) = variable.size() {
                check_sizes(size, expected_size.get())?;
            }

            let stream = WriteStream {
                index: self.last_index,
                subindex: self.last_subindex,
                new_data: &request[4..4 + size],
                offset: 0,
                is_last_segment: true,
            };

            variable.write(&stream)?;
        } else {
            if command & SIZE_SPECIFIED != 0 {
                if let Some(expected_size) = variable.size() {
                    let size = u32::from_le_bytes(request[4..8].try_into().unwrap()) as usize;

                    check_sizes(size, expected_size.get())?;
                }
            }

            self.state = State::SegmentedDownload {
                toggle_bit: 0,
                bytes_downloaded: 0,
                variable,
            };
        }

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

                // write data
                let stream = WriteStream {
                    index: self.last_index,
                    subindex: self.last_subindex,
                    new_data: &request[1..last_byte],
                    offset: *bytes_downloaded,
                    is_last_segment: no_more_data,
                };
                variable.write(&stream)?;
                *bytes_downloaded += last_byte - 1;

                // respond
                let response = [RESPONSE_SEGMENT_DOWNLOAD | *toggle_bit, 0, 0, 0, 0, 0, 0, 0];
                *toggle_bit ^= TOGGLE_BIT;
                Ok(Some(response))
            }
            _ => Err(SDOAbortCode::CommandSpecifierError),
        }
    }
    fn init_upload(&mut self, request: &[u8; 8]) -> RequestResult {
        let variable = self.od.find(self.last_index, self.last_subindex)?;

        let mut response = [RESPONSE_UPLOAD, 0, 0, 0, 0, 0, 0, 0];
        response[1..4].copy_from_slice(&request[1..4]);

        if let Some(size) = variable.size() {
            let size = size.get();
            response[0] |= SIZE_SPECIFIED;
            if size <= 4 {
                response[0] |= EXPEDITED;
                response[0] |= (4 - size as u8) << 2;

                let mut read_stream_data = ReadStreamData {
                    index: self.last_index,
                    subindex: self.last_subindex,
                    buf: &mut response[4..4 + size],
                    total_bytes_read: &mut 0,
                    is_last_segment: false,
                };
                variable.read(ReadStream(&mut read_stream_data))?;
                return Ok(Some(response));
            } else {
                response[4..].copy_from_slice(&(size as u32).to_le_bytes());
            }
        }

        self.state = State::SegmentedUpload {
            toggle_bit: 0,
            bytes_uploaded: 0,
            variable,
        };

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

                let mut response = [0; 8];
                let bytes_uploaded_prev = *bytes_uploaded;
                let mut read_stream_data = ReadStreamData {
                    index: self.last_index,
                    subindex: self.last_subindex,
                    buf: &mut response[1..8],
                    total_bytes_read: bytes_uploaded,
                    is_last_segment: false,
                };
                let read_stream_data = variable.read(ReadStream(&mut read_stream_data))?.0;
                let size = *read_stream_data.total_bytes_read - bytes_uploaded_prev;

                let mut res_command = RESPONSE_SEGMENT_UPLOAD;
                res_command |= *toggle_bit; // add toggle bit
                res_command |= (7 - size as u8) << 1; // add number of bytes not used

                if read_stream_data.is_last_segment {
                    res_command |= NO_MORE_DATA; // nothing left in buffer
                }

                *toggle_bit ^= TOGGLE_BIT;

                response[0] = res_command;
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

fn check_sizes(given: usize, expected: usize) -> Result<(), SDOAbortCode> {
    match given.cmp(&expected) {
        Ordering::Less => Err(SDOAbortCode::TooShort),
        Ordering::Greater => Err(SDOAbortCode::TooLong),
        Ordering::Equal => Ok(()),
    }
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
