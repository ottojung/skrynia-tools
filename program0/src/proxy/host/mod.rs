use super::abi;
mod close;
mod dir;
mod io;
mod misc;
mod open;
mod path;
mod scope;
#[cfg(test)]
mod scope_tests;
mod wire;
pub use scope::PREFIX;

fn run(req: &[u8], dc: usize) -> Result<Vec<u8>, i32> {
    if dc < 4 || req.len() > wire::LIMIT + 16 {
        return Err(wire::OVERFLOW);
    }
    let result = match req.first().copied() {
        Some(wire::OPEN_DISK) => open::call(req),
        Some(wire::READ) => io::read(req, dc),
        Some(wire::WRITE) => io::write(req),
        Some(wire::CLOSE) => close::call(req),
        Some(wire::MKDIR) => path::mkdir(req),
        Some(wire::DELETE) => path::delete(req),
        Some(wire::RENAME) => path::rename(req),
        Some(wire::OPEN_DIR) => dir::open(req),
        Some(wire::NEXT_ENTRY) => dir::next(req, dc),
        Some(wire::SYNC) => misc::sync(req),
        Some(wire::RANDOM) => misc::random(req),
        Some(wire::TRY_LOCK) => misc::try_lock(req),
        Some(wire::RELEASE_LOCK) => misc::release_lock(req),
        _ => wire::reply(wire::INVALID, &[]),
    };
    if result.len() > dc {
        return Err(wire::OVERFLOW);
    }
    Ok(result)
}

pub fn dispatch(req: &[u8], dc: usize) -> Vec<u8> {
    match run(req, dc) {
        Ok(v) => v,
        Err(code) => code.to_le_bytes().to_vec(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn skrynia_host_cb(rp: u32, rl: u32, dp: u32, dc: u32) -> i32 {
    let req = unsafe { core::slice::from_raw_parts(rp as *const u8, rl as usize) };
    let result = dispatch(req, dc as usize);
    unsafe { core::ptr::copy_nonoverlapping(result.as_ptr(), dp as *mut u8, result.len()) };
    result.len() as i32
}

#[used]
static CALLBACK: extern "C" fn(u32, u32, u32, u32) -> i32 = skrynia_host_cb;
pub fn callback_index() -> i32 {
    skrynia_host_cb as *const () as usize as i32
}
