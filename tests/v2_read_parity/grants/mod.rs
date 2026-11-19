mod compare;
mod errors;
mod fixture;
mod lifecycle;
use super::process::Server;
pub fn verify(server: &Server) -> Result<(), String> {
    let (root_id, issuer) = server.root_grant()?;
    let backend = fixture::init(&root_id, &issuer)?;
    let v2root = server.root_user()?;
    let v3root = backend.issue_user(&root_id, &issuer).map_err(|e| e.code)?["userToken"]
        .as_str()
        .unwrap()
        .to_string();
    lifecycle::run(server, &backend, &v2root, &v3root)
}
