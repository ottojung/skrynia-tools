//! Bounded read of one HTTP request from a TLS stream.
use super::response::content_length;
use std::io;
use tokio::io::{AsyncRead, AsyncReadExt};

const MAX_HEAD: usize = 8 * 1024;
const MAX_BODY: usize = 16 * 1024;

pub async fn request<S>(stream: &mut S) -> io::Result<(String, Vec<u8>)>
where
    S: AsyncRead + Unpin,
{
    let mut data = Vec::new();
    let mut byte = [0u8; 1];
    while data.len() < MAX_HEAD {
        if stream.read(&mut byte).await? == 0 {
            return Err(io::Error::other("empty request"));
        }
        data.push(byte[0]);
        if data.ends_with(b"\r\n\r\n") {
            break;
        }
    }
    let end = data
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or_else(|| io::Error::other("no header terminator"))?;
    let head = String::from_utf8_lossy(&data[..end]).into_owned();
    let expected = content_length(&head);
    let mut body = data[end + 4..].to_vec();
    while body.len() < expected && body.len() < MAX_BODY {
        let mut buf = [0u8; 1024];
        let want = (expected - body.len())
            .min(buf.len())
            .min(MAX_BODY - body.len());
        let read = stream.read(&mut buf[..want]).await?;
        if read == 0 {
            break;
        }
        body.extend_from_slice(&buf[..read]);
    }
    Ok((head, body))
}
