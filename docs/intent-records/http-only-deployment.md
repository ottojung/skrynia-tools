$id-8256068991761247
title: HTTP-only application code deployment
date: 2026/11/16
source: @ottojung
kind: requirement

Skrynia Tools must build application source on the invoking client or CI
runner and publish directory-preserving files only over the authenticated
Skrynia HTTP object protocol. Deployment and rollback must not require
SSH, an independent deployment server, a separate static hosting server,
a remote filesystem, or any server-side application code execution.
A scoped Skrynia grant user token authorizes writes. Immutable releases,
version listing, one-key CAS activation and constant-work code rollback
remain essential; the management CLI and reusable GitHub Action must
implement these capabilities directly without cutover switches.
