use super::{SkryniaClient, native};
use anyhow::{Result, bail};
use std::io::Read;

impl SkryniaClient {
    pub(super) fn post(&self, frame: Vec<u8>) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        let response = self
            .http
            .post(self.endpoint.clone())
            .header("Origin", self.endpoint.origin().ascii_serialization())
            .header(native::EDGE_HEADER, &self.edge)
            .header("Content-Type", "application/octet-stream")
            .body(frame)
            .send()
            .map_err(|_| anyhow::anyhow!("transport failed; outcome unknown"))?;
        if response.status() != reqwest::StatusCode::OK {
            bail!(
                "unexpected HTTP status {}; outcome unknown",
                response.status()
            );
        }
        response
            .take((native::MAX_FRAME + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|_| anyhow::anyhow!("response failed; outcome unknown"))?;
        if bytes.len() > native::MAX_FRAME {
            bail!("response too large; outcome unknown");
        }
        Ok(bytes)
    }
}
