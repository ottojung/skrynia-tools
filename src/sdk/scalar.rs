//! Decode scalar machine results. Negative is an error, nonnegative a value.
use super::READ1_EOF;

/// Raw negative code (or EOF). Positive/zero values are not errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScalarError(pub i64);

impl ScalarError {
    pub fn is_eof(self) -> bool {
        self.0 == i64::from(READ1_EOF)
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

/// read/write: nonnegative is bytes transferred (0 at EOF is success);
/// negative is an error before progress.
pub fn bulk(raw: i64) -> Result<u64, ScalarError> {
    if raw < 0 {
        Err(ScalarError(raw))
    } else {
        Ok(raw as u64)
    }
}
