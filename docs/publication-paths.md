# Publication path inventory

Scope: current v3 source at the PR98 integration head plus its deployment
configuration. This is an observed inventory, not a release approval.

| Path / surface | Root | Kind | Publication status |
|---|---|---|---|
| `security/program0.sha256` | private operator disk | token verifier | must never be published; P0 reads it after token check |
| `programs/root.wasm` | private operator disk | root program code | private; the P0 loads it with `eval_delegate`, not via nginx |
| `programs/operator.cap` | private operator disk | test operator capability | private |
| `publication-requests/<8 hex>` | private operator disk | request-only mailbox | new: create-exclusive u32 ticket, path bytes, read-only to provider, deleted after import |
| `programs/active/<id>.bin` | private user disk | mutable active pointer | private; revoke any public entry before CAS replacement |
| `programs/blob/<hash>.bin` | private user disk | immutable sealed code | private; publish only the public app copy if one is created |
| `programs/ready/<id>/<hash>.ready` | private user disk | ready marker | private; activation checks it before switching |
| `programs/appdata/<cap>/<hash>.bin` | private user disk | application data | private; never published |
| `apps/<app>/...` | public root | release entry, versions, static assets | the only publishable namespace; explicit `syscall.publish`/`unpublish` via root, never a storage alias |

## Configuration and deploy integration

- The kernel resolves `syscall.publish`/`syscall.unpublish` from
  `SKRYNIA_SYSCALL_URL` discovery; startup fails closed if either import is
  unavailable. The provider cannot receive guest pointers or private tokens.
- The provider's `PATH_TO_SKRYNIA_DISK` and `PATH_TO_NGINX_SKRYNIA_ROOT` are
  distinct roots. It reads mailbox requests from a private operator-disk mount
  and creates/removes only the corresponding public symlink. No nginx alias of
  private disk storage is permitted.
- `v3-trusted-edge-gateway` config binds app hosts/origins and authenticates
  opaque requests to the kernel. It carries no publication authority and must
  not be given the provider's disk mount.
- `program0/integration/` and `fullchain-integration/` fixtures create the
  private kernel directories and pin compiled artifacts. A publication
  acceptance fixture must additionally mount a separate nginx root and assert
  inaccessible -> exact bytes -> inaccessible with source intact.
- Version switching and rollback operate on the single active pointer. The
  public entry must be revoked before that pointer or any referenced private
  object is deleted or replaced; the revoke-then-switch helper lives in
  `native-service/src/cb_guest/publish.rs`.

The legacy v2 HTTP upload Actions were removed from this integration head; no
remaining tool in this repository assumes private `open_disk` bytes are served
directly by nginx.
