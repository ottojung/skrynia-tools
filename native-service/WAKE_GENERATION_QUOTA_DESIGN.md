# Bounded wake generation with conditional acknowledgement

**Current implementation:** `push/hints/wake/{format,ops,sender}.rs`,
`push/hints/queue.rs`, `push/sender/{list,read}.rs` and
`appdata/ops/write/{put,del,wake_hint}.rs`.

For the reserved exact key `skrynia-wake-v1`, the server stores one 16-byte
atomic slot: `WSL1 || u64_le(generation) || dirty_byte || 3 zero bytes`.
Unknown or corrupt formats fail closed. Under the mutation lock, the server
writes the next generation with dirty=true **before** source commit.
A sender lists only the current dirty generation g, reads a generic 32-byte
invalidation, and ACK(g) clears dirty only if the stored generation remains g.
Mutation g+1 between sender read and ACK therefore remains pending.

There are no per-generation wake hint files and no prune/remove step on the
write path; failure to remove an old file cannot disable ordinary writes
or hide the latest generation behind a 1,024-record listing limit.
With no subscriber the wake path does nothing; source CAS and legacy exact-key
data hints are otherwise unchanged. A failed durable slot update prevents
source commit. A crash between slot update and data commit can leave a spurious
wake only. Generation overflow is an explicit, practically unreachable
fail-closed condition.

The native-service wake suite and cross-namespace tests exercise direct
reserved-key Put/Delete, simulated restart and corruption, stale/replayed ACK,
crash failures and over 1,024 mutations during sender outage. Legacy data-key
subscriptions continue to use their separate per-sequence queue and quota.

**Not evidence of production delivery:** the ordinary P1 WASM and browser
client must be rebuilt and verified against the integrated ABI; real provider
delivery, operational credentials, five-app migration, TLS and release
qualification remain separate acceptance gates.
