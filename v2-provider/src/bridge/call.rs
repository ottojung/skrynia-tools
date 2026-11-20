#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InvokeError {
    Unavailable,
}
pub trait Callback {
    fn invoke(&self, request: &[u8]) -> Result<Vec<u8>, InvokeError>;
}
