//! Select T6's userland `p2::Interpreter` as the P1 Eval capability.
//!
//! The interpreter invokes P1's own callback closure directly on P1's stack:
//! `CB_GET` submits one opaque request byte and `CB_PUT` drains the reply.
//! There is no P0 callback relay, no suspended P1, no privileged import and
//! no cross-store pointer. This adapter frames the opaque byte into a complete
//! appdata request before handing it to `Registry::invoke`'s callback, because
//! that callback decodes whole wire requests rather than single bytes.

use crate::wire::Request;

fn code(err: crate::p2::Error) -> i32 {
    match err {
        crate::p2::Error::Callback(c) => c,
        crate::p2::Error::NoCallback => -4,
        crate::p2::Error::Trap => -2,
        crate::p2::Error::InvalidWasm => -1,
        _ => -3,
    }
}

/// The target P1 capability: T6's in-process interpreter driven by P1's own
/// appdata callback closure. It replaces the P0-mediated no-callback path for
/// ordinary nested evaluation; no kernel ABI or gateway file is changed.
pub struct P1CallbackEval;

impl crate::cap::Eval for P1CallbackEval {
    type Err = i32;

    fn eval(
        &self,
        wasm: &[u8],
        input: &[u8],
        cb: Option<&mut dyn FnMut(&[u8]) -> Result<Vec<u8>, i32>>,
    ) -> Result<Vec<u8>, i32> {
        let Some(iid) = super::cb::current_id() else {
            return Err(-13);
        };
        let Some(cb) = cb else {
            return Err(-4);
        };
        let mut bridge = |request: &[u8]| -> Result<Vec<u8>, i32> {
            let key = match request.first() {
                Some(0) => b"cb".to_vec(),
                _ => return Err(-3),
            };
            let framed = crate::wenc::encode_req(&Request::AppGet {
                read_cap: iid.clone(),
                key,
            });
            cb(&framed)
        };
        crate::p2::Interpreter::new()
            .evaluate(wasm, input, Some(&mut bridge))
            .map_err(code)
    }
}
