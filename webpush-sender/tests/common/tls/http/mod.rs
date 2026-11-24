//! Minimal HTTP/1.1 exchange over an established TLS stream.
mod parse;
mod read;
mod response;

pub use super::Seen;
pub use super::config::ReplyPlan;
use std::io;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite};

pub async fn exchange<S>(
    mut stream: S,
    plan: &ReplyPlan,
    out: &mut Vec<Seen>,
    sni: Option<String>,
) -> io::Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let (head, body) = read::request(&mut stream).await?;
    if plan.delay_ms > 0 {
        tokio::time::sleep(Duration::from_millis(plan.delay_ms)).await;
    }
    response::write(&mut stream, plan).await?;
    out.push(parse::parse(&head, body, sni));
    Ok(())
}
