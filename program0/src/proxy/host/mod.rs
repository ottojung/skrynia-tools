mod close;
mod control;
mod directory;
mod eval;
mod io;
mod open;
mod paths;
mod wire;

// Release blocker (hAxVrL5ZstxkNOF5O2mBvS-refreshed): generic guest callback
// delegation is not yet implemented; privileged operations fail closed.
#[unsafe(no_mangle)]
pub extern "C" fn skrynia_host_cb(rp: u32, rl: u32, dp: u32, dc: u32) -> i32 {
    if dc < 4 || rl as usize > wire::LIMIT + 16 {
        return wire::OVERFLOW;
    }
    let req = unsafe { core::slice::from_raw_parts(rp as *const u8, rl as usize) };
    let result = match req.first().copied() {
        Some(wire::OPEN_DISK)
        | Some(wire::READ)
        | Some(wire::WRITE)
        | Some(wire::CLOSE)
        | Some(wire::SYNC)
        | Some(wire::MKDIR)
        | Some(wire::RENAME)
        | Some(wire::DELETE)
        | Some(wire::OPEN_DIRECTORY)
        | Some(wire::NEXT_ENTRY)
        | Some(wire::EXISTS)
        | Some(wire::RANDOM_FILL)
        | Some(wire::EVAL) => wire::reply(wire::DENIED, &[]),
        _ => wire::reply(wire::INVALID, &[]),
    };
    if result.len() > dc as usize {
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
