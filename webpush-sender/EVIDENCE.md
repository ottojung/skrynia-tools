# WebPush sender validation evidence — agent 7L1EYxnLyKBfxPBZySk7iW-thanked

> **HISTORICAL / NOT RUN at the reviewed head (`eeb94881…`).** This file is a
> verbatim record of two earlier PR65/PR67-era runs. Its source-inventory
> SHA-256s do **not** match the current tree, and four listed files no longer
> exist at head: `src/pump/clock.rs`, `src/pump/ledger.rs`,
> `tests/common/fakes/p1.rs` and `tests/https.rs`. TLS coverage now lives in
> `tests/common/tls/**`. No exact-head re-run was performed here. Do not use
> this document as exact-head proof; the CI `webpush-sender` and
> `sender-crate` jobs at the reviewed commit are the current gate.

Base: skrynia-tools PR65 head efe61173660b089df4e43dcd2840210d3070bfe2 (branch security/skrynia-p1-subscription-revocation-20261122).
Isolated branch: security/skrynia-webpush-sender-20261122. Host: marceline-dev.
Disk: 26G available on /dev/loop9 (>20GiB hard, <32GiB preferred).
Toolchain: rustc 1.93.0 (254b59607 2026-01-19) (built from a source tarball); cargo 1.93.0 (083ac5135 2025-12-15) (built from a source tarball).

## Commands (all offline; target in /tmp/opencode/wps-target-7L1E)
```
cargo fmt --all -- --check
CARGO_TARGET_DIR=/tmp/opencode/wps-target-7L1E cargo test --offline --lib --tests
CARGO_TARGET_DIR=/tmp/opencode/wps-target-7L1E cargo clippy --offline --all-targets -- -D warnings -D clippy::correctness -D clippy::suspicious -D clippy::perf -D clippy::complexity
```

## Results
- cargo fmt: PASS (no diff).
- tests: 8 unit (7 canonical structural-lint self-tests + full-tree structure check) + 15 integration = 23 PASS, 0 FAIL across 8 test binaries.
- structural: <=50 nonblank/noncomment LOC per .rs file, <=10 direct entries per dir, sibling-only internal imports (canonical 7-lint harness).
- clippy: PASS with -D warnings and the four strict lint groups.
- doc-tests: NOT RUN (rustdoc binary absent on this host); the crate has no doctest examples.
- no real HTTPS/DNS/TLS/network/secret/production or Ivanka touched; all effects injected.

