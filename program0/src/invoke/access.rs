//! Userland authority of the supplied gateway, not a kernel rule.
//! The operator-selected root can use both disk roots itself; its child
//! receives user-root paths and only descriptors explicitly opened for it.
use super::{Ctx, abi};

pub(super) fn check_path(path: &[u8]) -> Result<(), i32> {
    if path.starts_with(b"user/") {
        Ok(())
    } else if path.starts_with(b"operator/") {
        Err(-4)
    } else {
        Err(-3)
    }
}

impl Ctx {
    pub fn grant(&mut self, fd: i32) {
        if fd >= 2 {
            self.handles.push(fd);
        }
    }

    pub fn resolve(&self, raw: i32) -> Result<i32, i32> {
        match raw {
            0 => Ok(self.input),
            1 => Ok(self.output),
            fd if self.handles.contains(&fd) => Ok(fd),
            _ => Err(-2),
        }
    }

    pub fn close_fd(&mut self, raw: i32) -> i32 {
        // The root owns the transport descriptors; the guest cannot close
        // them, nor can it close the private CAS metadata descriptor.
        if raw < 2 {
            return -4;
        }
        let Ok(fd) = self.resolve(raw) else {
            return -2;
        };
        let rc = unsafe { abi::close(fd) };
        self.handles.retain(|&item| item != fd);
        rc
    }

    pub fn close_all(&mut self) {
        for fd in self.handles.drain(..) {
            unsafe { abi::close(fd) };
        }
    }
}
