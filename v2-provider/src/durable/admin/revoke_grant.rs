use super::persist;
use crate::bridge::Callback;
use crate::durable::{grants, lock};
use skrynia_tools::wire::v2compat::CompatError;

fn err(status: u16, code: &str) -> CompatError {
    CompatError::new(status, code)
}
pub fn grant<I: Callback>(i: &I, id: &str, token: &str) -> Result<(), CompatError> {
    let guard = lock::acquire(i).ok_or_else(|| err(503, "unavailable"))?;
    let result = run(i, id, token);
    if lock::release(i, guard).is_err() {
        return Err(err(503, "mutation_outcome_unknown"));
    }
    result
}
fn run<I: Callback>(i: &I, id: &str, token: &str) -> Result<(), CompatError> {
    let list = grants::list(i).map_err(|_| err(503, "unavailable"))?;
    let auth = skrynia_tools::wire::v2compat::grants::Authority::from_grants(list.clone())
        .map_err(|_| err(503, "unavailable"))?;
    let _ = auth.issuer(id, token)?;
    let mut remove = vec![id.to_string()];
    let mut changed = true;
    while changed {
        changed = false;
        for g in &list {
            if g.parent
                .as_deref()
                .is_some_and(|p| remove.contains(&p.to_string()))
                && !remove.contains(&g.id)
            {
                remove.push(g.id.clone());
                changed = true;
            }
        }
    }
    for gid in remove {
        persist::remove(i, &gid).map_err(|_| err(503, "unavailable"))?;
    }
    Ok(())
}
