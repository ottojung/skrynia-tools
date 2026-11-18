mod backend;
mod key;
mod mime;
mod mutate;
mod read;
mod reply;
mod route;
mod types;

pub use backend::{
    CompatBackend, CompatError, CompatResult, Meta, Mutation, MutationKind, Object, ObjectRead,
    Store, Unavailable,
};
pub use route::handle;
pub use types::{Request, Response};

#[cfg(test)]
mod tests;
pub use backend::grants;
pub use key::{credentials, scope};
