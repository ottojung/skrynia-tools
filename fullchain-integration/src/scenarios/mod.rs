//! Full-chain acceptance scenarios against real compiled artifacts.
pub(crate) mod keys;
mod negative;
mod p1;
mod positive;
pub(crate) mod transport;
mod wire;
pub use negative::run as run_negative;
pub use positive::run_positive;
