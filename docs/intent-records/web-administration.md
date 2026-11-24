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

Withdrawn intent (surfaced 2026/11/24): the earlier claim that a one-time Bash
bootstrap receives `SKRYNIA_URL` and scoped deployment authority via
environment variables or interactive prompts, publishes the interface and
prints a random access password, is no longer current intent. That bootstrap
(`deploy.sh`, `bootstrap/deploy.mjs`, `bootstrap/transport.mjs`) was deleted
in f92fbea5 and is forbidden by AGENTS.md; no replacement exists. Git carries
the withdrawn record; do not treat it as current intent.

The web interface must display every retained version of every app and
support atomically changing any application's active code release to
any previous or later retained version. Rollback does not perform data
migrations or copy release files.
