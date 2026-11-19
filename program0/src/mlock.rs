pub trait MutLock {
    type Err;
    fn try_acquire(&self) -> Result<bool, Self::Err>;
    fn release(&self) -> Result<(), Self::Err>;
}

#[derive(Default)]
pub struct NoLock;
impl MutLock for NoLock {
    type Err = ();
    fn try_acquire(&self) -> Result<bool, ()> {
        Ok(true)
    }
    fn release(&self) -> Result<(), ()> {
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub struct StdLock(std::sync::Mutex<bool>);
#[cfg(not(target_arch = "wasm32"))]
impl Default for StdLock {
    fn default() -> Self {
        Self(std::sync::Mutex::new(false))
    }
}
#[cfg(not(target_arch = "wasm32"))]
impl MutLock for StdLock {
    type Err = ();
    fn try_acquire(&self) -> Result<bool, ()> {
        let mut g = self.0.lock().unwrap_or_else(|e| e.into_inner());
        if *g {
            Ok(false)
        } else {
            *g = true;
            Ok(true)
        }
    }
    fn release(&self) -> Result<(), ()> {
        let mut g = self.0.lock().unwrap_or_else(|e| e.into_inner());
        *g = false;
        Ok(())
    }
}
