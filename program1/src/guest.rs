use skrynia_native_service::wenc::encode_res;
use skrynia_native_service::wire::{Response, STATUS_ERR};

fn unavailable() -> Vec<u8> {
    encode_res(&Response {
        status: STATUS_ERR,
        body: vec![0xff],
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn skrynia_main(_in_ptr: i32, _in_len: i32, out_ptr: i32, out_cap: i32) -> i32 {
    let body = unavailable();
    let len = body.len() as i32;
    if out_cap < len {
        return -10;
    }
    unsafe {
        core::ptr::copy_nonoverlapping(body.as_ptr(), out_ptr as *mut u8, body.len());
    }
    len
}
