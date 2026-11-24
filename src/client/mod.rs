//! Native application service client through a trusted edge.
mod store;
mod transport;
mod types;
use super::wire::native;
pub use types::{CreateResult, SkryniaClient, Store};
