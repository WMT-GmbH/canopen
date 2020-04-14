use super::*;
// use alloc::vec::Vec;
use crate::sdo::errors::SDOAbortCode;
use core::convert::TryInto;

pub struct SdoServer<'a> {
    _rx_cobid: u32,
    tx_cobid: u32,
    node: node::Node<'a>,
    state: State,
}

#[derive(Default)]
struct State {
    index: u16,
    subindex: u8,
    toggle_bit: u8,
    buffer: Vec<u8>,
}

impl SdoServer<'_> {
    pub fn new(rx_cobid: u32, tx_cobid: u32, node: node::Node) -> SdoServer {
        SdoServer {
            _rx_cobid: rx_cobid,
            tx_cobid,
            node,
            state: State::default(),
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

        // TODO result could be Result<Frame, SDOAbortCode>
        // then call send_response from here
        let result = match ccs {
            REQUEST_UPLOAD => self.init_upload(request),
            REQUEST_SEGMENT_UPLOAD => self.segmented_upload(command),
            REQUEST_DOWNLOAD => self.init_download(request),
            REQUEST_SEGMENT_DOWNLOAD => self.segmented_download(command, request),
            REQUEST_ABORTED => Ok(()),
            _ => Err(SDOAbortCode::CommandSpecifierError),
        };
        if let Err(abort_error) = result {
            self.abort(abort_error)
        }
    }

    fn init_upload(&mut self, request: [u8; 7]) -> Result<(), SDOAbortCode> {
        let index = u16::from_le_bytes(request[0..2].try_into().unwrap());
        let subindex = request[2];
        self.state.index = index;
        self.state.subindex = subindex;

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
            self.state.buffer = data;
            self.state.toggle_bit = 0;
        }

        response[0] = res_command;
        response[1..3].copy_from_slice(&index.to_le_bytes());
        response[3] = subindex;
        self.send_response(response);
        Ok(())
    }

    fn segmented_upload(&mut self, command: u8) -> Result<(), SDOAbortCode> {
        if command & TOGGLE_BIT != self.state.toggle_bit {
            return Err(SDOAbortCode::ToggleBitNotAlternated);
        }

        let size = self.state.buffer.len().min(7);
        let data: Vec<u8> = self.state.buffer.drain(..size).collect();
        let mut res_command = RESPONSE_SEGMENT_DOWNLOAD;
        res_command |= self.state.toggle_bit; // add toggle bit
        res_command |= (7 - size as u8) << 1; // add nof bytes not used

        if self.state.buffer.is_empty() {
            res_command |= NO_MORE_DATA; // nothing left in buffer
        }

        self.state.toggle_bit ^= TOGGLE_BIT;

        let mut response = [0; 8];
        response[0] = res_command;
        response[1..1 + size].copy_from_slice(&data);
        self.send_response(response);
        Ok(())
    }

    fn init_download(&mut self, _request: [u8; 7]) -> Result<(), SDOAbortCode> {
        Ok(())
    }

    fn segmented_download(&mut self, _command: u8, _request: [u8; 7]) -> Result<(), SDOAbortCode> {
        Ok(())
    }

    fn abort(&mut self, abort_error: SDOAbortCode) {
        let [index_lo, index_hi] = self.state.index.to_le_bytes();
        let subindex = self.state.subindex;
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
        self.node.network.send_message(self.tx_cobid, data);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::Network;
    use crate::node::Node;
    use crate::objectdictionary;
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
        fn send_message(&self, _can_id: u32, data: [u8; 8]) {
            self.sent_messages.borrow_mut().push(data);
        }
    }

    fn mock_server(network: &MockNetwork) -> SdoServer {
        let rx_cobid = 69;
        let tx_cobid = 420;

        let mut od = objectdictionary::ObjectDictionary::default();
        od.add_object(objectdictionary::Object::Variable {
            index: 1,
            subindex: 0,
        });
        od.add_object(objectdictionary::Object::Variable {
            index: 2,
            subindex: 0,
        });

        let node = Node { network, od };

        SdoServer::new(rx_cobid, tx_cobid, node)
    }

    #[test]
    fn test_expedited_upload() {
        let network = MockNetwork::new();
        let mut server = mock_server(&network);

        server.on_request(server.tx_cobid, &[64, 1, 0, 0, 0, 0, 0, 0]);
        assert_eq!(network.sent_messages.borrow()[0], [67, 1, 0, 0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_segmented_upload() {
        let network = MockNetwork::new();
        let mut server = mock_server(&network);

        server.on_request(server.tx_cobid, &[64, 2, 0, 0, 0, 0, 0, 0]);
        server.on_request(server.tx_cobid, &[96, 0, 0, 0, 0, 0, 0, 0]);

        assert_eq!(network.sent_messages.borrow()[0], [65, 2, 0, 0, 5, 0, 0, 0]);
        assert_eq!(network.sent_messages.borrow()[1], [37, 1, 2, 3, 4, 5, 0, 0]);
    }

    #[test]
    fn test_abort() {
        let network = MockNetwork::new();
        let mut server = mock_server(&network);
        server.on_request(server.tx_cobid, &[7 << 5, 0, 0, 0, 0, 0, 0, 0]); // invalid command specifier
        server.on_request(server.tx_cobid, &[64, 0, 0, 0, 0, 0, 0, 0]); // upload invalid index
        assert_eq!(
            network.sent_messages.borrow()[0],
            [128, 0, 0, 0, 1, 0, 4, 5]
        );
        assert_eq!(
            network.sent_messages.borrow()[1],
            [128, 0, 0, 0, 0, 0, 2, 6]
        );
    }

    #[test]
    fn test_bad_data() {
        let network = MockNetwork::new();

        let mut server = mock_server(&network);

        server.on_request(server.tx_cobid, &[0; 7]);
        server.on_request(server.tx_cobid, &[0; 9]);
        server.on_request(server.tx_cobid, &[]);
        assert!(network.sent_messages.borrow().is_empty());
    }
}
