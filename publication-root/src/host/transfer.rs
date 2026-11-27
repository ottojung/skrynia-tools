//! Kernel `read`/`write` progress handling, independent of the host target.
//!
//! ABI 3.1 returns the actual byte count: `0` is EOF/no progress, a short
//! positive count is valid, and a negative error before progress is an error.
//! A negative result after progress is a successful short transfer.

pub fn read_all<F>(mut read: F, dst: &mut [u8]) -> Result<usize, i32>
where
    F: FnMut(*mut u8, usize) -> i64,
{
    let mut at = 0;
    while at < dst.len() {
        let count = read(unsafe { dst.as_mut_ptr().add(at) }, dst.len() - at);
        if count < 0 {
            return if at == 0 { Err(count as i32) } else { Ok(at) };
        }
        if count == 0 {
            break;
        }
        let accepted = usize::try_from(count).map_err(|_| -6)?;
        if accepted > dst.len() - at {
            return Err(-5);
        }
        at += accepted;
    }
    Ok(at)
}

pub fn write_all<F>(mut write: F, src: &[u8]) -> Result<(), i32>
where
    F: FnMut(*const u8, usize) -> i64,
{
    let mut at = 0;
    while at < src.len() {
        let count = write(unsafe { src.as_ptr().add(at) }, src.len() - at);
        if count < 0 {
            // No replay: the caller removes only the file it just created.
            return Err(count as i32);
        }
        let accepted = usize::try_from(count).map_err(|_| -6)?;
        if accepted == 0 || accepted > src.len() - at {
            return Err(-5);
        }
        at += accepted;
    }
    Ok(())
}
