use serde::Serialize;
#[derive(Debug, Serialize)]
pub struct Audit {
    pub name: String,
    pub available_releases: usize,
    pub active_release: Option<String>,
    pub active_error: Option<String>,
    pub activation_generation: Option<u64>,
    pub generation_error: Option<String>,
    pub metadata_without_artifact: Vec<String>,
    pub artifact_without_metadata: Vec<String>,
    pub abandoned_staging: Vec<String>,
    pub abandoned_activation_links: Vec<String>,
    pub abandoned_workspaces: Vec<String>,
    pub invalid_entries: Vec<String>,
    pub clean: bool,
}
impl Audit {
    pub(super) fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            available_releases: 0,
            active_release: None,
            active_error: None,
            activation_generation: None,
            generation_error: None,
            metadata_without_artifact: vec![],
            artifact_without_metadata: vec![],
            abandoned_staging: vec![],
            abandoned_activation_links: vec![],
            abandoned_workspaces: vec![],
            invalid_entries: vec![],
            clean: false,
        }
    }
}
