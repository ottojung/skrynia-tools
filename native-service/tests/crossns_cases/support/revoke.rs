use super::Mem;
use skrynia_native_service::MutLock;
pub struct Revoke<'a>(pub &'a Mem, pub String, pub Vec<u8>);
impl MutLock for Revoke<'_> {
    type Err = ();
    fn try_acquire(&self) -> Result<bool, ()> {
        self.0
            .0
            .lock()
            .unwrap()
            .insert(self.1.clone(), self.2.clone());
        Ok(true)
    }
    fn release(&self) -> Result<(), ()> {
        Ok(())
    }
}
