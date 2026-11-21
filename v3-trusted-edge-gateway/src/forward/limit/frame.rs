use bytes::Bytes;
use std::task::Poll;

pub fn handle(
    polled: Poll<Option<Result<http_body::Frame<Bytes>, axum::Error>>>,
    remaining: &mut usize,
    overflowed: &mut bool,
) -> Poll<Option<Result<http_body::Frame<Bytes>, axum::Error>>> {
    match polled {
        Poll::Ready(Some(Ok(frame))) => {
            if let Some(data) = frame.data_ref() {
                if data.len() > *remaining {
                    *overflowed = true;
                    return Poll::Ready(Some(Err(axum::Error::new("body too large"))));
                }
                *remaining -= data.len();
            }
            Poll::Ready(Some(Ok(frame)))
        }
        other => other,
    }
}
