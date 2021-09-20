#![feature(const_fn_trait_bound)]
#![warn(rust_2018_idioms)]
#![no_std]

pub use network::Network;
pub use node::LocalNode;
pub use objectdictionary::{datatypes, ObjectDictionary};

pub mod network;
pub mod node;
pub mod objectdictionary;
pub mod sdo;
