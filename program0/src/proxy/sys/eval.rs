use super::{disk, disk_cmd, lock, proto, random};
pub(super) static mut APP_CTX: Option<Vec<u8>> = None;

fn dispatch_sys(req: &[u8]) -> Vec<u8> {
    match req.first().copied() {
        Some(proto::DISK_READ) => disk::read_cmd(req),
        Some(proto::DISK_WRITE_ATOMIC) => disk_cmd::write_cmd(req),
        Some(proto::DISK_REMOVE) => disk_cmd::remove_cmd(req),
        Some(proto::DISK_LIST) => disk_cmd::list_cmd(req),
        Some(proto::LOCK_ACQUIRE) => lock::acquire(),
        Some(proto::LOCK_RELEASE) => lock::release(),
        Some(proto::RANDOM_FILL) => random::fill(req),
        Some(proto::EVAL_APP) => super::eval_app::run(req),
        _ => proto::err(-1),
    }
}
fn app_cb(req: &[u8]) -> Vec<u8> {
    let iid = unsafe { (&*core::ptr::addr_of!(APP_CTX)).as_ref() };
    let Some(iid) = iid else {
        return proto::err(-13);
    };
    match skrynia_native_service::registry::appdata::cb::guest_callback(
        &super::cap::HostDisk,
        &super::cap::HostLock,
        "programs",
        iid,
        req,
    ) {
        Ok(v) => v,
        Err(e) => proto::err(e),
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn skrynia_host_cb(rp: u32, rl: u32, dp: u32, dc: u32) -> i32 {
    let req = unsafe { core::slice::from_raw_parts(rp as *const u8, rl as usize) };
    let in_app = unsafe { (&*core::ptr::addr_of!(APP_CTX)).is_some() };
    let res = if in_app {
        app_cb(req)
    } else {
        dispatch_sys(req)
    };
    if res.len() > dc as usize {
        return -10;
    }
    unsafe { core::ptr::copy_nonoverlapping(res.as_ptr(), dp as *mut u8, res.len()) };
    res.len() as i32
}
pub fn callback_index() -> i32 {
    skrynia_host_cb as *const () as usize as i32
}
pub fn handle(req: &[u8]) -> Vec<u8> {
    dispatch_sys(req)
}
