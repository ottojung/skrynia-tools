use std::io::Read;
use std::process::Child;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use super::wait::StderrBuf;

const STDERR_CAPTURE_MAX: usize = 64 * 1024;

pub struct StderrDrain {
    pub buf: StderrBuf,
    handle: Option<JoinHandle<()>>,
}

impl StderrDrain {
    pub fn snapshot(&self) -> String {
        super::wait::stderr_snapshot(&self.buf)
    }
    pub fn finish(&mut self) {
        let Some(h) = self.handle.take() else { return };
        for _ in 0..50 {
            if h.is_finished() {
                let _ = h.join();
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        drop(h);
    }
}

pub fn spawn_stderr_drain(child: &mut Child) -> StderrDrain {
    let buf: StderrBuf = Arc::new(Mutex::new(Vec::new()));
    let Some(mut stderr) = child.stderr.take() else {
        return StderrDrain { buf, handle: None };
    };
    let sink = Arc::clone(&buf);
    let handle = std::thread::spawn(move || {
        let mut chunk = [0u8; 4096];
        loop {
            match stderr.read(&mut chunk) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    let mut g = sink.lock().unwrap_or_else(|e| e.into_inner());
                    let remaining = STDERR_CAPTURE_MAX.saturating_sub(g.len());
                    if remaining > 0 {
                        g.extend_from_slice(&chunk[..n.min(remaining)]);
                    }
                }
            }
        }
    });
    StderrDrain { buf, handle: Some(handle) }
}
