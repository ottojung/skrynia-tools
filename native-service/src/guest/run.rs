pub fn handle(input_fd: i32, output_fd: i32) -> i32 {
    let result = run(input_fd, output_fd);
    unsafe {
        super::host::root_abi::close(input_fd);
        super::host::root_abi::close(output_fd);
    }
    match result {
        Ok(()) => 0,
        Err(code) => code,
    }
}
fn run(input_fd: i32, output_fd: i32) -> Result<(), i32> {
    for dir in [
        "programs",
        "programs/active",
        "programs/blob",
        "programs/ready",
        "programs/appdata",
    ] {
        super::host::root_disk_ls::ensure(dir)?;
    }
    let disk = super::host::root_disk::HostDisk;
    let lock = super::host::root_eval::HostLock;
    let registry = super::reg::Registry::open(&disk, "programs").map_err(|_| -5)?;
    let req = super::host::root_stream::read_all(input_fd)?;
    let response = super::handle::handle(
        &registry,
        &disk,
        &lock,
        &super::host::root_random::HostRandom,
        &super::host::root_eval::HostEval,
        &req,
    );
    super::host::root_stream::write_all(output_fd, &response)
}
