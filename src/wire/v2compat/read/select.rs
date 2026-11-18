use super::{cache, range};
use std::collections::BTreeMap;
pub struct Selection {
    pub status: u16,
    pub start: usize,
    pub end: usize,
}
impl Selection {
    fn new(status: u16, start: usize, end: usize) -> Self {
        Self { status, start, end }
    }
}
pub fn choose(headers: &BTreeMap<String, String>, tag: &str, size: usize) -> Selection {
    let full = || Selection::new(200, 0, size.saturating_sub(1));
    if cache::fresh(headers, tag) {
        return Selection::new(304, 0, 0);
    }
    if let Some(range) = cache::header(headers, "range")
        && cache::header(headers, "if-range").is_none_or(|v| v.as_bytes() == tag.as_bytes())
    {
        return match range::parse(range, size as u64) {
            Ok((start, end)) => Selection::new(206, start as usize, end as usize),
            Err(range::Error::Ignore) => full(),
            Err(range::Error::Unsatisfiable) => Selection::new(416, 0, 0),
        };
    }
    full()
}
