//! Native Skrynia HTTP operations, independent of app deployment.
mod grants;
mod read;
mod request;
mod store;
mod types;
pub use types::{CreateResult, SkryniaClient, Store};
