use super::cap::{call, wire};

fn request(op: u8, rel: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(5 + rel.len());
    out.push(op);
    wire::put_u32(&mut out, rel.len() as u32);
    out.extend_from_slice(rel);
    out
}

fn result(reply: Result<(i32, Vec<u8>), i32>) -> Result<u32, i32> {
    let (status, data) = reply?;
    if status < 0 {
        return Err(status);
    }
    let bytes: [u8; 4] = data.try_into().map_err(|_| -1)?;
    Ok(u32::from_le_bytes(bytes))
}

pub fn publish(rel: &[u8]) -> Result<u32, i32> {
    result(call(&request(wire::PUBLISH, rel)))
}

pub fn unpublish(rel: &[u8]) -> Result<u32, i32> {
    result(call(&request(wire::UNPUBLISH, rel)))
}

/// Revoke the old public entry before switching to a new immutable target.
pub fn switch(old: &[u8], new: &[u8]) -> Result<u32, i32> {
    if !old.is_empty() && unpublish(old)? != 0 {
        return Err(-1);
    }
    publish(new)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mediated_request_matches_root_wire_format() {
        let req = request(wire::PUBLISH, b"apps/hello/index.html");
        assert_eq!(req[0], 16);
        assert_eq!(wire::i32_at(&req[1..5]), 21);
        assert_eq!(&req[5..], b"apps/hello/index.html");
    }
}
