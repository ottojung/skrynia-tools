use skrynia_program0::cap::Eval;

pub struct GuardEval;
impl Eval for GuardEval {
    type Err = &'static str;
    fn eval(
        &self,
        w: &[u8],
        _: &[u8],
        _cb: Option<&mut dyn FnMut(&[u8]) -> Result<Vec<u8>, i32>>,
    ) -> Result<Vec<u8>, Self::Err> {
        if w == b"UNSAFE_IMPORT" {
            Err("unsafe import")
        } else {
            Ok(w.to_vec())
        }
    }
}
