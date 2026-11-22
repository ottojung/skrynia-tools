//! rustls server config that records the ClientHello SNI for assertions.
use super::certs::Materials;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::server::{ClientHello, ResolvesServerCert, ServerConfig};
use rustls::sign::CertifiedKey;
use std::sync::{Arc, Mutex};

pub fn config(materials: &Materials, sni: Arc<Mutex<Option<String>>>) -> ServerConfig {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let chain = vec![CertificateDer::from(materials.cert_der.clone())];
    let key = PrivateKeyDer::Pkcs8(materials.key_pk8.clone().into());
    let key = CertifiedKey::from_der(chain, key, &provider).expect("valid test key");
    ServerConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .expect("tls versions")
        .with_no_client_auth()
        .with_cert_resolver(Arc::new(Sni { key, seen: sni }))
}

#[derive(Debug)]
struct Sni {
    key: CertifiedKey,
    seen: Arc<Mutex<Option<String>>>,
}

impl ResolvesServerCert for Sni {
    fn resolve(&self, hello: ClientHello<'_>) -> Option<Arc<CertifiedKey>> {
        *self.seen.lock().unwrap() = hello.server_name().map(str::to_owned);
        Some(Arc::new(self.key.clone()))
    }
}
