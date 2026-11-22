//! Deterministic fakes and disposable fixtures. No real DNS/TLS/network.
#![allow(dead_code, unused_imports)]
mod build;
pub mod fakes;
mod fixtures;
mod res;
mod scan;
mod ssrf_cases;

pub use build::build;
pub use fakes::{FakeP1, FakeResolver, FixedRng, ManualClock, ScriptedTransport, one, resolver};
pub use fixtures::{CAP, HINT, KEY, NS, PRIV, config, frame, sub_json, vapid_key};
pub use res::res;
pub use scan::scan;
pub use ssrf_cases::{BAD_IPS, BAD_URLS};
