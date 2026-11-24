const DIR: &[u8] = b"publication-requests/";
const HEX: &[u8; 16] = b"0123456789abcdef";
const PRIVATE: [&[u8]; 6] = [
    b"security",
    b"programs",
    b"publication-requests",
    b"operator",
    b"private",
    b"tokens",
];

pub fn ticket(t: u32) -> Vec<u8> {
    let mut p = Vec::with_capacity(DIR.len() + 8);
    p.extend_from_slice(DIR);
    for shift in (0..8).rev() {
        p.push(HEX[((t >> (shift * 4)) & 0xf) as usize]);
    }
    p
}

/// Public paths only: bounded relative UTF-8, no traversal, no private root.
pub fn is_public(path: &[u8]) -> bool {
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
    for segment in text.split('/') {
        if segment.is_empty() || segment == "." || segment == ".." {
            return false;
        }
        if first && PRIVATE.contains(&segment.as_bytes()) {
            return false;
        }
        first = false;
    }
    path.starts_with(b"apps/")
}
