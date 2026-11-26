//! Decode scalar machine results. Negative is an error, nonnegative a value.
use super::{ERR_BADF, ERR_BOUNDS, ERR_INVAL, ERR_IO, ERR_RANGE, ERR_UNSUPPORTED, READ1_EOF};

/// Raw negative code (or EOF). Positive/zero values are not errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScalarError(pub i64);

impl ScalarError {
    pub fn is_eof(self) -> bool {
        self.0 == i64::from(READ1_EOF)
    }

    pub fn name(self) -> &'static str {
        match self.0 {
            x if x == i64::from(READ1_EOF) => "EOF",
            x if x == i64::from(ERR_BADF) => "EBADF",
            x if x == i64::from(ERR_INVAL) => "EINVAL",
            x if x == i64::from(ERR_UNSUPPORTED) => "ENOTSUP",
            x if x == i64::from(ERR_IO) => "EIO",
            x if x == i64::from(ERR_RANGE) => "ERANGE",
            x if x == i64::from(ERR_BOUNDS) => "BOUNDS",
            _ => "unknown",
        }
    }
}

/// read1: 0..255 is a byte; -1 is EOF; other negatives are errors.
pub fn read1(raw: i32) -> Result<u8, ScalarError> {
    if (0..=255).contains(&raw) {
        Ok(raw as u8)
    } else {
        Err(ScalarError(i64::from(raw)))
    }
}

/// write1 and status-only imports: 0 is success, any negative is an error.
pub fn write1(raw: i32) -> Result<(), ScalarError> {
    status(raw)
}

pub fn status(raw: i32) -> Result<(), ScalarError> {
    if raw == 0 {
        Ok(())
    } else {
        Err(ScalarError(i64::from(raw)))
    }
}

/// seek: a nonnegative result is the new absolute position.
pub fn seek(raw: i64) -> Result<u64, ScalarError> {
    if raw >= 0 {
        Ok(raw as u64)
    } else {
        Err(ScalarError(raw))
    }
}

/// cas1: the observed byte is returned scalar, including on mismatch.
pub fn cas1(raw: i32) -> Result<u8, ScalarError> {
    read1(raw)
}
