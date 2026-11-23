//! Disposable full-chain WebPush acceptance harness (DRAFT, marceline-dev).
//! Real compiled kernel PR125 -> dumb P0 -> push P1 WASM via the real Rust
//! trusted edge -> real sender HttpP1 adapter -> disposable local HTTPS push
//! provider. No production, main, DNS, real TLS, live secrets or browser claim.
mod caps;
mod chain;
mod evidence;
mod provider;
mod run;
mod scenarios;

fn main() {
    run::execute();
}
