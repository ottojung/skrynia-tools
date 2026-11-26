use super::{Fake, Host};

impl Host for Fake {
    fn open(&mut self, _: &[u8], mode: u32) -> Result<i32, i32> {
        if mode == 1 {
            return Ok(7);
        }
        let ticket = self.next_ticket.wrapping_sub(1);
        if self.collisions > 0 {
            self.collisions -= 1;
            return Err(-3);
        }
        Ok(100 + ticket as i32)
    }
    fn read(&mut self, fd: i32, dst: &mut [u8]) -> Result<usize, i32> {
        if fd == 7 {
            let mut verifier = self.verifier.to_vec();
            verifier.extend_from_slice(&self.verifier_extra);
            return Ok(super::take(&verifier, &mut self.vpos, dst));
        }
        Ok(super::take(&self.input, &mut self.pos, dst))
    }
    fn close(&mut self, _: i32) -> Result<(), i32> {
        if self.fail_close { Err(-5) } else { Ok(()) }
    }
    fn mkdir(&mut self, _: &[u8]) -> Result<(), i32> {
        Ok(())
    }
    fn random(&mut self) -> Result<u32, i32> {
        if self.fail_random {
            return Err(-5);
        }
        let ticket = self.next_ticket;
        self.next_ticket = self.next_ticket.wrapping_add(1);
        Ok(ticket)
    }
    fn write(&mut self, _: i32, bytes: &[u8]) -> Result<(), i32> {
        if self.fail_write {
            return Err(-5);
        }
        self.written.push(bytes.to_vec());
        Ok(())
    }
    fn delete(&mut self, path: &[u8]) -> Result<(), i32> {
        self.deleted.push(path.to_vec());
        if self.deny_delete { Err(-5) } else { Ok(()) }
    }
    fn invoke(&mut self, publish: bool, ticket: u32) -> Option<i32> {
        (!self.trap).then(|| {
            self.invokes.push((publish, ticket));
            self.provider
        })
    }
}
