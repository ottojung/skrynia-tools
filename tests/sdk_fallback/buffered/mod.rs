use skrynia_tools::sdk::{BulkIo, CopyFailure, Progress, ScalarError, copy_buffered};
mod support;
use support::io;
#[test]
fn bounded_copy_repeats_partial_reads_and_writes() {
    let mut io = io(99);
    assert_eq!(
        copy_buffered(&mut io, 1, 2, 5, &mut [0; 4]).unwrap(),
        Progress {
            bytes: 5,
            eof: false
        }
    );
    assert_eq!(io.out, b"abcde");
    assert_eq!(
        copy_buffered(&mut io, 1, 2, 5, &mut [0; 4]).unwrap(),
        Progress {
            bytes: 1,
            eof: true
        }
    );
    assert_eq!(io.out, b"abcdef");
}
#[test]
fn failures_report_read_ahead_without_replay() {
    let mut io = io(2);
    let e = copy_buffered(&mut io, 1, 2, 6, &mut [0; 4]).unwrap_err();
    assert_eq!(
        (e.error, e.consumed, e.written),
        (CopyFailure::Write(ScalarError(-5)), 3, 2)
    );
    assert_eq!(io.out, b"ab");
    assert_eq!(io.at, 3);
    assert!(copy_buffered(&mut io, 1, 1, 6, &mut [0; 4]).is_err());
    assert!(copy_buffered(&mut io, 1, 2, 6, &mut []).is_err());
    assert_eq!(io.at, 3);
}
