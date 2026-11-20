# Skrynia v3 — dumb proxy root (Program 0)

This optional WASM is a pure one-to-one byte proxy to one operator-installed
unprivileged guest at programs/root.wasm within Skrynia's confined data
directory. It does not decode requests, route application names, maintain a
registry, authorize users, modify persistent application data, manage
versions, store grant secrets, or implement an administration interface.

It receives two forward-only root file descriptors and consumes only the
33-byte authentication header. After verification, it loads the fixed
operator-supplied guest WASM and passes the original input descriptor (now
advanced past the header) and output descriptor directly into generic
skrynia.eval(code, code_len, input_fd, output_fd, callback). The child
reads and writes through bounded streaming channels, with no copying or
buffering of request/response bodies in Program 0. The whole guest binary
is loaded into memory up to the 8 MiB module limit; that is independent of
request size.

This version is for generic two-FD Skrynia eval (kernel branch
feat/generic-fd-eval-abi), NOT the old 7-argument buffered eval API.
The child is unprivileged: it cannot import root filesystem, global RAM
or network APIs. With callback=-1 the child cannot call skrynia.invoke.
Consequently this is a verified authenticated streaming transport, but
not yet a working application storage/API delegation mechanism; a
reviewed generic callback proxy is still required. Missing, invalid,
or absent child modules fail closed; there is no legacy fallback.

The name programs/root.wasm denotes an operator-selected ordinary child,
not an independently recognized kernel Program 1 identity. The kernel
still only knows the root WASM supplied at SKRYNIA_ROOT_WASM. The kernel
does not know anything about the token or the child module.

NO automatic migration from the former root-owned registry is performed.
That previous code is retained separately as unshippable implementation
material, not as Program 0. Durable application capability services,
code version history, authenticated rollbacks and multi-tenant dispatch must
be reimplemented in ordinary user programs with a separately reviewed
capability boundary. Do not deploy to production from this intermediate state.

## Required operator token

The provided Program 0 rejects every request unless it starts with this
fixed binary authentication header:

    0x01 | 32-byte raw bearer token | opaque request bytes

The version and 32 token bytes are **consumed by Program 0**. They are
never included in the input passed to the evaluated user program. All
request bytes after the header remain opaque and unmodified. Short or
unsupported headers, wrong tokens, missing/invalid verifier files, and
I/O errors fail closed with no application response.

The operator provisions a **cryptographically random 256-bit token** out
of band to exactly the trusted clients that need access. Never place that
token in a repository, public web bundle, URL, access log, or deployment
manifest. In a browser, require explicit user entry and retain it only
in transient session memory, never in static JavaScript. Always use HTTPS
end to end; otherwise possession of the request body allows replay.

Instead of storing the bearer token itself, Program 0 reads the raw
32-byte SHA-256 digest of that token from this private, operator-owned
file under the configured Skrynia data root:

    security/program0.sha256

This is a **binary digest**, not a text hex string. Provision the parent
directory with mode 0700 and the file with 0600, before starting the
server. Generate the token with a cryptographically secure generator,
compute SHA-256 of its **32 raw bytes**, store only that digest on disk,
and deliver the token to authorized clients via a secure out-of-band
channel. Do not expose the verifier through any app or static host.
The operator can rotate access by atomically replacing this digest;
Program 0 reads it afresh for every request, so previously authorized
tokens stop working immediately. Deleting or corrupting the verifier
disables forwarding.

The token is a **root-level bearer capability**: anybody possessing it
can make any API request that this Program 0 chooses to proxy. It is
not a substitute for per-application, per-object or delegated grants.
Authentication is mandatory even when the forwarded request is empty.
Skrynia itself has no knowledge of this header, token, or verification.
The current kernel returns an HTTP error on auth denial; no distinct
HTTP 401 response is implemented at the transport layer.

The authentication header and the body are independently handled:
Program 0 checks and removes only the header and delegates the two
remaining streams to the generic eval API. The current fixed demo guest
path is not a production application registration policy.

## Generic evaluated-program syscall forwarding (draft)

The optional Program 0 may delegate a generic, **one-to-one kernel API
relay** through the evaluated guest's skrynia.invoke callback. No
Skrynia server code knows about programs other than operator-supplied
Program 0, and Program 0 contains no v2 routes, appdata rules, grant
tree, code-release registry or deployment management.

