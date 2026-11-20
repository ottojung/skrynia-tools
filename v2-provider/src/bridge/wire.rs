use super::InvokeError;
pub fn key_req(op: u8, key: &str) -> Option<Vec<u8>> {
    let len = u16::try_from(key.len()).ok()?;
    let mut out = vec![op];
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(key.as_bytes());
    Some(out)
}
pub fn key_body_req(op: u8, key: &str, body: &[u8]) -> Option<Vec<u8>> {
    let mut out = key_req(op, key)?;
    out.extend_from_slice(body);
    Some(out)
}
pub fn ok(reply: &[u8]) -> Result<(), InvokeError> {
    if reply == [0] {
        Ok(())
    } else {
        Err(InvokeError::Unavailable)
    }
}
