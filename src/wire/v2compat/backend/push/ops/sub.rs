use super::{CompatError, Record, Request, SubView, pauth, pparse, pvalidate, store::Store};
use super::{GrantAuth, Ids};
pub fn subscribe<S: Store, G: GrantAuth, I: Ids>(
    s: &mut S,
    g: &G,
    ids: &I,
    req: &Request,
) -> Result<SubView, CompatError> {
    let token = pauth::bearer(req)?;
    let body = pparse::sub(&req.body)?;
    pvalidate::validate_sub(&body)?;
    let grant_id = g.can_subscribe(token, &body.locations)?;
    let secret = ids.secret();
    let rec = Record {
        id: ids.id(),
        grant_id: grant_id.clone(),
        locations: body.locations,
        events: body.events.0,
        endpoint: body.endpoint,
        keys: body.keys,
        cap_hash: pauth::hash(&secret),
    };
    if let Some(old) = s.find_dedup(&grant_id, &rec) {
        s.set_cap_hash(&old.id, pauth::hash(&secret))?;
        return Ok(SubView {
            id: old.id,
            capability: secret,
            deduped: true,
        });
    }
    let id = rec.id.clone();
    s.insert(rec)?;
    Ok(SubView {
        id,
        capability: secret,
        deduped: false,
    })
}
