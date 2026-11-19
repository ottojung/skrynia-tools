use super::wmut;
use super::{StateErr, bootstrap_root, cap};

pub fn handle_wire<D: cap::Disk>(disk: &D, req: crate::protocol::wire::Request) -> Vec<u8> {
    let ok = |b: Vec<u8>| {
        crate::protocol::wenc::encode_res(&crate::protocol::wire::Response {
            status: crate::protocol::wire::STATUS_OK,
            body: b,
        })
    };
    let err = |c: &str| {
        crate::protocol::wenc::encode_res(&crate::protocol::wire::Response {
            status: crate::protocol::wire::STATUS_ERR,
            body: c.as_bytes().to_vec(),
        })
    };
    match req {
        crate::protocol::wire::Request::V2Bootstrap { token } => match bootstrap_root(disk, &token)
        {
            Ok(()) => ok(Vec::new()),
            Err(StateErr::Exists) => err("already_exists"),
            Err(_) => err("state_uncertain"),
        },
        crate::protocol::wire::Request::V2Mutate {
            token,
            key,
            kind,
            expected,
            body,
        } => wmut::run(disk, token, key, kind, expected, body),
        _ => err("invalid_state"),
    }
}
