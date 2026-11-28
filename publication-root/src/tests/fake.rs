use std::collections::HashMap;

#[derive(Default)]
pub struct Fake {
    pub input: Vec<u8>,
    pub verifier: [u8; 32],
    pub verifier_extra: Vec<u8>,
    pub fail_random: bool,
    pub files: HashMap<u32, Vec<u8>>,
    pub next_ticket: u32,
    pub collisions: u8,
    pub collided: Vec<Vec<u8>>,
    pub open_error: Option<i32>,
    pub provider: i32,
    pub trap: bool,
    pub deny_delete: bool,
    pub mkdir_fail: Option<i32>,
    pub mkdirs: Vec<Vec<u8>>,
    pub fail_write: bool,
    pub fail_close: bool,
    pub invokes: Vec<(bool, u32)>,
    pub written: Vec<Vec<u8>>,
    pub deleted: Vec<Vec<u8>>,
    pub pos: usize,
    pub vpos: usize,
}

impl Fake {
    pub(super) fn open_error(&mut self, name: &[u8]) -> Option<i32> {
        if let Some(code) = self.open_error {
            return Some(code);
        }
        if self.collisions == 0 {
            return None;
        }
        self.collisions -= 1;
        self.collided.push(name.to_vec());
        Some(-5)
    }
    pub(super) fn exists(&self, name: &[u8]) -> bool {
        self.collided.iter().any(|p| p == name)
    }
    pub(super) fn write(&mut self, bytes: &[u8]) -> Result<(), i32> {
        if self.fail_write {
            return Err(-5);
        }
        self.written.push(bytes.to_vec());
        Ok(())
    }
}
