use crate::bridge::Callback;
use crate::bridge::sys::{fs, path};
use std::collections::HashMap;
fn walk<I: Callback>(
    i: &I,
    dir: &str,
    prefix: &str,
    out: &mut HashMap<String, u64>,
) -> Result<(), ()> {
    let names = fs::list_dir(i, dir).map_err(|_| ())?;
    for name in names {
        if name.starts_with('.') {
            continue;
        }
        let full = format!("{dir}/{name}");
        let key = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}/{name}")
        };
        if path::is_dir(i, &full).map_err(|_| ())? == 1 {
            walk(i, &full, &key, out)?;
        } else if let Some(bytes) = crate::bridge::sys::io::read_all(i, &full).map_err(|_| ())? {
            out.insert(key, bytes.len() as u64);
        }
    }
    Ok(())
}
pub fn object_sizes<I: Callback>(i: &I) -> Result<HashMap<String, u64>, ()> {
    let mut out = HashMap::new();
    if path::exists(i, "v2/objects").map_err(|_| ())? != 1 {
        return Ok(out);
    }
    walk(i, "v2/objects", "", &mut out)?;
    Ok(out)
}
