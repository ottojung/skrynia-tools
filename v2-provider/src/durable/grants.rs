use crate::bridge::Callback;
use crate::bridge::sys;
use skrynia_tools::wire::v2compat::grants::{Authority, Grant};
const DIR: &str = "v2/state/grants";

pub fn list<I: Callback>(i: &I) -> Result<Vec<Grant>, ()> {
    let names = match sys::fs::list_dir(i, DIR) {
        Ok(v) => v,
        Err(_) => return Err(()),
    };
    let mut grants = Vec::new();
    for name in names {
        if !name.ends_with(".json") {
            continue;
        }
        let path = format!("{DIR}/{name}");
        let Some(bytes) = sys::io::read_all(i, &path).map_err(|_| ())? else {
            continue;
        };
        let grant: Grant = serde_json::from_slice(&bytes).map_err(|_| ())?;
        grants.push(grant);
    }
    if grants.is_empty() {
        return Err(());
    }
    Ok(grants)
}
pub fn load<I: Callback>(i: &I) -> Result<Authority, ()> {
    Authority::from_grants(list(i)?).map_err(|_| ())
}
