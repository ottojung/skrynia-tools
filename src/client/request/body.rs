use super::{Response, Result, SkryniaClient};
use anyhow::bail;
use std::io::Read;

const MAX_JSON_BYTES: u64 = 1_048_576;

impl SkryniaClient {
    pub(crate) fn json_response(response: Response) -> Result<serde_json::Value> {
        if response
            .content_length()
            .is_some_and(|length| length > MAX_JSON_BYTES)
        {
            bail!("Skrynia JSON response exceeds 1 MiB");
        }
        let mut bytes = Vec::new();
        response.take(MAX_JSON_BYTES + 1).read_to_end(&mut bytes)?;
        if bytes.len() as u64 > MAX_JSON_BYTES {
            bail!("Skrynia JSON response exceeds 1 MiB");
        }
        Ok(serde_json::from_slice(&bytes)?)
    }
}
