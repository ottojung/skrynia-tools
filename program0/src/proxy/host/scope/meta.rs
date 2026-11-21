pub mod op {
    pub const OPEN_DISK: u32 = 1 << 0;
    pub const READ: u32 = 1 << 1;
    pub const WRITE: u32 = 1 << 2;
    pub const CLOSE: u32 = 1 << 3;
    pub const SYNC: u32 = 1 << 4;
    pub const MKDIR: u32 = 1 << 5;
    pub const RENAME: u32 = 1 << 6;
    pub const DELETE: u32 = 1 << 7;
    pub const OPEN_DIR: u32 = 1 << 8;
    pub const NEXT: u32 = 1 << 9;
    pub const EXISTS: u32 = 1 << 10;
    pub const RANDOM: u32 = 1 << 11;
    pub const ALL: u32 = (1 << 12) - 1;
}
#[derive(Clone, Copy)]
pub struct Policy {
    pub ops: u32,
    pub prefix_len: u16,
    pub prefix: [u8; 192],
}
impl Policy {
    pub fn prefix(&self) -> &[u8] {
        &self.prefix[..self.prefix_len as usize]
    }
    pub fn allows(&self, bit: u32) -> bool {
        self.ops & bit == bit
    }
}
