# Real backend + Chromium browser integration

Reference integration harness for `web-admin-v3` against a nonzero-style
userland registry backend with **real filesystem-persistent storage** and
**real Chromium browser UI tests**.

- `registry_service.mjs`: filesystem-backed registry service implementing
  app listing, app state/version history, manifest reads, and ABA-safe
  constant-work active-pointer CAS activation/rollback (`revision`/pointer
  version). State persists on disk across process restarts.
- `gateway.mjs`: local HTTP server serving `web-admin-v3` and a dumb
  `POST /_skrynia/` transport that forwards opaque JSON service requests to
  the registry service (no Program 0 application semantics in gateway).
  Requires a gateway session credential on service requests when configured
  (separately provisioned, **not** a Program 0 bearer token).
- `gateway_adapter.js`: browser capability adapter using same-origin
  `POST /_skrynia/`; fetches gateway session credential dynamically at
  runtime (test harness bootstrap, not in static `web-admin-v3/` bundle);
  **no root/Program 0 bearer token in browser**.
- `test.html`: drives the actual `web-admin-v3` UI (vault unlock/init,
  refresh, manifest inspect, activation/rollback) and reports results.
- `run.mjs`: seeds storage, runs backend CAS/ABA/conflict checks, executes
  Chromium browser phases, restarts gateway on same storage to verify
  persistence and post-restart CAS continuity.

Run:

```sh
node tests/web-admin-v3/real_backend/run.mjs
```

The run verifies authenticated/unauthenticated gateway access, immutable ready
release history, ABA-safe constant-work pointer CAS activation/rollback,
Chromium UI flows, and persistence across restart.

This is a protocol/reference backend integration harness; it does not run the
registry service as a WASM guest inside a Skrynia kernel/Program 0 stack.
