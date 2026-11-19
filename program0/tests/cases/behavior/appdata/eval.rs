use skrynia_program0::cap::Eval;
use skrynia_program0::wdec::decode_res;
use skrynia_program0::wenc::encode_req;
use skrynia_program0::wire::{Request, STATUS_ERR, STATUS_OK};

pub struct GuestEval;
impl Eval for GuestEval {
    type Err = ();
    fn eval(
        &self,
        w: &[u8],
        i: &[u8],
        cb: Option<&mut dyn FnMut(&[u8]) -> Result<Vec<u8>, i32>>,
    ) -> Result<Vec<u8>, ()> {
        if w != b"guest" {
            return Ok(w.to_vec());
        }
        let cb = cb.ok_or(())?;
        let me = i.to_vec();
        let get = encode_req(&Request::AppGet {
            app: me.clone(),
            key: b"k".to_vec(),
        });
        let v = decode_res(&cb(&get).map_err(|_| ())?).map_err(|_| ())?;
        assert_eq!(v.status, STATUS_OK);
        let put = encode_req(&Request::AppPut {
            app: me.clone(),
            key: b"k".to_vec(),
            value: [v.body.as_slice(), b"+cb"].concat(),
        });
        let p = decode_res(&cb(&put).map_err(|_| ())?).map_err(|_| ())?;
        assert_eq!(p.status, STATUS_OK);
        let bad = encode_req(&Request::AppGet {
            app: vec![9u8; 32],
            key: b"k".to_vec(),
        });
        let b = decode_res(&cb(&bad).map_err(|_| ())?).map_err(|_| ())?;
        assert_eq!(b.status, STATUS_ERR);
        Ok(b"done".to_vec())
    }
}
