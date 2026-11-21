#[unsafe(no_mangle)]
pub extern "C" fn exists(_: u32, _: u32) -> i32 {
    -1
}
#[unsafe(no_mangle)]
pub extern "C" fn open_directory(_: u32, _: u32) -> i32 {
    -1
}
#[unsafe(no_mangle)]
pub extern "C" fn next_entry(_: i32, _: u32, _: u32) -> i32 {
    -1
}
#[unsafe(no_mangle)]
pub extern "C" fn sync(_: i32) -> i32 {
    -1
}
#[unsafe(no_mangle)]
pub extern "C" fn secure_random_bits(_: u32, _: u32) -> i32 {
    -1
}
#[unsafe(no_mangle)]
pub extern "C" fn eval(_: u32, _: u32, _: i32, _: i32, _: i32) -> i32 {
    -1
}
#[unsafe(no_mangle)]
pub extern "C" fn eval_delegate(_: u32, _: u32, _: i32, _: i32, _: i32) -> i32 {
    -1
}
