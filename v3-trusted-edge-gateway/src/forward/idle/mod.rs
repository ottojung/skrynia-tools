mod poll;

use bytes::Bytes;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::time::{Sleep, sleep};

pub struct IdleBody<B> {
    inner: B,
    idle: Duration,
    sleep: Pin<Box<Sleep>>,
    done: bool,
}

impl<B> IdleBody<B> {
    pub fn new(inner: B, idle: Duration) -> Self {
        Self {
            inner,
            idle,
            sleep: Box::pin(sleep(idle)),
            done: false,
        }
    }
}

impl<B> http_body::Body for IdleBody<B>
where
    B: http_body::Body<Data = Bytes, Error = axum::Error> + Unpin,
{
    type Data = Bytes;
    type Error = axum::Error;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        let this = &mut *self;
        poll::poll(this, cx)
    }

    fn is_end_stream(&self) -> bool {
        self.done || self.inner.is_end_stream()
    }
}
