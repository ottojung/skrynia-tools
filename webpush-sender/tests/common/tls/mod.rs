//! Localhost-only fake HTTPS push endpoint. Disposable CA/cert per instance;
//! trust is injected only through HttpsTransport::for_test.
mod accept;
pub mod cases;
mod certs;
mod config;
mod handshake;
mod http;
pub mod probe;
mod server;

pub use config::ReplyPlan;
pub use server::FakePushService;

pub const HOST: &str = "push.example.test";
pub const OTHER_HOST: &str = "other.example.test";

#[derive(Debug, Clone, Default)]
pub struct Seen {
    pub method: String,
    pub target: String,
    pub host: String,
    pub sni: Option<String>,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}
