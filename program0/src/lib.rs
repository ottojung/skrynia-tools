//! Operator-supplied Program 0: token verifier and dumb byte proxy only.
//! Target ABI (0ccc90f intents / K0 choices), no kernel application policy,
//! no CLI, no old `eval_delegate` / `fs_delegate_scope` imports.
#[cfg(any(target_arch = "wasm32", test))]
mod abi;
#[cfg(any(target_arch = "wasm32", test))]
mod alloc;
#[cfg(any(target_arch = "wasm32", test))]
mod auth;
#[cfg(any(target_arch = "wasm32", test))]
mod code;
#[cfg(any(target_arch = "wasm32", test))]
mod entry;
#[cfg(all(test, not(target_arch = "wasm32")))]
mod fake;
#[cfg(any(target_arch = "wasm32", test))]
mod invoke;
#[cfg(test)]
mod tests;
mod token;
