use super::{CompatResult, Response, State, bpush, push_quota, reply};

pub fn apply(
    state: &mut State,
    grant_id: String,
    body: bpush::SubReq,
    secret: String,
    cap_hash: String,
    id_new: String,
) -> CompatResult<Response> {
    if let Some(i) = state.subscriptions.iter().position(|x| {
        x.grant_id == grant_id
            && x.locations == body.locations
            && x.events == body.events.0
            && x.endpoint == body.endpoint
            && x.keys == body.keys
    }) {
        state.subscriptions[i].cap_hash = cap_hash;
        let view = bpush::SubView {
            id: state.subscriptions[i].id.clone(),
            capability: secret,
            deduped: true,
        };
        return Ok(reply::json(
            200,
            serde_json::to_vec(&view).unwrap_or_default(),
        ));
    }
    push_quota::check(state, &grant_id)?;
    state.subscriptions.push(bpush::Record {
        id: id_new.clone(),
        grant_id,
        locations: body.locations,
        events: body.events.0,
        endpoint: body.endpoint,
        keys: body.keys,
        cap_hash,
    });
    let view = bpush::SubView {
        id: id_new,
        capability: secret,
        deduped: false,
    };
    Ok(reply::json(
        201,
        serde_json::to_vec(&view).unwrap_or_default(),
    ))
}
