use super::{cap, protocol, wmut};
fn err(code: &str) -> Vec<u8> {
    protocol::wenc::encode_res(&protocol::wire::Response {
        status: protocol::wire::STATUS_ERR,
        body: code.as_bytes().to_vec(),
    })
}
pub fn handle_wire<D: cap::Disk, R: cap::Random<Err = D::Err>>(
    disk: &D,
    random: &R,
    req: protocol::wire::Request,
) -> Vec<u8> {
    match req {
        protocol::wire::Request::V2Bootstrap { .. } => err("invalid_state"),
        protocol::wire::Request::V2Mutate {
            token,
            key,
            kind,
            expected,
            body,
        } => {
            if super::state::ensure_root(disk, random).is_err() {
                return err("state_uncertain");
            }
            wmut::run(disk, token, key, kind, expected, body)
        }
        _ => err("invalid_state"),
    }
}
