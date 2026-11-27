#[cfg(target_arch = "wasm32")]
pub struct Abi {
    staging: buf::Staging,
}

#[cfg(target_arch = "wasm32")]
impl Abi {
    pub const fn new() -> Self {
        Self {
            staging: buf::Staging::new(),
        }
    }

    pub fn reset(&mut self) {
        self.staging.reset();
    }
}

#[cfg(target_arch = "wasm32")]
pub(super) use self::buf::Staging;
#[cfg(target_arch = "wasm32")]
pub(super) use self::transfer::{read_all, write_all};
#[cfg(target_arch = "wasm32")]
pub(super) use super::abi;
#[cfg(target_arch = "wasm32")]
mod buf;
#[cfg(target_arch = "wasm32")]
mod fs;
#[cfg(target_arch = "wasm32")]
mod imp;
#[cfg(target_arch = "wasm32")]
mod ops;
mod transfer;
#[cfg(test)]
mod transfer_tests;

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
