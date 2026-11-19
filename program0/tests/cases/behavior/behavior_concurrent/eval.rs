use skrynia_program0::cap::Eval;

pub struct TagEval;
impl Eval for TagEval {
    type Err = ();
    fn eval(
        &self,
        w: &[u8],
        _: &[u8],
        _cb: Option<&mut dyn FnMut(&[u8]) -> Result<Vec<u8>, i32>>,
    ) -> Result<Vec<u8>, ()> {
        Ok(w.to_vec())
    }
}
