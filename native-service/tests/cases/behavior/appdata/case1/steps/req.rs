use skrynia_native_service::wire::{AppPrecond, Request};

pub fn put_req(wc: &[u8], expect: AppPrecond, val: &[u8]) -> Request {
    Request::AppPut {
        write_cap: wc.to_vec(),
        key: b"nested/key".to_vec(),
        expect,
        value: val.to_vec(),
    }
}
pub fn get_req(rc: &[u8]) -> Request {
    Request::AppGet {
        read_cap: rc.to_vec(),
        key: b"nested/key".to_vec(),
    }
}
pub fn del_req(wc: &[u8], expect: Vec<u8>) -> Request {
    Request::AppDelete {
        write_cap: wc.to_vec(),
        key: b"nested/key".to_vec(),
        expect,
    }
}
