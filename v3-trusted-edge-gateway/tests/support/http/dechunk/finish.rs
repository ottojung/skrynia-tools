pub fn finish(mut data: &[u8]) -> Result<(), String> {
    loop {
        let j = data.windows(2).position(|w| w == b"\r\n").ok_or_else(|| {
            format!(
                "missing final CRLF/trailer terminator: {} remaining bytes",
                data.len()
            )
        })?;
        if j == 0 {
            return if data.len() == 2 {
                Ok(())
            } else {
                Err(format!(
                    "trailing bytes after final chunk: {} bytes",
                    data.len()
                ))
            };
        }
        if !data[..j].contains(&b':') {
            return Err("invalid trailer".to_string());
        }
        data = &data[j + 2..];
    }
}
