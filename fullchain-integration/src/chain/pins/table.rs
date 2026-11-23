//! Immutable SHA256 pins for the five disposable full-chain artifacts.
//! These values are byte-hashed at runtime before any process is spawned.
pub const PINS: [(&str, &str); 5] = [
    (
        "kernel_v3_main_1c5639d",
        "ac301e18fcabf2cbf9f246fbaf1b7985e4062b6225da125f873138a2ed77401c",
    ),
    (
        "p0_d57ab27a.wasm",
        "d57ab27aacc01f6d76b272aed8621ce9af0f3000ed1de2309ec533cbf181a5fa",
    ),
    (
        "p1_hintbudget_bounded_20261123.wasm",
        "e9e5e1f3543bfbfabce1788a6a9535bed37ef15187ffb50a5aa714d27775ca41",
    ),
    (
        "trusted_edge_9fa4df14",
        "9fa4df141d5bf12ca90e4b9c69438dfe12e522a62f6da80a3f80c26a8fd0f07f",
    ),
    (
        "echo_dc28e490.wasm",
        "dc28e490b107084224562cc02a185e99c64727d6dd185eab5645989aa9c17ec5",
    ),
];
