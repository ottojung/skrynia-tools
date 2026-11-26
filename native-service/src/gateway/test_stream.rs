//! Byte-stream call framing: one flush, no replay after an uncertain result.
use super::fake::Fake;
use super::{eval, ops, stream};

#[test]
fn stream_call_is_one_flush_and_no_replay() {
    let fake = Fake::new(vec![0, 0, 0, 0, 7, 8]);
    let out = stream::call(&fake, ops::CTX_MAIN, &[1, 2, 3], 16).expect("call");
    assert_eq!(out, vec![0, 0, 0, 0, 7, 8]);
    assert_eq!(*fake.outbound.borrow(), vec![1, 2, 3]);
    assert_eq!(*fake.gets.borrow(), 7);
}

#[test]
fn no_callback_nested_eval_returns_child_output() {
    let fake = Fake::new(vec![0, 0, 0, 0, 5, 6]);
    assert_eq!(
        eval::call(&fake, ops::CTX_NESTED, &[15, 0], 16),
        Ok(vec![5, 6])
    );
}
