//! Target error classification shared by the compiled guest and host tests.
//!
//! Only the kernel's explicit ENOENT means absence. EBADF, EIO, ENOTSUP and
//! unsafe-path failures stay hard errors, so a broken descriptor or a failed
//! operation is never silently reinterpreted as a missing object.
pub const E_NOENT: i32 = -8;

pub fn is_missing(code: i32) -> bool {
    code == E_NOENT
}

#[cfg(test)]
mod tests {
    use super::is_missing;

    #[test]
    fn only_explicit_enoent_means_absence() {
        assert!(is_missing(-8));
        for failure in [0, -1, -2, -3, -4, -5, -6, -7, 1, 255] {
            assert!(!is_missing(failure), "{failure} must not mean missing");
        }
    }
}
