use super::{process, req, wire};
use skrynia_program0::wire::{Request, V2Kind};
pub fn private_root(server: &process::Server) -> [u8; 32] {
    let attempted = [7u8; 32];
    let bootstrap = Request::V2Bootstrap { token: attempted };
    assert_eq!(wire::err(&server.url, bootstrap).unwrap(), "invalid_state");
    let probe = req(attempted, V2Kind::Create, b"probe", None, b"untrusted");
    assert_eq!(
        wire::err(&server.url, probe).unwrap(),
        "invalid_grant_user_token"
    );
    let path = server.data_dir().join("v2auth/root-secret");
    use std::os::unix::fs::PermissionsExt;
    let mode = std::fs::metadata(&path).unwrap().permissions().mode();
    assert_eq!(
        mode & 0o077,
        0,
        "private root credential must be owner-only"
    );
    let bytes = std::fs::read(&path).unwrap();
    let root: [u8; 32] = bytes.try_into().unwrap();
    assert_ne!(root, attempted);
    root
}
