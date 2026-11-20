mod expect;
mod revoke;
mod set;
use super::lock::with_prog_lock;
use super::{Disk, MutLock, RegErr, Registry};
