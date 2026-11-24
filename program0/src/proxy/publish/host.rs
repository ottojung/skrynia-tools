/// Deterministic boundary for the operator-disk mailbox and named syscalls.
/// On wasm32 `Abi` implements this with host imports; tests inject a fake.
pub trait Host {
    fn random_u32(&mut self) -> Result<u32, i32>;
    fn create_request(&mut self, ticket: u32) -> Result<i32, i32>;
    fn write_all(&mut self, fd: i32, bytes: &[u8]) -> Result<(), i32>;
    fn finish(&mut self, fd: i32) -> Result<(), i32>;
    fn remove_request(&mut self, ticket: u32) -> Result<(), i32>;
    fn invoke(&mut self, publish: bool, ticket: u32) -> i32;
}