## Source inventory (sha256)
| 7b579362cc73eb2a3c9d0a5984b8b50987308f24c1fe1ff21856cb350681693b  Cargo.toml
| 8b62b6d844be64ba7b3e0e044da0000a5dd8c6f73900395768aefcfa4f0d5311  Cargo.lock
| 59fe88a6c9a10ed4bdd2569f8524370df9246ce8a530c28fe2e84e1cfebf82a1  README.md
| a55bcb056c27584b39269a8e318d285a305fc11e6a56a09a2d94999b3c9732f0  src/config.rs
| fd7bca1cc73e0a1c5b5cb2835b5830592b2ee3a8de619f6ec8136f87a358c842  src/error.rs
| d94d51d898851fb238109652fa7f9de915c9611695703c2f1ea48c71a4fc88a8  src/lib.rs
| 38ea0104dc949b5961b575ecb76455f4b1c815b272026dda0702567113d67e78  src/p1/client.rs
| 831980cb41dea0f2677022d1bc54f09995e1600d4951e09a19b5e89b7dcef363  src/p1/decode.rs
| ce63dd7153096b81d3625f95e46bfc7df01a420c03a77eb2b454a29b085d07da  src/p1/mod.rs
| d87898fffa55b72f4bc43822fd3a2e08485acc293b8cb1e034aa9e3c7d018b14  src/p1/wire.rs
| 8555c6cd1179d48e4bb7cf521e017c9c1bc715bd785124e73e92ee88edcb1cec  src/pump/clock.rs
| 231893bd475b341122108e573b2d79360a95bbcdfb158e427b2eaad9a921cda8  src/pump/cycle.rs
| 87ed1d92d1edc96dd983131b1b7eac3eb2a95f3d2819e45f3732b655bf279077  src/pump/deliver.rs
| 4ab94ad99ebed4f1f1f475fcf9b0d80efec57dde794586e7fa54d26836806b3b  src/pump/ledger.rs
| 34f93e93027502c4554d6c7f56c0fd4cf4884c12b8bc2c02eec50cd2f15719a4  src/pump/mod.rs
| cd556d5870ae44caf431a09c56ea0ba555bfde143aa1c1a618f692ab1b29c315  src/pump/report.rs
| 197f9f9e9cb0898eb8ae8150ec3545e97953ddb1cc3b260b90cd646c4205a706  src/pump/retry.rs
| 9b9d17e0cffffae08eae5dc66f2710da0319b6db10029532732205e8ba75f8d7  src/pump/run.rs
| 279f68f8c33465a7a4e8e651071b7430a95d20efc7db2be18e7bb6e4cae142a1  src/pump/seqs.rs
| caf0fcea8c40b1a5cb2ebd3ec3bfa4358474852a9e5aa99446ea04817e756f87  src/pump/step.rs
| eca4caeac29c1fa2e022b041b624f1bd6931b1ca184a0b2ca5140433fe0c6059  src/push/encrypt.rs
| c9d8d01f312a0086e20ed7d56d36a613f6d1a4761a13573006529f12e629099a  src/push/keys.rs
| 1402bbdbfccf11f45d0b44452620fd38d94b88ac83a680a4cb453120c4b88932  src/push/mod.rs
| b3216a961be07882bfc6a960985748671d26c29ea10e610eae76dd217a87ed3b  src/push/request.rs
| 06cb46411800b9d655ffac79e9712385ea207ca14d9915d0657ac007feb84ab4  src/push/transport.rs
| 8254f0ac0d7a62fabd6af90067c110471fb0955707ed38eba80c7090b2cc5806  src/source_lints/imports/head.rs
| 10fb7cc2e5899ca310be5899cfb0f1b5ad40a3cab5a5ac4714024c1af5170ec3  src/source_lints/imports/mod.rs
| 53bb12d57503962b4cc7621a458b9d793e9a532ef76f59b8daff2d2b0a7d2239  src/source_lints/imports/traverse.rs
| b9e6bf5957717b66bcc9571b0981b251dc447fe12d479ce3768ba555799d726a  src/source_lints/lines.rs
| 7955ba430126b7fa129ccb73cf4caebc132891b37aaa6d4e6de14fda1b83f1c1  src/source_lints/mod.rs
| cc9e95c2fa19d7985f7f401de730faea507be39c8c5c67bc2abef500b7657da8  src/source_lints/test_extra.rs
| 344595faf2a31530170a40234941e5712b3b4af9457c5dcc9333bf1703961a38  src/source_lints/tests.rs
| 24bbeb3118846b5852c8650ad2b7d3128732e6a9ea5d330a3f3dba6cd88c9f38  src/source_lints/tree.rs
| 446ed0b87422ee5fb98bb023623c504008a3229a745078be24027b999617d1ad  src/source_lints/vendor_test.rs
| 1607e9f62e27bb569d431964d4528537f3c058141b5edfeb3ee9d7e7cbdc51cd  src/sub/endpoint.rs
| 4fe94c029dd7e04b1925d0384e99a6b64326dc1c375329e07243df7efacede9e  src/sub/host.rs
| 5bca84d2d49b538f81e8ddaf26f4e36cfac6f221fb47597e57c63448b1499953  src/sub/ip.rs
| a7669a38eae276e1157860fa19bb8dc48374625744aafdf6df9086bdb6771b85  src/sub/mod.rs
| d77c02d3d04f660756fcc4a788eb73e4dda557dfb9b06f704f4663b0e5ad0caf  src/sub/record.rs
| b9848a628b1f05c65da8e3c6e0dc59fcb375b8014d84bb33571aa93370bc2613  src/sub/scope.rs
| 68dfe35149b27772b39bd475bf4f11bb0f6154c08e60d211496e4252ac73121c  src/sub/subscription.rs
| 311419e0005246a9d54ebbf2a8782827231c393b593b3d8b9ef462b00e35db53  src/vapid/header.rs
| dde5979bc6075fb7a83dea64e5f095ed50e33befeffdb48438099fa008b12271  src/vapid/mod.rs
| ff8b2a9817233cd0c1cf53c80e43e3753b4688f14f9b4f9bea768efc72cc281f  src/vapid/sign.rs
| deaddd23814ec7e5ac0b5447c68786ad9f3ab70db9f9b9013aac58cc609bd971  tests/common/build.rs
| a6b9d68f74486b27618e84345b74fdf2fff7f707a2ae95a1cccf26be226d98e4  tests/common/fakes/list.rs
| e47fbd2cb79af6bd97ea96e722ed574d280018985662c45ce4649e3f18ee87fa  tests/common/fakes/mod.rs
| f4ecd15d522519fd57500640e91d8465247ab6d3f02634eebbdc8bfd177f23f2  tests/common/fakes/net.rs
| e9bc19e84327fcee1642a85ac6f766d4a62c50621e108d152216489951a2c39c  tests/common/fakes/p1.rs
| 2674e77c99e439f78469b7aaedd147cbf8e323d0d58b3aec9eaedd7c09c99400  tests/common/fakes/scripted.rs
| 86bc5352820d505378e42187a910d917d2513453a4b73693e3b839c57f3f0ea5  tests/common/fakes/time.rs
| b400cf7cba8d40049ee2bb969b249b1e0599781dc9f666b44daf51cab20a23f4  tests/common/fixtures.rs
| 69d297bae3c39ce38cdc1c3de330db2daed7efa86a209de779fa2cb88076a939  tests/common/mod.rs
| 2fb8dcae4fce478a36979b0cf0ea59a5e66e92d5d7654f71e0575b285302900d  tests/common/res.rs
| 8ba188f0d2b9897dfed2b047c855e25caad19e49534f1e000b1b0484203074a6  tests/common/scan.rs
| db170a0626f11071c68bffac7f86eea238647a8c084f662d741bd49a71161a84  tests/common/ssrf_cases.rs
| 581349062e1a89da7cb03843671f3a2d64220be5418f4052fb2c1322a2928f74  tests/delivery.rs
| e749edcef50cc50365b36db2c8beea2fb0f77d83b1520823f8b65d3286d9b09b  tests/negative.rs
| 86fda09752f4e01d918e757d615b68b9b2e79156e4f9a9337e885333681a8819  tests/negative2.rs
| fa33fb65ef54712a2eae883065443a1a18ba92f80479be0b1b918f5e0e5b1e61  tests/retry.rs
| 75c851d0adeb98ec351287c8b883336f50f3e7316f836c5bd633b212f95bbda6  tests/ssrf.rs
| 7daff75cb17e3058c94d9e087afc8f68226602c3c956b6443d8772f66aeebd6b  tests/vapid.rs
| 22ff2dab68370849325760cbf6ee56220d2fb854cdc80e65e3d4afdf8f4fbab7  tests/wire.rs

