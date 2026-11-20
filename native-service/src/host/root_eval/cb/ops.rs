#[cfg(target_arch = "wasm32")]
use super::map;

#[cfg(target_arch = "wasm32")]
type D = crate::root_disk::HostDisk;
#[cfg(target_arch = "wasm32")]
type L = super::super::lock::HostLock;

#[cfg(target_arch = "wasm32")]
pub fn put(
    d: &D,
    l: &L,
    dir: &str,
    id: &[u8],
    k: Vec<u8>,
    ex: crate::wire::AppPrecond,
    v: Vec<u8>,
) -> Vec<u8> {
    match crate::registry::appdata::ops::put(d, l, dir, crate::WriteAuth::SelfNs(id), &k, &ex, &v) {
        Ok(h) => map::ok(h.to_vec()),
        Err(e) => map::map_app_err(e),
    }
}

#[cfg(target_arch = "wasm32")]
pub fn del(d: &D, l: &L, dir: &str, id: &[u8], k: Vec<u8>, ex: Vec<u8>) -> Vec<u8> {
    match crate::registry::appdata::ops::delete(d, l, dir, crate::WriteAuth::SelfNs(id), &k, &ex) {
        Ok(()) => map::ok(Vec::new()),
        Err(e) => map::map_app_err(e),
    }
}
