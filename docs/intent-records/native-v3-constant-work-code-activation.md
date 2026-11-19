$id-0034008969876888
title: Native v3 immutable code release and constant-work activation
date: 2026/11/19
source: @ottojung
kind: requirement

Each application code version is an immutable content-addressed blob.
Authenticated application administrators can stage and explicitly seal
ready releases. Activation to any previously or subsequently sealed
ready release is an atomic single-pointer update requiring a version
revision compare-and-swap; work and memory are independent of the
number of historical releases and of the total code size, excluding
separate migrations and initial code publication. The active pointer
revision strictly increases with every activation, deactivation and
re-registration and never wraps or resets. This prevents A->B->A
stale activations, including across revocations. Unsealed or corrupted
code must not activate. Version enumeration and code publishing may
scale with the number or bytes of releases, independently of activation.
The native operator browser must list all application versions and may
activate an earlier or later ready version without a CLI.
