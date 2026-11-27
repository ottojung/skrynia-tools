//! Real target-ABI WASM guests for the two distinct host contracts.
//!
//! A child evaluated by the compiled dumb P0 must frame output as a relay
//! request, while a guest evaluated by P1's userland interpreter streams raw
//! bytes through GET/PUT. Both import exactly `p0.invoke` and export `_start`;
//! neither imports retired `skrynia` bulk names or exports `skrynia_main`.

/// Guest staged at `operator/programs/root.wasm` and evaluated as P0's child.
pub fn p0_child_echo_wasm() -> Vec<u8> {
    wat::parse_str(include_str!("fixtures/p0_child_echo.wat")).unwrap()
}

/// Guest registered in P1 and run by its `p0.invoke` interpreter; this is also
/// the artifact copied into `FC_ARTIFACTS` for the fullchain and scale gates.
pub fn p2_echo_wasm() -> Vec<u8> {
    wat::parse_str(include_str!("fixtures/p2_echo.wat")).unwrap()
}
