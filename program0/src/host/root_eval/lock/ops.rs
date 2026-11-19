#[cfg(target_arch = "wasm32")]
pub fn try_acquire() -> Result<bool, i32> {
    let result = unsafe { crate::root_abi::try_mutation_lock() };
    match result {
        1 => Ok(true),
        0 => Ok(false),
        code => Err(code),
    }
}

#[cfg(target_arch = "wasm32")]
pub fn release() -> Result<(), i32> {
    match unsafe { crate::root_abi::release_mutation_lock() } {
        0 => Ok(()),
        code => Err(code),
    }
}
