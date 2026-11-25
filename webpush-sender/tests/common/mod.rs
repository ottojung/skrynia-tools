//! Deterministic fakes and disposable fixtures. No real DNS or subscriber
//! traffic; the tls fake binds loopback only.
#![allow(dead_code, unused_imports)]
pub mod build;
pub mod claims;
pub mod fakes;
mod fixtures;
mod ledger_cases;
pub mod ledger_hard;
pub mod proc;
mod ssrf_cases;
#[cfg(feature = "test-transport")]
pub mod tls;

pub use build::build;
pub use build::res;
pub use claims::scan;
pub use claims::{jwt_aud, jwt_claims};
pub use fakes::{FakeP1, FakeResolver, FixedRng, ManualClock, ScriptedTransport, one, resolver};
pub use fixtures::{CAP, HINT, KEY, NS, PRIV, config, frame, sub_json, vapid_key};
pub use ssrf_cases::{BAD_IPS, BAD_URLS};
