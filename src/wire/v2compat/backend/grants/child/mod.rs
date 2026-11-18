mod fields;
mod limits;
mod validate;
pub struct Spec {
    pub locations: String,
    pub operations: Vec<String>,
    pub max_bytes: u64,
    pub max_objects: u64,
}
pub(super) use super::{Authority, CompatError, scope};
