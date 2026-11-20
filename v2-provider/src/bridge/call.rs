/// Generic callback transport. The owner delegates restricted v3 API access;
/// the Skrynia kernel neither interprets v2 nor knows Program 1.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InvokeError {
    Unavailable,
}
pub trait Callback {
    fn invoke(&self, request: &[u8]) -> Result<Vec<u8>, InvokeError>;
}
