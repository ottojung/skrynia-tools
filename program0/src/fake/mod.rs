//! Host fakes for the target ABI. Never compiled into WASM.
mod dir;
mod edit;
mod io;
mod scan;
mod state;
mod stream;
pub use dir::*;
pub use edit::*;
pub use io::*;
pub use path::*;
pub use ram::*;
pub use scan::*;
pub use setup::*;
mod path;
mod setup;
pub use state::*;
pub use stream::*;
mod ram;
use state::Kind;
pub use state::S;

pub fn inspect<T>(f: impl FnOnce(&State) -> T) -> T {
    S.with(|s| f(&s.borrow()))
}

pub unsafe fn open_ram() -> i32 {
    S.with(|s| s.borrow_mut().fd(Kind::Ram))
}

pub unsafe fn close(fd: i32) -> i32 {
    S.with(|s| {
        let mut b = s.borrow_mut();
        if b.get(fd).is_none() {
            return -2;
        }
        b.fds[fd as usize] = Kind::Closed;
        0
    })
}

pub unsafe fn truncate(fd: i32, size: u64) -> i32 {
    S.with(|s| {
        let mut b = s.borrow_mut();
        if !matches!(b.get(fd), Some(Kind::Ram)) {
            return -2;
        }
        let n = usize::try_from(size).unwrap_or(usize::MAX);
        if n > 64 * 1024 * 1024 {
            return -6;
        }
        b.ram.resize(n, 0);
        0
    })
}
