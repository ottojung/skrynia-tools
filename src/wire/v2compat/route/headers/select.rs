use super::super::{super::types::Request, range};
pub struct Sel {
    pub status: u16,
    pub start: u64,
    pub end: u64,
}
pub fn selection(req: &Request, tag: &str, size: u64) -> Sel {
    if fresh(req, tag) {
        return Sel {
            status: 304,
            start: 0,
            end: 0,
        };
    }
    if let Some(rh) = header(req, "range") {
        if header(req, "if-range").is_none_or(|v| v == tag) {
            return match range::parse(rh, size) {
                Ok((s, e)) => Sel {
                    status: 206,
                    start: s,
                    end: e,
                },
                Err(range::Error::Unsatisfiable) => Sel {
                    status: 416,
                    start: 0,
                    end: 0,
                },
                Err(range::Error::Ignore) => full(size),
            };
        }
    }
    full(size)
}
fn full(size: u64) -> Sel {
    Sel {
        status: 200,
        start: 0,
        end: size.saturating_sub(1),
    }
}
fn header<'a>(req: &'a Request, name: &str) -> Option<&'a str> {
    req.headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.as_str())
}
fn fresh(req: &Request, tag: &str) -> bool {
    header(req, "if-none-match").is_some_and(|v| {
        v.split(',')
            .map(str::trim)
            .any(|x| x == "*" || x == tag || x == format!("W/{tag}"))
    })
}
