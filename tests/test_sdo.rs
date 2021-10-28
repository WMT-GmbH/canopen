use core::num::NonZeroUsize;
use core::sync::atomic::AtomicU32;
use core::sync::atomic::AtomicU8;

use canopen::objectdictionary::datalink::{DataLink, ReadStream, WriteStream};
use canopen::objectdictionary::Variable;
use canopen::sdo::SDOAbortCode;
use canopen::{CanOpenNode, ObjectDictionary};
use embedded_can::Frame;
use std::sync::RwLock;

mod frame;
use frame::CanOpenFrame;

struct MockObject(RwLock<Vec<u8>>);
impl DataLink for MockObject {
    fn size(&self, _index: u16, _subindex: u8) -> Option<NonZeroUsize> {
        None
    }

    fn read(&self, read_stream: &mut ReadStream) -> Result<(), SDOAbortCode> {
        let data = self.0.read().unwrap();
        let unread_data = &data[*read_stream.total_bytes_read..];

        let new_data_len = if unread_data.len() <= read_stream.buf.len() {
            read_stream.is_last_segment = true;
            unread_data.len()
        } else {
            read_stream.buf.len()
        };

        read_stream.buf[..new_data_len].copy_from_slice(&unread_data[..new_data_len]);
        *read_stream.total_bytes_read += new_data_len;

        Ok(())
    }

    fn write(&self, write_stream: &WriteStream<'_>) -> Result<(), SDOAbortCode> {
        let mut buf = self.0.write().unwrap();
        if write_stream.offset == 0 {
            buf.clear();
        }
        buf.extend_from_slice(write_stream.new_data);
        Ok(())
    }
}

macro_rules! on_sdo_message {
    ($node:ident, $data:expr) => {
        $node.on_message(&CanOpenFrame::new($node.sdo_server.rx_cobid, $data).unwrap())
    };
}

#[test]
fn test_expedited_download() {
    let obj_1 = MockObject(RwLock::new(vec![]));
    let obj_2 = MockObject(RwLock::new(vec![]));

    let objects = [Variable::new(1, 0, &obj_1), Variable::new(2, 0, &obj_2)];
    let od = ObjectDictionary::new(&objects);

    let mut node = CanOpenNode::new(2, od);

    let response_0 = on_sdo_message!(node, &[0x22, 0x01, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04]); // size not specified
    let response_1 = on_sdo_message!(node, &[0x27, 0x02, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04]); // size specified

    assert_eq!(
        response_0.unwrap().data(),
        [0x60, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    );
    assert_eq!(
        response_1.unwrap().data(),
        [0x60, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    );

    assert_eq!(obj_1.0.read().unwrap().as_slice(), [1, 2, 3, 4]);
    assert_eq!(obj_2.0.read().unwrap().as_slice(), [1, 2, 3]);
}

#[test]
fn test_segmented_download() {
    let obj = MockObject(RwLock::new(vec![]));

    let objects = [Variable::new(1, 0, &obj)];
    let od = ObjectDictionary::new(&objects);

    let mut node = CanOpenNode::new(2, od);

    let response_0 = on_sdo_message!(node, &[0x21, 0x01, 0x00, 0x00, 0x13, 0x00, 0x00, 0x00]);
    let response_1 = on_sdo_message!(node, &[0x00, 0x41, 0x20, 0x6c, 0x6f, 0x6e, 0x67, 0x20]);
    let response_2 = on_sdo_message!(node, &[0x13, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67, 0x00]);

    assert_eq!(
        response_0.unwrap().data(),
        [0x60, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    );
    assert_eq!(
        response_1.unwrap().data(),
        [0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    );
    assert_eq!(
        response_2.unwrap().data(),
        [0x30, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    );

    assert_eq!(obj.0.read().unwrap().as_slice(), b"A long string");
}

#[test]
fn test_expedited_upload() {
    let obj = AtomicU32::new(0x04030201);
    let objects = [Variable::new(1, 0, &obj)];
    let od = ObjectDictionary::new(&objects);

    let mut node = CanOpenNode::new(2, od);

    let response_0 = on_sdo_message!(node, &[0x40, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    assert_eq!(
        response_0.unwrap().data(),
        [0x43, 0x01, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04]
    );
}

#[test]
fn test_segmented_upload() {
    let obj = MockObject(RwLock::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9]));

    let objects = [Variable::new(1, 0, &obj)];
    let od = ObjectDictionary::new(&objects);

    let mut node = CanOpenNode::new(2, od);

    let response_0 = on_sdo_message!(node, &[0x40, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    let response_1 = on_sdo_message!(node, &[0x60, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    let response_2 = on_sdo_message!(node, &[0x70, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

    assert_eq!(
        response_0.unwrap().data(),
        [0x40, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    );
    assert_eq!(
        response_1.unwrap().data(),
        [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07]
    );
    assert_eq!(
        response_2.unwrap().data(),
        [0x1b, 0x08, 0x09, 0x00, 0x00, 0x00, 0x00, 0x00]
    );
}

#[test]
fn test_segmented_upload_with_known_size() {
    let obj = "A long string";

    let objects = [Variable::new(1, 0, &obj)];
    let od = ObjectDictionary::new(&objects);

    let mut node = CanOpenNode::new(2, od);

    let response_0 = on_sdo_message!(node, &[0x40, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    let response_1 = on_sdo_message!(node, &[0x60, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    let response_2 = on_sdo_message!(node, &[0x70, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

    assert_eq!(
        response_0.unwrap().data(),
        [0x41, 0x01, 0x00, 0x00, 0x0d, 0x00, 0x00, 0x00]
    );
    assert_eq!(
        response_1.unwrap().data(),
        [0x00, 0x41, 0x20, 0x6c, 0x6f, 0x6e, 0x67, 0x20]
    );
    assert_eq!(
        response_2.unwrap().data(),
        [0x13, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67, 0x00]
    );
}

#[test]
fn test_abort() {
    let obj = AtomicU8::new(0);
    let objects = [Variable::new(0x0001, 0x00, &obj)];
    let od = ObjectDictionary::new(&objects);
    let mut node = CanOpenNode::new(2, od);
    let response_0 = on_sdo_message!(node, &[0xe0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]); // invalid command specifier
    let response_1 = on_sdo_message!(node, &[0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]); // upload invalid index
    let response_2 = on_sdo_message!(node, &[0x40, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00]); // upload invalid subindex
                                                                                               // TODO TOGGLE Bit not alternated
    assert_eq!(
        response_0.unwrap().data(),
        [0x80, 0x00, 0x00, 0x00, 0x01, 0x00, 0x04, 0x05]
    );
    assert_eq!(
        response_1.unwrap().data(),
        [0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x06]
    );
    assert_eq!(
        response_2.unwrap().data(),
        [0x80, 0x01, 0x00, 0x01, 0x11, 0x00, 0x09, 0x06]
    );
}

/*
#[test]
fn test_thread() {

    static OD: ObjectDictionary = ObjectDictionary { objects: &[] };
    let mut node = CanOpenNode::new(2, &network, &OD);
    let mut tpdo = TPDO(&OD);
    on_sdo_message!(node, &[]);
    let t = std::thread::spawn(move || tpdo.stuff());
    on_sdo_message!(node, &[]);
    t.join().unwrap();
}
*/
