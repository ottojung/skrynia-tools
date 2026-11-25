//! Fatal/transient disposition and terminal Gone handling.
use super::{
    FakeP1, FixedRng, ManualClock, ScriptedTransport, build, config, frame, resolver, sub_json,
    vapid_key,
};
use webpush_sender::Error;
use webpush_sender::push::proc::fatal;

pub fn checks() {
    assert!(fatal(&Error::Config("x")));
    assert!(fatal(&Error::Quota));
    assert!(fatal(&Error::P1(9)));
    assert!(fatal(&Error::Transport("status")));
    assert!(fatal(&Error::Transport("ledger-dir-sync")));
    assert!(!fatal(&Error::P1(11)), "routine Busy must not be fatal");
    assert!(!fatal(&Error::Exhausted));
    assert!(!fatal(&Error::Transport("gone")));
}
