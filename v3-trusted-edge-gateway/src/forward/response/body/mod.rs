mod poll;
mod watch;

use bytes::Bytes;
use futures_util::stream::Stream;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::sync::OwnedSemaphorePermit;
use tokio::time::Sleep;

type In = futures_util::stream::BoxStream<'static, Result<Bytes, reqwest::Error>>;
pub(super) type Permit = Arc<Mutex<Option<OwnedSemaphorePermit>>>;
pub use watch::limited_idle;

pub(super) struct IdleLimited {
    inner: In,
    max: usize,
    seen: usize,
    idle: Duration,
    sleep: Pin<Box<Sleep>>,
    deadline: Pin<Box<Sleep>>,
    done: bool,
    polls: Arc<AtomicUsize>,
    poll_file: Option<PathBuf>,
    _permit: Permit,
}

impl Stream for IdleLimited {
    type Item = Result<Bytes, std::io::Error>;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        poll::next(&mut self, cx)
    }
}
