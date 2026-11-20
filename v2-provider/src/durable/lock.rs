use crate::bridge::Callback;
use crate::bridge::sys;
pub struct Guard;
pub fn acquire<I: Callback>(i: &I) -> Option<Guard> {
    match sys::lock::acquire(i) {
        Ok(1) => Some(Guard),
        _ => None,
    }
}
pub fn release<I: Callback>(i: &I, _guard: Guard) -> Result<(), ()> {
    sys::lock::release(i).map_err(|_| ())
}
