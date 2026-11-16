//! Distinct HTTP rejection, transport failure and uncertain write outcomes.
use reqwest::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("Skrynia rejected request: HTTP {status}, {code}")]
    Rejected { status: StatusCode, code: String },
    #[error(
        "Skrynia returned HTTP {actual} for {operation}; expected {expected}; mutation outcome unconfirmed"
    )]
    UnexpectedMutationStatus {
        operation: &'static str,
        actual: StatusCode,
        expected: StatusCode,
    },
    #[error(
        "Skrynia returned HTTP {actual} for {operation}; expected {expected}; read cannot be trusted"
    )]
    UnexpectedReadStatus {
        operation: &'static str,
        actual: StatusCode,
        expected: StatusCode,
    },
    #[error("Skrynia transport failure: {source}")]
    Transport {
        #[source]
        source: reqwest::Error,
    },
    #[error("Skrynia mutation may have committed: {source}")]
    OutcomeUnknown {
        #[source]
        source: reqwest::Error,
    },
}
impl ClientError {
    pub fn rejected(status: StatusCode, body: &str) -> Self {
        let code = serde_json::from_str::<serde_json::Value>(body)
            .ok()
            .and_then(|v| v.get("error").and_then(|x| x.as_str()).map(str::to_owned))
            .unwrap_or_else(|| "unexpected_error_body".to_owned());
        Self::Rejected { status, code }
    }
}
#[cfg(test)]
use super::{SkryniaClient, Store};
#[cfg(test)]
mod large_test;
#[cfg(test)]
mod status_test;
#[cfg(test)]
mod tests;
