#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn skrynia_main(input_fd: i32, out_ptr: u32, out_cap: u32) -> i32 {
    use super::{handle, reg::Registry, root_disk::HostDisk, root_eval::HostEval};
    use super::{root_disk_ls, root_random::HostRandom};
    if root_disk_ls::ensure("programs").is_err() {
        return -5;
    }
    let disk = HostDisk;
    let registry = match Registry::open(&disk, "programs") {
        Ok(v) => v,
        Err(_) => return -5,
    };
    let req = match super::root_input::read_all(input_fd) {
        Ok(v) => v,
        Err(code) => return code,
    };
    let _ = unsafe { super::root_abi::close(input_fd) };
    let res = handle::handle(&registry, &disk, &HostRandom, &HostEval, &req);
    if res.len() > out_cap as usize {
        return -11;
    }
    if !res.is_empty() {
        unsafe { std::ptr::copy_nonoverlapping(res.as_ptr(), out_ptr as *mut u8, res.len()) };
    }
    res.len() as i32
}