The callback framing is deliberately small and independent of any app
or v2 protocol: requests begin with one opcode byte and typed fields,
and every reply begins with a signed little-endian 32-bit kernel status.
When READ succeeds, its reply appends exactly that many bytes.

  opcode 1 OPEN_DISK: u32 path-length | raw path | u32 mode-flags
                    => i32 opened fd or negative host error
  opcode 2 READ:      u32 fd | u32 maximum-bytes
                    => i32 n | exactly n raw bytes (n >= 0)
  opcode 3 WRITE:     u32 fd | u32 body-length | raw body
                    => i32 bytes-written or negative host error
  opcode 4 CLOSE:     u32 fd
                    => i32 close status

  opcode 5 SYNC:      u32 fd
                    => i32 sync status (file or directory)
  opcode 6 MKDIR:     u32 path-length | raw path | u32 flags
                    => i32 mkdir status
  opcode 7 RENAME:    u32 old-len | old path | u32 new-len | new path
                    => i32 rename status
  opcode 8 DELETE:    u32 path-length | raw path
                    => i32 deletion status
  opcode 9 OPEN_DIR:  u32 path-length | raw path
                    => i32 opened directory fd or negative host error
  opcode 10 NEXT:     u32 directory-fd | u32 maximum-name-bytes
                    => i32 n | exactly n bytes of entry name (n > 0);
                       n = 0 means EOF
  opcode 11 EXISTS:   u32 path-length | raw path
                    => i32 existence status, 0 false, 1 true, negative error
  opcode 12 RANDOM:   u32 requested bit-count
                    => i32 kernel status | ceil(bits/8) random bytes on
                       success; zero-count requests return no bytes

Lengths are u32 little-endian, fd is an opaque request-scoped host
descriptor encoded as u32, and unknown/trailing fields are rejected.
READ and WRITE are intentionally short (maximum 64 KiB per invocation);
clients must retry partial writes and read to EOF. READ and NEXT
check reply capacity *before* advancing their file/directory descriptor,
so a too-small reply buffer cannot silently discard data. RANDOM is
bounded to 64 KiB of output. Every descriptor belongs to one root
request; it cannot be reused after the request finishes.

For durable single-file replacement, an ordinary guest can open a new
file exclusively, write and sync it, close, rename within the same
filesystem and sync its parent directory. DELETE similarly needs
parent-directory sync. This is an atomic filesystem *replacement*
primitive; it is NOT a conditional compare-and-swap or transactional
grant/quota mutation, and a restarted kernel check is NOT a power-loss
test. The kernel maps these generic operations directly to its confined
filesystem API; Program 0 neither interprets app names nor coordinates
multiple file updates.

The owner-selected ordinary guest is a TRUSTED recipient of these
privileged generic APIs. Although it cannot directly import root
filesystem functions, it can ask the parent callback to access any
path within the configured kernel data root, INCLUDING the private
token-verifier and installed-program paths. A malicious or publicly
replaceable guest can therefore compromise the operator's data root.
Only install trusted immutable guest code under operator control;
real least-authority per-child delegation is an outstanding security
and multi-tenancy requirement. The Program0 bearer MUST NOT be exposed
to untrusted clients or browser bundles.

There is NO application-specific authorization in Program 0: anyone
with the operator's private bearer token has root-equivalent host API
authority for the confined Skrynia data root. The token is never
forwarded to the child and must NEVER be present in anonymous client
JavaScript. Future scoped grants, quotas and application protocol
rules belong in separately evaluated user programs, not the kernel or
this relay. The callback does not implement CAS, grant permissions, Web Push or
durable multi-key transactions, so it is not ready for production
stateful v2 parity.

The relay requires the generic-kernel parent/child callback authority
fix in Skrynia draft PR #111. Without it, the parent cannot read its
own privileged file descriptors and a nested eval can *lose* the outer
guest's descriptor restrictions, leaking access to private root files.
Do not deploy a callback-enabled Program 0 with the unfixed kernel.
The actual independently built Rust Program 0 and fixed Rust kernel
are qualified together in tests/v3_proxy, with all eight explicitly
run HTTP tests passing; the kernel still uses only POST /.
