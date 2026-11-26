//! Reusable client-side protocols for Skrynia and the browser.
#[cfg(target_arch = "wasm32")]
pub mod browser;
#[cfg(not(target_arch = "wasm32"))]
pub mod cas;
#[cfg(not(target_arch = "wasm32"))]
pub mod client;
#[cfg(not(target_arch = "wasm32"))]
pub mod event_set;
#[cfg(not(target_arch = "wasm32"))]
pub mod log;
pub mod p2;
pub mod sdk;
pub mod wire;
