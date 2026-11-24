//! Deterministic, injected capabilities. All values are disposable fixtures.
mod clock;
mod resolver;
mod rng;

pub use clock::TestClock;
pub use resolver::FixedResolver;
pub use rng::FixedRng;
