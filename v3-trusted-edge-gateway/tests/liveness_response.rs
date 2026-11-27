mod support;

#[test]
fn response_hold_saturates_then_recovers() {
    support::liveness::response_hold_saturates_then_recovers();
}

#[test]
fn trickle_exceeds_absolute_lifetime() {
    support::liveness::trickle_exceeds_absolute_lifetime();
}

#[test]
fn backpressure_nonreading_downstream() {
    support::liveness::backpressure_nonreading_downstream();
}

#[test]
fn chunk_parser_rejects_interrupted_responses() {
    use support::http::dechunk;
    assert_eq!(dechunk(b"3\r\nabc\r\n0\r\n\r\n").unwrap(), b"abc");
    assert_eq!(
        dechunk(b"1\r\na\r\n0\r\nX-Test: yes\r\n\r\n").unwrap(),
        b"a"
    );
    for bad in [
        &b"3\r\nabc\r\n"[..],
        &b"3\r\nab"[..],
        &b"xyz\r\n"[..],
        &b"1\r\naXX0\r\n\r\n"[..],
        &b"0\r\n"[..],
    ] {
        assert!(dechunk(bad).is_err(), "accepted malformed response");
    }
    assert!(dechunk(b"3\r\nab").unwrap_err().contains("expected 3 bytes"));
    assert!(dechunk(b"3\r\nabc\n").unwrap_err().contains("incomplete chunk"));
}
