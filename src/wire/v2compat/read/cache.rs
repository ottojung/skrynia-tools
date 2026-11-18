use std::collections::BTreeMap;
pub fn header<'a>(headers: &'a BTreeMap<String, String>, name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.as_str())
}
pub fn fresh(headers: &BTreeMap<String, String>, tag: &str) -> bool {
    header(headers, "if-none-match").is_some_and(|s| {
        s.split(',')
            .map(str::trim)
            .any(|v| v == "*" || v == tag || v == format!("W/{tag}"))
    })
}
