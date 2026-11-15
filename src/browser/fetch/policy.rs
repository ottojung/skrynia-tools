use web_sys::{RequestCache, RequestInit, RequestRedirect};

pub fn apply(init: &RequestInit, method: &str) {
    init.set_redirect(RequestRedirect::Error);
    if method == "GET" {
        init.set_cache(RequestCache::NoStore);
    }
}
