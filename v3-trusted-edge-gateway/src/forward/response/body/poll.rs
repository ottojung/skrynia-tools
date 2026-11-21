use bytes::Bytes;
use std::task::{Context, Poll};
use tokio::time::Instant;

pub fn next(
    this: &mut super::IdleLimited,
    cx: &mut Context<'_>,
) -> Poll<Option<Result<Bytes, std::io::Error>>> {
    if this.done {
        return Poll::Ready(None);
    }
    if this.sleep.as_mut().poll(cx).is_ready() {
        this.done = true;
        return Poll::Ready(Some(Err(std::io::Error::other("upstream idle timeout"))));
    }
    match this.inner.as_mut().poll_next(cx) {
        Poll::Ready(Some(Ok(bytes))) => match this.seen.checked_add(bytes.len()) {
            Some(v) if v <= this.max => {
                this.seen = v;
                this.sleep.as_mut().reset(Instant::now() + this.idle);
                Poll::Ready(Some(Ok(bytes)))
            }
            _ => {
                this.done = true;
                Poll::Ready(Some(Err(std::io::Error::other("response too large"))))
            }
        },
        Poll::Ready(Some(Err(_))) => {
            this.done = true;
            Poll::Ready(Some(Err(std::io::Error::other("upstream error"))))
        }
        Poll::Ready(None) => {
            this.done = true;
            Poll::Ready(None)
        }
        Poll::Pending => Poll::Pending,
    }
}
