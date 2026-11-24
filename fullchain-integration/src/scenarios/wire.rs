//! Minimal P1 request framing for setup ops (real binary protocol).
pub fn id(out: &mut Vec<u8>, v: &[u8]) {
    out.extend_from_slice(&(v.len() as u16).to_le_bytes());
    out.extend_from_slice(v);
}
pub fn blob(out: &mut Vec<u8>, v: &[u8]) {
    out.extend_from_slice(&(v.len() as u32).to_le_bytes());
    out.extend_from_slice(v);
}
pub fn req_ver(v: u8, op: u8, parts: &[&[u8]]) -> Vec<u8> {
    let mut o = vec![v, op]; // VERSION, op
    for p in parts {
        o.extend_from_slice(p);
    }
    o
}
pub fn req(op: u8, parts: &[&[u8]]) -> Vec<u8> {
    req_ver(1, op, parts)
}
pub fn idb(v: &[u8]) -> Vec<u8> {
    let mut o = Vec::new();
    id(&mut o, v);
    o
}
pub fn blb(v: &[u8]) -> Vec<u8> {
    let mut o = Vec::new();
    blob(&mut o, v);
    o
}
pub fn u64b(v: u64) -> Vec<u8> {
    v.to_le_bytes().to_vec()
}
