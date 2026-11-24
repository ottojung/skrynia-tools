#[cfg(target_arch = "wasm32")]
pub struct Abi;
#[cfg(target_arch = "wasm32")]
pub(super) use super::abi;
#[cfg(target_arch = "wasm32")]
mod fs;
#[cfg(target_arch = "wasm32")]
mod imp;
#[cfg(target_arch = "wasm32")]
mod ops;

/// Root-authority boundary. No evaluated child ever sees these operations.
pub trait Host {
    fn open(&mut self, path: &[u8], mode: u32) -> Result<i32, i32>;
    fn read(&mut self, fd: i32, dst: &mut [u8]) -> Result<usize, i32>;
    fn close(&mut self, fd: i32) -> Result<(), i32>;
    fn mkdir(&mut self, path: &[u8]) -> Result<(), i32>;
    fn random(&mut self) -> Result<u32, i32>;
    fn write(&mut self, fd: i32, bytes: &[u8]) -> Result<(), i32>;
    fn delete(&mut self, path: &[u8]) -> Result<(), i32>;
    fn invoke(&mut self, publish: bool, ticket: u32) -> Option<i32>;
}
