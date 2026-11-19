use serde_json::json;
use skrynia_tools::wire::v2compat::{DurableBackend, Mutation, MutationKind, Store};

pub struct Ctx {
    pub token: String,
    pub grant_id: String,
    pub user_id: String,
    pub issuer: String,
}

pub fn create_and_mutate<S, R>(b: &DurableBackend<S, R>, rt: &str) -> Ctx
where
    S: skrynia_tools::wire::v2compat::StateStore,
    R: skrynia_tools::wire::v2compat::RandomSource,
{
    let c = b
        .create_grant(
            rt,
            &json!({"locations":"t/*","operations":["create","modify","delete"],"maxBytes":4096,"maxObjects":2}),
        )
        .unwrap();
    assert_eq!(c["id"].as_str().unwrap().len(), 32);
    assert_eq!(c["issuerToken"].as_str().unwrap().len(), 64);
    let iss = c["issuerToken"].as_str().unwrap().to_string();
    let u = b.issue_user(c["id"].as_str().unwrap(), &iss).unwrap();
    assert_eq!(u["userId"].as_str().unwrap().len(), 32);
    assert_eq!(u["userToken"].as_str().unwrap().len(), 64);
    let t = u["userToken"].as_str().unwrap().to_string();
    b.mutate_object(Mutation {
        key: "t/a",
        kind: MutationKind::Create,
        token: &t,
        expected: None,
        body: b"hi",
    })
    .unwrap();
    Ctx {
        token: t,
        grant_id: u["grantId"].as_str().unwrap().into(),
        user_id: u["userId"].as_str().unwrap().into(),
        issuer: iss,
    }
}
