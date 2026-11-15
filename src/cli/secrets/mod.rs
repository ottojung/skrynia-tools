//! Private, durable credential persistence for administration.
mod parent;
mod read;
mod write;
pub use read::read;
pub use write::{replace, write_new};
#[cfg(test)]
mod tests;
