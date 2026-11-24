use super::res;

pub fn page(mut versions: Vec<[u8; 32]>, after: &Option<Vec<u8>>, limit: u16) -> Vec<u8> {
    if limit == 0 || limit > 256 || after.as_ref().is_some_and(|v| v.len() != 32) {
        return res::mk_err(1);
    }
    versions.sort_unstable();
    let start = versions.partition_point(|h| after.as_ref().is_some_and(|x| h.as_slice() <= x));
    let selected = &versions[start..versions.len().min(start + usize::from(limit))];
    let mut body = Vec::with_capacity(2 + selected.len() * 32);
    body.extend_from_slice(&(selected.len() as u16).to_le_bytes());
    for hash in selected {
        body.extend_from_slice(hash);
    }
    res::ok(body)
}
