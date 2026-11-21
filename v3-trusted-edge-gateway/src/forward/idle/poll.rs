use bytes::Bytes;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::time::Instant;

pub fn poll<B>(
    this: &mut super::IdleBody<B>,
    cx: &mut Context<'_>,
) -> Poll<Option<Result<http_body::Frame<Bytes>, axum::Error>>>
where
    B: http_body::Body<Data = Bytes, Error = axum::Error> + Unpin,
{
    if this.done {
        return Poll::Ready(None);
    }
    if this.sleep.as_mut().poll(cx).is_ready() {
        this.done = true;
        return Poll::Ready(Some(Err(axum::Error::new("upload idle timeout"))));
    }
    match Pin::new(&mut this.inner).poll_frame(cx) {
        Poll::Ready(Some(Ok(f))) => {
            this.sleep.as_mut().reset(Instant::now() + this.idle);
            Poll::Ready(Some(Ok(f)))
        }
        Poll::Ready(Some(Err(e))) => {
            this.done = true;
            Poll::Ready(Some(Err(e)))
        }
        Poll::Ready(None) => {
            this.done = true;
            Poll::Ready(None)
        }
        Poll::Pending => Poll::Pending,
    }
}
