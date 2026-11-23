//! Bounded HTTP/1.1 response writer and Content-Length parser.
use std::io;
use tokio::io::{AsyncWrite, AsyncWriteExt};

pub(super) fn content_len(head: &str) -> usize {
    head.lines()
        .find_map(|l| {
            let (n, v) = l.split_once(':')?;
            n.trim()
                .eq_ignore_ascii_case("content-length")
                .then(|| v.trim().parse().unwrap_or(0))
        })
        .unwrap_or(0)
}
pub async fn write_reply<S: AsyncWrite + Unpin>(
    s: &mut S,
    status: u16,
    hdrs: &[(String, String)],
    body: &[u8],
) -> io::Result<()> {
    let mut out = format!(
        "HTTP/1.1 {status} X\r\nContent-Length: {}\r\nConnection: close\r\n",
        body.len()
    )
    .into_bytes();
    for (n, v) in hdrs {
        out.extend_from_slice(format!("{n}: {v}\r\n").as_bytes());
    }
    out.extend_from_slice(b"\r\n");
    out.extend_from_slice(body);
    s.write_all(&out).await?;
    s.flush().await
}
