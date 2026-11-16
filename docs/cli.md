# Skrynia Tools CLI

**Implemented commands**, executed by the `skrynia-tools` native Rust binary (or an installed `skrynia` alias). See `skrynia-tools help` for a summary. Static application commands do not require a Skrynia token, an append-only log, or access to the Skrynia object store.

## Application releases

Set `SKRYNIA_APPS_DIR=/srv/skrynia-apps` to use a default hosting root for interactive `app` commands. An explicit `--apps-dir` overrides the environment. No application data or migration protocol is required.

```sh
skrynia-tools app new --name hello --directory ./apps/hello
skrynia-tools app deploy --name hello --repo https://github.com/OWNER/REPO --subdir apps/hello --apps-dir /srv/skrynia-apps [--commit FULL_SHA] [--builder-image IMAGE]
skrynia-tools app list --apps-dir /srv/skrynia-apps [--format table|json]
skrynia-tools app doctor --apps-dir /srv/skrynia-apps [--format table|json]
skrynia-tools app audit --name hello --apps-dir /srv/skrynia-apps [--format table|json]
skrynia-tools app current --name hello --apps-dir /srv/skrynia-apps [--format table|json]
skrynia-tools app versions --name hello --apps-dir /srv/skrynia-apps [--format table|json]
skrynia-tools app show --name hello --release RELEASE_ID --apps-dir /srv/skrynia-apps [--format table|json]
skrynia-tools app compare --name hello --from OLD_ID --to NEW_ID --apps-dir /srv/skrynia-apps [--format table|json]
skrynia-tools app verify --name hello [--release RELEASE_ID] --apps-dir /srv/skrynia-apps [--format table|json]
skrynia-tools app rollback --name hello --release RELEASE_ID --apps-dir /srv/skrynia-apps [--format table|json]
# Compatibility aliases:
skrynia-tools app inspect --name hello --apps-dir /srv/skrynia-apps
skrynia-tools app releases --name hello --apps-dir /srv/skrynia-apps
skrynia-tools app history --name hello --apps-dir /srv/skrynia-apps
skrynia-tools app activate --name hello --release RELEASE_ID --apps-dir /srv/skrynia-apps
```

`app new` fails if the destination already exists. `app deploy` clones the whole Git repository (HTTPS or `git@github.com:owner/repo.git`), checks out the specified or resolved commit, runs the Makefile's `build` target in the given subdirectory inside rootless Podman, validates the hierarchical `build/` output and switches one symlink atomically. It makes **no assumption about the files in `build/`**, preserves arbitrary regular files and nested directories, and records source/commit/subdirectory/builder metadata. Errors leave any previous active release intact. A build lock serializes source builds; a separate brief activation lock protects only the active-code pointer. The CLI host must have writable access to the static server's app directory; no server-side deployment endpoint is called.

The default format for `list`, `current`, `versions`, `history`, `rollback` and `activate` is a readable table or confirmation. `inspect`, `show` and legacy `releases` retain JSON output by default. All views accept `--format json|table`. The `versions` display lists the newest published snapshots first, marking the currently active one with `*` in tables and `"active": true` in JSON; legacy `releases` JSON remains unchanged. Legacy releases without a publication timestamp remain available and show `unknown`. `show` and `rollback` address a version **directly by its opaque release ID**, without enumerating the history. `list` includes registered apps without an active release. The `verify` command explicitly scans the requested release (default: currently active), recomputes its directory-tree SHA-256 and reports verified, mismatch or unverifiable legacy metadata. A mismatch or missing expected digest returns a nonzero exit status; verification reads all artifact bytes, unlike rollback.

The `doctor` command checks for a real deployment root, available Git, rootless Podman and initialized release directories. It is **diagnostic only**: it does not build applications, pull images, create the root or verify sufficient disk capacity or production web routing. Missing internal subdirectories are reported but can be created by an actual deployment.

The `compare` command reads two records directly and compares their source provenance (commits, repositories, builder images, timestamps), **not** their file contents. Code rollback never invokes application migrations or changes Skrynia object data.

## Remote CLI release management

