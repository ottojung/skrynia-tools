use crate::bridge::{Callback, InvokeError, sys};
use skrynia_tools::wire::v2compat::grants::Grant;
const DIR: &str = "v2/state/grants";

pub fn path(id: &str) -> String {
    format!("{DIR}/{id}.json")
}
pub fn save<I: Callback>(i: &I, grant: &Grant) -> Result<(), InvokeError> {
    let body = serde_json::to_vec(grant).map_err(|_| InvokeError::Unavailable)?;
    sys::fs::write_atomic(i, &path(&grant.id), &body)
}
pub fn remove<I: Callback>(i: &I, id: &str) -> Result<(), InvokeError> {
    if sys::path::delete(i, &path(id))? != 0 {
        return Err(InvokeError::Unavailable);
    }
    sys::fs::sync_dir(i, DIR)
}
pub fn hex<I: Callback>(i: &I, n: usize) -> Result<String, InvokeError> {
    Ok(hex::encode(sys::rand::bytes(i, n)?))
}