## LOC / entry counts
max code LOC file: 48 ./tests/ssrf.rs

## Negative/legacy-route scan
grep -r "_skrynia/push|push/subscriptions|configure-push-rules" src => no matches
clean

---

# Pinned-IP HTTPS transport — agent 3TvdurdpEuUtq9syXuqPtg-seizes

Base: PR67 exact tip `460f2762278a13f61b7e09b6656d51bef76f0359`
(branch `security/skrynia-webpush-sender-fixes-20261122`); PR66 exact
`0c340b9684df9532f1f375d37b96f03ec89d4b2c` was read only. Host marceline-dev
verified. Own isolated clone/branch
`security/skrynia-webpush-https-transport-20261122`. No PR66/PR67 source,
kernel, Program0, P1, v2, main, production, DNS, real credential or
subscriber endpoint touched. No real VAPID secret: tests use the literal
public-only header `vapid t=test.jwt, k=public-key`.

## Library semantics actually checked (not assumed)
- reqwest 0.12.28 `ClientBuilder::resolve` / `resolve_to_addrs` install a
  `DnsResolverWithOverrides` wrapping the chosen resolver
  (`async_impl/client.rs:2270-2283, 439-443`) — it replaces only TCP name
  resolution; `hyper-rustls 0.27.10` `DefaultServerNameResolver::resolve`
  derives SNI from `uri.host()` (`connector.rs:104, 157-168`), so cert
  verification uses the URL host, not the pinned IP.
- `no_proxy()` plus reqwest built with `default-features = false` (so the
  `system-proxy` feature is absent) prevent proxy/environment egress.
  A `dns::Resolve` implementation that always errors (`NoDns`) is installed
  under the override, so any lookup not covered by the single pin fails closed.
- `redirect(Policy::none())` is set on the per-request client.
- TLS uses rustls/ring only (`aws-lc-sys` was traced to a dev-dep default and
  removed); built-in roots stay on in production and are disabled only in the
  test adapter.

