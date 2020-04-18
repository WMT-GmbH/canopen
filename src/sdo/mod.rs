pub mod server;
pub use server::SdoServer;

pub mod errors;
pub use errors::SDOAbortCode;

const REQUEST_SEGMENT_DOWNLOAD: u8 = 0 << 5;
const REQUEST_DOWNLOAD: u8 = 1 << 5;
const REQUEST_UPLOAD: u8 = 2 << 5;
const REQUEST_SEGMENT_UPLOAD: u8 = 3 << 5;
const REQUEST_ABORTED: u8 = 4 << 5;
const _REQUEST_BLOCK_UPLOAD: u8 = 5 << 5;
const _REQUEST_BLOCK_DOWNLOAD: u8 = 6 << 5;

const RESPONSE_SEGMENT_UPLOAD: u8 = 0 << 5;
const RESPONSE_SEGMENT_DOWNLOAD: u8 = 1 << 5;
const RESPONSE_UPLOAD: u8 = 2 << 5;
const RESPONSE_DOWNLOAD: u8 = 3 << 5;
const RESPONSE_ABORTED: u8 = 4 << 5;
const _RESPONSE_BLOCK_DOWNLOAD: u8 = 5 << 5;
const _RESPONSE_BLOCK_UPLOAD: u8 = 6 << 5;

const EXPEDITED: u8 = 0x2;
const SIZE_SPECIFIED: u8 = 0x1;
const _BLOCK_SIZE_SPECIFIED: u8 = 0x2;
const _CRC_SUPPORTED: u8 = 0x4;
const NO_MORE_DATA: u8 = 0x1;
const _NO_MORE_BLOCKS: u8 = 0x80;
const TOGGLE_BIT: u8 = 0x10;
