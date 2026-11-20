mod close;
mod dir;
mod fdctl;
mod io;
mod open;
mod path;
mod rename;
mod special;
mod wire;
use super::abi;

#[unsafe(no_mangle)]
pub extern "C" fn skrynia_host_cb(rp: u32, rl: u32, dp: u32, dc: u32) -> i32 {
    if dc < 4 || rl as usize > wire::LIMIT + 64 {
        return wire::OVERFLOW;
    }
    let req = unsafe { core::slice::from_raw_parts(rp as *const u8, rl as usize) };
    let cap = dc as usize;
    let result = match req.first().copied() {
        Some(wire::OPEN_DISK) => open::call(req),
        Some(wire::READ) => io::read(req, cap),
        Some(wire::WRITE) => io::write(req),
        Some(wire::CLOSE) => close::call(req),
        Some(wire::SYNC) => fdctl::sync(req),
        Some(wire::SEEK) => fdctl::seek(req),
        Some(wire::TRUNCATE) => fdctl::truncate(req),
        Some(wire::RENAME) => rename::call(req),
        Some(wire::DELETE) => path::delete(req),
        Some(wire::MKDIR) => path::mkdir(req),
        Some(wire::EXISTS) => path::exists(req),
        Some(wire::IS_DIR) => path::is_dir(req),
        Some(wire::OPEN_DIR) => dir::open(req),
        Some(wire::NEXT_ENTRY) => dir::next(req, cap),
        Some(wire::RANDOM) => special::random(req, cap),
        Some(wire::OPEN_RAM) => special::open_ram(req),
        Some(wire::CAS_RAM) => special::cas_ram(req),
        _ => wire::reply(wire::INVALID, &[]),
    };
    if result.len() > cap {
        return wire::OVERFLOW;
    }
    unsafe { core::ptr::copy_nonoverlapping(result.as_ptr(), dp as *mut u8, result.len()) };
    result.len() as i32
}
#[used]
static CALLBACK: extern "C" fn(u32, u32, u32, u32) -> i32 = skrynia_host_cb;
pub fn callback_index() -> i32 {
    skrynia_host_cb as *const () as usize as i32
}
