use super::{call, wire};
pub struct CbLock;
impl crate::MutLock for CbLock {
    type Err = i32;
    fn try_acquire(&self) -> Result<bool, i32> {
        let (n, _) = call::call(&[wire::TRY_LOCK])?;
        match n {
            1 => Ok(true),
            0 => Ok(false),
            _ if n < 0 => Err(n),
            _ => Err(-1),
        }
    }
    fn release(&self) -> Result<(), i32> {
        let (n, _) = call::call(&[wire::RELEASE_LOCK])?;
        if n < 0 { Err(n) } else { Ok(()) }
    }
}
pub struct CbRandom;
impl crate::cap::Random for CbRandom {
    type Err = i32;
    fn fill(&self, out: &mut [u8]) -> Result<(), i32> {
        let bits = (out.len() as u32).saturating_mul(8);
        let mut req = vec![wire::RANDOM];
        wire::put_u32(&mut req, bits);
        let (n, b) = call::call(&req)?;
        if n < 0 {
            return Err(n);
        }
        if b.len() < out.len() {
            return Err(-1);
        }
        out.copy_from_slice(&b[..out.len()]);
        Ok(())
    }
}
