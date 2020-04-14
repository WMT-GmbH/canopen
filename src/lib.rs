/*#![no_std]
#[macro_use]
extern crate alloc;*/

pub mod network;
pub mod objectdictionary;
pub mod node;
pub mod sdo;

#[cfg(test)]
#[macro_use]
extern crate std;
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
