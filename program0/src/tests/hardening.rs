//! Nested EVAL staging bounds, malformed frames and trap-cleanup regressions.
use super::support::{call, code, setup};
use super::{alloc, fake, invoke};

fn eval_frame(n: usize, bytes: &[u8]) -> Vec<u8> {
    let mut f = vec![15];
    f.extend_from_slice(&(n as u32).to_le_bytes());
    f.extend_from_slice(bytes);
    f
}

#[test]
fn oversized_op15_code_is_rejected_before_any_write() {
    let cap = (alloc::SLOT_SIZE - alloc::SLOT_CODE_AT) as usize;
    let (_i, _o, _r) = setup(vec![]);
    let f = eval_frame(cap + 1, &vec![0xA5; cap + 1]);
    assert_eq!(code(&call(&f)), alloc::ERR_RANGE);
    fake::inspect(|s| {
        assert_eq!(s.eval_calls, 0);
        let at = (alloc::BASE + alloc::SLOT_CODE_AT) as usize;
        assert!(s.ram[at..at + cap].iter().all(|&b| b == 0));
        let meta = (alloc::BASE + alloc::SLOT_SIZE + alloc::META_AT) as usize;
        assert!(
            s.ram[meta..meta + alloc::SLOTS as usize]
                .iter()
                .all(|&b| b == 0)
        );
    });
}

#[test]
fn malformed_op15_frames_never_eval_or_stage() {
    let (_i, _o, _r) = setup(vec![]);
    let mut huge = vec![15];
    huge.extend_from_slice(&u32::MAX.to_le_bytes());
    assert_eq!(code(&call(&huge)), -1);
    let mut trailing = eval_frame(3, b"abc");
    trailing.push(0);
    assert_eq!(code(&call(&trailing)), -1);
    fake::inspect(|s| assert_eq!(s.eval_calls, 0));
}

#[test]
fn nested_trap_leaves_the_parent_context_usable() {
    let (_i, _o, _r) = setup(vec![]);
    fake::S.with(|s| s.borrow_mut().eval_result = -9);
    assert_eq!(code(&call(&eval_frame(5, b"child"))), -9);
    assert_eq!(invoke::invoke(0, invoke::STATUS, 0), 0);
    let mut exists = vec![14];
    exists.extend_from_slice(&12u32.to_le_bytes());
    exists.extend_from_slice(b"user/missing");
    assert_eq!(code(&call(&exists)), 0);
}
