pub mod durable;
pub mod grants;
pub use self::store::{Mutation, MutationKind, Object, ObjectRead, Store};
pub(super) use super::scope;
pub(super) use super::types::Response;
mod store;

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
    fn health(&self) -> Response;
    fn version(&self) -> Response;
    fn push_vapid(&self) -> Response;
}
pub trait CompatBackend: Meta + Store {}

#[derive(Clone, Debug, Default)]
pub struct Unavailable;
impl Meta for Unavailable {
    fn health(&self) -> Response {
        super::reply::error(503, "unavailable")
    }
    fn version(&self) -> Response {
        self.health()
    }
    fn push_vapid(&self) -> Response {
        self.health()
    }
}
impl Store for Unavailable {}
impl CompatBackend for Unavailable {}
