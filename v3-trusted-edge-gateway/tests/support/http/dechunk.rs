pub fn dechunk(data: &[u8]) -> Vec<u8> {
    let mut i = 0;
    let mut out = Vec::new();
    loop {
        let mut j = i;
        while j + 1 < data.len() && &data[j..j + 2] != b"\r\n" {
            j += 1;
        }
        if j + 1 > data.len() {
            break;
        }
        let line = std::str::from_utf8(&data[i..j]).unwrap_or("");
        let size =
            usize::from_str_radix(line.split(';').next().unwrap_or("").trim(), 16).unwrap_or(0);
        i = j + 2;
        if size == 0 {
            break;
        }
        if i + size > data.len() {
            out.extend_from_slice(&data[i..]);
            break;
        }
        out.extend_from_slice(&data[i..i + size]);
        i += size;
        if i + 2 <= data.len() && &data[i..i + 2] == b"\r\n" {
            i += 2;
        }
    }
    out
}
