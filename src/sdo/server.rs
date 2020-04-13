use super::*;
use core::convert::TryInto;

pub struct SdoServer<'a> {
    _rx_cobid: u32,
    tx_cobid: u32,
    node: node::Node<'a>,
    state: State,
}

struct State {
    index: Option<u16>,
    subindex: Option<u8>,
}

impl State {
    fn new() -> State {
        State {
            index: None,
            subindex: None,
        }
    }
}

impl SdoServer<'_> {
    pub fn new(rx_cobid: u32, tx_cobid: u32, node: node::Node) -> SdoServer {
        SdoServer {
            _rx_cobid: rx_cobid,
            tx_cobid,
            node,
            state: State::new(),
        }
    }

    pub fn on_request(&mut self, _can_id: u32, data: &[u8]) {
        if data.len() != 8 {
            return;
        }
        let command = data[0];
        let request: [u8; 7] = data[1..8].try_into().unwrap();
        //let (command, request) = data.split_first().unwrap();
        let ccs = command & 0xE0;

        // TODO result could be Result<Frame, SdoAbortedError>
        // then call send_response from here
        let result = match ccs {
            REQUEST_UPLOAD => self.init_upload(request),
            REQUEST_SEGMENT_UPLOAD => self.segmented_upload(command),
            REQUEST_DOWNLOAD => self.init_download(request),
            REQUEST_SEGMENT_DOWNLOAD => self.segmented_download(command, request),
            REQUEST_ABORTED => Ok(()),
            _ => Err(SdoAbortedError(0x0504_0001)),
        };
        if let Err(abort_error) = result {
            self.abort(abort_error)
        }
    }

    fn init_upload(&mut self, request: [u8; 7]) -> Result<(), SdoAbortedError> {
        let index = u16::from_le_bytes(request[0..2].try_into().unwrap());
        let subindex = request[2];
        self.state.index = Some(index);
        self.state.subindex = Some(subindex);

        let data = self.node.get_data(index, subindex)?;
        let mut res_command = RESPONSE_UPLOAD | SIZE_SPECIFIED;
        let mut response = [0; 8];

        let size = data.len();
        if size <= 4 {
            res_command |= EXPEDITED;
            res_command |= (4 - size as u8) << 2;
            response[4..4 + size].copy_from_slice(&data);
        } else {
            response[4..].copy_from_slice(&(size as u32).to_le_bytes());
            // self._buffer = bytearray(data)
            // self._toggle = 0
        }

        response[0] = res_command;
        response[1..3].copy_from_slice(&index.to_le_bytes());
        response[3] = subindex;
        print!("init upload: ");
        self.send_response(response);
        Ok(())
    }

    fn segmented_upload(&mut self, _command: u8) -> Result<(), SdoAbortedError> {
        print!("segmented_upload: ");
        Ok(())
    }

    fn init_download(&mut self, request: [u8; 7]) -> Result<(), SdoAbortedError> {
        print!("{:?}", request);
        Ok(())
    }

    fn segmented_download(
        &mut self,
        _command: u8,
        _request: [u8; 7],
    ) -> Result<(), SdoAbortedError> {
        println!("segmented_download: ");
        Ok(())
    }

    fn abort(&mut self, abort_error: SdoAbortedError) {
        let [index_lo, index_hi] = self.state.index.unwrap_or_default().to_le_bytes();
        let subindex = self.state.subindex.unwrap_or_default();
        let code = abort_error.to_le_bytes();
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

        print!("{}: ", abort_error);
        self.send_response(data);
    }

    fn send_response(&mut self, data: [u8; 8]) {
        println!("{:?}", data);
        self.node
            .network
            .borrow_mut()
            .transmit(self.tx_cobid, &data)
            .ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hal;
    use crate::node::Node;
    use core::cell::RefCell;

    struct Network {
        expected_id: u32,
        expected_response: [u8; 8],
    }

    impl hal::MyTransmitter for Network {
        fn transmit(&mut self, can_id: u32, data: &[u8]) -> Result<(), ()> {
            assert_eq!(self.expected_id, can_id);
            assert_eq!(self.expected_response, data);
            Ok(())
        }
    }

    fn test_request(request: &[u8], expected_response: [u8; 8]) {
        let tx_cobid = 420;

        let network = RefCell::new(Network {
            expected_id: tx_cobid,
            expected_response,
        });

        let node = Node { network: &network };

        let mut server = SdoServer::new(42, 420, node);

        server.on_request(0, request);
    }

    #[test]
    fn test_init_upload() {
        let data = [64, 2, 3, 4, 5, 6, 7, 8];
        test_request(&data, [67, 2, 3, 4, 1, 2, 3, 4])
    }

    #[test]
    fn test_abort() {
        // invalid command specifier
        let data = [7 << 5, 0, 0, 0, 0, 0, 0, 0];
        test_request(&data, [128, 0, 0, 0, 1, 0, 4, 5]);
    }

    #[test]
    fn test_bad_data() {
        test_request(&[0; 7], [1; 8]);
        test_request(&[0; 9], [1; 8]);
        test_request(&[], [1; 8]);
    }
}
