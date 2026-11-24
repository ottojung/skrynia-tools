//! Small FIFO helper so containment tests stay under structural line limits.

mod roots;

use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::FileTypeExt;
use std::path::Path;

pub fn mkfifo(path: &Path) {
    let name = CString::new(path.as_os_str().as_bytes()).expect("cstr");
    let rc = unsafe { libc::mkfifo(name.as_ptr(), 0o600) };
    assert_eq!(rc, 0, "mkfifo failed");
    assert!(
        std::fs::symlink_metadata(path)
            .expect("meta")
            .file_type()
            .is_fifo()
    );
}
