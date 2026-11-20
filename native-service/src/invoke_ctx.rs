#[cfg(all(target_arch = "wasm32", feature = "cb-guest"))]
static mut CURRENT: Option<Vec<u8>> = None;

#[cfg(all(target_arch = "wasm32", feature = "cb-guest"))]
pub fn set(id: Vec<u8>) {
    unsafe { CURRENT = Some(id) };
}
#[cfg(all(target_arch = "wasm32", feature = "cb-guest"))]
pub fn clear() {
    unsafe { CURRENT = None };
}
#[cfg(all(target_arch = "wasm32", feature = "cb-guest"))]
pub fn get() -> Option<Vec<u8>> {
    unsafe { (&*core::ptr::addr_of!(CURRENT)).clone() }
}
