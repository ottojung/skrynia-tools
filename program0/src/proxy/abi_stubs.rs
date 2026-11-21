#[unsafe(no_mangle)]
pub extern "C" fn open_disk(_: u32, _: u32, _: u32) -> i32 {
    -1
}
#[unsafe(no_mangle)]
pub extern "C" fn read(_: i32, _: u32, _: u32) -> i32 {
    -1
}
#[unsafe(no_mangle)]
pub extern "C" fn write(_: i32, _: u32, _: u32) -> i32 {
    -1
}
#[unsafe(no_mangle)]
pub extern "C" fn close(_: i32) -> i32 {
    -1
}
#[unsafe(no_mangle)]
pub extern "C" fn mkdir(_: u32, _: u32, _: u32) -> i32 {
    -1
}
#[unsafe(no_mangle)]
pub extern "C" fn rename(_: u32, _: u32, _: u32, _: u32) -> i32 {
    -1
}
#[unsafe(no_mangle)]
pub extern "C" fn delete(_: u32, _: u32) -> i32 {
    -1
}
