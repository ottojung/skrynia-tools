pub fn identifier(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(is_lower_hex)
}
pub fn sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(is_lower_hex)
}
fn is_lower_hex(c: u8) -> bool {
    c.is_ascii_digit() || (b'a'..=b'f').contains(&c)
}
