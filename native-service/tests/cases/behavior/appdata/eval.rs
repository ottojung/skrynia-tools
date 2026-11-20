use skrynia_native_service::cap::Eval;
use skrynia_native_service::wdec::decode_res;
use skrynia_native_service::wenc::encode_req;
use skrynia_native_service::wire::{AppPrecond, Request, STATUS_ERR, STATUS_OK};

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
        let me = i.to_vec(); // invoke_id (self namespace)
        // read own key (should exist with value "v")
        let get = encode_req(&Request::AppGet {
            read_cap: me.clone(),
            key: b"k".to_vec(),
        });
        let v = decode_res(&cb(&get).map_err(|_| ())?).map_err(|_| ())?;
        if v.status != STATUS_OK {
            return Err(());
        }
        // conditional write own key: expect current hash, write v+cb
        let h = skrynia_native_service::auth::hash_bytes(&v.body).to_vec();
        let put = encode_req(&Request::AppPut {
            write_cap: me.clone(),
            key: b"k".to_vec(),
            expect: AppPrecond::Present(h),
            value: [v.body.as_slice(), b"+cb"].concat(),
        });
        let p = decode_res(&cb(&put).map_err(|_| ())?).map_err(|_| ())?;
        if p.status != STATUS_OK {
            return Err(());
        }
        // cross-app authority: try to read a different namespace
        let bad = encode_req(&Request::AppGet {
            read_cap: vec![9u8; 32],
            key: b"k".to_vec(),
        });
        let b = decode_res(&cb(&bad).map_err(|_| ())?).map_err(|_| ())?;
        if b.status != STATUS_ERR {
            return Err(());
        }
        Ok(b"done".to_vec())
    }
}
