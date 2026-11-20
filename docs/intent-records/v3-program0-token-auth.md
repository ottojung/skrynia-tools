$id-0679002719090531
title: Program 0 authenticates before forwarding API calls
date: 2026/11/19
source: @ottojung
kind: requirement

The Skrynia server knows and executes only the operator-supplied Program 0
and exposes generic APIs. The optional Program 0 supplied by skrynia-tools
must authenticate its callers using a high-entropy token known only to
authorized clients and the operator. Program 0 must reject all requests
without the correct token before invoking any further API or guest code,
must strip the token before forwarding, and must not embed it in a public
program or downloaded browser artifact. The root operator can revoke or
rotate the token without a server reboot, and failures must be fail-closed.
Only Program 0 knows the authentication protocol; the server does not
contain application-specific authentication, code registries or privileged
special-purpose Program 1 configuration. Program 0 is otherwise a dumb
generic API proxy, not a registry or application server.
