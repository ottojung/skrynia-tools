//! Outcome (3) acceptance: exact-key wake fan-out on ordinary P1 at the RED pins.
mod common;

#[test]
fn two_key_red_regression_wakes_designated_key() {
    common::issue5z::two_key_red();
}

#[test]
fn both_traveller_keys_and_113_profile_threads() {
    common::issue5z_prims::both_keys();
}

#[test]
fn restart_and_stale_ack_aba() {
    common::issue5z_prims::restart_aba();
}

#[test]
fn ambiguous_duplicate_ack_is_noop() {
    common::issue5z_prims::ambiguous();
}
