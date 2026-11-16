use super::{ClientError, Response, Result, SkryniaClient, StatusCode};
impl SkryniaClient {
    pub(crate) fn success(response: Response) -> Result<Response> {
        if response.status().is_success() {
            return Ok(response);
        }
        let status = response.status();
        let body = response.text().unwrap_or_default();
        Err(ClientError::rejected(status, &body).into())
    }
    pub(crate) fn committed(
        response: Response,
        expected: StatusCode,
        operation: &'static str,
    ) -> Result<Response> {
        let actual = response.status();
        if actual.is_success() && actual != expected {
            return Err(ClientError::UnexpectedMutationStatus {
                operation,
                actual,
                expected,
            }
            .into());
        }
        Self::success(response)
    }
    pub(crate) fn fetched(
        response: Response,
        expected: StatusCode,
        operation: &'static str,
    ) -> Result<Response> {
        let actual = response.status();
        if actual.is_success() && actual != expected {
            return Err(ClientError::UnexpectedReadStatus {
                operation,
                actual,
                expected,
            }
            .into());
        }
        Self::success(response)
    }
}
