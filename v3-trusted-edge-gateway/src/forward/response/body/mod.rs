mod poll;

use bytes::Bytes;
use futures_util::stream::{Stream, StreamExt};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::time::{Sleep, sleep};

type In = futures_util::stream::BoxStream<'static, Result<Bytes, reqwest::Error>>;

pub fn limited_idle(
    stream: impl Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static,
    max: usize,
    idle: Duration,
) -> impl Stream<Item = Result<Bytes, std::io::Error>> + Send {
    IdleLimited {
        inner: stream.boxed(),
        max,
        seen: 0,
        idle,
        sleep: Box::pin(sleep(idle)),
        done: false,
    }
}

pub(super) struct IdleLimited {
    inner: In,
    max: usize,
    seen: usize,
    idle: Duration,
    sleep: Pin<Box<Sleep>>,
    done: bool,
}

impl Stream for IdleLimited {
    type Item = Result<Bytes, std::io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = &mut *self;
        poll::next(this, cx)
    }
}