## Commands and results (one bounded build; CARGO_BUILD_JOBS=2; target in /tmp/opencode)
```
cargo generate-lockfile                      # crates.io index only; no subscriber traffic
cargo fmt --all && cargo fmt --all -- --check
cargo test --locked --all-targets            # 8 unit + 29 integration PASS, 0 FAIL
rustc --test src/source_lints/mod.rs -o /tmp/opencode/skrynia-lint-tests-3Tvd && /tmp/opencode/skrynia-lint-tests-3Tvd   # 7/7 PASS
cargo clippy --locked --all-targets -- -D warnings -D clippy::correctness -D clippy::suspicious -D clippy::perf -D clippy::complexity   # PASS
HTTPS_PROXY=http://127.0.0.1:9 HTTP_PROXY=... ALL_PROXY=... \
  cargo test --locked --offline --test https pinned_ip_defeats_bad_dns_and_proxy_env  # PASS
```
All seven canonical structural lints pass (<=50 nonblank/noncomment LOC per
.rs after rustfmt, <=10 direct entries per directory, sibling-only imports).
Disk stayed 32-33 GiB free (>20 GiB hard floor). doc-tests NOT RUN (no rustdoc
on host); no doctest examples exist. GitHub CI NOT yet observed for the new
commit at review time.

## Live transport proofs (historical `tests/https.rs`, 6/6 PASS; file removed at head)
Disposable CA + SAN-bound leaf generated per service by the openssl CLI in a
unique temp dir; loopback `127.0.0.1:0` only; SNI captured in a rustls
`ResolvesServerCert` implementation. Trust and connect port are injected only
via `HttpsTransport::for_test`, never in production. Proofs:
1. live 201 with correct Host, SNI, SAN-verified certificate, POST target,
   Authorization/Content-Encoding aes128gcm/TTL/Topic/base64 body and no
   unexpected header (no secret leakage);
2. pinned IP wins; a different pinned IP fails; hostile proxy env cannot
   redirect egress;
3. foreign CA and hostname outside SAN both fail closed with no request seen;
4. 302 is returned, never followed (exactly one request); 201/403/429/500 are
   surfaced verbatim;
5. Retry-After capped to 60000 ms, short overall timeout fails, oversized
   response body fails;
6. production constructor refuses port != 443, rejects non-lowercase host,
   and needs no injected trust.

## Source hashes (sha256)
```
e42720f0d10c79b8b376fdd8d8e718165ba7cd6cd74b2aceea737ddc3b7e2a34  Cargo.toml
d7df34193cf9520d26cac0e0da300ae93d1d09deddbee6cf4449827dfe3ee4bd  src/push/https/client.rs
d1dedf090d0d1b93e1199342c37ea6664cc8dcea0447bef0c5681a3975a3e4f1  src/push/https/guard.rs
3f27d53c632620138487907d29eb234952a8c2199d61cfc3bac39e2445a44cd7  src/push/https/mod.rs
e8ec25061b45896472b2efc749d77cda8482509325d9ba7463fdb55637d989c7  src/push/https/reply.rs
cdbd7af9bc8eb027d3df9a269d6124b9a52547084475c1fd75cfed9668bfcbd9  src/push/https/request.rs
8daf2d651894c1a57b8de1a3ce876da3e4ff2b6a08e44ffacb6f302f3c02f42d  src/push/https/trust.rs
12bdc38aee59ac1acd1a8aad75aed60389710038bf036cb57f3f6b58663fe73e  tests/common/tls/mod.rs
b19b6140b505a0cd304f4aceb63d80f9d64d06b371b699d6a1febebe89c3c918  tests/common/tls/probe.rs
e16669cacee3d6e75c0a6415903176e9d76ae6fdb896c2c60ed2458ad4d119f8  tests/common/tls/server.rs
f5e597170616e83b13d300fddc0c482282642d131858a15b212a13685abc6113  tests/common/tls/accept.rs
71ad5eb636d7bc62bd1c42e3030301bc6bc69836a5df78e00bf0b86f6c6600c5  tests/common/tls/handshake.rs
dd56e788c18760fc404a17ffed870e584a3705399c68cd46acb1779ce5df2450  tests/https.rs  (REMOVED at head; historical pin only)
```

## Separate blockers (documented, deliberately NOT fixed here)
Out of scope for this narrowly scoped transport change and left for their own
reviewed fixes: VAPID `sub` claim JSON escaping, TTL > 24h bound, single fixed
Topic / dedupe, `Reply.retry_after_ms` ignored by the pump, and the in-memory
replay Ledger ceiling. These are P1/pump business logic, not transport.
(Historical: `Reply.retry_after_ms` is now consumed by `src/pump/retry.rs`;
this list is not a claim about the current head.)

## Gate status
This is source + localhost fake-endpoint evidence only. It is NOT actual FCM
production acceptance, NOT a real VAPID credential delivery, and NOT a whole
release GO. PR67 remains under independent audit; keep all PRs DRAFT and do
not merge without that audit and fresh pinned-SHA CI.
