use super::{Audit, files};
use std::path::Path;

pub fn check(root: &Path, name: &str, out: &mut Audit) {
    match files::generation::current(root, name) {
        Ok(value) => out.activation_generation = Some(value),
        Err(error) => out.generation_error = Some(error.to_string()),
    }
}
