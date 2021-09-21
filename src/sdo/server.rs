use core::cmp;

use crate::objectdictionary::{ObjectDictionary, Variable};
use crate::sdo::errors::SDOAbortCode;
use crate::Network;

use super::*;
use crate::objectdictionary::datalink::WriteStream;
use core::cmp::Ordering;

type RequestResult = Result<Option<[u8; 8]>, SDOAbortCode>;

enum State<'a> {
    None,
    SegmentedUpload {
        toggle_bit: u8,
        variable: &'a Variable<'a>,
        bytes_uploaded: usize,
    },
    SegmentedDownload {
        toggle_bit: u8,
        variable: &'a Variable<'a>,
        bytes_downloaded: usize,
    },
}

pub struct SdoServer<'a, N: Network> {
    _rx_cobid: u32,
    tx_cobid: u32,
    network: &'a N,
    od: &'a ObjectDictionary<'a>,
    last_index: u16,
    last_subindex: u8,
    state: State<'a>,
}

impl<'a, N: Network> SdoServer<'a, N> {
    pub fn new(rx_cobid: u32, tx_cobid: u32, network: &'a N, od: &'a ObjectDictionary<'a>) -> Self {
        SdoServer {
            _rx_cobid: rx_cobid,
            tx_cobid,
            network,
            od,
            last_index: 0,
            last_subindex: 0,
            state: State::None,
        }
    }

    pub fn on_request(&mut self, data: &[u8; 8]) {
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
            Ok(None) => {}
            Ok(Some(response)) => self.send_response(response),
            Err(abort_code) => self.abort(abort_code),
        };
    }

    fn set_index(&mut self, request: &[u8; 8]) {
        self.last_index = ((request[2] as u16) << 8) + request[1] as u16;
        self.last_subindex = request[3];
    }

    fn init_download(&mut self, request: &[u8; 8]) -> RequestResult {
        let variable = self.od.get(self.last_index, self.last_subindex)?;

        let command = request[0];
        if command & EXPEDITED != 0 {
            let size = match command & SIZE_SPECIFIED {
                0 => 4,
                _ => 4 - ((command >> 2) & 0x3) as usize,
            };
            if let Some(expected_size) = variable.datalink.size() {
                check_sizes(size, expected_size.get())?;
            }

            let stream = WriteStream {
                index: self.last_index,
                subindex: self.last_subindex,
                new_data: &request[4..4 + size],
                offset: 0,
                is_last_segment: true,
            };

            variable.datalink.write(&stream)?;
        } else {
            if command & SIZE_SPECIFIED != 0 {
                if let Some(expected_size) = variable.datalink.size() {
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
                variable.datalink.write(&stream)?;
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
        let variable = self.od.get(self.last_index, self.last_subindex)?;
        let mut response = [RESPONSE_UPLOAD, 0, 0, 0, 0, 0, 0, 0];
        response[1..4].copy_from_slice(&request[1..4]);

        if let Some(size) = variable.datalink.size() {
            let size = size.get();
            response[0] |= SIZE_SPECIFIED;
            if size <= 4 {
                response[0] |= EXPEDITED;
                response[0] |= (4 - size as u8) << 2;
                variable.datalink.read(&mut response[4..4 + size], 0)?;
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
                let total_size = variable.datalink.size().unwrap().get(); // TODO
                let size = cmp::min(total_size - *bytes_uploaded, 7);
                variable
                    .datalink
                    .read(&mut response[1..1 + size], *bytes_uploaded)?;

                *bytes_uploaded += size;

                let mut res_command = RESPONSE_SEGMENT_UPLOAD;
                res_command |= *toggle_bit; // add toggle bit
                res_command |= (7 - size as u8) << 1; // add number of bytes not used

                if *bytes_uploaded == total_size {
                    res_command |= NO_MORE_DATA; // nothing left in buffer
                }

                *toggle_bit ^= TOGGLE_BIT;

                response[0] = res_command;
                Ok(Some(response))
            }
            _ => Err(SDOAbortCode::CommandSpecifierError),
        }
    }

    fn abort(&mut self, abort_error: SDOAbortCode) {
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

        self.send_response(data);
    }

    fn send_response(&mut self, data: [u8; 8]) {
        self.network.send_message(self.tx_cobid, data);
    }
}

fn check_sizes(given: usize, expected: usize) -> Result<(), SDOAbortCode> {
    match given.cmp(&expected) {
        Ordering::Less => Err(SDOAbortCode::TooShort),
        Ordering::Greater => Err(SDOAbortCode::TooLong),
        Ordering::Equal => Ok(()),
    }
}
