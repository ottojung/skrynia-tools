mod verify;
use std::{net::SocketAddr, path::PathBuf};
pub struct Config {
    pub bind: SocketAddr,
    pub endpoint: reqwest::Url,
    pub token_path: PathBuf,
}
impl Config {
    pub fn load() -> Result<Self, String> {
        let bind: SocketAddr = std::env::var("SKRYNIA_V2_GATEWAY_BIND")
            .map_err(|_| "missing bind")?
            .parse()
            .map_err(|_| "bad bind")?;
        let endpoint =
            reqwest::Url::parse(&std::env::var("SKRYNIA_V3_URL").map_err(|_| "missing upstream")?)
                .map_err(|_| "invalid upstream")?;
        verify::check(bind, &endpoint)?;
        let token_path = std::env::var_os("SKRYNIA_OPERATOR_TOKEN_FILE")
            .ok_or("missing private token file")?
            .into();
        Ok(Self {
            bind,
            endpoint,
            token_path,
        })
    }
}
#[cfg(test)]
pub(crate) use verify::check;
