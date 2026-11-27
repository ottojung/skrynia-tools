impl super::Gateway {
    pub fn stderr_text(&mut self) -> String {
        let _ = self.child.kill();
        let _ = self.child.wait();
        self.stderr.finish();
        self.stderr.snapshot()
    }
}
impl Drop for super::Gateway {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        self.stderr.finish();
        if std::thread::panicking() {
            let stderr = self.stderr.snapshot();
            if !stderr.trim().is_empty() {
                eprintln!("--- gateway stderr ({}) ---\n{stderr}", self.addr);
            }
        }
    }
}
