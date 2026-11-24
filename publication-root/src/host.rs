#[cfg(target_arch = "wasm32")]
pub struct Abi;

/// Root-authority boundary. No evaluated child ever sees these operations.
pub trait Host {
    fn open_read(&mut self, path: &[u8]) -> Result<i32, i32>;
    fn read(&mut self, fd: i32, dst: &mut [u8]) -> Result<usize, i32>;
    fn close(&mut self, fd: i32) -> Result<(), i32>;
    fn mkdir(&mut self, path: &[u8]) -> Result<(), i32>;
    fn random(&mut self) -> Result<u32, i32>;
    fn create(&mut self, path: &[u8]) -> Result<i32, i32>;
    fn write(&mut self, fd: i32, bytes: &[u8]) -> Result<(), i32>;
    fn sync(&mut self, fd: i32) -> Result<(), i32>;
    fn delete(&mut self, path: &[u8]) -> Result<(), i32>;
    fn invoke(&mut self, publish: bool, ticket: u32) -> Option<i32>;
}
