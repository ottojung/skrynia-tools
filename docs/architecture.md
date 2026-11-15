# Architecture proposal

**Status: proposed design, not implemented.** Skrynia Tools is a separate client library and CLI for the existing Skrynia API. It does not require server extensions.

## Objective

Provide reusable client-side protocol state machines, usable from native Rust and browser/Wasm applications, with a CLI for ordinary clients and remote administrators. The Rust implementation must own protocol semantics; JavaScript binds to it, not reimplements it.

## Layers

1. **Wire client:** typed `ObjectKey`, `LocationMatcher`, `ByteDigest`, tokens and structured errors; public GET/HEAD, authenticated create/replace/delete, CAS, paginated listing, grants, user tokens and push registration.
2. **Reusable protocols:** `CasCell`, `AppendLog`, `EventSet`, `SnapshotStore`, later `SealedEpochLog`. Every protocol has a versioned wire format, capability requirements, proof obligations, bounded-work controls and unknown-outcome recovery rules.
3. **Native `skrynia` CLI:** app developer commands and admin provisioning; it uses the same Rust library, and adds local secure credential storage, diagnostics, output and journaling.
4. **Browser/Wasm adapter:** Rust core compiled to Wasm, browser fetch and optional Service Worker/Web Push integration, with a thin TypeScript facade. Rust applications targeting Wasm may directly link the Rust crate.

Start with one Rust core crate; split into additional crates only when a real transport, CLI or Wasm build boundary calls for it. Feature-gate optional facilities to keep browser bundles small.

## Inversion of control

Protocol engines depend on explicit, narrow injected effects: store transport, grant transport, clock, randomness, credential store, and diagnostics as relevant. Avoid direct OS files, global state, or hardwired HTTP libraries in protocol algorithms. Implement native and Wasm transports separately. Accommodate browser-local futures without imposing a native thread executor on the core.

Raw operations are **not** generic writes: `get`, `head`, `create`, `replace`, `delete`, and `list_page` retain their distinct semantics. Conditional replace/delete require SHA-256 of **actual previously read object bytes**; do not rely on static-server ETags. Expose whether a mutable head was read authoritatively: a stale cache is not fresh linearizable state.

Listings expose bounded pages and `next` cursors. Pages are not one atomic snapshot. Push notifications are duplicate-prone, coalesced `changed` wake-ups; always reconcile by reading durable data.

## Rust public API shape

Validated keys/matchers/digests/credentials are newtypes with private constructors and documented invariants. Use explicit protocol descriptors and version IDs, `Result<Success, DomainError>`, and typed `OutcomeUnknown`. A protocol may require a **bundle of separate user tokens**, since one Skrynia grant supports only one matcher, not a disjoint key-set. A local token scope is never proof of actual authorization.

Example conceptual interfaces: `CasCell<T>::read/compare_update`, `AppendLog<T>::initialize/append/read_from/follow`, and `SnapshotStore<T>::read_snapshot/transact`. These are **not implemented** and the final API names should follow executable protocol tests.

## Consistency and trust

Skrynia enforces one-key operations, resource-bounded grants and public reads. It provides no multi-key physical transaction, trusted client computation, key-private read or global app registry. A client protocol can offer logical atomicity only for cooperating writers within declared authority/trust assumptions. Reader hash/signature validation is distinct from preventing malicious writes. A browser never receives root or issuer credentials; independently untrusted publishers need separate HTTPS origins.

## Evidence and release criteria

For each public protocol require a normative wire-format version, golden interoperability bytes, deterministic state-machine tests, crash/unknown-outcome injection at each durable step, adversarial cases, bounded retry behavior, and real-server integration. Run native checks first, then browser tests once bindings exist. Do not describe proposed commands as implemented.
