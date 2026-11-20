# Optional native Skrynia v3 browser and Node client

This is a dependency-free, **POST-only** client for one optional Program 0
in skrynia-tools. It is not a client for v2 HTTP grants, REST object GET,
or arbitrary independently supplied Program 0 WASM modules.

Import createNativeClient, request, sha256, hex, unhex from index.mjs.
Browser use is limited to a same-origin HTTPS proxy such as
https://tools.a.vau.place/_skrynia/. HTTP is allowed only for
loopback development servers. The client rejects redirects, URL
credentials and cross-origin browser endpoints. It never includes bearer
capabilities in the URL, cookies or referrer.

Secrets and capabilities are Uint8Array(32), not strings. Native packets
contain a version byte (1), an opcode and length-prefixed payloads.
New application registration is opcode 15 with both operator-cap and
per-app admin-cap; opcode 1 has been retired. AppGet 7, AppPut 6 and
AppDelete 8 operate with independent read and write capabilities.
Ready-version operations 9–14 support staging, sealing, activation by
monotone revision compare-and-swap, active inspection, and cursor-paged
historical versions/app IDs. Revisions are BigInt, never floating-point.

An installation program must pre-provision a 32-byte operator capability
in the privately owned Program 0 disk directory before requests can
register applications. The **operator cap must never be embedded in an
HTML/JS bundle or a public config file**. The admin interface should
retrieve it only after decrypting its user-unlocked credential vault.

Examples:

~~~js
const native = createNativeClient('/_skrynia/');
const ids = await native.programs(); // first 256 IDs
const current = await native.current(ids[0]); // live/hash/revision
const ready = await native.versions(ids[0]); // hashes, 256/page
const updated = await native.activate(appAdminCap,
  current.revision, ready[0]); // fixed-work single-pointer CAS
~~~

Mutations have **no automatic retries**. A lost HTTP response, invalid
success frame, transport failure or ambiguous disk/application error may
mean the mutation already committed; the SDK throws
NativeOutcomeUnknown. The caller must perform a fresh authoritative
read and explicitly reconcile with operator intent before attempting a
new mutation. NativeRejected means the optional root returned a
definitive error such as unauthorized, stale revision or busy.

Requests and responses are bounded to approximately 16 MiB by this
prototype client and Program 0. This is not streaming support for
arbitrarily large uploads. Directory listings are not atomic snapshots.
No code migration or storage garbage collection happens on activation;
operator-owned web hosts must serve JS/HTML assets independently
because Skrynia v3 itself only handles POST /.

Run the offline unit suite with:

~~~sh
node --test tests/native-client/unit.test.mjs
~~~

To exercise a real **disposable** kernel and independently compiled
operator-provisioned WASM:

~~~sh
SKRYNIA_V3_SERVER_BIN=/path/to/v3-server \
SKRYNIA_V3_PROGRAM0_WASM=/path/to/user-built-root.wasm \
node --test tests/native-client/live.test.mjs
~~~

No test contacts production servers or reuses an existing data directory.
