use sha2::{Digest, Sha256};
use skrynia_tools::wire::v2compat::{DurableBackend, Mutation, MutationKind, Store};

pub fn check<S, R>(b: &DurableBackend<S, R>, ctx: &super::setup::Ctx)
where
    S: skrynia_tools::wire::v2compat::StateStore,
    R: skrynia_tools::wire::v2compat::RandomSource,
{
    let old = hex::encode(Sha256::digest(b"hi"));
    b.mutate_object(Mutation {
        key: "t/a",
        kind: MutationKind::Replace,
        token: &ctx.token,
        expected: Some(&old),
        body: b"ok",
    })
    .unwrap();
    assert!(
        b.mutate_object(Mutation {
            key: "t/a",
            kind: MutationKind::Replace,
            token: &ctx.token,
            expected: Some(&hex::encode(Sha256::digest(b"x"))),
            body: b"no",
        })
        .is_err()
    );
    b.revoke_user(&ctx.grant_id, &ctx.user_id, &ctx.issuer)
        .unwrap();
    assert!(
        b.mutate_object(Mutation {
            key: "t/a",
            kind: MutationKind::Delete,
            token: &ctx.token,
            expected: None,
            body: &[],
        })
        .is_err()
    );
}
