use super::{BufferedCopyFailure, BulkIo, CopyFailure, ScalarError};

pub(super) fn count(raw: i64, capacity: usize) -> Result<usize, ScalarError> {
    if raw < 0 {
        Err(ScalarError(raw))
    } else if raw as u64 > capacity as u64 {
        Err(ScalarError(-5))
    } else {
        Ok(raw as usize)
    }
}

pub(super) fn write_chunk<I: BulkIo>(
    io: &mut I,
    dst: i32,
    buf: &[u8],
    consumed: u64,
    written: &mut u64,
) -> Result<(), BufferedCopyFailure> {
    let mut at = 0;
    while at < buf.len() {
        let n = io.write(dst, &buf[at..]);
        let len = count(n, buf.len() - at)
            .and_then(|n| if n == 0 { Err(ScalarError(-5)) } else { Ok(n) })
            .map_err(|e| BufferedCopyFailure {
                error: CopyFailure::Write(e),
                consumed,
                written: *written,
            })?;
        *written += len as u64;
        at += len;
    }
    Ok(())
}
