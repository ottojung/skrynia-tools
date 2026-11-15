//! Private, durable credential persistence for administration.
mod lock;
mod parent;
mod read;
mod write;
pub use lock::lock;
pub use read::read;
pub use write::{replace, write_new};
#[cfg(test)]
mod tests;
