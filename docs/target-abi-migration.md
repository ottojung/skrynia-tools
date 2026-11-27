# Target ABI migration: tools and SDK (T1)

This is derivative architecture for owner head `0ccc90ff7cfa8ee1a62b4a1e90b6b08677976323`
(`docs/intent-records/` in Skrynia main), not a new intent record. K0's scalar,
seek, CAS, eval and wire choices are explicit implementation assumptions, not
owner intent; the owner may override them. No production cutover, no CLI, no
legacy wire parity requirement and no old-ABI final qualification as acceptance.

## Scope of this change

`src/sdk/` is a portable, kernel-independent SDK layer plus independently
runnable tests. Existing clients, Program 0, applications, publication,
rollback, grants, push and web admin are intentionally left intact: their WASM
modules still import retired bulk names and need a later, coordinated port
after K1/K2/K3 heads land. This change does not claim the full port.

## Fixed ABI

The 19 fixed imports use `u64` shared-RAM addresses and lengths, never WASM
linear-memory pointers. `read1(fd) -> i32` yields 0..255, `-1` is EOF;
`write1(fd, u32) -> i32` yields 0 or an error; `seek(fd, i64) -> i64` is a
signed relative displacement returning the new absolute position, with a
forward-only input skip clamped at EOF; `cas1` returns the observed byte
scalar. Negative values are error codes (`-2` EBADF, `-3` EINVAL, `-4`
ENOTSUP, `-5` EIO, `-6` ERANGE, `-7` bounds, `-8` ENOENT). Only genuine
ENOENT maps to `-8`; EBADF and EIO are failures and must never be read as an
absent object. No fixed bulk read/write import,
no global RAM mutex and no engine fuel/budget is used by this layer.

## Typed FFI discovery and deployment

Discovery accepts `GET /syscalls_list` (`functions`) and `POST /version`
(`syscalls`), with ordered `i32`/`i64` parameter and result types; empty lists
are valid. Calls use the version-1 tagged little-endian wire: `0x01`, count,
then `0x32` + 4 bytes or `0x40` + 8 bytes per argument; responses use the same
layout for results. Deployment checks every module import against the exact
discovered signature before execution. Unknown, mismatched or duplicate
imports make the module ineligible; a module with an unresolved optional
import on an unused path is never selected. A failed call is final: this SDK
has no fallback replay path.

**K6 count extension (explicit implementation choice).** The v1 count byte
stays literal for 0..=254 values; 255 or more values use the `0xff` sentinel
followed by a checked 4-byte little-endian `u32` count. Extended counts below
255 are rejected as non-minimal. The decoder derives the exact frame length
from the declared result types, so truncated, oversized and mistyped replies
fail before execution; one frame is bounded by 64 MiB of checked arithmetic.
Every frame with at most 254 values is byte-identical to the original v1 wire,
including the publication provider's single-i32 `[0x01,1,0x32,u32-le]` frame.
This is an SDK/K6 decision recorded for integration, not a new owner intent.

## Eval, invoke and userland fallback

`eval` supplies zero or one integer-signature function. Selector `-1` means
no function and requires zero context handles; a nonnegative selector indexes
the evaluator's table and must match the child's single import exactly. The
function runs in the originating context; `arg0`/`arg1` are opaque i32
handles, not RAM addresses. Program 0 remains a token-checking dumb proxy
whose `invoke` protocol is userland-only; the kernel has no application
policy, no implicit child imports and no mandatory child memory.

Derived copy is a software-provider concern. `copy_bytes` is an internal byte
pump over `read1`/`write1` with documented partial progress and no replay;
it is deliberately **not** the four-integer `copy` FFI. The owner has not
fixed copy's argument meanings, order, widths or result, so no module or
signature is generated for it. An HTTP provider may omit copy and needs no
machine-context access; native Rust acceleration stays optional.

## Independence and evidence

A1 prepares application/SDK compatibility tests independently while the
kernel lanes are implemented. The gates run here are this SDK's own tests,
default rustfmt, structural lints and targeted Clippy; the existing old-ABI
qualification remains historical evidence only and is not reused as target
acceptance. K1 posted a conflicting error-code choice in its lane issue; T1
follows K0 until K0 arbitrates, then rebases the scalar constants.
