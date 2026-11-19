#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn skrynia_main(input_fd: i32, output_fd: i32) -> i32 {
    use super::root_disk::HostDisk;
    use super::root_disk_ls;
    use super::root_eval::HostEval;
    use super::root_eval::HostLock;
    use super::root_random::HostRandom;
    use super::{handle, reg::Registry};
    for d in [
        "programs",
        "programs/active",
        "programs/blob",
        "programs/appdata",
    ] {
        if root_disk_ls::ensure(d).is_err() {
            return -5;
        }
    }
    let disk = HostDisk;
    let lock = HostLock;
    let registry = match Registry::open(&disk, "programs") {
        Ok(v) => v,
        Err(_) => return -5,
    };
    let req = match super::root_stream::read_all(input_fd) {
        Ok(v) => v,
        Err(code) => return code,
    };
    let _ = unsafe { super::root_abi::close(input_fd) };
    let res = handle::handle(&registry, &disk, &lock, &HostRandom, &HostEval, &req);
    let result = super::root_stream::write_all(output_fd, &res);
    let _ = unsafe { super::root_abi::close(output_fd) };
    match result {
        Ok(()) => 0,
        Err(code) => code,
    }
}
