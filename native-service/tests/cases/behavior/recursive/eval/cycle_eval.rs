use super::{Arc, Eval, MemDisk, NoLock, Registry};

pub(crate) struct CycleEval {
    pub(crate) reg: Arc<Registry>,
    pub(crate) disk: Arc<MemDisk>,
    pub(crate) lock: Arc<NoLock>,
    pub(crate) a: Vec<u8>,
    pub(crate) b: Vec<u8>,
}

impl Eval for CycleEval {
    type Err = ();
    fn eval(
        &self,
        w: &[u8],
        i: &[u8],
        _cb: Option<&mut dyn FnMut(&[u8]) -> Result<Vec<u8>, i32>>,
    ) -> Result<Vec<u8>, ()> {
        match w {
            b"A" => self
                .reg
                .invoke(&*self.disk, &*self.lock, self, &self.b, i)
                .map_err(|_| ()),
            b"B" if !i.is_empty() => self
                .reg
                .invoke(&*self.disk, &*self.lock, self, &self.a, &i[1..])
                .map_err(|_| ()),
            _ => Ok(b"done".to_vec()),
        }
    }
}
