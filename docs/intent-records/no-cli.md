$id-2918869708597732
title: Remove the Skrynia Tools command-line interface entirely
date: 2026/11/17
source: @ottojung
kind: requirement

Skrynia Tools must not expose any CLI functionality. Remove its
CLI executable, command parser, installation package, tests and
documentation. Administration must take place through the web interface,
which sends opaque binary POST through an operator gateway.

Scope clarification (provenance reconciled 2026/11/25 on the hall board,
[bootstrap-intent-design]): owner messages 2026-11-17 19:53:50 and
2026-11-17 20:38:44 retain a single one-time, trusted-host Bash installation
bootstrap for Skrynia Tools. It is a one-shot installer with no subcommands:
not a user-facing CLI and not an HTTP-based deployment action. It uses pinned
reviewed Tools source and scoped native admin/read authority, and must never
reintroduce a v2 `/store` grant, a legacy HTTP deployment facade, or a kernel
bootstrap.

Withdrawn 2026/11/24 in f92fbea5: only the obsolete LEGACY v2 HTTP deploy
routes/actions (`action/deploy/*`, `action/publish-stable/*`,
`.github/workflows/web-admin.yml`) and the v2-grant-bearing `bootstrap/*`
transport. Those legacy HTTP surfaces are forbidden by AGENTS.md and stay
withdrawn; their removal did not withdraw the one-time bootstrap concept
above. The 2026/11/24 record that labelled the whole bootstrap withdrawn was
over-broad; Git carries its history.
