use embedded_can::Frame;

use canopen::objectdictionary::OdData;
use canopen::sdo::SdoServer;
use canopen::{CanOpenService, NodeId};
use frame::CanOpenFrame;

mod frame;
macro_rules! on_sdo_message {
    ($node:ident, $od:ident, $data:expr) => {
        $node.on_message(&CanOpenFrame::new($node.rx_cobid, $data).unwrap(), &mut $od)
    };
}

#[test]
fn test_expedited_download() {
    #[derive(OdData)]
    struct Data {
        #[canopen(index = 1)]
        obj_1: [u8; 4],
        #[canopen(index = 2)]
        obj_2: [u8; 3],
    }

    let mut od = Data {
        obj_1: [0; 4],
        obj_2: [0; 3],
    }
    .into_od();

    let mut sdo_server = SdoServer::new(NodeId::new(2).unwrap());

    let response_0 = on_sdo_message!(
        sdo_server,
        od,
        &[0x22, 0x01, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04]
    ); // size not specified
    let response_1 = on_sdo_message!(
        sdo_server,
        od,
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

    assert_eq!(od.data().obj_1.as_slice(), &[1, 2, 3, 4]);
    assert_eq!(od.data().obj_2.as_slice(), &[1, 2, 3]);
}

#[test]
fn test_segmented_download() {
    #[derive(OdData)]
    struct Data {
        #[canopen(index = 1)]
        obj: [u8; 13],
    }

    let mut od = Data { obj: [0; 13] }.into_od();

    let mut sdo_server = SdoServer::new(NodeId::new(2).unwrap());

    // REQUEST_DOWNLOAD|SIZE_SPECIFIED, index=1, subindex=0, len=13
    let response_0 = on_sdo_message!(
        sdo_server,
        od,
        &[0x21, 0x01, 0x00, 0x00, 0x0d, 0x00, 0x00, 0x00]
    );
    // REQUEST_SEGMENT_DOWNLOAD, data
    let response_1 = on_sdo_message!(
        sdo_server,
        od,
        &[0x00, 0x41, 0x20, 0x6c, 0x6f, 0x6e, 0x67, 0x20]
    );
    // REQUEST_SEGMENT_DOWNLOAD|TOGGLE_BIT|NO_MORE_DATA|unused_bytes=1, data
    let response_2 = on_sdo_message!(
        sdo_server,
        od,
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

    assert_eq!(od.data().obj.as_slice(), b"A long string");
}

#[test]
fn test_expedited_upload() {
    #[derive(OdData)]
    struct Data {
        #[canopen(index = 1)]
        obj: u32,
    }

    let mut od = Data { obj: 0x04030201 }.into_od();

    let mut sdo_server = SdoServer::new(NodeId::new(2).unwrap());

    let response_0 = on_sdo_message!(
        sdo_server,
        od,
        &[0x40, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    );
    assert_eq!(
        response_0.unwrap().data(),
        [0x43, 0x01, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04]
    );
}

#[test]
fn test_segmented_upload() {
    #[derive(OdData)]
    struct Data {
        #[canopen(index = 1)]
        obj: [u8; 9],
    }

    let mut od = Data {
        obj: [1, 2, 3, 4, 5, 6, 7, 8, 9],
    }
    .into_od();

    let mut sdo_server = SdoServer::new(NodeId::new(2).unwrap());

    let response_0 = on_sdo_message!(
        sdo_server,
        od,
        &[0x40, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    );
    let response_1 = on_sdo_message!(
        sdo_server,
        od,
        &[0x60, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    );
    let response_2 = on_sdo_message!(
        sdo_server,
        od,
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

/*
TODO
#[test]
fn test_segmented_upload_with_known_size() {
    #[derive(OdData)]
    struct Data {
        #[canopen(index = 1, readonly)]
        obj: &'static str,
    }

    let mut od = Data {
        obj: "A long string",
    }
    .into_od();

    let mut sdo_server = SdoServer::new(NodeId::new(2).unwrap());

    let response_0 = on_sdo_message!(
        sdo_server,
        od,
        &[0x40, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    );
    let response_1 = on_sdo_message!(
        sdo_server,
        od,
        &[0x60, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    );
    let response_2 = on_sdo_message!(
        sdo_server,
        od,
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
*/
#[test]
fn test_abort() {
    #[derive(OdData)]
    struct Data {
        #[canopen(index = 1)]
        obj: u8,
    }

    let mut od = Data { obj: 0 }.into_od();

    let mut sdo_server = SdoServer::new(NodeId::new(2).unwrap());

    let response_0 = on_sdo_message!(
        sdo_server,
        od,
        &[0xe0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    ); // invalid command specifier
    let response_1 = on_sdo_message!(
        sdo_server,
        od,
        &[0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    ); // upload invalid index
    let response_2 = on_sdo_message!(
        sdo_server,
        od,
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
