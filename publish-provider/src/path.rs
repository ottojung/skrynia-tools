//! Kernel `clean_relative` grammar plus the mailbox contract: UTF-8,
//! non-empty, no NUL/backslash/newline, no `//`, no trailing `/`, no `..`,
//! at most 4096 bytes, and never the private mailbox directory itself.

pub const MAX: usize = 4096;

/// Path bytes outside the kernel grammar.
#[derive(Debug, PartialEq, Eq)]
pub struct InvalidPath;

/// Returns the accepted normal components, dropping `.` segments.
pub fn clean(bytes: &[u8]) -> Result<Vec<String>, InvalidPath> {
    if bytes.is_empty() || bytes.len() > MAX {
        return Err(InvalidPath);
    }
    if bytes
        .iter()
        .any(|byte| matches!(byte, 0 | b'\\' | b'\n' | b'\r'))
    {
        return Err(InvalidPath);
    }
    let text = std::str::from_utf8(bytes).map_err(|_| InvalidPath)?;
    if text.starts_with('/') || text.contains("//") || text.ends_with('/') {
        return Err(InvalidPath);
    }
    let mut out = Vec::new();
    for part in text.split('/') {
        match part {
            "." => {}
            ".." | "" => return Err(InvalidPath),
            normal => out.push(normal.to_owned()),
        }
    }
    if out.is_empty() || out[0] == "publication-requests" {
        return Err(InvalidPath);
    }
    let joined = out.iter().map(String::len).sum::<usize>() + out.len() - 1;
    if joined > MAX {
        return Err(InvalidPath);
    }
    Ok(out)
}
