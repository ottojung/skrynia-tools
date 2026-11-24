use super::Host;
use std::collections::HashMap;

#[derive(Default)]
pub struct Fake {
    pub collisions: u8,
    pub next_ticket: u32,
    pub files: HashMap<u32, Vec<u8>>,
    pub fds: HashMap<i32, u32>,
    pub invokes: Vec<(bool, u32)>,
    pub provider: i32,
    pub written: Vec<Vec<u8>>,
}

impl Host for Fake {
    fn random_u32(&mut self) -> Result<u32, i32> {
        let t = self.next_ticket;
        self.next_ticket = self.next_ticket.wrapping_add(1);
        Ok(t)
    }
    fn create_request(&mut self, ticket: u32) -> Result<i32, i32> {
        if self.collisions > 0 || self.files.contains_key(&ticket) {
            self.collisions = self.collisions.saturating_sub(1);
            return Err(-3);
        }
        self.files.insert(ticket, Vec::new());
        let fd = 100 + ticket as i32;
        self.fds.insert(fd, ticket);
        Ok(fd)
    }
    fn write_all(&mut self, fd: i32, bytes: &[u8]) -> Result<(), i32> {
        let Some(t) = self.fds.get(&fd).copied() else {
            return Err(-7);
        };
        self.files.entry(t).or_default().extend_from_slice(bytes);
        self.written.push(bytes.to_vec());
        Ok(())
    }
    fn finish(&mut self, _: i32) -> Result<(), i32> {
        Ok(())
    }
    fn remove_request(&mut self, ticket: u32) -> Result<(), i32> {
        self.files.remove(&ticket);
        Ok(())
    }
    fn invoke(&mut self, publish: bool, ticket: u32) -> i32 {
        self.invokes.push((publish, ticket));
        self.provider
    }
}
