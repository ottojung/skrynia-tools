//! Bounded request-head/body reader for the disposable local HTTPS provider.
use std::io;
use tokio::io::{AsyncRead, AsyncReadExt};

pub async fn read_req<S: AsyncRead + Unpin>(s: &mut S) -> io::Result<(String, Vec<u8>)> {
    let mut data = Vec::new();
    let mut b = [0u8; 1];
    while data.len() < 8192 {
        if s.read(&mut b).await? == 0 {
            return Err(io::Error::other("empty"));
        }
        data.push(b[0]);
        if data.ends_with(b"\r\n\r\n") {
            break;
        }
    }
    let end = data
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or_else(|| io::Error::other("no term"))?;
    let head = String::from_utf8_lossy(&data[..end]).into_owned();
    let want = content_len(&head);
    let mut body = data[end + 4..].to_vec();
    while body.len() < want && body.len() < 16384 {
        let mut buf = [0u8; 1024];
        let n = (want - body.len()).min(buf.len());
        let got = s.read(&mut buf[..n]).await?;
        if got == 0 {
            break;
        }
        body.extend_from_slice(&buf[..got]);
    }
    Ok((head, body))
}
fn content_len(head: &str) -> usize {
    head.lines()
        .find_map(|l| {
            let (n, v) = l.split_once(':')?;
            n.trim()
                .eq_ignore_ascii_case("content-length")
                .then(|| v.trim().parse().unwrap_or(0))
        })
        .unwrap_or(0)
}
