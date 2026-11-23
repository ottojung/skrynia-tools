# Abandoned P1 per-object sentinel workaround (do not merge)

A local-only P1 source branch (`antonina/p1-push-kernel-list-compat-20261122`,
from `efe6117`) trialled a durable `has-subs` marker so `enqueue_hints` could
skip `disk.list()` on absent nested directories. It was **never pushed**, and
has been deleted. Decisive real-binary testing with the owner's corrected
generic kernel PR125 exact `856c029c6e428706a475b2fb06dbe2e2357eb105` showed
the clean PR65-head P1 WASM works unchanged (31 named PASS / ALL_OK=True push E2E PASS). The kernel
fix maps ENOENT -> ERR_NOT_FOUND(-2) in `fs/dir/path.rs::open_path` only,
preserving ENOTDIR/EACCES/other errors as real I/O/permission failures.

Do NOT invent per-object sentinel indices or globally treat ERR_IO(-5) as
"empty" in P1 `Disk::list`: that would conceal genuine ENOSPC/EACCES/corruption
and break fail-closed semantics. Evidence: `fault_inject_list.log` (ENOTDIR
injected -> AppPut err 7, fail closed).
