#![allow(dead_code, unused_imports)]

mod fixture;
mod net;

pub use fixture::Fixture;
pub use net::{call_at, client, wait_ready};

use std::path::{Path, PathBuf};

pub fn call(f: &Fixture, name: &str, ticket: u32) -> u32 {
    call_at(&f.base, name, ticket)
}

pub fn request(f: &Fixture, ticket: u32, path: &[u8]) {
    request_at(&f.requests, ticket, path);
}

pub fn request_at(dir: &Path, ticket: u32, path: &[u8]) {
    let name = format!("{ticket:08x}");
    std::fs::write(dir.join(name), path).expect("write request");
}

pub fn source(f: &Fixture, rel: &str, content: &[u8]) -> PathBuf {
    let path = f.disk.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("source parents");
    }
    std::fs::write(&path, content).expect("source file");
    path
}

pub fn target(f: &Fixture, rel: &str) -> Option<PathBuf> {
    std::fs::read_link(f.nginx.join(rel)).ok()
}

pub fn code(f: &Fixture, name: &str, ticket: u32, path: &[u8]) -> u32 {
    request(f, ticket, path);
    call(f, name, ticket)
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
