# Intent Records

Record current directly confirmed user intent, distinct from derived implementation decisions. See [applications](applications.md) for current deployment intent. Follow Volodyslav's format:

```text
$id-<16 random decimal digits>
title: Human-readable intent title
date: YYYY/MM/DD
source: @ottojung
kind: requirement

Self-contained statement of current user intent.
```

IDs must be random, unique, stable for an active intent, and not mnemonic. Kinds include requirement, preference, constraint, accepted-tradeoff and rejected-concern. An agent task prompt, existing repository artifact or inference must never be attributed to `@ottojung` as direct intent. Surface conflicts explicitly and remove superseded intent; Git carries its history.

- [No CLI](no-cli.md) — web administration replaces the removed CLI entirely.
- [App versions and rollback](app-management.md) — app inventory and constant-work code rollback intent.
- [Web administration](web-administration.md) — self-hosted web admin with release history, atomic backward and forward activation.
- [App origins](app-origins.md) — stable per-app hosts and relative-link releases.
