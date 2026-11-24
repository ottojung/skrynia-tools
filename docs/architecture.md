# Skrynia Tools architecture

Skrynia is a generic WASM execution kernel. Application protocols belong to
ordinary user-supplied programs. Tools provides optional components:

- **Program 0:** operator-selected root WASM, authenticating a private token
  and forwarding opaque bytes. Generic callbacks delegate bounded authority;
  application storage, code registry and push semantics are not root functions.
- **Ordinary native service:** capability-authorized data, immutable sealed code
  versions, a revision-checked active pointer, and exact-key change hints.
- **Trusted edge:** authenticates an app-scoped caller and forwards opaque
  binary requests to the kernel with the private root token.
- **Native clients:** Rust and browser WASM adapters for the ordinary service.
  Optional higher-level data protocols have independent correctness assumptions.
- **Web administration:** static HTML/JS on a separate trusted HTTPS origin;
  native admin operations, local encrypted vault, version discovery and
  atomic activation. No command-line administrative interface.
- **Web Push sender:** separate process with narrowly authorized hint access and
  outbound delivery. Durable service state, not notifications, is authoritative.

The static host is separate from execution. A stable origin and release
entrypoint preserve each application's navigation and service worker boundary.
Building from an exact Git commit and publishing immutable static assets are
deployment concerns, not kernel operations.

## Costs and failure boundaries

Stage code by content identity, seal it before activation, and activate by one
revision-checked mutation of the fixed-size active pointer. This operation does
not copy release bytes, scan historical releases or migrate application data.
Listing history and verifying artifacts have their own costs.

A rejected CAS is distinct from a missing or uncertain response. Reconcile
the authoritative state before another attempt. Separate static asset retention,
data migration and user-held authorization from code activation.

See [protocols](protocols.md), [security](security.md), [browser SDK](browser.md),
and [web administration](../web-admin-v3/README.md).
