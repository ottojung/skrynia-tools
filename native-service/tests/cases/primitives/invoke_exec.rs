use crate::cases::support::{MemDisk, NoLock, Rng};
use skrynia_native_service::cap::Eval;
use skrynia_native_service::reg::Registry;
use skrynia_native_service::{handle, wdec, wenc, wire};

struct RunEval;
impl Eval for RunEval {
    type Err = ();
    fn eval(
        &self,
        wasm: &[u8],
        input: &[u8],
        _cb: Option<&mut dyn FnMut(&[u8]) -> Result<Vec<u8>, i32>>,
    ) -> Result<Vec<u8>, ()> {
        Ok([wasm, b":", input].concat())
    }
}

#[test]
fn invoke_runs_registered_code() {
    let disk = MemDisk::default();
    let lock = NoLock;
    let admin = [3u8; 32];
    let reg = Registry::open(&disk, "programs").unwrap();
    let iid = reg
        .register(&disk, &lock, &admin, b"CODE".to_vec())
        .unwrap();
    let req = wenc::encode_req(&wire::Request::Invoke {
        invoke_id: iid,
        input: b"IN".to_vec(),
    });
    let raw = handle::handle(&reg, &disk, &lock, &Rng(1), &RunEval, &req);
    let res = wdec::decode_res(&raw).unwrap();
    assert_eq!(res.status, wire::STATUS_OK);
    assert_eq!(res.body, b"CODE:IN".to_vec());
}

#[test]
fn guest_entry_has_no_no_eval_stub() {
    let src = include_str!("../../../src/cb_guest/entry.rs");
    assert!(
        !src.contains("NoEval"),
        "guest still carries the NoEval stub"
    );
    assert!(
        src.contains("MediatorEval"),
        "guest is not wired to delegated eval"
    );
}
