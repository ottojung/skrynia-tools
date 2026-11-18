mod backend;
mod key;
mod mime;
mod read;
mod reply;
mod route;
mod types;

pub use backend::{
    CompatBackend, CompatError, CompatResult, Meta, Object, ObjectRead, Store, Unavailable,
};
pub use route::handle;
pub use types::{Request, Response};

#[cfg(test)]
mod tests_route;
#[cfg(test)]
mod tests_store;
