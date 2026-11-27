use skrynia_tools::sdk::ByteIo;

#[derive(Default)]
pub struct FakeIo {
    pub input: Vec<u8>,
    pub at: usize,
    pub written: Vec<u8>,
    pub fail_write: bool,
}

impl ByteIo for FakeIo {
    fn read1(&mut self, _fd: i32) -> i32 {
        match self.input.get(self.at) {
            Some(byte) => {
                self.at += 1;
                i32::from(*byte)
            }
            None => -1,
        }
    }
    fn write1(&mut self, _fd: i32, byte: u32) -> i32 {
        if self.fail_write {
            return -2;
        }
        self.written.push(byte as u8);
        0
    }
    fn seek(&mut self, _fd: i32, shift: i64) -> i64 {
        if shift < 0 {
            return -4;
        }
        let at = (self.at as i64)
            .saturating_add(shift)
            .min(self.input.len() as i64);
        self.at = at as usize;
        at
    }
}
