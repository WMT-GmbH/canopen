#![no_std]
#[macro_use]
extern crate alloc;
pub mod network;
pub mod node;
pub mod objectdictionary;
pub mod sdo;

#[cfg(test)]
#[macro_use]
extern crate std;
