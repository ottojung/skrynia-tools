//! Authenticate, reserve a CAS owner tag, and evaluate one private code buffer.
use super::{abi, alloc, auth, code, invoke};

#[unsafe(no_mangle)]
pub extern "C" fn skrynia_main(input_fd: i32, output_fd: i32) -> i32 {
    let result = serve(input_fd, output_fd);
    unsafe {
        abi::close(input_fd);
        abi::close(output_fd);
    }
    result.err().unwrap_or(0)
}

fn serve(input_fd: i32, output_fd: i32) -> Result<(), i32> {
    auth::require(input_fd)?;
    let wasm = code::load()?;
    let ram = unsafe { abi::open_ram() };
    if ram < 0 {
        return Err(ram);
    }
    let result = (|| {
        alloc::ensure(ram)?;
        let slot = alloc::acquire(ram)?;
        invoke::enter(input_fd, output_fd, ram, slot, input_fd, output_fd);
        let rc = unsafe {
            abi::eval(
                wasm.as_ptr() as u64,
                wasm.len() as u64,
                invoke::index(),
                input_fd,
                output_fd,
            )
        };
        invoke::leave();
        alloc::reclaim(ram, slot);
        alloc::release(ram, slot);
        if rc < 0 { Err(rc as i32) } else { Ok(()) }
    })();
    unsafe { abi::close(ram) };
    result
}
