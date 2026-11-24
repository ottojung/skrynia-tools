use super::{abi, auth, code, host};

#[unsafe(no_mangle)]
pub extern "C" fn skrynia_main(input_fd: i32, output_fd: i32) -> i32 {
    let result = match auth::require(input_fd).and_then(|()| code::load()) {
        Ok(wasm) => {
            // Dumb P0: delegate generic User-root data capabilities to the single
            // operator-selected trusted ordinary root guest; no app policy/metadata in P0.
            unsafe { abi::fs_delegate_scope(0, 0, 0, 0xff) };
            let r = unsafe {
                abi::eval_delegate(
                    wasm.as_ptr() as u32,
                    wasm.len() as u32,
                    input_fd,
                    output_fd,
                    host::callback_index(),
                )
            };
            unsafe { abi::fs_delegate_scope(0, 0, 0, 0) };
            r
        }
        Err(error) => error,
    };
    unsafe {
        abi::close(input_fd);
        abi::close(output_fd);
    }
    result
}
