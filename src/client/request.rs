use super::{errors::ClientError, types::SkryniaClient};
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
        Err(ClientError::rejected(status, &body).into())
    }
    pub(super) fn mutating(response: reqwest::Result<Response>) -> Result<Response> {
        response.map_err(|source| ClientError::OutcomeUnknown { source }.into())
    }
    pub(super) fn not_found(status: StatusCode) -> bool {
        status == StatusCode::NOT_FOUND
    }
}
