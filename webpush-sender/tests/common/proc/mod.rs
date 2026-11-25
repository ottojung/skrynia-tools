//! Hermetic operator-runtime tests: hostile files, locks, loop disposition.
pub mod auth;
pub mod auth_pri;
pub mod auth_pri2;
pub mod cfg;
pub mod dir;
pub mod gone;
pub mod lock;
pub mod lock2;
pub mod policy;

pub(super) use super::{
    FakeP1, FixedRng, ManualClock, ScriptedTransport, build, config, frame, res, resolver,
    sub_json, vapid_key,
};

/// All single-process hermetic checks; two-process tests are separate.
pub fn hermetic() {
    super::claims::config_err_redacted();
    cfg::load_and_hostile();
    dir::dir_checks();
    lock::hostile();
    policy::checks();
    gone::gone_and_success();
    auth::auth_rejected_once();
    auth::auth_stops_before_later_seq();
    auth_pri::auth_not_masked_by_later_transient_sub();
    auth_pri::auth_outweighs_prior_ack_failure_across_seqs();
    auth_pri2::busy_does_not_mask_later_auth();
    gone::transient_no_ack();
}
