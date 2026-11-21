/// Interpret the kernel `skrynia.try_mutation_lock()` return value.
///
/// Kernel contract (PR #112): `1` acquired, `0` contended/nonreentrant,
/// any negative value is an error. This pure helper is shared by the
/// ordinary-guest callback capability and host-side regression tests.
pub fn try_lock_status(n: i32) -> Result<bool, i32> {
    match n {
        1 => Ok(true),
        0 => Ok(false),
        code => Err(code),
    }
}
