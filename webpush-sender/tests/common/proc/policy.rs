//! Fatal/transient disposition and terminal Gone handling.
use super::{
    FakeP1, FixedRng, ManualClock, ScriptedTransport, build, config, frame, resolver, sub_json,
    vapid_key,
};
use webpush_sender::Error;
use webpush_sender::push::proc::fatal;

pub fn checks() {
    use webpush_sender::p1::P1Failure;
    assert!(fatal(&Error::Config("x")));
    assert!(fatal(&Error::Auth("edge")));
    assert!(fatal(&Error::Auth("push")));
    assert_eq!(P1Failure::from_status(401), P1Failure::Auth);
    assert_eq!(P1Failure::from_status(403), P1Failure::Auth);
    assert_eq!(P1Failure::from_status(503), P1Failure::Io);
    assert_eq!(P1Failure::from_status(400), P1Failure::Reject);
    assert!(fatal(&Error::Config("x")));
    assert!(fatal(&Error::Quota));
    assert!(fatal(&Error::P1(9)));
    assert!(fatal(&Error::Transport("status")));
    assert!(fatal(&Error::Transport("ledger-dir-sync")));
    assert!(!fatal(&Error::P1(11)), "routine Busy must not be fatal");
    assert!(!fatal(&Error::Exhausted));
    assert!(!fatal(&Error::Transport("gone")));
}
