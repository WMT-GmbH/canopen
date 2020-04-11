pub mod server;
pub use self::server::*;

pub mod errors;
pub use errors::SdoAbortedError;

const REQUEST_SEGMENT_DOWNLOAD: u8 = 0 << 5;
const REQUEST_DOWNLOAD: u8 = 1 << 5;
const REQUEST_UPLOAD: u8 = 2 << 5;
const REQUEST_SEGMENT_UPLOAD: u8 = 3 << 5;
const REQUEST_ABORTED: u8 = 4 << 5;
const _REQUEST_BLOCK_UPLOAD: u8 = 5 << 5;
const _REQUEST_BLOCK_DOWNLOAD: u8 = 6 << 5;