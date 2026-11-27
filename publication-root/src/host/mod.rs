#[cfg(target_arch = "wasm32")]
pub struct Abi {
    staging: stage::Staging,
}

#[cfg(target_arch = "wasm32")]
impl Abi {
    pub const fn new() -> Self {
        Self {
            staging: stage::Staging::new(),
        }
    }

    pub fn reset(&mut self) {
        self.staging.reset();
    }
}

#[cfg(target_arch = "wasm32")]
pub(crate) use self::stage::Staging;
#[cfg(target_arch = "wasm32")]
pub(super) use super::abi;
#[cfg(target_arch = "wasm32")]
mod fs;
#[cfg(target_arch = "wasm32")]
mod imp;
#[cfg(target_arch = "wasm32")]
mod ops;
#[cfg(any(target_arch = "wasm32", test))]
mod ram;
#[cfg(test)]
mod ram_tests;
#[cfg(target_arch = "wasm32")]
mod stage;
#[cfg(target_arch = "wasm32")]
mod stage_claim;
#[cfg(target_arch = "wasm32")]
mod stage_io;
#[cfg(target_arch = "wasm32")]
mod stage_seek;

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
