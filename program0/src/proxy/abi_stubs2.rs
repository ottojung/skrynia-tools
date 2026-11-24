pub unsafe extern "C" fn open_directory(_: u32, _: u32) -> i32 {
    -1
}
pub unsafe extern "C" fn next_entry(_: i32, _: u32, _: u32) -> i32 {
    -1
}
pub unsafe extern "C" fn sync(_: i32) -> i32 {
    -1
}
pub unsafe extern "C" fn secure_random_bits(_: u32, _: u32) -> i32 {
    -1
}
#[allow(dead_code)]
pub unsafe extern "C" fn eval(_: u32, _: u32, _: i32, _: i32, _: i32) -> i32 {
    -1
}
pub unsafe extern "C" fn eval_delegate(_: u32, _: u32, _: i32, _: i32, _: i32) -> i32 {
    -1
}
pub unsafe extern "C" fn try_mutation_lock() -> i32 {
    -1
}
pub unsafe extern "C" fn release_mutation_lock() -> i32 {
    -1
}
pub unsafe extern "C" fn fs_delegate_scope(_: i32, _: u32, _: u32, _: u32) -> i32 {
    -1
}
#[allow(dead_code)]
pub unsafe extern "C" fn publish(_: u32) -> i32 {
    -1
}
#[allow(dead_code)]
pub unsafe extern "C" fn unpublish(_: u32) -> i32 {
    -1
}
