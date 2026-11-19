mod api;
mod auth;
mod mem;
mod model;
mod ops;
mod parse;
mod store;
#[cfg(test)]
mod tests;
mod validate;
#[allow(unused_imports)]
pub(crate) use super::CompatResult;
pub(crate) use super::{CompatError, Request, Response, reply, scope};
#[allow(unused_imports)]
pub use api::{delete, subscribe, update};
pub use mem::Mem;
pub use model::{Events, Keys, Record, SubReq, SubView, UpdateReq};
pub use ops::{GrantAuth, Ids};
pub use parse::{sub as parse_sub, update as parse_update};
pub use store::Store;
#[allow(unused_imports)]
pub use validate::{validate_sub, validate_update};
pub trait Push {
    fn push_subscribe(&self, req: &Request) -> Response;
    fn push_update(&self, id: &str, req: &Request) -> Response;
    fn push_delete(&self, id: &str, req: &Request) -> Response;
}
