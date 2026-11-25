//! Hermetic operator-runtime tests: hostile files, locks, loop disposition.
pub mod cfg;
pub mod dir;
pub mod gone;
pub mod lock;
pub mod lock2;
pub mod policy;

pub(super) use super::{
    FakeP1, FixedRng, ManualClock, ScriptedTransport, build, config, frame, resolver, sub_json,
    vapid_key,
};

/// All single-process hermetic checks; two-process tests are separate.
pub fn hermetic() {
    super::claims::config_err_redacted();
    cfg::load_and_hostile();
    dir::dir_checks();
    lock::hostile();
    policy::checks();
    gone::gone_and_success();
    gone::transient_no_ack();
}
