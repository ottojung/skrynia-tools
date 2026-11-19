pub mod delivery;
pub mod durable;
pub mod grants;
pub mod push;
pub use self::store::{Mutation, MutationKind, Object, ObjectRead, Store};
pub(crate) use super::reply;
pub(crate) use super::scope;
use super::types::Response as MetaResponse;
pub(crate) use super::types::{Request, Response};
pub use push::Push;
mod store;
mod unavail;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompatError {
    pub status: u16,
    pub code: String,
}
impl CompatError {
    pub fn new(status: u16, code: impl Into<String>) -> Self {
        Self {
            status,
            code: code.into(),
        }
    }
}
pub type CompatResult<T> = Result<T, CompatError>;

pub trait Meta {
    fn health(&self) -> MetaResponse;
    fn version(&self) -> MetaResponse;
    fn push_vapid(&self) -> MetaResponse;
}
pub trait CompatBackend: Meta + Store + Push {}

#[derive(Clone, Debug, Default)]
pub struct Unavailable;
impl Meta for Unavailable {
    fn health(&self) -> MetaResponse {
        reply::error(503, "unavailable")
    }
    fn version(&self) -> MetaResponse {
        self.health()
    }
    fn push_vapid(&self) -> MetaResponse {
        self.health()
    }
}
impl Store for Unavailable {}
impl CompatBackend for Unavailable {}
