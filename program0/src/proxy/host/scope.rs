/// Operator-delegated filesystem scope for privileged root callbacks.
/// Baked at build time; an empty prefix denies every path (fail closed).
pub const PREFIX: &str = match option_env!("SKRYNIA_P0_FS_PREFIX") {
    Some(p) => p,
    None => "",
};
pub const WRITE_OK: bool = option_env!("SKRYNIA_P0_FS_WRITE").is_some();
pub const MAX_FDS: usize = 16;

/// Accept only plain descendant paths under a delegated prefix.
pub fn path_in(prefix: &str, path: &[u8]) -> bool {
    let Ok(s) = core::str::from_utf8(path) else {
        return false;
    };
    if prefix.is_empty() || s.len() > 256 || s.starts_with('/') || s.contains('\0') {
        return false;
    }
    if s.split('/').any(|p| p.is_empty() || p == "." || p == "..") {
        return false;
    }
    s.strip_prefix(prefix).is_some_and(|r| r.starts_with('/'))
}

pub fn path_ok(path: &[u8]) -> bool {
    path_in(PREFIX, path)
}

pub fn write_in(prefix: &str, write: bool, path: &[u8]) -> bool {
    write && path_in(prefix, path)
}

/// Bounded set of root FDs minted for this guest; no implicit FD rights.
#[derive(Clone, Copy)]
pub struct Fds {
    items: [i32; MAX_FDS],
}
impl Fds {
    pub const fn new() -> Self {
        Self {
            items: [-1; MAX_FDS],
        }
    }
    pub fn grant(&mut self, fd: i32) -> bool {
        if fd < 0 || self.has(fd) {
            return false;
        }
        if let Some(slot) = self.items.iter_mut().find(|x| **x < 0) {
            *slot = fd;
            return true;
        }
        false
    }
    pub fn has(&self, fd: i32) -> bool {
        self.items.contains(&fd)
    }
    pub fn revoke(&mut self, fd: i32) {
        if let Some(slot) = self.items.iter_mut().find(|x| **x == fd) {
            *slot = -1;
        }
    }
}

static mut DELEGATED: Fds = Fds::new();
pub fn grant(fd: i32) -> bool {
    unsafe { (*core::ptr::addr_of_mut!(DELEGATED)).grant(fd) }
}
pub fn allows(fd: i32) -> bool {
    unsafe { (*core::ptr::addr_of!(DELEGATED)).has(fd) }
}
pub fn revoke(fd: i32) {
    unsafe { (*core::ptr::addr_of_mut!(DELEGATED)).revoke(fd) }
}
