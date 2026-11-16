use super::Audit;
impl Audit {
    pub(super) fn finalize(&mut self) {
        self.metadata_without_artifact.sort();
        self.artifact_without_metadata.sort();
        self.abandoned_staging.sort();
        self.abandoned_activation_links.sort();
        self.abandoned_workspaces.sort();
        self.invalid_entries.sort();
        self.clean = self.active_error.is_none()
            && self.generation_error.is_none()
            && self.metadata_without_artifact.is_empty()
            && self.artifact_without_metadata.is_empty()
            && self.abandoned_staging.is_empty()
            && self.abandoned_activation_links.is_empty()
            && self.abandoned_workspaces.is_empty()
            && self.invalid_entries.is_empty();
    }
}
