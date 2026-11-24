$id-2918869708597732
title: Remove the Skrynia Tools command-line interface entirely
date: 2026/11/17
source: @ottojung
kind: requirement

Skrynia Tools must not expose any CLI functionality. Remove its
CLI executable, command parser, installation package, tests and
documentation. Administration must take place through the web interface,
which sends opaque binary POST through an operator gateway. There is no
one-time Bash installation bootstrap, no HTTP-based deployment action and
no publishing workflow: those surfaces are withdrawn, not a separate
administration channel.

Withdrawn intent (surfaced 2026/11/24): the earlier claim that a one-time Bash
installation bootstrap and noninteractive GitHub Actions for application
publishing "remain distinct from a user-facing CLI" is no longer current
intent. `bootstrap/*`, `deploy.sh`, `action/deploy/*`,
`action/publish-stable/*` and `.github/workflows/web-admin.yml` were deleted
in f92fbea5, and AGENTS.md forbids reintroducing HTTP-based deployment
actions. Git carries the withdrawn record; do not treat it as current intent.
