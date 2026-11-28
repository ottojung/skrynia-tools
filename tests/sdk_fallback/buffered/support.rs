use super::BulkIo;
pub(super) struct Io {
    input: Vec<u8>,
    pub(super) at: usize,
    pub(super) out: Vec<u8>,
    fail_after: usize,
}
impl BulkIo for Io {
    fn read(&mut self, _: i32, b: &mut [u8]) -> i64 {
        let n = b.len().min(self.input.len() - self.at).min(3);
        b[..n].copy_from_slice(&self.input[self.at..self.at + n]);
        self.at += n;
        n as i64
    }
    fn write(&mut self, _: i32, b: &[u8]) -> i64 {
        if self.out.len() >= self.fail_after {
            return -5;
        }
        let n = b.len().min(2).min(self.fail_after - self.out.len());
        self.out.extend_from_slice(&b[..n]);
        n as i64
    }
}
pub(super) fn io(fail_after: usize) -> Io {
    Io {
        input: b"abcdef".to_vec(),
        at: 0,
        out: vec![],
        fail_after,
    }
}
