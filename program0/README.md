# Skrynia v3 — dumb proxy root (Program 0)

This optional WASM is a pure one-to-one byte proxy to one operator-installed
unprivileged guest at programs/root.wasm within Skrynia's confined data
directory. It does not decode requests, route application names, maintain a
registry, authorize users, modify persistent application data, manage
versions, store grant secrets, or implement an administration interface.

It receives the two forward-only root file descriptors, reads the complete
request, loads the operator's fixed guest module, invokes that WASM with the
opaque bytes, and forwards the returned bytes to output. It provides no
callback for skrynia.invoke and cannot grant an untrusted guest disk/RAM
access. Missing or invalid guest returns an error, never a fallback service.
Input is capped at 16 MiB, child code at 8 MiB, output at 8 MiB. These are
temporary implementation bounds, NOT true streaming: Skrynia's current
eval guest ABI is memory-buffered. To make the proxy fully streaming, the
kernel must expose a safe guest FD execution ABI, preserving child isolation.

NO automatic migration from the former root-owned registry is performed.
That previous code remains in a separate unshippable development branch,
not in the kernel or in this clean Program 0. Durable application capability services,
code version history, authenticated rollbacks and multi-tenant dispatch must
be reimplemented in ordinary user programs with a separately reviewed
capability boundary. Do not deploy to production from this intermediate state.
