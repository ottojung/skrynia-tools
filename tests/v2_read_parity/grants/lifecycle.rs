use super::{Server, compare, fixture::Backend};
use serde_json::json;
pub fn run(server: &Server, backend: &Backend, v2root: &str, v3root: &str) -> Result<(), String> {
    let spec = json!({"locations":"parity/*","operations":["create","modify"],
        "maxBytes":4096,"maxObjects":2});
    let bytes = serde_json::to_vec(&spec).unwrap();
    let (old, new) = compare::each(
        &server.url,
        backend,
        "POST",
        "/grants",
        v2root,
        v3root,
        &bytes,
    )?;
    if old.0 != new.0 || old.0 != 200 {
        return Err(format!("grant create {old:?} {new:?}"));
    }
    for key in ["locations", "operations", "maxBytes", "maxObjects"] {
        if old.1[key] != new.1[key] {
            return Err(format!("grant response mismatch {key}"));
        }
    }
    let v2id = old.1["id"].as_str().ok_or("missing v2 id")?;
    let v3id = new.1["id"].as_str().ok_or("missing v3 id")?;
    let v2issuer = old.1["issuerToken"].as_str().ok_or("missing v2 issuer")?;
    let v3issuer = new.1["issuerToken"].as_str().ok_or("missing v3 issuer")?;
    super::errors::check(server, backend, v2root, v3root)?;
    super::errors::issue_and_inspect(server, backend, (v2id, v2issuer), (v3id, v3issuer))
}
