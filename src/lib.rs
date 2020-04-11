extern crate failure;
#[macro_use]
extern crate failure_derive;


pub mod canopen;
pub mod frames;
mod split;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
