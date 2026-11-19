use std::{fs::File, io::Read, path::Path};

const MAX_DIAGNOSTIC_BYTES: usize = 4096;

pub(super) fn read_bounded(path: &Path) -> String {
    let Ok(mut file) = File::open(path) else {
        return String::from("<stderr log unavailable>");
    };
    let mut buf = Vec::new();
    if file.read_to_end(&mut buf).is_err() {
        return String::from("<stderr log unreadable>");
    }
    if buf.len() > MAX_DIAGNOSTIC_BYTES {
        let start = buf.len() - MAX_DIAGNOSTIC_BYTES;
        let mut out = String::from("...[truncated]...\n");
        out.push_str(&String::from_utf8_lossy(&buf[start..]));
        return redact(&out);
    }
    redact(&String::from_utf8_lossy(&buf))
}

fn redact(text: &str) -> String {
    // Keep diagnostics useful but bounded; avoid dumping huge or obviously sensitive blobs.
    // This is a conservative best-effort redaction for common token-like env assignments.
    let mut out = String::with_capacity(text.len());
    for line in text.lines() {
        let lower = line.to_ascii_lowercase();
        if lower.contains("token") || lower.contains("secret") || lower.contains("authorization") {
            out.push_str("<redacted-diagnostic-line>");
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    out
}
