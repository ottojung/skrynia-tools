# Proposed delivery and validation sequence

**Status: planning only.** The Rust scaffold and CI are present; none of these client protocols or CLI commands is implemented.

## Phase 0 — dependable raw substrate

Build typed keys, matchers, byte-SHA256 digests, token/credential types, explicit transport interfaces, errors and public GET / authenticated create, replace, delete, CAS and paginated list. Prove the transport layer against a pinned Skrynia server and deterministic injected fakes.

## Phase 1 — minimal useful protocols

Deliver `CasCell`, then create-only `AppendLog` with lost-response recovery, concurrency and bounded scans. Implement the ordinary native CLI for raw operations, diagnostics and logs. Define a versioned public app manifest format and **pure** provisioning-plan generator before remote app mutations.

## Phase 2 — secure app provisioning

Implement exact grant lifecycle, private credential store, journaling, redacted JSON and explicit ambiguous-result handling. Add `admin app plan/create/inspect/issue-user/revoke-user/diagnose` plus local `app new` templates. End-to-end tests use isolated test grants and verify scope separation, quota errors and loss of issuer responses.

## Phase 3 — portable execution

Add WebAssembly/browser `fetch` transport and narrow TypeScript interface over the **same Rust protocol algorithms**. Keep optional features modular and Wasm artifacts small. Test real browsers, cross-language formats and origin restrictions.

## Phase 4 — advanced coordination

Add immutable snapshot transactions, event sets, sealed-epoch garbage collection, then evaluate task queues and cryptographic helpers as separately specified modules. Do not claim garbage collection correct before seal/revocation and registered-consumer safety are model-tested.

## Release gates and open decision

Every public protocol must have normative versioned format, proof obligations, deterministic fault injection (including ambiguous commits), adversarial tests, integration tests against real Skrynia, native/Wasm parity where relevant, and CLI output/security tests. No unexecuted tests may be reported green. Choose the SDK/CLI Free Software license explicitly before public distribution; server AGPL does not automatically dictate client library license.
