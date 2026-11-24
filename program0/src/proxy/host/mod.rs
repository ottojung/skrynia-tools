use super::abi;

mod close;
mod control;
mod directory;
mod io;
mod open;
mod path;
mod paths;
mod wire;
#[cfg(test)]
mod wire_test;

#[unsafe(no_mangle)]
pub extern "C" fn skrynia_host_cb(rp: u32, rl: u32, dp: u32, dc: u32) -> i32 {
    if dc < 4 || rl as usize > wire::LIMIT + 16 {
        return wire::OVERFLOW;
    }
    let req = unsafe { core::slice::from_raw_parts(rp as *const u8, rl as usize) };
    let result = dispatch(req, dc as usize);
    if result.len() > dc as usize {
        return wire::OVERFLOW;
    }
    unsafe { core::ptr::copy_nonoverlapping(result.as_ptr(), dp as *mut u8, result.len()) };
    result.len() as i32
}
fn dispatch(req: &[u8], dc: usize) -> Vec<u8> {
    match req.first().copied() {
        Some(wire::OPEN_DISK) => open::call(req),
        Some(wire::READ) => io::read(req, dc),
        Some(wire::WRITE) => io::write(req),
        Some(wire::CLOSE) => close::call(req),
        Some(wire::MKDIR) => paths::mkdir(req),
        Some(wire::DELETE) => paths::delete(req),
        Some(wire::RENAME) => paths::rename(req),
        Some(wire::OPEN_DIRECTORY) => directory::open(req),
        Some(wire::NEXT_ENTRY) => directory::next(req, dc),
        Some(wire::SYNC) => control::sync(req),
        Some(wire::RANDOM_FILL) => control::random(req, dc),
        Some(wire::TRY_LOCK) => control::try_lock(req),
        Some(wire::RELEASE_LOCK) => control::release_lock(req),
        Some(wire::EXISTS) => paths::exists(req),
        Some(wire::EVAL) => wire::reply(wire::DENIED, &[]),
        _ => wire::reply(wire::INVALID, &[]),
    }
}
#[used]
static CALLBACK: extern "C" fn(u32, u32, u32, u32) -> i32 = skrynia_host_cb;
pub fn callback_index() -> i32 {
    skrynia_host_cb as *const () as usize as i32
}
