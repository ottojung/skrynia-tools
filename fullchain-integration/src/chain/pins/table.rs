//! Immutable SHA256 pins for the five disposable full-chain artifacts.
//! These values are byte-hashed at runtime before any process is spawned.
pub const PINS: [(&str, &str); 5] = [
    (
        "kernel_PR125_856c029",
        "4c6d34f93f59a5377e64fdeae319a200d358b7ca5290162ad0a6b618f62954dc",
    ),
    (
        "p0_d57ab27a.wasm",
        "d57ab27aacc01f6d76b272aed8621ce9af0f3000ed1de2309ec533cbf181a5fa",
    ),
    (
        "p1_push_efe6117.wasm",
        "cf1322c3d41009c19c65771b353259b5f9c0d98157ec9075513edcd390c3477a",
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
