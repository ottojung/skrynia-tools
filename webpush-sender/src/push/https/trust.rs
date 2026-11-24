//! Trust roots are an injected capability. Only the feature-gated loopback
//! constructor can narrow them to a disposable CA; production keeps webpki.
#[cfg(feature = "test-transport")]
use reqwest::Certificate;
use reqwest::blocking::ClientBuilder;

pub enum Trust {
    System,
    #[cfg(feature = "test-transport")]
    Test(Vec<u8>),
}

impl Trust {
    pub fn production() -> Self {
        Self::System
    }

    #[cfg(feature = "test-transport")]
    pub fn test(ca_der: &[u8]) -> Self {
        Self::Test(ca_der.to_vec())
    }

    pub fn apply(&self, builder: ClientBuilder) -> reqwest::Result<ClientBuilder> {
        match self {
            Self::System => Ok(builder.tls_built_in_root_certs(true)),
            #[cfg(feature = "test-transport")]
            Self::Test(der) => Ok(builder
                .tls_built_in_root_certs(false)
                .add_root_certificate(Certificate::from_der(der)?)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_keeps_built_in_webpki_roots() {
        assert!(matches!(Trust::production(), Trust::System));
    }
}
