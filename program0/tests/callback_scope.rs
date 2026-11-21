use skrynia_program0::dispatch;

fn word(v: u32) -> Vec<u8> {
    v.to_le_bytes().to_vec()
}

fn code(req: &[u8]) -> i32 {
    let r = dispatch(req, 512);
    assert!(r.len() >= 4, "short callback reply");
    i32::from_le_bytes(r[..4].try_into().unwrap())
}

fn path_req(op: u8, p: &[u8]) -> Vec<u8> {
    let mut r = vec![op];
    r.extend_from_slice(&word(p.len() as u32));
    r.extend_from_slice(p);
    r
}

#[test]
fn open_disk_denied_without_delegated_prefix() {
    let mut req = path_req(1, b"programs/app/a.json");
    req.extend_from_slice(&word(1));
    assert_eq!(code(&req), -1);
}

#[test]
fn path_mutations_denied_without_write_delegation() {
    assert_eq!(code(&path_req(6, b"programs/app/a.json")), -1);
    let mut req = path_req(7, b"programs/app/a.json");
    req.extend_from_slice(&word(b"programs/app/b.json".len() as u32));
    req.extend_from_slice(b"programs/app/b.json");
    assert_eq!(code(&req), -1);
}

#[test]
fn guessed_root_fd_denied() {
    let mut req = vec![2u8];
    req.extend_from_slice(&word(71));
    req.extend_from_slice(&word(16));
    assert_eq!(code(&req), -1);
}
