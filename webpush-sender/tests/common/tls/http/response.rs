//! Response serialization and header lookup for the fake endpoint.
use super::ReplyPlan;
use std::io;
use tokio::io::{AsyncWrite, AsyncWriteExt};

pub fn content_length(head: &str) -> usize {
    head.lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.trim()
                .eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse().unwrap_or(0))
        })
        .unwrap_or(0)
}

pub async fn write<S>(stream: &mut S, plan: &ReplyPlan) -> io::Result<()>
where
    S: AsyncWrite + Unpin,
{
    let mut out = format!(
        "HTTP/1.1 {} X\r\nContent-Length: {}\r\nConnection: close\r\n",
        plan.status,
        plan.body.len()
    )
    .into_bytes();
    for (name, value) in &plan.headers {
        out.extend_from_slice(format!("{name}: {value}\r\n").as_bytes());
    }
    out.extend_from_slice(b"\r\n");
    out.extend_from_slice(&plan.body);
    stream.write_all(&out).await?;
    stream.flush().await
}
