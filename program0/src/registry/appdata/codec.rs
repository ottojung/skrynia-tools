// Object storage format: raw value bytes (v1). CAS hash is SHA-256(value).
pub fn encode_value(value: &[u8]) -> Vec<u8> {
    value.to_vec()
}

pub fn decode_value(bytes: &[u8]) -> Option<Vec<u8>> {
    Some(bytes.to_vec())
}
