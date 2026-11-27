use super::parse;

#[test]
fn rejects_oversize_and_full_record() {
    let mut old = vec![0u8; 4182];
    old[16..20].copy_from_slice(&4096u32.to_be_bytes());
    old[20] = 65;
    assert_eq!(parse(&old), Err("body-size"));
    old.truncate(135);
    old[16..20].copy_from_slice(&49u32.to_be_bytes());
    assert_eq!(parse(&old), Err("record-size"));
    old[16..20].copy_from_slice(&4096u32.to_be_bytes());
    assert!(parse(&old).is_ok());
}

#[test]
fn final_record_requires_delimiter_and_tag() {
    let mut body = vec![0u8; 86 + 16];
    body[16..20].copy_from_slice(&4096u32.to_be_bytes());
    body[20] = 65;
    assert_eq!(parse(&body), Err("short"));
    body.push(0);
    assert!(parse(&body).is_ok());
}
