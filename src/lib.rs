//#![no_std]
#[warn(clippy::all)]
pub mod network;
pub mod node;
pub mod sdo;
mod split;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
