//! Native Skrynia HTTP operations, independent of app deployment.
mod cas;
mod errors;
mod grants;
mod path;
mod read;
mod request;
mod store;
mod types;
pub(super) use super::wire;
pub use errors::ClientError;
pub use types::{CreateResult, SkryniaClient, Store};
