mod finish;

/// Decode a complete HTTP/1.1 chunked response. A missing final chunk is
/// an interrupted stream, not successful delivery of a shorter body.
pub fn dechunk(mut data: &[u8]) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    loop {
        let j = data.windows(2).position(|w| w == b"\r\n").ok_or_else(|| {
            format!(
                "incomplete chunk header: missing CRLF after {} remaining bytes",
                data.len()
            )
        })?;
        let line = std::str::from_utf8(&data[..j])
            .map_err(|_| format!("invalid chunk header utf8: {} bytes", j))?;
        let size = usize::from_str_radix(
            line.split(';')
                .next()
                .ok_or_else(|| "empty chunk size".to_string())?
                .trim(),
            16,
        )
        .map_err(|_| format!("invalid chunk size: {line:?}"))?;
        data = &data[j + 2..];
        if size == 0 {
            return finish::finish(data).map(|()| out);
        }
        let end = size
            .checked_add(2)
            .ok_or_else(|| "chunk size overflow".to_string())?;
        if data.len() < end {
            return Err(format!(
                "incomplete chunk: expected {size} bytes + CRLF, got {} remaining bytes",
                data.len()
            ));
        }
        if &data[size..end] != b"\r\n" {
            return Err("chunk missing trailing CRLF".to_string());
        }
        out.extend_from_slice(&data[..size]);
        data = &data[end..];
    }
}
