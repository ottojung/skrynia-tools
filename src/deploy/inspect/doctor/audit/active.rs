use super::{Audit, inspect};
use std::path::Path;

pub fn check(root: &Path, name: &str, out: &mut Audit) {
    match inspect(root, name) {
        Ok(Some(active)) => out.active_release = Some(active.id),
        Ok(None) => {}
        Err(error) => out.active_error = Some(error.to_string()),
    }
}
