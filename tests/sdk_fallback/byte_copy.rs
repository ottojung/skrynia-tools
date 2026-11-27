use skrynia_tools::sdk::{CopyFailure, Progress, ScalarError, copy_bytes};

use super::support::FakeIo;

#[test]
fn byte_copy_reports_eof_and_partial_progress() {
    let mut io = FakeIo {
        input: vec![7, 9],
        ..FakeIo::default()
    };
    assert_eq!(
        copy_bytes(&mut io, 0, 1),
        Ok(Progress {
            bytes: 2,
            eof: true
        })
    );
    assert_eq!(io.written, vec![7, 9]);
    let mut failing = FakeIo {
        input: vec![1, 2, 3],
        fail_write: true,
        ..FakeIo::default()
    };
    let failed = copy_bytes(&mut failing, 0, 1);
    assert_eq!(failed, Err(CopyFailure::Write(ScalarError(-2))));
    assert_eq!(failing.at, 1);
}
