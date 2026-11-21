# Theorem Lab feedback: native v3 browser migration (disposable, not release)

Status: disposable design + browser-side implementation for Antonina `ygD` /
`w4wg`. **Not release-ready.** Kernel `4389433` and Program 0 remain security
blocked; no production/DNS/TLS/token/Antonina board changes were made.

## What changed in the browser

`source/theorem-lab/feedback.js` no longer uses the obsolete
`/_skrynia/store/theorem-lab/feedback-v1` GET/POST/PUT path or the
`X-Skrynia-Mode: public-write` header. It now:

- sends **POST-only** `application/octet-stream` requests to same-origin
  `/_skrynia/` (native P1 binary request bodies);
- writes through **OP_INVOKE (3)** to a registered, mediated P2 feedback handler
  using a **public invoke ID** configured at deploy time
  (`window.__THEOREM_LAB_FEEDBACK_V3__`);
- reads through **OP_APP_GET (7)** only when an explicitly configured public
  read-only cap exists, otherwise through OP_INVOKE (P2 read command);
- fails closed on missing/invalid endpoint, service, configuration or
  capability — there is no insecure GET/store fallback;
- preserves annotate/drag-pointer feedback, Copy JSON, Cancel, Save, and the
  CAS merge UX (append retries on conflict);
- keeps **write_cap, admin_cap, Program 0/root tokens entirely out of browser
  JS**; no Authorization/token headers are sent.

Configuration (public identifiers only):

```js
window.__THEOREM_LAB_FEEDBACK_V3__ = {
  endpoint: '/_skrynia/',
  invokeIdHex: '<public P2 invoke id, 32 bytes hex>',
  readCapHex: '<optional public read-only cap, 32 bytes hex>',
};
```

If the trusted edge requires an edge-signed anonymous visitor session before
it will accept `/_skrynia/` POSTs, that session provisioning must be provided
by the edge; do **not** embed a shared powerful token in static JS. The exact
dependency is documented below.

## Wire protocol assumptions (coordinate with `Jmf1...` on `ygD`)

These bytes follow the native-service wire format already used by the
integration kit (`OP_INVOKE=3`, `OP_APP_GET=7`, `OP_APP_PUT=6`, `VERSION=1`):

```
request := [0x01, op] ++ id16 ++ blob32
id16    := u16_le(len) ++ bytes            (32-byte invoke id / read cap)
blob32  := u32_le(len) ++ bytes            (P2 command JSON / app key)
P2 cmd  := {"v":1,"op":"read"} | {"v":1,"op":"append","entry":{...},"fragments":{...}}
P2 resp := {"ok":true, "state":{...}} | {"ok":true,"exists":bool,...} | {"ok":false,"code":409,...}
P1 resp := [0x01, status, u32_le(body_len), body...]   (status 0 = OK, 1 = ERR)
```

P1 registry invokable P2 registration, opcode/version assignment and the P2
feedback handler ABI are owned by the separate engineer `Jmf1...` on Antonina
`ygD`. This branch assumes the above stable opcode values; if `Jmf1...`
assigns different op bytes or a different P2 command envelope, only the small
encoder/command shapes in `feedback.js` need adjustment.

## Trust boundary

- Public edge (nginx/static) is POST-only and fail-closed; it never proxies
  directly to the native kernel and never injects/logs P0/root tokens (see
  the `p0-trusted-gateway-fix` work and `docs/static-host-v3-deployment.md`).
- A separate authorized trusted gateway performs independent app-scoped
  auth/session/capability checks and privately injects the P0 token. That
  gateway component is **not** implemented or validated here.

## Local executable checks (disposable)

- `node scripts/test-theorem-lab-feedback-boundary.mjs` — static boundary
  checks: no Program0/root token, no write/admin cap, no legacy store path,
  POST-only `/_skrynia/`, fail-closed config, no insecure fallback. **PASS.**
- `node scripts/test-theorem-lab-feedback-v3-mock.mjs` — headless Chromium
  mock acceptance: fail-closed when unconfigured; Save emits OP_INVOKE POSTs
  only; Copy JSON works; CAS conflict triggers append retry; no legacy store
  or non-POST requests; no capability/token/secret command fields and no auth
  or P0 token headers. **PASS.** (This is a mock gateway speaking the P1 wire
  format, not a compiled native stack.)

## Genuine compiled-stack acceptance (disposable, passing)

`scripts/test-theorem-lab-feedback-v3-native.py` runs the **exact matched
secure current-main artifacts** (no mocks) behind real nginx HTTPS, a separate
trusted-edge gateway and headless Chromium. Verified SHA-256 at runtime:

| Role | Artifact | SHA-256 |
| --- | --- | --- |
| Kernel (PR122 exact head `97a964d4`) | `wasm-build-record/main-secure-pr122-20261121/skrynia-server` | `8da114ff4c6c835621eca71a2840c4262c04b9ac1d37705b058dc2f3f9168d0f` |
| P0 (clean dumb PR53) | `hAx-p0-dumb-target/.../skrynia_program0.wasm` | `ec74523be4d537969b6a9b93d702ba5f550a1205ca7fa85c3dc3261c5e1645dc` |
| P1 (native service) | `artifacts/skrynia_native_service_aa6333a.wasm` | `d3686527bbf290704bf967b2e97f37dda8ff487b7d9f58db28d2a806712f4dd2` |
| P2 (echo guest) | `tests/guests/echo_guest.wasm` | `dc28e490b107084224562cc02a185e99c64727d6dd185eab5645989aa9c17ec5` |

The repro fixture `dumb_p0_http.py` on these exact artifacts also ran
`ALL_OK=True` (register / lookup / P2 invoke / AppPut / Get / CAS / restart and
bad-bearer / user-shadow / sibling negatives).

Executed checks (all **PASS**, deterministic across repeated runs):

- exact artifact hashes; real P2 registration via OP_REGISTER_AUTH and a real
  echo P2 invoke through the compiled kernel;
- nginx HTTPS up and `nginx -t` valid; headless Chromium drives the real
  `feedback.js` Save flow → `OK:browser-native-flow`;
- ephemeral app-scoped edge session cookie accepted by the trusted edge;
- no P0 token or admin/write-cap value in logs or the served page;
- public OP_APP_GET read returns the saved feedback state;
- deny bad Host (403), bad Origin (403), wrong session (403), non-POST (405),
  and wrong bearer at the kernel (fail closed);
- real CAS race (concurrent writer injected once) → browser append gets a 409
  conflict and retries successfully;
- kernel restart with the same dirs preserves state; A→B→A appdata CAS returns
  to the original value/hash.

The trusted edge holds the private P0 token and app caps; the browser only ever
sees a public invoke ID / read ID and an HttpOnly app-scoped session cookie.
The disposable test omits `Secure` on the cookie because it uses a self-signed
certificate; production must set `Secure`.

**Still blocked / not claimed:** no production deploy or publication; no real
Program0 root token; no Antonina board update posted by the agent (no board
access in this environment); P2 feedback-handler ABI and opcode assignment
still require sign-off from `Jmf1...` on `ygD` — the echo P2 here exercises the
invoke path and the trusted edge mediates appdata CAS for the fake records.
The kernel/P0 remain under independent ChR security review and five-app
acceptance; all PRs stay drafts. No release claim is made.
