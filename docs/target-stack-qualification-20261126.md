# Target stack integration checkpoint — 2026-11-26

This checkpoint qualifies the exact source and artifact sets listed below,
using disposable loopback kernels and synthetic credentials. It is not
production authorization, a main merge, or a whole-release sign-off.

## Source and artifact provenance

Kernel code db08cbc7496fafedc6a2e32df80395911b930879 (PR 134);
9b30d66536f4240363fe3638a099ddba0627df4d adds documentation only.
Tools combined implementation: 12d41dae9839bbadb1afae97d16ee236759acc28.
P0 source unchanged since 96ce876; P1 durability integrated in b742ebe;
publication-root private path fix integrated in 3897a54. Rust 1.93.0,
locked dependencies and WASM flags:
`-C link-arg=--export-table -C opt-level=z -C strip=symbols`.

| Artifact | SHA256 |
| --- | --- |
| kernel | 0339ca954b40a5907936add3edde5e146e625599008899dec6f6c816f6d2a7f0 |
| P0 | 91042b8930a8f9463e9436266b2de2942fe5015f2a14a849ccc68ea316359d88 |
| P1 | 29389bfdfd2ba2ff67dec30330f39a4dafe63f49e9142d5c17bf5e980bcad216 |
| Theorem Lab P2 | ecfb44df23fd25b3388dddf2b332508b1285072538bb4a55b75b4d7c423ae268 |
| publication root | 3ef41bea72bc11c7a18dae0c39a7e2f2c5e9390565509a46203e2a861f175073 |
| provider | 2de66285b15dd0cb5ef83b758cf12175a34743e46005e3218ecf8d3f4565c53f |

P1 has 1,513,075 bytes and passed the single-import/entrypoint check.
The provider was built at 37eea06; its source is unchanged in the
combined candidate. Evidence artifacts and raw logs are retained on
marceline-dev in /workspace/skrynia-final-qualification-20261126 and
the T12 evidence directory.

## Local gates

| Gate | Observed result |
| --- | --- |
| Kernel make test at 9b30d66 | 125 passed, 0 failed/ignored |
| Kernel GitHub CI | success, run 45906075597 |
| Tools host workspace at 12d41da | 249 passed, 0 failed/ignored |
| Tools workspace formatting and strict Clippy | pass |
| Real P0 qualified operator-root denial and descriptor isolation | pass |
| Real descriptor operations and random bytes | pass |
| Real shared mutation lease contention and trap recovery | pass |
| Real registry restart, obstruction recovery and P2 adversarial suite, durable P1 | pass |
| Real nested callback (including persistence and malformed input) | 12/12 pass |
| Real publication root + provider + nginx, including 300 overlaps | 27/27 pass |

The native-components CI now requires the compiled descriptor, random,
lease, isolation, nested P2 and publication overlap gates. The older
acceptance wrapper was replaced with explicit artifact pins and no
historical default. See the workflow and the individual fixture scripts.

Five actual app browser gates also passed on the earlier 415e624 kit;
those results are separate from final durable P1 / publication-root
qualification. Trusted-edge duplex/fullchain, scale, Chromium admin,
genuine ENOSPC and final app kit replay remain separate gates. Earlier
passing results must not be attributed to a later artifact without a
replay or evidence that the relevant source and artifact are unchanged.
