type Record<'a> = (&'a [u8], &'a [u8], &'a [u8]);

pub(super) fn parse(body: &[u8]) -> Result<Record<'_>, &'static str> {
    if body.len() > 4096 {
        return Err("body-size");
    }
    if body.len() < 16 + 4 + 1 + 65 + 17 {
        return Err("short");
    }
    let salt = &body[0..16];
    let idlen = body[20] as usize;
    if idlen != 65 || body.len() < 21 + idlen + 16 {
        return Err("idlen");
    }
    let keyid = &body[21..21 + idlen];
    let ct = &body[21 + idlen..];
    let rs = u32::from_be_bytes(body[16..20].try_into().unwrap()) as usize;
    if rs <= ct.len() {
        return Err("record-size");
    }
    Ok((salt, keyid, ct))
}

#[cfg(test)]
mod tests;
