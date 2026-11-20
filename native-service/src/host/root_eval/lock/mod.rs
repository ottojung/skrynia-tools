mod ops;

#[cfg(target_arch = "wasm32")]
pub struct HostLock;

#[cfg(target_arch = "wasm32")]
impl crate::MutLock for HostLock {
    type Err = i32;
    fn try_acquire(&self) -> Result<bool, i32> {
        ops::try_acquire()
    }
    fn release(&self) -> Result<(), i32> {
        ops::release()
    }
}
