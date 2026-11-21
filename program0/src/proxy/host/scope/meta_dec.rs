use super::{meta::Policy, path};

fn hex(c: u8) -> u8 {
    if c < 10 { b'0' + c } else { b'a' + c - 10 }
}
pub fn meta_path(program: &[u8]) -> Option<Vec<u8>> {
    let mut out = b"security/program/".to_vec();
    for &b in program {
        match b {
            b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'-' | b'_' | b'.' => out.push(b),
            b'/' => out.extend_from_slice(b"%2F"),
            _ => {
                out.push(b'%');
                out.push(hex(b >> 4));
                out.push(hex(b & 15));
            }
        }
    }
    Some(out)
}
pub fn parse(buf: &[u8]) -> Option<Policy> {
    if buf.len() < 8 {
        return None;
    }
    let ops = u32::from_le_bytes(buf[0..4].try_into().ok()?);
    let plen = u32::from_le_bytes(buf[4..8].try_into().ok()?) as usize;
    if plen == 0 || plen > 192 || 8 + plen > buf.len() {
        return None;
    }
    if !path::clean_prefix(&buf[8..8 + plen]) {
        return None;
    }
    let mut prefix = [0u8; 192];
    prefix[..plen].copy_from_slice(&buf[8..8 + plen]);
    Some(Policy {
        ops,
        prefix_len: plen as u16,
        prefix,
    })
}
