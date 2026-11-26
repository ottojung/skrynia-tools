//! Per-invocation byte queues. Each root instance has one live context.
use std::cell::RefCell;
use std::collections::VecDeque;

#[derive(Default)]
pub struct Ctx {
    pub input: i32,
    pub output: i32,
    pub ram: i32,
    pub base: u64,
    pub in_handle: i32,
    pub out_handle: i32,
    pub frame: Vec<u8>,
    pub reply: VecDeque<u8>,
    pub cb_in: VecDeque<u8>,
    pub cb_out: VecDeque<u8>,
    pub status: i32,
    pub pos: u64,
}
thread_local! { static CURRENT: RefCell<Option<Ctx>> = const { RefCell::new(None) }; }

pub fn enter(input: i32, output: i32, ram: i32, base: u64, ih: i32, oh: i32) {
    let mut c = Ctx::default();
    c.input = input;
    c.output = output;
    c.ram = ram;
    c.base = base;
    c.in_handle = ih;
    c.out_handle = oh;
    CURRENT.with(|s| *s.borrow_mut() = Some(c));
}
pub fn leave() {
    CURRENT.with(|s| *s.borrow_mut() = None);
}
pub fn set(outer: Option<Ctx>) {
    CURRENT.with(|s| *s.borrow_mut() = outer);
}
pub fn swap(child: Ctx) -> Option<Ctx> {
    CURRENT.with(|s| s.borrow_mut().replace(child))
}
pub fn feed_cb(byte: u8) {
    CURRENT.with(|s| {
        if let Some(c) = s.borrow_mut().as_mut() {
            c.cb_in.push_back(byte)
        }
    });
}
pub fn take() -> Option<Ctx> {
    CURRENT.with(|s| s.borrow_mut().take())
}
pub fn with<T>(f: impl FnOnce(&mut Ctx) -> T) -> Option<T> {
    CURRENT.with(|s| s.borrow_mut().as_mut().map(f))
}
