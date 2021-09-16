mod my_od;

use core::cell::RefCell;

use canopen::objectdictionary::{RefCellDataLink, Variable};
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

fn get_data_link_value(node: &LocalNode<MockNetwork>, variable: &Variable) -> Vec<u8> {
    let link = node
        .data_store
        .get_data_link(variable)
        .expect("No data link found");
    match link {
        RefCellDataLink(link) => link.borrow().to_vec(),
        _ => panic!("Data Link not a RefCell"),
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

    node.on_message(&[64, 1, 0, 0, 0, 0, 0, 0]);
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

    node.on_message(&[64, 2, 0, 0, 0, 0, 0, 0]);
    node.on_message(&[96, 0, 0, 0, 0, 0, 0, 0]);

    assert_eq!(network.sent_messages.borrow()[0], [65, 2, 0, 0, 5, 0, 0, 0]);
    assert_eq!(network.sent_messages.borrow()[1], [5, 1, 2, 3, 4, 5, 0, 0]);

    assert_eq!(
        get_data_link_value(&node, &my_od::OD_DUMMY0002),
        vec![1, 2, 3, 4, 5]
    );
}

#[test]
fn test_expedited_download() {
    let network = MockNetwork::default();
    let od = my_od::get_od();
    let mut node = LocalNode::new(2, &network, &od);

    node.on_message(&[34, 1, 0, 0, 1, 2, 3, 4]); // size not specified
    node.on_message(&[39, 2, 0, 0, 1, 2, 3, 4]); // size specified

    assert_eq!(network.sent_messages.borrow()[0], [96, 1, 0, 0, 0, 0, 0, 0]);
    assert_eq!(network.sent_messages.borrow()[1], [96, 2, 0, 0, 0, 0, 0, 0]);

    assert_eq!(
        get_data_link_value(&node, &my_od::OD_DUMMY0001),
        vec![1, 2, 3, 4]
    );
    assert_eq!(
        get_data_link_value(&node, &my_od::OD_DUMMY0002),
        vec![1, 2, 3]
    );
}

#[test]
fn test_segmented_download() {
    let network = MockNetwork::default();
    let od = my_od::get_od();
    let mut node = LocalNode::new(2, &network, &od);

    node.on_message(&[33, 0, 32, 0, 13, 0, 0, 0]);
    node.on_message(&[0, 65, 32, 108, 111, 110, 103, 32]);
    node.on_message(&[19, 115, 116, 114, 105, 110, 103, 0]);

    assert_eq!(
        network.sent_messages.borrow()[0],
        [96, 0, 32, 0, 0, 0, 0, 0]
    );
    assert_eq!(network.sent_messages.borrow()[1], [32, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(network.sent_messages.borrow()[2], [48, 0, 0, 0, 0, 0, 0, 0]);

    assert_eq!(
        get_data_link_value(&node, &my_od::OD_WRITABLE_STRING),
        b"A long string"
    );
}

#[test]
fn test_abort() {
    let network = MockNetwork::default();
    let od = my_od::get_od();
    let mut node = LocalNode::new(2, &network, &od);
    node.on_message(&[7 << 5, 0, 0, 0, 0, 0, 0, 0]); // invalid command specifier
    node.on_message(&[64, 0, 0, 0, 0, 0, 0, 0]); // upload invalid index
    node.on_message(&[64, 3, 48, 2, 0, 0, 0, 0]); // upload invalid subindex
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

    node.on_message(&[0; 7]);
    node.on_message(&[0; 9]);
    node.on_message(&[]);
    assert!(network.sent_messages.borrow().is_empty());
}
