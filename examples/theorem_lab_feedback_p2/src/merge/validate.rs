use super::Entry;
use crate::{MAX_CHAIN, MAX_FRAGMENT, MAX_FRAGMENTS, MAX_ID, MAX_TEXT};
use alloc::collections::BTreeMap;
use alloc::string::String;

fn safe_id(s: &str) -> bool {
    let ok = !s.is_empty()
        && s.len() <= MAX_ID
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_:.@".contains(&b));
    ok
}
fn safe_ref(s: &str) -> bool {
    s.starts_with("sha256:") && s.len() == 71 && s[7..].bytes().all(|b| b.is_ascii_hexdigit())
}
pub fn validate_entry(e: &Entry, frags: &BTreeMap<String, String>) -> Result<(), &'static str> {
    if !safe_id(&e.id) {
        return Err("bad_id");
    }
    if e.comment.is_empty() || e.comment.len() > MAX_TEXT {
        return Err("bad_comment");
    }
    if e.page_url.len() > 2048 || e.html_chain.len() > MAX_CHAIN {
        return Err("bad_meta");
    }
    if !e.html_chain.iter().all(|r| safe_ref(r)) {
        return Err("bad_ref");
    }
    if frags.len() > MAX_FRAGMENTS {
        return Err("too_many_fragments");
    }
    for (k, v) in frags {
        if !safe_ref(k) || v.len() > MAX_FRAGMENT {
            return Err("bad_fragment");
        }
    }
    let _ = String::new();
    Ok(())
}
