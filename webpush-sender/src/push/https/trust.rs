//! Trust roots are an injected capability. Only the test constructor can
//! narrow them to a disposable CA; production keeps the built-in webpki set.
use reqwest::Certificate;
use reqwest::blocking::ClientBuilder;

pub enum Trust {
    System,
    Test(Vec<u8>),
}

impl Trust {
    pub fn production() -> Self {
        Self::System
    }

    pub fn test(ca_der: &[u8]) -> Self {
        Self::Test(ca_der.to_vec())
    }

    pub fn apply(&self, builder: ClientBuilder) -> reqwest::Result<ClientBuilder> {
        match self {
            Self::System => Ok(builder.tls_built_in_root_certs(true)),
            Self::Test(der) => Ok(builder
                .tls_built_in_root_certs(false)
                .add_root_certificate(Certificate::from_der(der)?)),
        }
    }
}
