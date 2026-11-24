mod read;
mod write;
pub(super) use super::{Disk, appdata, map_push_err};
pub(super) use crate::{
    MutLock,
    reg::{RegErr, Registry},
};
