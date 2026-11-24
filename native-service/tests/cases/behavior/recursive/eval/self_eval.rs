use super::{Arc, Eval, MemDisk, NoLock, Registry};

pub(crate) struct SelfEval {
    pub(crate) reg: Arc<Registry>,
    pub(crate) disk: Arc<MemDisk>,
    pub(crate) lock: Arc<NoLock>,
    pub(crate) me: Vec<u8>,
}

impl Eval for SelfEval {
    type Err = ();
    fn eval(
        &self,
        w: &[u8],
        i: &[u8],
        _cb: Option<&mut dyn FnMut(&[u8]) -> Result<Vec<u8>, i32>>,
    ) -> Result<Vec<u8>, ()> {
        if w == b"self" && !i.is_empty() {
            self.reg
                .invoke(&*self.disk, &*self.lock, self, &self.me, &i[1..])
                .map_err(|_| ())
        } else {
            Ok(vec![i.len() as u8])
        }
    }
}
