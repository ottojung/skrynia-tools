use super::{IdleLimited, Permit};
use futures_util::StreamExt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::OwnedSemaphorePermit;
use tokio::time::{Instant, sleep, sleep_until};

pub fn spawn(permit: Permit, deadline: Instant) {
    tokio::spawn(async move {
        tokio::time::sleep_until(deadline).await;
        if let Ok(mut g) = permit.lock() {
            let _ = g.take();
        }
    });
}
pub fn limited_idle(
    stream: impl futures_util::Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send + 'static,
    max: usize,
    idle: Duration,
    permit: OwnedSemaphorePermit,
    deadline: Instant,
    polls: Arc<AtomicUsize>,
    poll_file: Option<PathBuf>,
) -> impl futures_util::Stream<Item = Result<bytes::Bytes, std::io::Error>> + Send {
    let permit = Arc::new(Mutex::new(Some(permit)));
    spawn(permit.clone(), deadline);
    IdleLimited {
        inner: stream.boxed(),
        max,
        seen: 0,
        idle,
        sleep: Box::pin(sleep(idle)),
        deadline: Box::pin(sleep_until(deadline)),
        done: false,
        polls,
        poll_file,
        _permit: permit,
    }
}
impl Drop for super::IdleLimited {
    fn drop(&mut self) {
        if let Ok(mut g) = self._permit.lock() {
            g.take();
        }
        if let Some(path) = &self.poll_file {
            let _ = std::fs::write(path, self.polls.load(Ordering::Relaxed).to_string());
        }
    }
}
