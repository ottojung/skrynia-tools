use super::meta::Policy;

pub fn map_path(p: &Policy, guest: &[u8]) -> Option<Vec<u8>> {
    if guest.contains(&0) || guest.contains(&b'\\') {
        return None;
    }
    let clean = clean_rel(guest)?;
    let base = p.prefix();
    let mut out = Vec::with_capacity(base.len() + 1 + clean.len());
    out.extend_from_slice(base);
    if !clean.is_empty() {
        if out.last().copied() != Some(b'/') {
            out.push(b'/');
        }
        out.extend_from_slice(clean);
    }
    Some(out)
}
fn clean_rel(path: &[u8]) -> Option<&[u8]> {
    if path.is_empty() {
        return Some(path);
    }
    let s = if path[0] == b'/' { &path[1..] } else { path };
    for part in s.split(|&b| b == b'/') {
        if part.is_empty() || part == b"." || part == b".." {
            return None;
        }
    }
    Some(s)
}
pub fn clean_prefix(p: &[u8]) -> bool {
    !p.is_empty()
        && !p.contains(&0)
        && !p.contains(&b'\\')
        && p[0] != b'/'
        && !p
            .split(|&b| b == b'/')
            .any(|x| x.is_empty() || x == b"." || x == b"..")
}
