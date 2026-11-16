use super::types::{Artifact, Entry, Modification};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Serialize, Default)]
pub struct Changes {
    pub added: Vec<Entry>,
    pub removed: Vec<Entry>,
    pub modified: Vec<Modification>,
}
impl Changes {
    pub fn between(
        before: &BTreeMap<String, Artifact>,
        after: &BTreeMap<String, Artifact>,
    ) -> Self {
        let mut changes = Self::default();
        for (path, old) in before {
            match after.get(path) {
                None => changes.removed.push(Entry {
                    path: path.clone(),
                    artifact: old.clone(),
                }),
                Some(new) if old != new => changes.modified.push(Modification {
                    path: path.clone(),
                    before: old.clone(),
                    after: new.clone(),
                }),
                Some(_) => {}
            }
        }
        for (path, new) in after {
            if !before.contains_key(path) {
                changes.added.push(Entry {
                    path: path.clone(),
                    artifact: new.clone(),
                });
            }
        }
        changes
    }
}
