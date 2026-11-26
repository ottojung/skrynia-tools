#![no_std]

mod callback;
mod stream;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() {
    let mut request = [0u8; 4096];
    let Ok(len) = stream::receive(&mut request) else {
        return;
    };
    if len == 0 {
        return;
    }
    let trap = request[0] == 255;
    if trap {
        request[0] = 1;
    }
    let mut reply = [0u8; 4096];
    let n = callback::call(&request[..len], &mut reply);
    if n < 0 {
        return;
    }
    if trap {
        // Deliberate guest trap after a completed framed request. The kernel
        // must recover so the next request on this stack is served normally.
        core::arch::wasm32::unreachable();
    }
    let mut frame = [0u8; 4105];
    if stream::send(&mut frame, &reply[..n as usize]) < 0 {
        return;
    }
}
