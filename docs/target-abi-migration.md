# Target ABI migration: tools and SDK (T1)

This is derivative architecture for the owner-approved 2026-11-27 change and
the published kernel source `b37cd95da8cdb14517ecf9e3236afc3611d41b56`
(`docs/intent-records/` in Skrynia main). It is not a new intent record.
K0's scalar, seek, CAS, eval and wire choices are explicit integration
decisions, not owner intent. No production cutover, no CLI, no legacy wire
parity requirement and no old-ABI qualification reused as acceptance.

## Scope of this change

`src/sdk/` is the portable, kernel-independent SDK layer with independently
runnable tests. Program 0 is migrated on its own topic branch; native-service
and publication-root sources are owned by the P1 lane. This crate checks the
hand-authored child WAT fixtures, the SDK discovery/deployment contract, the
exact kernel pin and the ABI identity in disposable CI. Ownership boundaries
are recorded, not silently crossed.

## Fixed ABI 3.1

The 21 fixed imports live in the `skrynia_v3_1` namespace; the retired
`skrynia` namespace is never linked. Every fixed buffer pointer and length is
`u64` and addresses the caller's exported, non-shared linear memory named
`memory`; scalar-only callers need no memory. Descriptor offsets (including
`cas1`) are not linear-memory pointers. Buffer ranges are validated before
effects; no borrowed view survives a host call or reentry.

`read(fd, ptr, len)` and `write(fd, ptr, len)` return an `i64` byte count:
`0` at EOF (or for a zero-length request), `0..len` for a partial transfer
that can be continued at the advanced cursor, or a negative error before
progress. `read1(fd) -> i32` yields `0..255` with `-1` at EOF;
`write1(fd, u32) -> i32` yields 0 or an error. `seek(fd, i64) -> i64` is a
signed relative displacement returning the new absolute position, with a
forward-only input skip clamped at EOF; `cas1` returns the observed byte
scalar. Negative values are error codes (`-1` EOF, `-2` EBADF, `-3` EINVAL,
`-4` ENOTSUP, `-5` EIO, `-6` ERANGE, `-7` bounds, `-8` ENOENT). Only genuine
ENOENT maps to `-8`; EBADF and EIO are failures and must never be read as an
absent object. No global RAM mutex and no engine fuel/budget is used here.

Optional derived `copy` stays a software-provider accelerator. The bounded
portable `copy_buffered` reference explicitly uses bulk read/write with short
transfer progress and no replay. It is **not** the four-integer `copy` FFI:
the owner has not fixed that signature, so no import or guessed argument
order is generated. An HTTP provider may omit copy.

## Typed FFI discovery and deployment

Discovery accepts `GET /syscalls_list` (`functions`) and `POST /version`
(`syscalls` and `"abi":"skrynia_v3_1"`), with ordered `i32`/`i64` parameter
and result types; empty lists are valid. `parse_version` rejects a missing or
mismatched ABI identity before any import is used. Calls use the version-1
tagged little-endian wire: `0x01`, count, then `0x32` + 4 bytes or `0x40` +
8 bytes per argument; responses use the same layout for results. Deployment
checks every module import against its namespace and exact ordered types:
fixed imports use the 21-name SDK inventory, operator-defined named FFI
imports use the `syscall` discovery list, and any other namespace is
forbidden. Unknown, mismatched or duplicate imports make the module
ineligible; a module with an unresolved optional import on an unused path is
never selected. A failed call is final: no fallback replay after execution.

**K6 count extension (explicit implementation choice).** The v1 count byte
stays literal for `0..=254` values; 255 or more use the `0xff` sentinel plus
a checked little-endian `u32` count. Extended counts below 255 are rejected
as non-minimal. The decoder derives the exact frame length from the declared
result types, so truncated, oversized and mistyped replies fail before
execution; one frame is bounded by 64 MiB of checked arithmetic. Every frame
with at most 254 values is byte-identical to the original v1 wire.

## Eval, invoke and userland fallback

`eval(code, code_size, function, arg0, arg1)` reads code bytes from the
evaluator's exported linear memory and the kernel snapshots them before
validation and reentrant execution. The callback contract is unchanged:
selector `-1` means no function and zero context handles; a nonnegative
selector indexes the evaluator's table and must match the child's single
import exactly. The function runs in its originating context; `arg0`/`arg1`
are opaque i32 handles, not RAM addresses. Program 0 remains a token-checking
dumb proxy whose `invoke` protocol is userland-only. A child imports exactly
one userland `p0.invoke`; fixed host imports are never injected into
children.

## Evidence boundary

CI pins the exact published kernel source and checks both the source and
compiled-binary ABI identity. The gates run the SDK's own tests, default
rustfmt, structural lints and strict Clippy. Old-ABI qualification artifacts
remain historical and are not promoted to target acceptance. P0, P1 and
kernel lane evidence is integrated by exact topic commit, not by replaying
superseded results.
