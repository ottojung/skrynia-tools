use crate::{Callback, InvokeBackend};
use skrynia_tools::wire::v2compat::{self, Request, Response};

pub fn dispatch<I: Callback>(request: &Request, invoke: &I) -> Response {
    v2compat::handle(&InvokeBackend { invoke }, request)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameError {
    TooLarge,
    Malformed,
    Encode,
}

/// JSON envelope for the arbitrary guest-evaluation input/output bytes.
/// HTTP methods are DATA in this envelope, never kernel HTTP methods.
pub fn dispatch_bytes<I: Callback>(data: &[u8], invoke: &I) -> Result<Vec<u8>, FrameError> {
    if data.len() > 8 * 1024 * 1024 {
        return Err(FrameError::TooLarge);
    }
    let request: Request = serde_json::from_slice(data).map_err(|_| FrameError::Malformed)?;
    let response = dispatch(&request, invoke);
    let bytes = serde_json::to_vec(&response).map_err(|_| FrameError::Encode)?;
    if bytes.len() > 10 * 1024 * 1024 {
        return Err(FrameError::TooLarge);
    }
    Ok(bytes)
}
