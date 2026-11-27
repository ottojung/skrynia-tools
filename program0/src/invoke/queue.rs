//! Per-invocation byte queues. Each root instance has one live context.
use std::cell::RefCell;
use std::collections::VecDeque;

#[derive(Default)]
pub struct Ctx {
    pub input: i32,
    pub output: i32,
    pub ram: i32,
    pub slot: u64,
    pub in_handle: i32,
    pub out_handle: i32,
    pub frame: Vec<u8>,
    pub reply: VecDeque<u8>,
    pub reply_active: bool,
    pub frame_failed: bool,
    pub status: i32,
    pub handles: Vec<i32>,
    pub nested: bool,
}
thread_local! { static CURRENT: RefCell<Option<Ctx>> = const { RefCell::new(None) }; }

pub fn enter(input: i32, output: i32, ram: i32, slot: u64, ih: i32, oh: i32) {
    let c = Ctx {
        input,
        output,
        ram,
        slot,
        in_handle: ih,
        out_handle: oh,
        ..Ctx::default()
    };
    CURRENT.with(|s| *s.borrow_mut() = Some(c));
}
pub fn leave() {
    CURRENT.with(|s| {
        if let Some(mut c) = s.borrow_mut().take() {
            c.close_all();
        }
    });
}
pub fn set(outer: Option<Ctx>) {
    CURRENT.with(|s| *s.borrow_mut() = outer);
}
pub fn take() -> Option<Ctx> {
    CURRENT.with(|s| s.borrow_mut().take())
}
pub fn with<T>(f: impl FnOnce(&mut Ctx) -> T) -> Option<T> {
    CURRENT.with(|s| s.borrow_mut().as_mut().map(f))
}
