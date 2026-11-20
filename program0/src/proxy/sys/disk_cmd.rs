use super::disk::{allowed, raw_read};
use super::disk_io::{raw_list, raw_remove, raw_write};
use super::proto;
pub fn read_cmd(req: &[u8]) -> Vec<u8> {
    let mut i = 1usize;
    let Some(path) = proto::get_str(req, &mut i) else {
        return proto::err(-1);
    };
    if !allowed(&path) {
        return proto::err(-4);
    }
    match raw_read(&path) {
        Ok(None) => vec![proto::OK, proto::NONE],
        Ok(Some(b)) => {
            let mut r = vec![proto::OK, proto::SOME];
            proto::put_bytes(&mut r, &b);
            r
        }
        Err(e) => proto::err(e),
    }
}
pub fn write_cmd(req: &[u8]) -> Vec<u8> {
    let mut i = 1usize;
    let (Some(path), Some(bytes)) = (proto::get_str(req, &mut i), proto::get_bytes(req, &mut i))
    else {
        return proto::err(-1);
    };
    if !allowed(&path) {
        return proto::err(-4);
    }
    match raw_write(&path, &bytes) {
        Ok(()) => proto::ok_empty(),
        Err(e) => proto::err(e),
    }
}
pub fn remove_cmd(req: &[u8]) -> Vec<u8> {
    let mut i = 1usize;
    let Some(path) = proto::get_str(req, &mut i) else {
        return proto::err(-1);
    };
    if !allowed(&path) {
        return proto::err(-4);
    }
    match raw_remove(&path) {
        Ok(()) => proto::ok_empty(),
        Err(e) => proto::err(e),
    }
}
pub fn list_cmd(req: &[u8]) -> Vec<u8> {
    let mut i = 1usize;
    let Some(dir) = proto::get_str(req, &mut i) else {
        return proto::err(-1);
    };
    if !allowed(&dir) {
        return proto::err(-4);
    }
    match raw_list(&dir) {
        Ok(names) => {
            let mut r = vec![proto::OK];
            proto::put_u32(&mut r, names.len() as u32);
            for n in names {
                proto::put_bytes(&mut r, n.as_bytes());
            }
            r
        }
        Err(e) => proto::err(e),
    }
}
