use super::{
    CompatResult, DurableBackend, RandomSource, Request, Response, StateStore, bpush, push_auth,
    push_sub_tx,
};
pub fn run<S: StateStore, R: RandomSource>(
    b: &DurableBackend<S, R>,
    req: &Request,
) -> CompatResult<Response> {
    let token = push_auth::bearer(req)?;
    let body = bpush::parse_sub(&req.body)?;
    bpush::validate_sub(&body)?;
    let secret = b.random_hex(32)?;
    let cap_hash = push_auth::hash(&secret);
    let id_new = b.random_hex(16)?;
    b.change(|state| {
        let grant_id = push_auth::subscribe_grant(state, token, &body.locations)?;
        push_sub_tx::apply(
            state,
            grant_id,
            body,
            secret.clone(),
            cap_hash,
            id_new.clone(),
        )
    })
}
