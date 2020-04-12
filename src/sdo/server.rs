use super::*;
use core::convert::TryInto;

#[derive(Debug)]
pub struct SdoServer {
    rx_cobid: u32,
    tx_cobid: u32,
    node: node::Node,
}

impl SdoServer {
    pub fn new(rx_cobid: u32, tx_cobid: u32, node: node::Node) -> SdoServer {
        SdoServer {
            rx_cobid,
            tx_cobid,
            node,
        }
    }

    pub fn on_request(&self, _can_id: u32, data: &[u8]) {
        if data.len() != 8 {
            // TODO return
        }
        let command = data[0];
        let request: [u8; 7] = data[1..8].try_into().unwrap();
        //let (command, request) = data.split_first().unwrap();
        let ccs = command & 0xE0;

        let result = match ccs {
            REQUEST_UPLOAD => self.init_upload(request),
            REQUEST_SEGMENT_UPLOAD => self.segmented_upload(command),
            REQUEST_DOWNLOAD => self.init_download(request),
            REQUEST_SEGMENT_DOWNLOAD => self.segmented_download(command, request),
            REQUEST_ABORTED => Ok(()),
            _ => Err(SdoAbortedError { code: 0x05040001 }),
        };
        if result.is_err() {
            self.abort(result.unwrap_err())
        }
    }

    fn init_upload(&self, request: [u8; 7]) -> Result<(), SdoAbortedError> {
        let index = u16::from_le_bytes(request[0..2].try_into().unwrap());
        let subindex = request[2];

        let data = self.node.get_data(index, subindex)?;
        let mut res_command = RESPONSE_UPLOAD | SIZE_SPECIFIED;
        let mut response = [0u8; 8];

        let size = data.len() as u32;
        if size <= 4 {
            res_command |= EXPEDITED;
            res_command |= (4 - size as u8) << 2;
            for (i, b) in data.iter().enumerate() {
                response[4 + i] = *b;
            }
        } else {
            let size_bytes = size.to_le_bytes();
            for (i, b) in size_bytes.iter().enumerate() {
                response[4 + i] = *b;
            }
            // self._buffer = bytearray(data)
            // self._toggle = 0
        }

        response[0] = res_command;
        response[1] = index.to_le_bytes()[0];
        response[2] = index.to_le_bytes()[1];
        response[3] = subindex;
        print!("init upload: ");
        self.send_response(response);
        Ok(())
    }

    fn segmented_upload(&self, _command: u8) -> Result<(), SdoAbortedError> {
        print!("segmented_upload: ");
        Ok(())
    }

    fn init_download(&self, request: [u8; 7]) -> Result<(), SdoAbortedError> {
        print!("{:?}", request);
        Ok(())
    }

    fn segmented_download(&self, _command: u8, _request: [u8; 7]) -> Result<(), SdoAbortedError> {
        println!("segmented_download: ");
        Ok(())
    }

    fn abort(&self, abort_error: SdoAbortedError) {
        let index = 420u16.to_le_bytes();
        let subindex = 50u8;
        let code = abort_error.code.to_le_bytes();
        let data: [u8; 8] = [
            RESPONSE_ABORTED,
            index[0],
            index[1],
            subindex,
            code[0],
            code[1],
            code[2],
            code[3],
        ];

        print!("abort: ");
        self.send_response(data);
    }

    fn send_response(&self, data: [u8; 8]) {
        println!("{:?}", data);
    }
}
