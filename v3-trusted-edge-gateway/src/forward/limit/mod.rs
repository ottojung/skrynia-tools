mod frame;

use axum::body::Body;
use bytes::Bytes;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct LimitedBody {
    inner: Body,
    remaining: usize,
    overflowed: bool,
    done: bool,
}

impl LimitedBody {
    pub fn new(inner: Body, max: usize) -> Self {
        Self {
            inner,
            remaining: max,
            overflowed: false,
            done: false,
        }
    }
}

impl http_body::Body for LimitedBody {
    type Data = Bytes;
    type Error = axum::Error;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        if self.done {
            return Poll::Ready(None);
        }
        if self.overflowed {
            self.done = true;
            return Poll::Ready(Some(Err(axum::Error::new("body too large"))));
        }
        let this = &mut *self;
        let polled = Pin::new(&mut this.inner).poll_frame(cx);
        frame::handle(polled, &mut this.remaining, &mut this.overflowed)
    }

    fn is_end_stream(&self) -> bool {
        self.done || self.inner.is_end_stream()
    }
}
