use crate::cases::support::{MemDisk, NoLock};
use skrynia_native_service::{cap::Disk, reg::Registry, store};

/// Number of staged and sealed ready versions in the exact scale gate.
pub const N: usize = 1000;

/// Build one ordinary P1 program and `N` distinct sealed ready versions.
pub fn build<D: Disk>(d: &D, lock: &NoLock, admin: &[u8]) -> (Vec<u8>, Vec<[u8; 32]>)
where
    D::Err: std::fmt::Debug,
{
    let reg = Registry::open(d, "reg").unwrap();
    let first = reg.register(d, lock, admin, b"v-base".to_vec()).unwrap();
    let mut hashes = vec![store::code_hash(b"v-base")];
    for i in 0..N {
        let code = format!("v-{i:04}");
        let h = reg.stage_release(d, lock, admin, code.as_bytes()).unwrap();
        reg.seal_release(d, lock, admin, &h).unwrap();
        hashes.push(h);
    }
    (first, hashes)
}

pub type Fixture = (Registry, MemDisk, NoLock, Vec<u8>, Vec<u8>, Vec<[u8; 32]>);

pub fn setup() -> Fixture {
    let d = MemDisk::default();
    let lock = NoLock;
    let admin = vec![9u8; 32];
    let (iid, hashes) = build(&d, &lock, &admin);
    (
        Registry::open(&d, "reg").unwrap(),
        d,
        lock,
        admin,
        iid,
        hashes,
    )
}

pub fn sorted(hashes: &[[u8; 32]]) -> Vec<[u8; 32]> {
    let mut v = hashes.to_vec();
    v.sort();
    v
}
