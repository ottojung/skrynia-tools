use super::{Fake, Host};

impl Host for Fake {
    fn open(&mut self, path: &[u8], mode: u32) -> Result<i32, i32> {
        if mode == 1 {
            return Ok(7);
        }
        if let Some(code) = self.open_error(path) {
            return Err(code);
        }
        Ok(100 + self.next_ticket.wrapping_sub(1) as i32)
    }
    fn exists(&mut self, path: &[u8]) -> bool {
        Fake::exists(self, path)
    }
    fn read(&mut self, fd: i32, dst: &mut [u8]) -> Result<usize, i32> {
        if fd == 7 {
            let verifier = [self.verifier.as_slice(), self.verifier_extra.as_slice()].concat();
            return Ok(super::take(&verifier, &mut self.vpos, dst));
        }
        Ok(super::take(&self.input, &mut self.pos, dst))
    }
    fn close(&mut self, _: i32) -> Result<(), i32> {
        if self.fail_close { Err(-5) } else { Ok(()) }
    }
    fn mkdir(&mut self, path: &[u8]) -> Result<(), i32> {
        self.mkdirs.push(path.to_vec());
        self.mkdir_fail.map_or(Ok(()), Err)
    }
    fn random(&mut self) -> Result<u32, i32> {
        if self.fail_random {
            return Err(-5);
        }
        let t = self.next_ticket;
        self.next_ticket = t.wrapping_add(1);
        Ok(t)
    }
    fn write(&mut self, _: i32, bytes: &[u8]) -> Result<(), i32> {
        Fake::write(self, bytes)
    }
    fn delete(&mut self, path: &[u8]) -> Result<(), i32> {
        self.deleted.push(path.to_vec());
        if self.deny_delete { Err(-5) } else { Ok(()) }
    }
    fn invoke(&mut self, publish: bool, ticket: u32) -> Option<i32> {
        super::record_invoke(self, publish, ticket)
    }
}