Any implemented code-management command, including `app deploy`, `list`, `versions`, `current`, `show`, `compare`, `verify`, `doctor`, `rollback` and `activate`, can run on an authorized host by adding:

```text
--ssh-host HOST --ssh-user DEPLOY_USER --ssh-key PRIVATE_KEY_FILE
--ssh-known-hosts PINNED_KNOWN_HOSTS_FILE [--ssh-port PORT]
```

For example:

```sh
skrynia-tools app versions --name hello --apps-dir /srv/skrynia-apps \
  --ssh-host deploy.example.org --ssh-user deployer \
  --ssh-key ~/.ssh/skrynia-deploy --ssh-known-hosts ~/.ssh/skrynia-hosts

skrynia-tools app rollback --name hello --release RELEASE_ID \
  --apps-dir /srv/skrynia-apps --ssh-host deploy.example.org \
  --ssh-user deployer --ssh-key ~/.ssh/skrynia-deploy \
  --ssh-known-hosts ~/.ssh/skrynia-hosts
```

The CLI opens an SSH connection with strict host-key checking, no agent forwarding, no proxy commands and no inherited SSH configuration. It sends a small JSON request to `skrynia-tools app rpc --spec-stdin true` on the remote machine. The restricted RPC handler checks `SKRYNIA_DEPLOY_ROOT`, accepts only listed release operations, and uses the **same local code** for formatting and atomic rollback. The `--apps-dir` value must identify the authorized directory **on the remote host**; `SKRYNIA_APPS_DIR` can supply that default. The private key must be a non-symlinked, owner-only regular file; the trusted host-key file must be supplied explicitly.

SSH failures and timeouts do not prove that a remote activation failed: inspect `app current` before retrying. Remote deploys have a 30-minute command timeout; management commands have a two-minute limit. The remote command does not expose object storage credentials.

## Optional object data

```sh
skrynia-tools object get --url URL --key KEY [--output FILE]
skrynia-tools object create --url URL --token-file PRIVATE_FILE --key KEY --file FILE
skrynia-tools object replace --url URL --token-file PRIVATE_FILE --key KEY --file FILE [--if-sha256 LOWERCASE_HEX]
skrynia-tools object delete --url URL --token-file PRIVATE_FILE --key KEY [--if-sha256 LOWERCASE_HEX]
skrynia-tools object list --url URL --token-file PRIVATE_FILE --locations 'a.myapp.*'
```

GET returns raw binary bytes (stdout if no output file). Listing follows the Skrynia key cursor and prints one JSON page per line; it does not promise a cross-page snapshot. Conditional replace/delete use the SHA-256 digest **of object bytes**, not an arbitrary static-server ETag. Token files must be owner-only regular files (e.g. mode 0600).

## Optional create-only log

```sh
skrynia-tools log init --url URL --token-file PRIVATE_FILE --prefix chat.log
skrynia-tools log append --url URL --token-file PRIVATE_FILE --prefix chat.log --operation-id 0123456789abcdef0123456789abcdef --file message.bin
skrynia-tools log read --url URL --prefix chat.log [--from 1]
```

A successful append returns its integer slot number; read output is newline-delimited JSON with position, operation ID and hex payload. Appends scan a contiguous prefix and can be expensive; a caller-supplied operation ID enables duplicate suppression across retries. Create-only writers must cooperate and must not delete or mutate live entries; see [protocols](protocols.md).

## Optional app data provisioning

```sh
skrynia-tools admin plan --name hello --max-bytes 1048576 --max-objects 100 [--id 32_HEX_DIGITS]
skrynia-tools admin create --name hello --url URL --parent-token-file PRIVATE_FILE --credentials-out PRIVATE_FILE --max-bytes 1048576 --max-objects 100 [--id 32_HEX_DIGITS]
skrynia-tools admin inspect --credentials-file PRIVATE_FILE
skrynia-tools admin resume --credentials-file PRIVATE_FILE --url URL
# Only after investigating whether the previous user-token issuance committed:
skrynia-tools admin resume --credentials-file PRIVATE_FILE --url URL --accept-duplicate-token-risk true
skrynia-tools admin revoke-user --credentials-file PRIVATE_FILE --url URL --confirm-grant-id ID
skrynia-tools admin revoke-grant --credentials-file PRIVATE_FILE --url URL --confirm-grant-id ID
# Only after investigating an interrupted DELETE:
skrynia-tools admin revoke-grant --credentials-file PRIVATE_FILE --url URL --confirm-grant-id ID --retry-after-uncertain true
```

