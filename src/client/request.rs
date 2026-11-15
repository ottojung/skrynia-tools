use super::types::SkryniaClient;
use anyhow::{Result, bail};
use reqwest::{
    StatusCode,
    blocking::{RequestBuilder, Response},
};

impl SkryniaClient {
    pub(super) fn auth(&self, request: RequestBuilder) -> Result<RequestBuilder> {
        let Some(token) = &self.token else {
            bail!("missing Skrynia user token");
        };
        Ok(request.bearer_auth(token))
    }
    pub(super) fn success(response: Response) -> Result<Response> {
        if response.status().is_success() {
            return Ok(response);
        }
        let status = response.status();
        let body = response.text().unwrap_or_default();
        bail!(
            "Skrynia HTTP {status}: {}",
            body.chars().take(300).collect::<String>()
        )
    }
    pub(super) fn mutating(response: reqwest::Result<Response>) -> Result<Response> {
        response.map_err(|e| anyhow::anyhow!("mutation outcome unknown: {e}"))
    }
    pub(super) fn not_found(status: StatusCode) -> bool {
        status == StatusCode::NOT_FOUND
    }
}
