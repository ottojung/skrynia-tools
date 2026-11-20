use super::abi;
#[unsafe(no_mangle)]
pub extern "C" fn skrynia_main(input_fd: i32, output_fd: i32) -> i32 {
    // Pass two opaque forward-only descriptors unchanged. The kernel
    // transfers ownership to an isolated operator-selected Program 1.
    unsafe { abi::run_program1(input_fd, output_fd) }
}
