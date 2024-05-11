use embedded_can::Frame;

use canopen::objectdictionary::od_cell::OdCell;
use canopen::objectdictionary::OdData;
use canopen::sdo::client::{ReadInto, ReadResult, SdoBuffer, SdoClient};
use canopen::sdo::SdoServer;
use canopen::{CanOpenService, NodeId, ObjectDictionary};
use frame::CanOpenFrame;

mod frame;
macro_rules! on_sdo_message {
    ($node:ident, $od:ident, $data:expr) => {
        $node.on_message(
            &CanOpenFrame::new($node.rx_cobid, &$data).unwrap(),
            &mut $od,
        )
    };
}

#[test]
fn test_expedited_download() {
    #[derive(OdData)]
    struct Data {
        #[canopen(index = 1)]
        obj_1: OdCell<[u8; 4]>,
        #[canopen(index = 2)]
        obj_2: OdCell<[u8; 3]>,
    }

    let mut od = Data {
        obj_1: OdCell::new([0; 4]),
        obj_2: OdCell::new([0; 3]),
    }
    .into_od();

    let mut sdo_server = SdoServer::new(NodeId::NODE_ID_2);

    let response_0 = on_sdo_message!(
        sdo_server,
        od,
        [0x22, 0x01, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04]
    ); // size not specified
    let response_1 = on_sdo_message!(
        sdo_server,
        od,
        [0x27, 0x02, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04]
    ); // size specified

    assert_eq!(
        response_0.unwrap().data(),
        [0x60, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    );
    assert_eq!(
        response_1.unwrap().data(),
        [0x60, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    );

    assert_eq!(od.data.obj_1.get().as_slice(), &[1, 2, 3, 4]);
    assert_eq!(od.data.obj_2.get().as_slice(), &[1, 2, 3]);
}

#[test]
fn test_segmented_download() {
    #[derive(OdData)]
    struct Data {
        #[canopen(index = 1)]
        obj: OdCell<[u8; 13]>,
    }

    let mut od = Data {
        obj: OdCell::new([0; 13]),
    }
    .into_od();

    let mut sdo_server = SdoServer::new(NodeId::NODE_ID_2);

    // REQUEST_DOWNLOAD|SIZE_SPECIFIED, index=1, subindex=0, len=13
    let response_0 = on_sdo_message!(
        sdo_server,
        od,
        [0x21, 0x01, 0x00, 0x00, 0x0d, 0x00, 0x00, 0x00]
    );
    // REQUEST_SEGMENT_DOWNLOAD, data
    let response_1 = on_sdo_message!(
        sdo_server,
        od,
        [0x00, 0x41, 0x20, 0x6c, 0x6f, 0x6e, 0x67, 0x20]
    );
    // REQUEST_SEGMENT_DOWNLOAD|TOGGLE_BIT|NO_MORE_DATA|unused_bytes=1, data
    let response_2 = on_sdo_message!(
        sdo_server,
        od,
        [0x13, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67, 0x00]
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

    assert_eq!(od.data.obj.get().as_slice(), b"A long string");
}

fn read<B: ReadInto, OD, const N: usize>(
    index: u16,
    subindex: u8,
    server: &mut SdoServer,
    od: &mut ObjectDictionary<OD, N>,
    buf: &mut B,
) {
    let mut sdo_buffer = SdoBuffer::new();
    let (mut sdo_producer, sdo_consumer) = sdo_buffer.split();
    let mut sdo_client = SdoClient::new(NodeId::NODE_ID_2, sdo_consumer);
    let mut sdo_reader = sdo_client.read(index, subindex);

    loop {
        match sdo_reader.poll(buf).unwrap() {
            ReadResult::NextRequest(message) => {
                let response: CanOpenFrame = server.on_message(&message.into_frame(), od).unwrap();
                sdo_producer
                    .enqueue(response.data().try_into().unwrap())
                    .unwrap();
            }
            ReadResult::Done => break,
            ReadResult::Waiting => unreachable!(),
        }
    }
}

#[test]
fn test_expedited_upload() {
    #[derive(OdData)]
    struct Data {
        #[canopen(index = 1)]
        obj: u32,
    }

    let mut od = Data { obj: 0x04030201 }.into_od();

    let mut sdo_server = SdoServer::new(NodeId::NODE_ID_2);

    let mut data = 0_u32;
    read(1, 0, &mut sdo_server, &mut od, &mut data);

    assert_eq!(data, 0x04030201);
}

#[test]
fn test_segmented_upload() {
    #[derive(OdData)]
    struct Data {
        #[canopen(index = 1)]
        obj: OdCell<[u8; 9]>,
    }

    let mut od = Data {
        obj: OdCell::new([1, 2, 3, 4, 5, 6, 7, 8, 9]),
    }
    .into_od();

    let mut sdo_server = SdoServer::new(NodeId::NODE_ID_2);

    let mut data = Vec::new();
    read(1, 0, &mut sdo_server, &mut od, &mut data);

    assert_eq!(data, [1, 2, 3, 4, 5, 6, 7, 8, 9]);
}

#[test]
fn test_segmented_upload_with_known_size() {
    #[derive(OdData)]
    struct Data {
        #[canopen(index = 1, read_only)]
        obj: &'static str,
    }

    let mut od = Data {
        obj: "A long string",
    }
    .into_od();

    let mut sdo_server = SdoServer::new(NodeId::NODE_ID_2);

    let mut data = Vec::new();
    read(1, 0, &mut sdo_server, &mut od, &mut data);

    assert_eq!(data, b"A long string");
}

#[test]
fn test_abort() {
    #[derive(OdData)]
    struct Data {
        #[canopen(index = 1)]
        obj: u8,
    }

    let mut od = Data { obj: 0 }.into_od();

    let mut sdo_server = SdoServer::new(NodeId::NODE_ID_2);

    let response_0 = on_sdo_message!(
        sdo_server,
        od,
        [0xe0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    ); // invalid command specifier
    let response_1 = on_sdo_message!(
        sdo_server,
        od,
        [0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    ); // upload invalid index
    let response_2 = on_sdo_message!(
        sdo_server,
        od,
        [0x40, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00]
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
