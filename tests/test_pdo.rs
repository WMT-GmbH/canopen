use canopen::objectdictionary::OdData;
use canopen::pdo::{DefaultTPDO, TPDO};
use canopen::NodeId;

#[derive(OdData)]
struct Data {
    #[canopen(index = 0x1800, subindex = 0x01)]
    #[canopen(index = 0x1800, subindex = 0x02)]
    #[canopen(index = 0x1800, subindex = 0x03)]
    #[canopen(index = 0x1800, subindex = 0x05)]
    #[canopen(index = 0x1800, subindex = 0x06)]
    #[canopen(index = 0x1A00, subindex = 0x00)]
    #[canopen(index = 0x1A00, subindex = 0x01)]
    #[canopen(index = 0x1A00, subindex = 0x02)]
    #[canopen(index = 0x1A00, subindex = 0x03)]
    #[canopen(index = 0x1A00, subindex = 0x04)]
    #[canopen(index = 0x1A00, subindex = 0x05)]
    #[canopen(index = 0x1A00, subindex = 0x06)]
    #[canopen(index = 0x1A00, subindex = 0x07)]
    #[canopen(index = 0x1A00, subindex = 0x08)]
    tpdo: TPDO,
}

#[test]
fn tpdo() {
    let mut od = Data {
        tpdo: DefaultTPDO::TPDO1.new(NodeId::NODE_ID_0, |_, new| Ok(new)),
    }
    .into_od();

    dbg!(od.read(0x1800, 0x01).unwrap().as_bytes());
    dbg!(od.read(0x1800, 0x02).unwrap().as_bytes());
    dbg!(od.read(0x1A00, 0x00).unwrap().as_bytes());
    dbg!(od.read(0x1A00, 0x01).unwrap().as_bytes());
}
