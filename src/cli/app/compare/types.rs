use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Artifact {
    pub kind: &'static str,
    pub sha256: Option<String>,
    pub executable: Option<bool>,
}
#[derive(Clone, Debug, Serialize)]
pub struct Entry {
    pub path: String,
    pub artifact: Artifact,
}
#[derive(Clone, Debug, Serialize)]
pub struct Modification {
    pub path: String,
    pub before: Artifact,
    pub after: Artifact,
}
