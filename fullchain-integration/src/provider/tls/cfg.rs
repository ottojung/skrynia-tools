use super::Materials;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::server::{ClientHello, ResolvesServerCert, ServerConfig};
use rustls::sign::CertifiedKey;
use std::sync::{Arc, Mutex};

struct Sni {
    key: CertifiedKey,
    seen: Arc<Mutex<Option<String>>>,
}
impl std::fmt::Debug for Sni {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Sni")
    }
}
impl ResolvesServerCert for Sni {
    fn resolve(&self, hello: ClientHello<'_>) -> Option<Arc<CertifiedKey>> {
        *self.seen.lock().unwrap() = hello.server_name().map(str::to_owned);
        Some(Arc::new(self.key.clone()))
    }
}
pub fn server_config(m: &Materials, sni: Arc<Mutex<Option<String>>>) -> ServerConfig {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let chain = vec![CertificateDer::from(m.cert_der.clone())];
    let key = PrivateKeyDer::Pkcs8(m.key_pk8.clone().into());
    let key = CertifiedKey::from_der(chain, key, &provider).expect("key");
    ServerConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .expect("tls")
        .with_no_client_auth()
        .with_cert_resolver(Arc::new(Sni { key, seen: sni }))
}
