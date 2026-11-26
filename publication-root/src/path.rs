const DIR: &[u8] = b"user/publication-requests/";
pub fn mailbox_dir() -> &'static [u8] {
    b"user/publication-requests"
}
const HEX: &[u8; 16] = b"0123456789abcdef";
const PRIVATE: [&[u8]; 6] = [
    b"security",
    b"programs",
    b"publication-requests",
    b"operator",
    b"private",
    b"tokens",
];

pub fn mailbox(ticket: u32) -> Vec<u8> {
    let mut out = Vec::with_capacity(DIR.len() + 8);
    out.extend_from_slice(DIR);
    for shift in (0..8).rev() {
        out.push(HEX[((ticket >> (shift * 4)) & 0xf) as usize]);
    }
    out
}

/// Generic public-path grammar. App/version authority belongs to the caller,
/// never to this root program. Private roots are never publishable.
pub fn valid(path: &[u8]) -> bool {
    if path.is_empty()
        || path.len() > 4096
        || path[0] == b'/'
        || path.ends_with(b"/")
        || path.contains(&0)
        || path.contains(&b'\\')
    {
        return false;
    }
    let Ok(text) = core::str::from_utf8(path) else {
        return false;
    };
    if text.contains(['\n', '\r']) {
        return false;
    }
    let mut first = true;
    for part in text.split('/') {
        if part.is_empty() || part == "." || part == ".." {
            return false;
        }
        if first && PRIVATE.contains(&part.as_bytes()) {
            return false;
        }
        first = false;
    }
    true
}
