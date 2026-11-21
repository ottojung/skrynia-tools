use std::io::Read;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

pub fn hang_first_then_hello() -> (SocketAddr, Arc<AtomicBool>, Arc<AtomicUsize>) {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = l.local_addr().unwrap();
    let stop = Arc::new(AtomicBool::new(false));
    let count = Arc::new(AtomicUsize::new(0));
    let f = stop.clone();
    let c = count.clone();
    thread::spawn(move || {
        l.set_nonblocking(true).ok();
        let mut held = Vec::new();
        while !f.load(Ordering::Relaxed) {
            if let Ok((s, _)) = l.accept() {
                let n = c.fetch_add(1, Ordering::Relaxed) + 1;
                if n == 1 {
                    s.set_nonblocking(false).ok();
                    held.push(s);
                } else {
                    respond_ok(s);
                }
            }
            thread::sleep(Duration::from_millis(10));
        }
        drop(held);
    });
    (addr, stop, count)
}

fn respond_ok(mut s: TcpStream) {
    thread::spawn(move || {
        let mut buf = [0u8; 4096];
        let _ = s.set_read_timeout(Some(Duration::from_secs(2)));
        let _ = s.read(&mut buf);
        let _ = std::io::Write::write_all(
            &mut s,
            b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok",
        );
    });
}
