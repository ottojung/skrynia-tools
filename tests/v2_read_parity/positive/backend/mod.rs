mod meta;
mod model;
mod store;
pub use model::RootStore;
pub(super) use skrynia_tools::wire::v2compat::{
    CompatBackend, CompatError, CompatResult, Meta, Object, ObjectRead, Response, Store,
    Unavailable,
};
pub(super) use skrynia_tools::wire::v2compat::{Mutation, MutationKind};
