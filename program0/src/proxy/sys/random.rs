use super::proto;
pub fn fill(req: &[u8]) -> Vec<u8> {
    let mut i = 1usize;
    let n = proto::get_u32(req, &mut i).unwrap_or(0) as usize;
    if n == 0 || n > 4096 {
        return proto::err(-1);
    }
    let mut out = vec![0u8; n];
    let bits = (n as u32).saturating_mul(8);
    let rc = unsafe { super::super::abi::secure_random_bits(out.as_mut_ptr() as u32, bits) };
    if rc < 0 {
        return proto::err(rc);
    }
    let mut r = vec![proto::OK];
    proto::put_bytes(&mut r, &out);
    r
}
