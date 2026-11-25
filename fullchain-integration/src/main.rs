//! Disposable full-chain WebPush acceptance harness (DRAFT, marceline-dev).
//! Real v3 main kernel -> dumb P0 -> native P1 WASM via the real Rust
//! trusted edge -> real sender HttpP1 adapter -> disposable local HTTPS push
//! provider. No production, main, DNS, real TLS, live secrets or browser claim.
mod bridge;
mod caps;
mod chain;
mod evidence;
mod provider;
mod run;
mod scenarios;

// Flat re-exports keep every cross-module import at exactly one `super`/`crate`
// hop, satisfying the sibling-only structural lint.
pub use caps::{FixedResolver, FixedRng, TestClock};
pub use chain::{EdgeProc, Kernel, free_port, manifest, op_cap, transport, verify, verify_all};
pub use provider::{
    FakePush, Materials, ReplyPlan, Seen, SubKeys, Verified, check, read_req, server_config,
    write_reply,
};
pub use scenarios::{LoopbackPush, blb, call, fixture, idb, must, req, req_ver, u64b};

fn main() {
    if std::env::var_os("FC_SENDER_BRIDGE").is_some() {
        bridge::execute();
    } else {
        run::execute();
    }
}
