//! Callback frame codec and P0-mediated eval fail-closed behaviour.
use super::fake::Fake;
use super::{eval, ops, relay};

#[test]
fn frame_deframe_and_truncation() {
    let framed = relay::frame(b"appdata-request");
    assert_eq!(
        relay::deframe(&framed).expect("deframe"),
        vec![b"appdata-request".to_vec()]
    );
    assert_eq!(relay::deframe(&framed[..6]), Err(ops::ERR_INVAL));
}

#[test]
fn cb_reply_uses_synchronous_in_process_frames() {
    let fake = Fake::new(vec![]);
    relay::cb_reply(&fake, ops::CTX_NESTED, b"callback-answer").expect("reply");
    assert_eq!(
        relay::deframe(&fake.cb_out.borrow()),
        Ok(vec![b"callback-answer".to_vec()])
    );
}

#[test]
fn pending_callback_fails_closed_in_p0_mediated_eval() {
    let mut fake = Fake::new(vec![0, 0, 0, 0, 9]);
    fake.active = true;
    for byte in relay::frame(b"AppGet") {
        fake.cb_in.borrow_mut().push_back(byte as i32);
    }
    assert_eq!(
        eval::call(&fake, ops::CTX_NESTED, &[15, 0], 16),
        Err(ops::ERR_NOTSUP)
    );
}

#[test]
fn trapped_status_is_reported_not_pretended_success() {
    struct Trap;
    impl super::Transport for Trap {
        fn invoke(&self, _ctx: i32, op: i32, _value: i32) -> i64 {
            if op == ops::STATUS { ops::TRAPPED } else { 0 }
        }
    }
    assert_eq!(eval::call(&Trap, 0, &[15], 16), Err(ops::TRAPPED));
}
