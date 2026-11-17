//! Reusable client-side protocols for Skrynia and optional native administration.
#[cfg(target_arch = "wasm32")]
pub mod browser;
#[cfg(not(target_arch = "wasm32"))]
pub mod cas;
#[cfg(not(target_arch = "wasm32"))]
pub mod cli;
#[cfg(not(target_arch = "wasm32"))]
pub mod client;
#[cfg(not(target_arch = "wasm32"))]
pub mod event_set;
#[cfg(not(target_arch = "wasm32"))]
pub mod http_app;
#[cfg(not(target_arch = "wasm32"))]
pub mod log;
pub mod wire;
