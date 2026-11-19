mod req;
pub use req::Request;

pub const VERSION: u8 = 1;
pub const OP_REGISTER: u8 = 1;
pub const OP_SET: u8 = 2;
pub const OP_INVOKE: u8 = 3;
pub const OP_LOOKUP: u8 = 4;
pub const OP_REVOKE: u8 = 5;
pub const OP_V2_BOOTSTRAP: u8 = 6;
pub const OP_V2_MUTATE: u8 = 7;
pub const STATUS_OK: u8 = 0;
pub const STATUS_ERR: u8 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum V2Kind {
    Create,
    Replace,
    Delete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub status: u8,
    pub body: Vec<u8>,
}
