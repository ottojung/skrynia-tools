$id-5926197117582876
title: Protocol-independent applications
date: 2026/11/15
source: @ottojung
kind: requirement

Skrynia Tools applications must not be assumed to use append-only logs. The deployment lifecycle must work without any particular data protocol, including for applications that have no persistent Skrynia data. Optional client-side protocols are independently chosen by the application.

$id-3506129748450785
title: Primary GitHub subdirectory application deployment
date: 2026/11/15
source: @ottojung
kind: requirement

Skrynia Tools must primarily deploy applications from a GitHub repository URL and a subdirectory within that repository, preserving the existing Skrynia Git-based workflow. The historical contract builds the chosen full Git commit from its specified subdirectory, keeping the full monorepo checkout accessible, runs `make build` producing `build/`, and supports immutable releases, atomic activation and rollback. Deployment must not require the Skrynia server to implement application protocols.
