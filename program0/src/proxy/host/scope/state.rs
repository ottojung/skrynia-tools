use super::{fd, meta, meta_load};
use core::cell::UnsafeCell;

pub struct Ctx {
    pub policy: Option<meta::Policy>,
    pub fds: fd::Table,
}
struct Slot(UnsafeCell<Option<Ctx>>);
unsafe impl Sync for Slot {}
static STATE: Slot = Slot(UnsafeCell::new(None));

pub fn enter(program: &[u8]) {
    let policy = meta_load::load(program);
    unsafe {
        *STATE.0.get() = Some(Ctx {
            policy,
            fds: fd::Table::new(),
        })
    };
}
pub fn exit() {
    unsafe { *STATE.0.get() = None };
}
pub fn with<R>(f: impl FnOnce(&mut Ctx) -> R) -> Option<R> {
    unsafe { (&mut *STATE.0.get()).as_mut().map(f) }
}
