mod model;
mod policy;
mod queue;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_remove;
pub use model::{Delivery, Outcome, Pending};
pub use policy::Policy;
pub use queue::Queue;
pub trait Clock {
    fn now_ms(&self) -> u64;
}
pub trait Transport {
    fn send(&mut self, sub: &str, endpoint: &str) -> Outcome;
}
pub trait Durable {
    type Error: std::fmt::Display;
    fn load(&self) -> Result<Vec<Pending>, Self::Error>;
    fn save(&self, items: &[Pending]) -> Result<(), Self::Error>;
}
