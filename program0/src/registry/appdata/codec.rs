const VER: u8 = 1;

pub fn encode(key: &[u8], value: &[u8]) -> Option<Vec<u8>> {
    let klen = u16::try_from(key.len()).ok()?;
    let mut out = Vec::with_capacity(3 + key.len() + value.len());
    out.push(VER);
    out.extend_from_slice(&klen.to_le_bytes());
    out.extend_from_slice(key);
    out.extend_from_slice(value);
    Some(out)
}

pub fn decode(bytes: &[u8], key: &[u8]) -> Option<Vec<u8>> {
    if bytes.first().copied()? != VER || bytes.len() < 3 {
        return None;
    }
    let klen = u16::from_le_bytes([bytes[1], bytes[2]]) as usize;
    if bytes.len() < 3 + klen || &bytes[3..3 + klen] != key {
        return None;
    }
    Some(bytes[3 + klen..].to_vec())
}
