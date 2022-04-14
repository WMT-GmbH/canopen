use core::sync::atomic::AtomicU32;
use core::sync::atomic::AtomicU8;

use canopen::objectdictionary::datalink::{DataLink, ReadData, WriteStream};
use canopen::objectdictionary::odcell::ODCell;
use canopen::objectdictionary::{ODError, Variable};
use canopen::sdo::SdoServer;
use canopen::{CanOpenService, NodeId};
use embedded_can::Frame;

mod frame;
use frame::CanOpenFrame;

struct MockObject<const N: usize>([u8; N]);
impl<const N: usize> DataLink for MockObject<N> {
    fn read(&self, _: u16, _: u8) -> Result<ReadData, ODError> {
        Ok(self.0.as_slice().into())
    }

    fn write(&mut self, write_stream: WriteStream<'_>) -> Result<(), ODError> {
        write_stream.write_into(&mut self.0)?;
        Ok(())
    }
}

macro_rules! on_sdo_message {
    ($node:ident, $data:expr) => {
        $node.on_message(&CanOpenFrame::new($node.rx_cobid, $data).unwrap())
    };
}

#[test]
fn test_expedited_download() {
    let obj_1 = ODCell::new(MockObject([0; 4]));
    let obj_2 = ODCell::new(MockObject([0; 3]));

    let od = [
        Variable::new_datalink_cell(1, 0, &obj_1),
        Variable::new_datalink_cell(2, 0, &obj_2),
    ];

    let mut sdo_server = SdoServer::new(NodeId::new(2).unwrap(), &od);

    let response_0 = on_sdo_message!(
        sdo_server,
        &[0x22, 0x01, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04]
    ); // size not specified
    let response_1 = on_sdo_message!(
        sdo_server,
        &[0x27, 0x02, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04]
    ); // size specified

    assert_eq!(
        response_0.unwrap().data(),
        [0x60, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    );
    assert_eq!(
        response_1.unwrap().data(),
        [0x60, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    );

    assert_eq!(obj_1.borrow().0.as_slice(), [1, 2, 3, 4]);
    assert_eq!(obj_2.borrow().0.as_slice(), [1, 2, 3]);
}

#[test]
fn test_segmented_download() {
    let obj = ODCell::new(MockObject([0; 13]));

    let od = [Variable::new_datalink_cell(1, 0, &obj)];

    let mut sdo_server = SdoServer::new(NodeId::new(2).unwrap(), &od);

    // REQUEST_DOWNLOAD|SIZE_SPECIFIED, index=1, subindex=0, len=13
    let response_0 = on_sdo_message!(
        sdo_server,
        &[0x21, 0x01, 0x00, 0x00, 0x0d, 0x00, 0x00, 0x00]
    );
    // REQUEST_SEGMENT_DOWNLOAD, data
    let response_1 = on_sdo_message!(
        sdo_server,
        &[0x00, 0x41, 0x20, 0x6c, 0x6f, 0x6e, 0x67, 0x20]
    );
    // REQUEST_SEGMENT_DOWNLOAD|TOGGLE_BIT|NO_MORE_DATA|unused_bytes=1, data
    let response_2 = on_sdo_message!(
        sdo_server,
        &[0x13, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67, 0x00]
    );

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

    assert_eq!(obj.borrow().0.as_slice(), b"A long string");
}

#[test]
fn test_expedited_upload() {
    let obj = AtomicU32::new(0x04030201);
    let od = [Variable::new_datalink_ref(1, 0, &obj, None)];

    let mut sdo_server = SdoServer::new(NodeId::new(2).unwrap(), &od);

    let response_0 = on_sdo_message!(
        sdo_server,
        &[0x40, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    );
    assert_eq!(
        response_0.unwrap().data(),
        [0x43, 0x01, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04]
    );
}

#[test]
fn test_segmented_upload() {
    let obj = ODCell::new(MockObject([1, 2, 3, 4, 5, 6, 7, 8, 9]));

    let od = [Variable::new_datalink_cell(1, 0, &obj)];

    let mut sdo_server = SdoServer::new(NodeId::new(2).unwrap(), &od);

    let response_0 = on_sdo_message!(
        sdo_server,
        &[0x40, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    );
    let response_1 = on_sdo_message!(
        sdo_server,
        &[0x60, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    );
    let response_2 = on_sdo_message!(
        sdo_server,
        &[0x70, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    );

    assert_eq!(
        response_0.unwrap().data(),
        [0x41, 0x01, 0x00, 0x00, 0x09, 0x00, 0x00, 0x00]
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

    let od = [Variable::new(1, 0, obj)];

    let mut sdo_server = SdoServer::new(NodeId::new(2).unwrap(), &od);

    let response_0 = on_sdo_message!(
        sdo_server,
        &[0x40, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    );
    let response_1 = on_sdo_message!(
        sdo_server,
        &[0x60, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    );
    let response_2 = on_sdo_message!(
        sdo_server,
        &[0x70, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    );

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
    let od = [Variable::new_datalink_ref(0x0001, 0x00, &obj, None)];

    let mut sdo_server = SdoServer::new(NodeId::new(2).unwrap(), &od);
    let response_0 = on_sdo_message!(
        sdo_server,
        &[0xe0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    ); // invalid command specifier
    let response_1 = on_sdo_message!(
        sdo_server,
        &[0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    ); // upload invalid index
    let response_2 = on_sdo_message!(
        sdo_server,
        &[0x40, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00]
    ); // upload invalid subindex
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
