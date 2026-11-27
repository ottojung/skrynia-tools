//! Partial-progress semantics of the fixed bulk transfer helpers.
use super::transfer::{read_all, write_all};

#[test]
fn read_repeats_partial_transfers_and_stops_at_eof() {
    let counts = [2i64, 3, 0];
    let mut calls = counts.into_iter();
    let mut dst = [0u8; 8];
    let got = read_all(
        |ptr, len| {
            let at = 8 - len;
            let n = calls.next().unwrap_or(0);
            let bytes = unsafe { core::slice::from_raw_parts_mut(ptr, n as usize) };
            bytes.fill(7);
            assert_eq!(at, 8 - len);
            n.min(len as i64)
        },
        &mut dst,
    )
    .unwrap();
    assert_eq!(got, 5);
    assert_eq!(&dst[..5], &[7, 7, 7, 7, 7]);
}

#[test]
fn read_negative_before_progress_is_an_error() {
    let err = read_all(|_, _| -2, &mut [0u8; 4]).unwrap_err();
    assert_eq!(err, -2);
}

#[test]
fn read_negative_after_progress_returns_the_count() {
    let counts = [2i64, -5];
    let mut calls = counts.into_iter();
    let got = read_all(|_, _| calls.next().unwrap_or(0), &mut [0u8; 4]).unwrap();
    assert_eq!(got, 2);
}

#[test]
fn write_repeats_partial_transfers() {
    let counts = [2i64, 2, 1];
    let mut calls = counts.into_iter();
    write_all(|_, len| calls.next().unwrap_or(0).min(len as i64), b"hello").unwrap();
}

#[test]
fn write_zero_progress_and_oversized_count_fail_closed() {
    assert_eq!(write_all(|_, _| 0, b"abc").unwrap_err(), -5);
    assert_eq!(write_all(|_, _| 99, b"abc").unwrap_err(), -5);
    assert_eq!(write_all(|_, _| -5, b"abc").unwrap_err(), -5);
}
