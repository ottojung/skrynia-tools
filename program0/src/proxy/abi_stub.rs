//! Host-only link stubs. Kernel imports are real only in the wasm32 build.
#[allow(dead_code)]
pub unsafe fn open_disk(_p: u32, _n: u32, _m: u32) -> i32 {
    71
}
#[allow(dead_code)]
pub unsafe fn read(_fd: i32, _d: u32, _n: u32) -> i32 {
    0
}
#[allow(dead_code)]
pub unsafe fn write(_fd: i32, _p: u32, _n: u32) -> i32 {
    0
}
#[allow(dead_code)]
pub unsafe fn close(_fd: i32) -> i32 {
    0
}
#[allow(dead_code)]
pub unsafe fn mkdir(_p: u32, _n: u32, _m: u32) -> i32 {
    0
}
#[allow(dead_code)]
pub unsafe fn delete(_p: u32, _n: u32) -> i32 {
    0
}
#[allow(dead_code)]
pub unsafe fn rename(_a: u32, _an: u32, _b: u32, _bn: u32) -> i32 {
    0
}
#[allow(dead_code)]
pub unsafe fn open_directory(_p: u32, _n: u32) -> i32 {
    72
}
#[allow(dead_code)]
pub unsafe fn next_entry(_fd: i32, _d: u32, _c: u32) -> i32 {
    0
}
#[allow(dead_code)]
pub unsafe fn sync(_fd: i32) -> i32 {
    0
}
#[allow(dead_code)]
pub unsafe fn secure_random_bits(_d: u32, _b: u32) -> i32 {
    0
}
#[allow(dead_code)]
pub unsafe fn try_mutation_lock() -> i32 {
    1
}
#[allow(dead_code)]
pub unsafe fn release_mutation_lock() -> i32 {
    0
}
#[allow(dead_code)]
pub unsafe fn eval(_c: u32, _cl: u32, _i: i32, _o: i32, _cb: i32) -> i32 {
    0
}
