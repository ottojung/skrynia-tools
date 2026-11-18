use super::types::Response;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompatError {
    pub status: u16,
    pub code: String,
}

pub type CompatResult<T> = Result<T, CompatError>;

/// Injected capability for v2-compatible operations over Program 0.
pub trait CompatBackend {
    fn health(&self) -> Response;
    fn version(&self) -> Response;
    fn push_vapid(&self) -> Response;
}

#[derive(Clone, Debug, Default)]
pub struct Unavailable;

impl CompatBackend for Unavailable {
    fn health(&self) -> Response {
        Response {
            status: 503,
            headers: Default::default(),
            body: br#"{"error":"unavailable"}"#.to_vec(),
        }
    }
    fn version(&self) -> Response {
        self.health()
    }
    fn push_vapid(&self) -> Response {
        self.health()
    }
}
