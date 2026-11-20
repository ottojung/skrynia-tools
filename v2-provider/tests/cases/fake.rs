use skrynia_v2_provider::{Callback, InvokeError};
use std::cell::RefCell;
#[derive(Default)]
pub struct Fake {
    state: RefCell<(bool, usize)>,
}
impl Callback for Fake {
    fn invoke(&self, request: &[u8]) -> Result<Vec<u8>, InvokeError> {
        let failure = || Err(InvokeError::Unavailable);
        let Some((&op, tail)) = request.split_first() else {
            return failure();
        };
        match op {
            1 => {
                let Some(len) = tail.get(..4) else {
                    return failure();
                };
                let n = u32::from_le_bytes(len.try_into().unwrap()) as usize;
                let Some(path) = tail.get(4..4 + n) else {
                    return failure();
                };
                if tail.get(4 + n..) != Some(&1u32.to_le_bytes()) {
                    return failure();
                }
                if path == b"v2/objects/public/missing.txt" {
                    return Ok((-2i32).to_le_bytes().to_vec());
                }
                if path != b"v2/objects/public/file.txt" {
                    return failure();
                }
                self.state.replace((true, 0));
                Ok((99i32).to_le_bytes().to_vec())
            }
            2 => {
                if tail.len() != 8 {
                    return failure();
                }
                if i32::from_le_bytes(tail[..4].try_into().unwrap()) != 99 {
                    return failure();
                }
                let mut state = self.state.borrow_mut();
                if !state.0 {
                    return failure();
                }
                let bytes = b"abcdef";
                let n = (bytes.len() - state.1)
                    .min(u32::from_le_bytes(tail[4..].try_into().unwrap()) as usize);
                let mut out = (n as i32).to_le_bytes().to_vec();
                out.extend_from_slice(&bytes[state.1..state.1 + n]);
                state.1 += n;
                Ok(out)
            }
            4 if tail == 99i32.to_le_bytes() => {
                self.state.replace((false, 0));
                Ok(0i32.to_le_bytes().to_vec())
            }
            _ => failure(),
        }
    }
}
