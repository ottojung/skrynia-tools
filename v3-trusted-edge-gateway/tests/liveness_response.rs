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
