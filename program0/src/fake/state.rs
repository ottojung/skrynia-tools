//! Deterministic in-memory machine state for host-only tests.
use std::cell::RefCell;

#[derive(Clone)]
pub enum Kind {
    Input,
    Output,
    Ram,
    Disk(usize),
    Dir(usize),
}
pub struct State {
    pub input: Vec<u8>,
    pub inp: usize,
    pub output: Vec<u8>,
    pub ram: Vec<u8>,
    pub rpos: usize,
    pub files: Vec<(String, Vec<u8>)>,
    pub fpos: Vec<usize>,
    pub dirs: Vec<String>,
    pub fds: Vec<Kind>,
    pub eval_code: Vec<u8>,
    pub eval_result: i64,
    pub eval_calls: usize,
}
impl State {
    pub fn new() -> Self {
        Self {
            input: vec![],
            inp: 0,
            output: vec![],
            ram: vec![],
            rpos: 0,
            files: vec![],
            fpos: vec![],
            dirs: vec![],
            fds: vec![],
            eval_code: vec![],
            eval_result: 0,
            eval_calls: 0,
        }
    }
    pub fn fd(&mut self, k: Kind) -> i32 {
        self.fds.push(k);
        self.fpos.push(0);
        (self.fds.len() - 1) as i32
    }
    pub fn get(&self, fd: i32) -> Option<&Kind> {
        self.fds.get(fd as usize)
    }
}
thread_local! { pub static S: RefCell<State> = RefCell::new(State::new()); }
