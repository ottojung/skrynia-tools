use super::{invoke::Host, stream};
#[unsafe(no_mangle)]
pub extern "C" fn skrynia_main(input_fd: i32, output_fd: i32) -> i32 {
    let result = stream::receive(input_fd).and_then(|req| {
        let response = crate::dispatch_bytes(&req, &Host).map_err(|_| -4)?;
        stream::send(output_fd, &response)
    });
    match result {
        Ok(()) => 0,
        Err(e) => e,
    }
}
