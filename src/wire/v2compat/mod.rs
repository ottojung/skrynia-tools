//! v2 grant-based HTTP compatibility facade scaffold.
mod backend;
mod reply;
mod route;
mod types;
pub use backend::{CompatBackend, CompatError, CompatResult, Unavailable};
pub use route::handle;
pub use types::{Request, Response};

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests2;
#[cfg(test)]
mod tests3;
#[cfg(test)]
mod tests4;
#[cfg(test)]
mod tests5;
