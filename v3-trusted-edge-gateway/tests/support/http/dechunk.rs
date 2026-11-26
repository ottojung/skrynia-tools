/// Decode a complete HTTP/1.1 chunked response. A missing final chunk is
/// an interrupted stream, not successful delivery of a shorter body.
pub fn dechunk(mut data: &[u8]) -> Result<Vec<u8>, &'static str> {
    let mut out = Vec::new();
    loop {
        let j = data
            .windows(2)
            .position(|w| w == b"\r\n")
            .ok_or("chunk header")?;
        let line = std::str::from_utf8(&data[..j]).map_err(|_| "chunk header")?;
        let size = usize::from_str_radix(line.split(';').next().ok_or("chunk size")?.trim(), 16)
            .map_err(|_| "chunk size")?;
        data = &data[j + 2..];
        if size == 0 {
            return finish(data).map(|()| out);
        }
        let end = size.checked_add(2).ok_or("chunk size overflow")?;
        if data.len() < end || &data[size..end] != b"\r\n" {
            return Err("incomplete chunk");
        }
        out.extend_from_slice(&data[..size]);
        data = &data[end..];
    }
}

fn finish(mut data: &[u8]) -> Result<(), &'static str> {
    loop {
        let j = data
            .windows(2)
            .position(|w| w == b"\r\n")
            .ok_or("missing final CRLF")?;
        if j == 0 {
            return if data.len() == 2 {
                Ok(())
            } else {
                Err("trailing bytes")
            };
        }
        if !data[..j].contains(&b':') {
            return Err("invalid trailer");
        }
        data = &data[j + 2..];
    }
}
