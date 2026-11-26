//! Root entrypoint: token check, per-request RAM slot, one eval, no replay.
use super::{abi, alloc, auth, code, invoke};

#[unsafe(no_mangle)]
pub extern "C" fn skrynia_main(input_fd: i32, output_fd: i32) -> i32 {
    let result = serve(input_fd, output_fd);
    unsafe {
        abi::close(input_fd);
        abi::close(output_fd);
    }
    match result {
        Ok(()) => 0,
        Err(e) => e,
    }
}

fn serve(input_fd: i32, output_fd: i32) -> Result<(), i32> {
    let ram = unsafe { abi::open_ram() };
    if ram < 0 {
        return Err(ram);
    }
    let mut pos = 0u64;
    let result = (|| {
        alloc::ensure(ram)?;
        let (index, base) = alloc::acquire(ram)?;
        let outcome = (|| {
            auth::require(input_fd, ram, &mut pos, base)?;
            code::put_path(ram, &mut pos, base + alloc::SLOT_PATH_AT, code::GUEST_PATH)?;
            let size = code::stage(
                ram,
                &mut pos,
                base + alloc::SLOT_CODE_AT,
                base + alloc::SLOT_PATH_AT,
            )?;
            invoke::enter(input_fd, output_fd, ram, base, input_fd, output_fd, pos);
            let rc = unsafe {
                abi::eval(
                    base + alloc::SLOT_CODE_AT,
                    size,
                    invoke::index(),
                    input_fd,
                    output_fd,
                )
            };
            invoke::leave();
            if rc < 0 { Err(rc as i32) } else { Ok(()) }
        })();
        alloc::release(ram, index);
        outcome
    })();
    unsafe { abi::close(ram) };
    result
}
