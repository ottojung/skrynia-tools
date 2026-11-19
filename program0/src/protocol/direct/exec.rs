use super::{Disk, Eval, err, ok, store_io};
pub fn invoke<D: Disk, E: Eval>(disk: &D, eval: &E, id: &[u8], input: &[u8]) -> Vec<u8> {
    let code = match store_io::get(disk, "programs", id) {
        Ok(code) => code,
        Err(_) => return err(4),
    };
    match eval.eval(&code, input) {
        Ok(output) => ok(output),
        Err(_) => err(4),
    }
}
