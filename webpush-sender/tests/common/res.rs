pub fn res(status: u8, body: &[u8]) -> Vec<u8> {
    let mut out = vec![1u8, status];
    out.extend_from_slice(&(body.len() as u32).to_le_bytes());
    out.extend_from_slice(body);
    out
}
