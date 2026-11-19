use super::{
    CompatError, CompatResult, DurableBackend, RandomSource, Request, Response, StateStore, bpush,
    push_auth, reply,
};
pub fn update<S: StateStore, R: RandomSource>(
    b: &DurableBackend<S, R>,
    id: &str,
    req: &Request,
) -> CompatResult<Response> {
    let token = push_auth::cap(req)?;
    let body = bpush::parse_update(&req.body)?;
    bpush::validate_update(&body)?;
    b.change(|state| {
        let rec = state
            .subscriptions
            .iter()
            .find(|x| x.id == id)
            .ok_or_else(|| CompatError::new(404, "subscription_not_found"))?;
        push_auth::check_cap(rec, token)?;
        let x = state.subscriptions.iter_mut().find(|x| x.id == id).unwrap();
        x.endpoint = body.endpoint;
        x.keys = body.keys;
        Ok(())
    })?;
    Ok(reply::json(200, b"{}".to_vec()))
}
pub fn delete<S: StateStore, R: RandomSource>(
    b: &DurableBackend<S, R>,
    id: &str,
    req: &Request,
) -> CompatResult<Response> {
    let token = push_auth::cap(req)?;
    b.change(|state| {
        let rec = state
            .subscriptions
            .iter()
            .find(|x| x.id == id)
            .ok_or_else(|| CompatError::new(404, "subscription_not_found"))?;
        push_auth::check_cap(rec, token)?;
        state.subscriptions.retain(|x| x.id != id);
        Ok(())
    })?;
    Ok(reply::json(200, b"{}".to_vec()))
}
