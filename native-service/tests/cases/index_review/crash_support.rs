//! Fail-injecting Disk for COW crash-boundary regressions.
use skrynia_native_service::index::{self, Scope};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
};

#[derive(Default)]
pub struct CrashDisk {
    pub files: RefCell<BTreeMap<String, Vec<u8>>>,
    step: Cell<usize>,
    fail: Cell<Option<(usize, bool)>>,
}
impl CrashDisk {
    pub fn mutation(&self) -> (bool, bool) {
        let n = self.step.get() + 1;
        self.step.set(n);
        match self.fail.get() {
            Some((at, after)) if at == n => (!after, after),
            _ => (false, false),
        }
    }
    pub fn arm(&self, at: usize, after: bool) {
        self.step.set(0);
        self.fail.set(Some((at, after)));
    }
    pub fn recover_io(&self) {
        self.fail.set(None);
    }
}
