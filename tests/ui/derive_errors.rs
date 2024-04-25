use canopen::objectdictionary::OdData;

#[derive(OdData)]
struct Test {
    a: u8,
    #[canopen()]
    b: u8,
    #[canopen(index = 0xFFFF_FFFF)]
    c: u8,
    #[canopen(index = 1, read_only, write_only)]
    d: u8,
}

#[derive(OdData)]
struct Test2 {
    #[canopen(index = 1)]
    a1: u8,
    #[canopen(index = 1)]
    a2: u8,
    #[canopen(index = 3)]
    c1: u8,
    #[canopen(index = 2)]
    b: u8,
    #[canopen(index = 3)]
    c2: u8,
}

fn main() {}
