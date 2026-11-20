use super::{abi, code, wire};

pub fn call(req: &[u8]) -> Vec<u8> {
    let mut at = 1;
    let (Some(path), Some(input_fd), Some(output_fd)) = (
        wire::bytes(req, &mut at),
        wire::u32_at(req, &mut at),
        wire::u32_at(req, &mut at),
    ) else {
        return wire::reply(wire::INVALID, &[]);
    };
    if at != req.len() || input_fd == output_fd {
        return wire::reply(wire::INVALID, &[]);
    }
    let code = match code::read_from(path) {
        Ok(bytes) => bytes,
        Err(error) => return wire::reply(error, &[]),
    };
    let result = unsafe {
        abi::eval(
            code.as_ptr() as u32,
            code.len() as u32,
            input_fd as i32,
            output_fd as i32,
            super::callback_index(),
        )
    };
    wire::reply(result, &[])
}
