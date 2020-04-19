mod my_od;

use core::cell::RefCell;

use canopen::objectdictionary::{Object, RefCellDataLink};
use canopen::LocalNode;
use canopen::Network;

#[derive(Default)]
pub struct MockNetwork {
    sent_messages: RefCell<Vec<[u8; 8]>>,
}

impl Network for MockNetwork {
    fn send_message(&self, _can_id: u32, data: [u8; 8]) {
        self.sent_messages.borrow_mut().push(data);
    }
}

#[test]
fn test_expedited_upload() {
    let network = MockNetwork::default();
    let od = my_od::get_od();
    let mut node = LocalNode::new(2, &network, &od);

    node.data_store.add_data_link(
        &my_od::OD_DUMMY0001,
        RefCellDataLink(RefCell::new(vec![1, 2, 3, 4])),
    );

    node.sdo_server
        .on_request(&mut node.data_store, &[64, 1, 0, 0, 0, 0, 0, 0]);
    assert_eq!(network.sent_messages.borrow()[0], [67, 1, 0, 0, 1, 2, 3, 4]);
}

#[test]
fn test_segmented_upload() {
    let network = MockNetwork::default();
    let od = my_od::get_od();
    let mut node = LocalNode::new(2, &network, &od);

    node.data_store.add_data_link(
        &my_od::OD_DUMMY0002,
        RefCellDataLink(RefCell::new(vec![1, 2, 3, 4, 5])),
    );

    node.sdo_server
        .on_request(&mut node.data_store, &[64, 2, 0, 0, 0, 0, 0, 0]);
    node.sdo_server
        .on_request(&mut node.data_store, &[96, 0, 0, 0, 0, 0, 0, 0]);

    assert_eq!(network.sent_messages.borrow()[0], [65, 2, 0, 0, 5, 0, 0, 0]);
    assert_eq!(network.sent_messages.borrow()[1], [5, 1, 2, 3, 4, 5, 0, 0]);
}

#[test]
fn test_expedited_download() {
    let network = MockNetwork::default();
    let od = my_od::get_od();
    let mut node = LocalNode::new(2, &network, &od);

    node.sdo_server
        .on_request(&mut node.data_store, &[34, 1, 0, 0, 1, 2, 3, 4]); // size not specified
    node.sdo_server
        .on_request(&mut node.data_store, &[39, 2, 0, 0, 1, 2, 3, 4]); // size specified

    assert_eq!(network.sent_messages.borrow()[0], [96, 1, 0, 0, 0, 0, 0, 0]);
    assert_eq!(network.sent_messages.borrow()[1], [96, 2, 0, 0, 0, 0, 0, 0]);
}

#[test]
fn test_abort() {
    let network = MockNetwork::default();
    let od = my_od::get_od();
    let mut node = LocalNode::new(2, &network, &od);
    node.sdo_server
        .on_request(&mut node.data_store, &[7 << 5, 0, 0, 0, 0, 0, 0, 0]); // invalid command specifier
    node.sdo_server
        .on_request(&mut node.data_store, &[64, 0, 0, 0, 0, 0, 0, 0]); // upload invalid index
    node.sdo_server
        .on_request(&mut node.data_store, &[64, 3, 48, 2, 0, 0, 0, 0]); // upload invalid subindex
                                                                        // TODO TOGGLE Bit not alternated
    assert_eq!(
        network.sent_messages.borrow()[0],
        [128, 0, 0, 0, 0x01, 0x00, 0x04, 0x05]
    );
    assert_eq!(
        network.sent_messages.borrow()[1],
        [128, 0, 0, 0, 0x00, 0x00, 0x02, 0x06]
    );
    assert_eq!(
        network.sent_messages.borrow()[2],
        [128, 3, 48, 2, 0x11, 0x00, 0x09, 0x06]
    );
}

#[test]
fn test_bad_data() {
    let network = MockNetwork::default();

    let od = my_od::get_od();
    let mut node = LocalNode::new(2, &network, &od);

    node.sdo_server.on_request(&mut node.data_store, &[0; 7]);
    node.sdo_server.on_request(&mut node.data_store, &[0; 9]);
    node.sdo_server.on_request(&mut node.data_store, &[]);
    assert!(network.sent_messages.borrow().is_empty());
}
