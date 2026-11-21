use skrynia_native_service::wire::{AppPrecond, Request};
pub fn put(wc: &[u8], k: &[u8], ex: AppPrecond, v: &[u8]) -> Request {
    Request::AppPut {
        write_cap: wc.to_vec(),
        key: k.to_vec(),
        expect: ex,
        value: v.to_vec(),
    }
}
pub fn get(rc: &[u8], k: &[u8]) -> Request {
    Request::AppGet {
        read_cap: rc.to_vec(),
        key: k.to_vec(),
    }
}
pub fn del(wc: &[u8], k: &[u8], ex: Vec<u8>) -> Request {
    Request::AppDelete {
        write_cap: wc.to_vec(),
        key: k.to_vec(),
        expect: ex,
    }
}
