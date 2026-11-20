$id-9817296693638097
title: v2 API as an ordinary user program under generic v3 evaluation
date: 2026/11/19
source: @ottojung
kind: requirement

The v3 Skrynia kernel must know only its operator-supplied Program 0, and
must never contain a dedicated special Program 1, a v2 HTTP facade, or
application-specific routing. The user may implement v2 API semantics as
an ordinary evaluated program, conventionally called Program 1, which
communicates through generic evaluation callbacks to the v3 API provider.
Program 0 remains a dumb token-authenticated proxy of v3 API calls.
HTTP method/path/header compatibility is handled outside the kernel when
needed by an edge adapter. Preserve exact v2 authority, CAS/quota behavior,
revoke and push contracts through an independently qualified ordinary guest;
do not claim compatibility from read-only response parity.
