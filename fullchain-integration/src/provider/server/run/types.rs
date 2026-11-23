//! Captured request observation and configurable reply plan.
#[derive(Debug, Clone, Default)]
pub struct Seen {
    pub method: String,
    pub target: String,
    pub host: String,
    pub sni: Option<String>,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}
impl Seen {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }
}
#[derive(Clone)]
pub struct ReplyPlan {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}
impl ReplyPlan {
    pub fn status(status: u16) -> Self {
        Self {
            status,
            headers: Vec::new(),
            body: b"ok".to_vec(),
        }
    }
    pub fn retry(mut self, secs: u64) -> Self {
        self.headers.push(("Retry-After".into(), secs.to_string()));
        self
    }
}
