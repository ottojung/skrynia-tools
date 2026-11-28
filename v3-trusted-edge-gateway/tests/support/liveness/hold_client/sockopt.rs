use std::net::TcpStream;

pub(super) fn set_small_rcvbuf(s: &TcpStream) {
    #[cfg(unix)]
    {
        use std::os::unix::io::AsRawFd;
        let buf: libc::c_int = 4096;
        // SAFETY: the fd is valid while `s` lives; `buf` is a valid c_int.
        let rc = unsafe {
            libc::setsockopt(
                s.as_raw_fd(),
                libc::SOL_SOCKET,
                libc::SO_RCVBUF,
                &buf as *const _ as *const libc::c_void,
                std::mem::size_of_val(&buf) as libc::socklen_t,
            )
        };
        assert_eq!(
            rc,
            0,
            "setsockopt(SO_RCVBUF) failed: {}",
            std::io::Error::last_os_error()
        );
    }
    #[cfg(not(unix))]
    let _ = s;
}
