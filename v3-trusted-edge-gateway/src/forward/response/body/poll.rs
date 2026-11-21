use bytes::Bytes;
use std::sync::atomic::Ordering;
use std::task::{Context, Poll};
use tokio::time::Instant;

#[rustfmt::skip]
pub fn next(this: &mut super::IdleLimited, cx: &mut Context<'_>) -> Poll<Option<Result<Bytes, std::io::Error>>> {
    let n = this.polls.fetch_add(1, Ordering::Relaxed) + 1;
    if let Some(p) = &this.poll_file { let _ = std::fs::write(p, n.to_string()); }
    if this.done { return Poll::Ready(None); }
    if this.deadline.as_mut().poll(cx).is_ready() { return err(this, "response lifetime exceeded"); }
    if this.sleep.as_mut().poll(cx).is_ready() { return err(this, "upstream idle timeout"); }
    match this.inner.as_mut().poll_next(cx) {
        Poll::Ready(Some(Ok(b))) => chunk(this, b),
        Poll::Ready(Some(Err(_))) => err(this, "upstream error"),
        Poll::Ready(None) => { this.done = true; Poll::Ready(None) }
        Poll::Pending => Poll::Pending,
    }
}
#[rustfmt::skip]
fn err(this: &mut super::IdleLimited, msg: &'static str) -> Poll<Option<Result<Bytes, std::io::Error>>> {
    this.done = true; Poll::Ready(Some(Err(std::io::Error::other(msg))))
}
#[rustfmt::skip]
fn chunk(this: &mut super::IdleLimited, bytes: Bytes) -> Poll<Option<Result<Bytes, std::io::Error>>> {
    match this.seen.checked_add(bytes.len()) {
        Some(v) if v <= this.max => { this.seen = v; this.sleep.as_mut().reset(Instant::now() + this.idle); Poll::Ready(Some(Ok(bytes))) }
        _ => err(this, "response too large"),
    }
}
