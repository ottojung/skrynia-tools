//! Scripted response plan for the fake endpoint.
pub struct ReplyPlan {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    pub delay_ms: u64,
}

impl ReplyPlan {
    pub fn status(status: u16) -> Self {
        Self {
            status,
            headers: Vec::new(),
            body: b"ok".to_vec(),
            delay_ms: 0,
        }
    }

    pub fn retry(mut self, seconds: u64) -> Self {
        self.headers
            .push(("Retry-After".into(), seconds.to_string()));
        self
    }

    pub fn redirect(location: &str) -> Self {
        let mut out = Self::status(302);
        out.headers.push(("Location".into(), location.into()));
        out
    }

    pub fn slow(delay_ms: u64) -> Self {
        Self {
            delay_ms,
            ..Self::status(201)
        }
    }
}
