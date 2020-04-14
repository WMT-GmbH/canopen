use super::*;
use core::convert::TryInto;

pub struct SdoServer<'a> {
    _rx_cobid: u32,
    tx_cobid: u32,
    node: node::Node<'a>,
    state: Option<State>,
}

struct State {
    index: u16,
    subindex: u8,
    toggle_bit: u8,
}

impl SdoServer<'_> {
    pub fn new(rx_cobid: u32, tx_cobid: u32, node: node::Node) -> SdoServer {
        SdoServer {
            _rx_cobid: rx_cobid,
            tx_cobid,
            node,
            state: None,
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
        self.state = Some(State {
            index,
            subindex,
            toggle_bit: 0,
        });

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
            // self._buffer = bytearray(data) TODO
        }

        response[0] = res_command;
        response[1..3].copy_from_slice(&index.to_le_bytes());
        response[3] = subindex;
        print!("init upload: ");
        self.send_response(response);
        Ok(())
    }

    fn segmented_upload(&mut self, command: u8) -> Result<(), SdoAbortedError> {
        print!("segmented_upload: ");
        if command & _TOGGLE_BIT != self.state.as_ref().unwrap().toggle_bit {
            // TODO unwrap
            return Err(SdoAbortedError(0x0503_0000));
        }
        /*
        data = self._buffer[:7]
        size = len(data)

        # Remove sent data from buffer
        del self._buffer[:7]

        res_command = RESPONSE_SEGMENT_UPLOAD
        # Add toggle bit
        res_command |= self._toggle
        # Add nof bytes not used
        res_command |= (7 - size) << 1
        if not self._buffer:
            # Nothing left in buffer
            res_command |= NO_MORE_DATA
        # Toggle bit for next message
        self._toggle ^= TOGGLE_BIT

        response = bytearray(8)
        response[0] = res_command
        response[1:1 + size] = data
        self.send_response(response)
        */
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
        let (index, subindex) = match &self.state {
            Some(state) => (state.index, state.subindex),
            None => (0, 0),
        };

        let [index_lo, index_hi] = index.to_le_bytes();
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
        self.node.network.send_message(self.tx_cobid, data);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::Network;
    use crate::node::Node;
    use core::cell::RefCell;

    pub struct MockNetwork {
        sent_messages: RefCell<Vec<[u8; 8]>>,
    }

    impl MockNetwork {
        pub fn new() -> MockNetwork {
            MockNetwork {
                sent_messages: RefCell::new(vec![]),
            }
        }
    }

    impl Network for MockNetwork {
        fn send_message(&self, can_id: u32, data: [u8; 8]) {
            dbg!(can_id, data);
            println!("sent");
            self.sent_messages.borrow_mut().push(data);
        }
    }

    fn test_request(network: &MockNetwork, request: &[u8]) {
        let rx_cobid = 69;
        let tx_cobid = 420;

        let node = Node { network };

        let mut server = SdoServer::new(rx_cobid, tx_cobid, node);

        server.on_request(0, request);
    }

    #[test]
    fn test_init_upload() {
        let data = [64, 2, 3, 4, 5, 6, 7, 8];
        let network = MockNetwork::new();
        test_request(&network, &data);
        dbg!(network.sent_messages.borrow()[0]);

        assert_eq!(network.sent_messages.borrow()[0], [67, 2, 3, 4, 1, 2, 3, 4]);
    }

    #[test]
    fn test_abort() {
        // invalid command specifier
        let data = [7 << 5, 0, 0, 0, 0, 0, 0, 0];
        let network = MockNetwork::new();
        test_request(&network, &data);
        assert_eq!(
            network.sent_messages.borrow()[0],
            [128, 0, 0, 0, 1, 0, 4, 5]
        );
    }

    #[test]
    fn test_bad_data() {
        let network = MockNetwork::new();

        test_request(&network, &[0; 7]);
        test_request(&network, &[0; 9]);
        test_request(&network, &[]);
        assert!(network.sent_messages.borrow().is_empty());
    }
}
