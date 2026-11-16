use std::{
    os::unix::process::CommandExt,
    process::{Child, Command},
};

/// Give a subprocess its own process group so timeout cleanup reaches descendants.
pub fn isolate(cmd: &mut Command) {
    cmd.process_group(0);
}
pub fn terminate(child: &mut Child) {
    if let Ok(pid) = i32::try_from(child.id()) {
        // Negative PID targets the isolated child process group, never our own.
        unsafe {
            libc::kill(-pid, libc::SIGKILL);
        }
    }
    let _ = child.kill();
    let _ = child.wait();
}
