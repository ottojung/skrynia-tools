$id-2006686696634046
title: Web-based Skrynia Tools administration and atomic release management
date: 2026/11/17
source: @ottojung
kind: requirement

The Skrynia Tools administration interface must be a web application, not
a command-line admin interface. It is served as static assets on a separate
trusted origin and sends opaque binary POST through an operator gateway. The
gateway holds the server-only Program 0 token, which never enters public JS,
URLs or logs. Release identity, history and activation are enforced by the
ordinary nonzero WASM service behind that gateway, not by a bootstrap step.

Scope clarification (provenance reconciled 2026/11/25 on the hall board,
[bootstrap-intent-design]): owner messages 2026-11-17 19:53:50 and
2026-11-17 20:38:44 retain a one-time, trusted-host Bash bootstrap that reads
its configuration from the environment or `/dev/tty`, publishes the web admin
assets through the operator's native publication helper and prints a stable
link plus a strong random access password. It is not a user-facing CLI and
grants no browser capability before a successful unlock; it never carries a
root, Program 0, operator or sender token into public assets, URLs or logs.

Withdrawn 2026/11/24 in f92fbea5: the previous `deploy.sh` / `bootstrap/*`
transport that carried the then-v2 grant, and the HTTP-based `action/deploy/*`,
`action/publish-stable/*` and `.github/workflows/web-admin.yml` surfaces.
Those legacy HTTP/v2-grant surfaces are forbidden by AGENTS.md and stay
withdrawn; the one-time bootstrap concept above is not. Git carries the
withdrawn record.

The web interface must display every retained version of every app and
support atomically changing any application's active code release to
any previous or later retained version. Rollback does not perform data
migrations or copy release files.
