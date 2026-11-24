#![no_std]
mod stream;
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}
mod callback;
use callback::call;
#[unsafe(no_mangle)]
pub extern "C" fn skrynia_main(input: i32, output: i32) -> i32 {
    let mut req = [0u8; 4096];
    let len = match stream::receive(input, &mut req) {
        Ok(n) => n,
        Err(e) => return e,
    };
    if len == 0 {
        return -1;
    }
    let trap = req[0] == 255;
    if trap {
        req[0] = 1;
    }
    let mut out = [0u8; 512];
    let mut n = call(&req[..len], &mut out);
    if n == 4 && req[0] == 1 {
        let fd = i32::from_le_bytes(out[..4].try_into().unwrap());
        if fd >= 0 {
            if trap {
                panic!("deliberate test trap with live fd");
            }
            let mut next = [0u8; 9];
            next[0] = 2;
            next[1..5].copy_from_slice(&fd.to_le_bytes());
            next[5..].copy_from_slice(&128u32.to_le_bytes());
            n = call(&next, &mut out);
            next[0] = 4;
            call(&next[..5], &mut [0u8; 4]);
        }
    }
    if n < 0 {
        return n;
    }
    stream::send(output, &out[..n as usize])
}
