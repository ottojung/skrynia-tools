use super::super::host::Host;
use std::collections::HashMap;

#[derive(Default)]
pub struct Fake {
    pub input: Vec<u8>,
    pub verifier: [u8; 32],
    pub files: HashMap<u32, Vec<u8>>,
    pub next_ticket: u32,
    pub collisions: u8,
    pub provider: i32,
    pub trap: bool,
    pub deny_delete: bool,
    pub invokes: Vec<(bool, u32)>,
    pub written: Vec<Vec<u8>>,
    pub pos: usize,
    pub vpos: usize,
}

impl Fake {
    pub fn with_verifier(token: &[u8; 32]) -> Self {
        use sha2::{Digest, Sha256};
        Self {
            verifier: Sha256::digest(token).into(),
            ..Self::default()
        }
    }
}

impl Host for Fake {
    fn open_read(&mut self, _: &[u8]) -> Result<i32, i32> {
        Ok(7)
    }
    fn read(&mut self, fd: i32, dst: &mut [u8]) -> Result<usize, i32> {
        if fd == 7 {
            let left = self.verifier.len().saturating_sub(self.vpos);
            let n = left.min(dst.len());
            dst[..n].copy_from_slice(&self.verifier[self.vpos..self.vpos + n]);
            self.vpos += n;
            return Ok(n);
        }
        let remaining = self.input.len().saturating_sub(self.pos);
        let n = remaining.min(dst.len());
        dst[..n].copy_from_slice(&self.input[self.pos..self.pos + n]);
        self.pos += n;
        Ok(n)
    }
    fn close(&mut self, _: i32) -> Result<(), i32> {
        Ok(())
    }
    fn mkdir(&mut self, _: &[u8]) -> Result<(), i32> {
        Ok(())
    }
    fn random(&mut self) -> Result<u32, i32> {
        let t = self.next_ticket;
        self.next_ticket = self.next_ticket.wrapping_add(1);
        Ok(t)
    }
    fn create(&mut self, _: &[u8]) -> Result<i32, i32> {
        let t = self.next_ticket.wrapping_sub(1);
        if self.collisions > 0 || self.files.contains_key(&t) {
            self.collisions = self.collisions.saturating_sub(1);
            return Err(-3);
        }
        let fd = 100 + t as i32;
        Ok(fd)
    }
    fn write(&mut self, _: i32, bytes: &[u8]) -> Result<(), i32> {
        self.written.push(bytes.to_vec());
        Ok(())
    }
    fn sync(&mut self, _: i32) -> Result<(), i32> {
        Ok(())
    }
    fn delete(&mut self, _: &[u8]) -> Result<(), i32> {
        if self.deny_delete { Err(-5) } else { Ok(()) }
    }
    fn invoke(&mut self, publish: bool, ticket: u32) -> Option<i32> {
        if self.trap {
            return None;
        }
        self.invokes.push((publish, ticket));
        Some(self.provider)
    }
}
