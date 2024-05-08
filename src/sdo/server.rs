use embedded_can::{Id, StandardId};

use crate::objectdictionary::datalink::WriteData;
use crate::objectdictionary::{ObjectDictionary, OdPosition};
use crate::{CanOpenService, NodeId, SdoMessage};

use super::*;

type RequestResult = Result<Option<[u8; 8]>, SDOAbortCode>;

enum State {
    None,
    SegmentedDownload {
        toggle_bit: u8,
        od_position: OdPosition,
        bytes_downloaded: usize,
    },
    SegmentedUpload {
        toggle_bit: u8,
        od_position: OdPosition,
        bytes_uploaded: usize,
    },
}

pub struct SdoServer {
    pub rx_cobid: StandardId,
    pub tx_cobid: StandardId,
    last_index: u16,
    last_subindex: u8,
    state: State,
}

impl<F: embedded_can::Frame, T, const N: usize> CanOpenService<F, T, N> for SdoServer {
    fn on_message(&mut self, frame: &F, od: &mut ObjectDictionary<T, N>) -> Option<F> {
        if frame.id() != Id::Standard(self.rx_cobid) {
            return None;
        }
        if let Ok(data) = frame.data().try_into() {
            self.on_request(data, od).map(SdoMessage::into_frame)
        } else {
            None
        }
    }
}

impl SdoServer {
    pub fn new(node_id: NodeId) -> Self {
        SdoServer {
            rx_cobid: node_id.sdo_rx_cobid(),
            tx_cobid: node_id.sdo_tx_cobid(),
            last_index: 0,
            last_subindex: 0,
            state: State::None,
        }
    }

    pub fn on_request<T, const N: usize>(
        &mut self,
        data: &[u8; 8],
        od: &mut ObjectDictionary<T, N>,
    ) -> Option<SdoMessage> {
        let ccs = data[0] & 0xE0;

        let result = match ccs {
            REQUEST_DOWNLOAD => {
                self.state = State::None;
                self.set_index(data);
                self.init_download(data, od)
            }
            REQUEST_SEGMENT_DOWNLOAD => self.segmented_download(data, od),
            REQUEST_UPLOAD => {
                self.state = State::None;
                self.set_index(data);
                self.init_upload(data, od)
            }
            REQUEST_SEGMENT_UPLOAD => self.segmented_upload(data[0], od),
            REQUEST_ABORTED => {
                self.state = State::None;
                Ok(None)
            }
            _ => Err(SDOAbortCode::CommandSpecifierError),
        };
        match result {
            Ok(None) => None,
            Ok(Some(response)) => Some(SdoMessage::new(self.tx_cobid, response)),
            Err(abort_code) => {
                self.state = State::None;
                Some(self.abort(abort_code))
            }
        }
    }

    fn set_index(&mut self, request: &[u8; 8]) {
        self.last_index = ((request[2] as u16) << 8) + request[1] as u16;
        self.last_subindex = request[3];
    }

    fn init_download<T, const N: usize>(
        &mut self,
        request: &[u8; 8],
        od: &mut ObjectDictionary<T, N>,
    ) -> RequestResult {
        let od_position = od.search(self.last_index, self.last_subindex)?;
        let (link, info) = od.get_plus(od_position);
        if info.get(od_position).flags.is_read_only() {
            return Err(SDOAbortCode::ReadOnlyError);
        }

        // unpack command
        let mut stream = unpack_init_download_request(request);
        stream.index = self.last_index;
        stream.subindex = self.last_subindex;

        // write data
        link.write(&stream, info)?; // TODO write even if no data?

        // update state
        if !stream.is_last_segment {
            self.state = State::SegmentedDownload {
                toggle_bit: 0,
                bytes_downloaded: 0,
                od_position,
            };
        }

        // respond
        let mut response = [RESPONSE_DOWNLOAD, 0, 0, 0, 0, 0, 0, 0];
        response[1..4].copy_from_slice(&request[1..4]);

        Ok(Some(response))
    }

    fn segmented_download<T, const N: usize>(
        &mut self,
        request: &[u8; 8],
        od: &mut ObjectDictionary<T, N>,
    ) -> RequestResult {
        match &mut self.state {
            State::SegmentedDownload {
                toggle_bit,
                od_position,
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
                let (link, info) = od.get_plus(*od_position);
                link.write(&stream, info)?;

                // respond
                let response = [RESPONSE_SEGMENT_DOWNLOAD | *toggle_bit, 0, 0, 0, 0, 0, 0, 0];
                *toggle_bit ^= TOGGLE_BIT;
                *bytes_downloaded += last_byte - 1;
                if no_more_data {
                    self.state = State::None;
                }
                Ok(Some(response))
            }
            _ => Err(SDOAbortCode::CommandSpecifierError), // TODO why this error
        }
    }
    fn init_upload<T, const N: usize>(
        &mut self,
        request: &[u8; 8],
        od: &mut ObjectDictionary<T, N>,
    ) -> RequestResult {
        let od_position = od.search(self.last_index, self.last_subindex)?;
        let (link, info) = od.get_plus(od_position);
        if info.get(od_position).flags.is_write_only() {
            return Err(SDOAbortCode::WriteOnlyError);
        }

        let mut response = [RESPONSE_UPLOAD | SIZE_SPECIFIED, 0, 0, 0, 0, 0, 0, 0];
        response[1..4].copy_from_slice(&request[1..4]);

        let is_expedited = fill_upload_response(
            link.read(self.last_index, self.last_subindex)?.as_bytes(),
            &mut response,
        );

        if !is_expedited {
            self.state = State::SegmentedUpload {
                toggle_bit: 0,
                bytes_uploaded: 0,
                od_position,
            };
        }

        Ok(Some(response))
    }

    fn segmented_upload<T, const N: usize>(
        &mut self,
        command: u8,
        od: &mut ObjectDictionary<T, N>,
    ) -> RequestResult {
        match &mut self.state {
            State::SegmentedUpload {
                toggle_bit,
                bytes_uploaded,
                od_position,
            } => {
                if command & TOGGLE_BIT != *toggle_bit {
                    return Err(SDOAbortCode::ToggleBitNotAlternated);
                }

                let mut response = [RESPONSE_SEGMENT_UPLOAD | *toggle_bit, 0, 0, 0, 0, 0, 0, 0];
                *toggle_bit ^= TOGGLE_BIT;

                let no_more_data = fill_segmented_upload_response(
                    od.get(*od_position)
                        .read(self.last_index, self.last_subindex)?
                        .as_bytes(),
                    &mut response,
                    bytes_uploaded,
                );
                if no_more_data {
                    self.state = State::None;
                }

                Ok(Some(response))
            }
            _ => Err(SDOAbortCode::CommandSpecifierError),
        }
    }

    fn abort(&mut self, abort_error: SDOAbortCode) -> SdoMessage {
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

        SdoMessage::new(self.tx_cobid, data)
    }
}

fn unpack_init_download_request(request: &[u8; 8]) -> WriteData {
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
