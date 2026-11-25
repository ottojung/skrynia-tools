$id-2918869708597732
title: Remove the Skrynia Tools command-line interface entirely
date: 2026/11/17
source: @ottojung
kind: requirement

Skrynia Tools must not expose any CLI functionality. Administration takes
place through the web interface, which sends opaque binary POST through an
operator gateway. There is no user-facing command-line admin interface.

A ONE-TIME trusted-host Bash installation bootstrap remains current intent
(direct user message 2026-11-17 19:53:50 UTC; confirmed again 2026-11-17
20:38:44 UTC: no administrative CLI, but the one-time bootstrap is retained).
It is not a CLI and not a general deployment service: it is run once by the
operator, takes only env values or /dev/tty prompts, and publishes the web
admin assets. In native v3 it must use the operator-supplied publication
root/provider and private artifact root, never a v2 grant, `/store`, an
HTTP deployment facade or direct nginx writes. The kernel's separate
no-bootstrap position concerns the kernel, not this tools-side bootstrap.

Commit f92fbea5 deleted only the obsolete LEGACY v2 HTTP surfaces
(`bootstrap/*` v2 scripts, `deploy.sh` v2 form, `action/deploy/*`,
`action/publish-stable/*`, `.github/workflows/web-admin.yml`). It did not
withdraw the one-time bootstrap itself. The 2026/11/24 record calling the
whole bootstrap withdrawn was incorrect; Git carries its history.
