use super::fake::{M, R};
use super::{check_cas, check_list, seed};
use serde_json::json;
use std::sync::atomic::AtomicU64;

fn setup() -> (super::http::B, String) {
    let b = super::http::B::open(M::default(), R(AtomicU64::new(1))).unwrap();
    let issuer = "a".repeat(64);
    b.bootstrap_root("r", &issuer).unwrap();
    let root = b.issue_user("r", &issuer).unwrap();
    let rt = root["userToken"].as_str().unwrap();
    let spec = json!({"locations":"apps/*","operations":["create","modify","delete","list","delegate"],"maxBytes":500000,"maxObjects":500});
    let child = b.create_grant(rt, &spec).unwrap();
    let user = b
        .issue_user(
            child["id"].as_str().unwrap(),
            child["issuerToken"].as_str().unwrap(),
        )
        .unwrap();
    (b, user["userToken"].as_str().unwrap().into())
}
pub fn run() {
    let (b, t) = setup();
    let vault = b"encrypted-vault".to_vec();
    seed::seed(&b, &t, &vault);
    check_list::all_apps(&b, &t);
    check_cas::rollback(&b, &t, &vault);
}
