use super::{abi, auth, code, stream};
const MAX_OUTPUT: usize = 8 * 1024 * 1024;

#[unsafe(no_mangle)]
pub extern "C" fn skrynia_main(input_fd: i32, output_fd: i32) -> i32 {
    let result = forward(input_fd, output_fd);
    unsafe {
        abi::close(input_fd);
        abi::close(output_fd);
    }
    match result {
        Ok(()) => 0,
        Err(code) => code,
    }
}
fn forward(input_fd: i32, output_fd: i32) -> Result<(), i32> {
    auth::require(input_fd)?;
    let wasm = code::load()?;
    let input = stream::receive(input_fd)?;
    let mut output = vec![0u8; MAX_OUTPUT];
    let len = unsafe {
        abi::eval(
            wasm.as_ptr() as u32,
            wasm.len() as u32,
            input.as_ptr() as u32,
            input.len() as u32,
            -1,
            output.as_mut_ptr() as u32,
            output.len() as u32,
        )
    };
    if len < 0 {
        return Err(len);
    }
    stream::send(output_fd, &output[..len as usize])
}
