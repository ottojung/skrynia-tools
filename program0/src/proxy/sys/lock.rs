use super::proto;
pub fn acquire() -> Vec<u8> {
    let n = unsafe { super::super::abi::try_mutation_lock() };
    if n == 0 {
        vec![proto::OK, proto::TRUE]
    } else if n == -3 {
        vec![proto::OK, proto::FALSE]
    } else {
        proto::err(n)
    }
}
pub fn release() -> Vec<u8> {
    let n = unsafe { super::super::abi::release_mutation_lock() };
    if n < 0 {
        proto::err(n)
    } else {
        proto::ok_empty()
    }
}
