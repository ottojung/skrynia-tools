use super::{MutateKind, MutateReq, cap};

pub fn run<D: cap::Disk>(
    disk: &D,
    token: [u8; 32],
    key: Vec<u8>,
    kind: crate::protocol::wire::V2Kind,
    expected: Option<[u8; 32]>,
    body: Vec<u8>,
) -> Vec<u8> {
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
    let Ok(key) = core::str::from_utf8(&key) else {
        return err("invalid_key");
    };
    let kind = match kind {
        crate::protocol::wire::V2Kind::Create => MutateKind::Create,
        crate::protocol::wire::V2Kind::Replace => MutateKind::Replace,
        crate::protocol::wire::V2Kind::Delete => MutateKind::Delete,
    };
    match super::mutate(
        disk,
        MutateReq {
            token,
            key,
            kind,
            expected,
            body: &body,
        },
    ) {
        Ok(_) => ok(Vec::new()),
        Err(e) => err(e.code()),
    }
}
