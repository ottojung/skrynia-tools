//! Gateway operation selectors and scalar status values.

pub const GET: i32 = 0;
pub const PUT: i32 = 1;
pub const FLUSH: i32 = 2;
pub const CB_GET: i32 = 3;
pub const CB_PUT: i32 = 4;
pub const STATUS: i32 = 5;

pub const IDLE: i64 = 0;
pub const ACTIVE: i64 = 1;
pub const TRAPPED: i64 = 2;
pub const EOF: i64 = -1;
pub const ERR_INVAL: i64 = -3;
pub const ERR_NOTSUP: i64 = -4;
pub const ERR_LIMIT: i64 = -11;

/// The evaluator assigns context handles; these are local names only.
pub const CTX_MAIN: i32 = 0;
pub const CTX_NESTED: i32 = 1;
