use super::{CompatError, Keys, Mem};
pub fn update_target(
    m: &mut Mem,
    id: &str,
    endpoint: String,
    keys: Keys,
) -> Result<(), CompatError> {
    let x = m
        .items
        .iter_mut()
        .find(|x| x.id == id)
        .ok_or_else(not_found)?;
    x.endpoint = endpoint;
    x.keys = keys;
    Ok(())
}
pub fn set_cap_hash(m: &mut Mem, id: &str, cap_hash: String) -> Result<(), CompatError> {
    let x = m
        .items
        .iter_mut()
        .find(|x| x.id == id)
        .ok_or_else(not_found)?;
    x.cap_hash = cap_hash;
    Ok(())
}
pub fn remove(m: &mut Mem, id: &str) -> Result<(), CompatError> {
    let n = m.items.len();
    m.items.retain(|x| x.id != id);
    if m.items.len() == n {
        Err(not_found())
    } else {
        Ok(())
    }
}
fn not_found() -> CompatError {
    CompatError::new(404, "subscription_not_found")
}
