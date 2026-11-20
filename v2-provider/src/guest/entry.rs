use super::invoke::Host;

#[unsafe(no_mangle)]
pub extern "C" fn skrynia_main(
    request_ptr: u32,
    request_len: u32,
    output_ptr: u32,
    output_capacity: u32,
) -> i32 {
    if request_len == 0 || request_len > 8 * 1024 * 1024 || output_capacity > 10 * 1024 * 1024 {
        return -10;
    }
    let req =
        unsafe { core::slice::from_raw_parts(request_ptr as *const u8, request_len as usize) };
    let Ok(answer) = crate::dispatch_bytes(req, &Host) else {
        return -4;
    };
    if answer.len() > output_capacity as usize {
        return -11;
    }
    unsafe {
        core::ptr::copy_nonoverlapping(answer.as_ptr(), output_ptr as *mut u8, answer.len());
    }
    answer.len() as i32
}
