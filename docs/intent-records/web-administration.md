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

Current intent (direct user messages 2026-11-17 19:53:50 and 20:38:44 UTC): a
ONE-TIME trusted-host bootstrap receives the operator URL and scoped native
deployment authority from environment variables or interactive /dev/tty
prompts, publishes this web interface as immutable static assets, and prints a
stable link plus a freshly generated random access password. The bootstrap is
not an administrative CLI and does not grant general deployment commands.

In native v3 the one-time bootstrap must publish through the already
provisioned publication root/provider and private artifact root (reusing the
native publication helper semantics, never writing nginx config or the
servable tree directly), and the printed password must actually protect the
browser's scoped native admin/read session: an encrypted bootstrap vault may
contain only that scoped session material, never Program 0/root, operator,
sender or other server-only tokens, and no plaintext secret may enter public
assets, URLs or logs. The same-origin trusted gateway is an existing
operator prerequisite.

Commit f92fbea5 removed the obsolete LEGACY v2 deploy routes and v2 grant
form; it did not withdraw this one-time bootstrap. The 2026/11/24 record
calling the whole bootstrap withdrawn was incorrect; Git carries its history.

The web interface must display every retained version of every app and
support atomically changing any application's active code release to
any previous or later retained version. Rollback does not perform data
migrations or copy release files.
