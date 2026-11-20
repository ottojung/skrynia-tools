mod run;
use super::{handle, host, reg};
#[unsafe(no_mangle)]
pub extern "C" fn skrynia_main(input_fd: i32, output_fd: i32) -> i32 {
    run::handle(input_fd, output_fd)
}
