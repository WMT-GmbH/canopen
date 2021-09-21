use core::cell::RefCell;

use canopen::objectdictionary::datalink::DataLink;
use canopen::objectdictionary::Variable;
use canopen::sdo::SDOAbortCode;
use canopen::Network;
use canopen::{LocalNode, ObjectDictionary};
use core::sync::atomic::AtomicU8;

#[derive(Default)]
pub struct MockNetwork {
    sent_messages: RefCell<Vec<[u8; 8]>>,
}

impl Network for MockNetwork {
    fn send_message(&self, _can_id: u32, data: [u8; 8]) {
        self.sent_messages.borrow_mut().push(data);
    }
}

struct MockObject(RefCell<Vec<u8>>);
impl DataLink for MockObject {
    fn size(&self) -> usize {
        self.0.borrow().len()
    }

    fn read(&self, buf: &mut [u8], offset: usize) -> Result<(), SDOAbortCode> {
        let data = self.0.borrow();
        buf.copy_from_slice(&data[offset..offset + buf.len()]);
        Ok(())
    }

    fn write(&self, data: &[u8], _offset: usize, _no_more_data: bool) -> Result<(), SDOAbortCode> {
        let mut buf = self.0.borrow_mut();
        if _offset == 0 {
            buf.clear();
        }
        buf.extend_from_slice(data);
        Ok(())
    }
}

#[test]
fn test_expedited_upload() {
    let obj = MockObject(RefCell::new(vec![1, 2, 3, 4]));
    let objects = [Variable::new(1, 0, &obj)];
    let od = ObjectDictionary::new(&objects);

    let network = MockNetwork::default();

    let mut node = LocalNode::new(2, &network, &od);

    node.on_message(&[0x40, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    assert_eq!(
        network.sent_messages.borrow()[0],
        [0x43, 0x01, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04]
    );
}

#[test]
fn test_segmented_upload() {
    let obj = MockObject(RefCell::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9]));

    let objects = [Variable::new(1, 0, &obj)];
    let od = ObjectDictionary::new(&objects);

    let network = MockNetwork::default();

    let mut node = LocalNode::new(2, &network, &od);

    node.on_message(&[0x40, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    node.on_message(&[0x60, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    node.on_message(&[0x70, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

    assert_eq!(
        network.sent_messages.borrow()[0],
        [0x41, 0x01, 0x00, 0x00, 0x09, 0x00, 0x00, 0x00]
    );
    assert_eq!(
        network.sent_messages.borrow()[1],
        [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07]
    );
    assert_eq!(
        network.sent_messages.borrow()[2],
        [0x1b, 0x08, 0x09, 0x00, 0x00, 0x00, 0x00, 0x00]
    );
}

#[test]
fn test_expedited_download() {
    let obj_1 = MockObject(RefCell::new(vec![]));
    let obj_2 = MockObject(RefCell::new(vec![]));

    let objects = [Variable::new(1, 0, &obj_1), Variable::new(2, 0, &obj_2)];
    let od = ObjectDictionary::new(&objects);

    let network = MockNetwork::default();

    let mut node = LocalNode::new(2, &network, &od);

    node.on_message(&[0x22, 0x01, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04]); // size not specified
    node.on_message(&[0x27, 0x02, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04]); // size specified

    assert_eq!(
        network.sent_messages.borrow()[0],
        [0x60, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    );
    assert_eq!(
        network.sent_messages.borrow()[1],
        [0x60, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    );

    assert_eq!(obj_1.0.borrow().as_slice(), [1, 2, 3, 4]);
    assert_eq!(obj_2.0.borrow().as_slice(), [1, 2, 3]);
}

#[test]
fn test_segmented_download() {
    let obj = MockObject(RefCell::new(vec![]));

    let objects = [Variable::new(1, 0, &obj)];
    let od = ObjectDictionary::new(&objects);

    let network = MockNetwork::default();

    let mut node = LocalNode::new(2, &network, &od);

    node.on_message(&[0x21, 0x01, 0x00, 0x00, 0x13, 0x00, 0x00, 0x00]);
    node.on_message(&[0x00, 0x41, 0x20, 0x6c, 0x6f, 0x6e, 0x67, 0x20]);
    node.on_message(&[0x13, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67, 0x00]);

    assert_eq!(
        network.sent_messages.borrow()[0],
        [0x60, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    );
    assert_eq!(
        network.sent_messages.borrow()[1],
        [0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    );
    assert_eq!(
        network.sent_messages.borrow()[2],
        [0x30, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    );

    assert_eq!(obj.0.borrow().as_slice(), b"A long string");
}

#[test]
fn test_abort() {
    let obj = AtomicU8::new(0);
    let objects = [Variable::new(0x0001, 0x00, &obj)];
    let od = ObjectDictionary::new(&objects);
    let network = MockNetwork::default();
    let mut node = LocalNode::new(2, &network, &od);
    node.on_message(&[0xe0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]); // invalid command specifier
    node.on_message(&[0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]); // upload invalid index
    node.on_message(&[0x40, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00]); // upload invalid subindex
                                                                        // TODO TOGGLE Bit not alternated
    assert_eq!(
        network.sent_messages.borrow()[0],
        [0x80, 0x00, 0x00, 0x00, 0x01, 0x00, 0x04, 0x05]
    );
    assert_eq!(
        network.sent_messages.borrow()[1],
        [0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x06]
    );
    assert_eq!(
        network.sent_messages.borrow()[2],
        [0x80, 0x01, 0x00, 0x01, 0x11, 0x00, 0x09, 0x06]
    );
}

#[test]
fn test_bad_data() {
    let network = MockNetwork::default();

    let od = ObjectDictionary { objects: &[] };
    let mut node = LocalNode::new(2, &network, &od);

    node.on_message(&[0; 7]);
    node.on_message(&[0; 9]);
    node.on_message(&[]);
    assert!(network.sent_messages.borrow().is_empty());
}