The plan is side-effect-free. Reuse its `--id` with `admin create` to preserve the same proposed data namespace; otherwise each invocation generates a new random ID. The private pending file records the proposed name, prefix, permissions and quotas before the first remote mutation. A successful issuer response is durably saved before its user token is requested. Creation requires a **parent user token with delegate authority**, not the parent issuer token. The CLI securely creates a private pending credential file before remote mutations, then preserves issuer and issued user tokens there. The terminal prints only nonsecret identifiers. The credential file is an explicit recovery journal. `admin inspect` prints a **nonsecret summary** of issuance, ready, or revocation states. The persisted raw issuer response is saved in `grant_saved` before any user-token request. `admin resume` can issue the user token from `grant_saved` without needing the parent token, and is idempotent for a fully `ready` journal. Before making a user-token request, the CLI durably records `issuing_user`. If the user-token response was lost, the operation might have committed, so **reissuing requires an explicit duplicate-risk acknowledgement**; doing so can leave another valid user token outside the journal. Legacy raw issuer-response files require the same acknowledgement because their issuance history is unknown. `pending_grant_create` cannot be resumed safely because a lost one-time issuer response cannot be recovered by repeating grant creation. The CLI never automatically retries that request.

Credential operations use an owner-only persistent `.lock` sidecar file to serialize updates across concurrent local processes. The journal and sidecar remain private; inspection/resumption never prints issuer or user tokens. No automatic grant-create issuer recovery or guaranteed exactly-once user issuance is claimed. Revocation is explicit and journaled.

## Current boundaries

GitHub Actions can deploy remotely through the SSH composite action under `action/deploy`; the CLI itself has no managed remote build service, hosted app registry, push-subscription management or sealed-epoch collection. CAS-backed immutable-manifest snapshots are available as a separate initial client protocol. Future commands for these must be implemented and qualified separately.


### Revoking optional app credentials

Use `admin revoke-user` to invalidate the **one user token recorded in the credentials file**, while keeping the child grant and its issuer credential available. Use `admin revoke-grant` to revoke the **entire child grant subtree**. Both commands require an exact `--confirm-grant-id` matching the private journal, serialize access with its owner-only lock, and write a durable `revoking_user` or `revoking_grant` state **before** sending DELETE.

After a confirmed success or an expected Skrynia 404 error code (`user_not_found` or `grant_not_found`), the journal transitions to `user_revoked` (removes the user token) or `grant_revoked` (removes all issuer/user credentials). Repeating a completed operation is idempotent. On a failed/uncertain response, the journal remains pending and the CLI refuses another DELETE without `--retry-after-uncertain true`. Inspect the state first; the flag is an **explicit operator retry decision**, not automatic recovery.

Revocation only applies to grants represented by a wrapped child-grant journal; it does not accept a raw root-credentials file or independently create grants. Per Skrynia's API contract, **revoking a grant does not delete previously stored object bytes**. Revocation does not roll back application code, migrate data, free historical release artifacts or clean up clients that copied credentials earlier.

### Crash-recovery release auditing

\`app audit --name NAME\` reports complete published versions plus any release metadata whose artifact directory is missing, artifact directories without a matching metadata record, abandoned \`.RELEASE_ID.staging\` directories, and malformed or unsafe release entries. It prints the report even when incomplete records are found, but exits nonzero so automation can detect inconsistencies. \`--format json\` exposes \`clean\`, \`available_releases\`, and per-category arrays. The same action works over restricted SSH.

Auditing is **read-only**: it never prunes historical releases, removes staging files, changes the active symlink, performs a source build or touches Skrynia object data. It is O(number of release entries) rather than O(1); the \`rollback\` algorithm is unchanged. An audit can observe an in-progress build between separate directory writes, so repeat after deployment completes before treating transient results as persistent corruption. For content integrity use the separate \`app verify\` command.
