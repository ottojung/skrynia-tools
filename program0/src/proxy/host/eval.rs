use super::wire;

pub fn call(_req: &[u8]) -> Vec<u8> {
    wire::reply(wire::DENIED, &[])
}
