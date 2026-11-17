# Skrynia Tools CLI

Use standard nested help and version flags to inspect the supported
commands and their required arguments:

~~~sh
skrynia-tools --help
skrynia-tools --version
skrynia-tools admin grant --help
skrynia-tools app deploy --help
~~~

The CLI reports unknown options and missing required arguments with
a nonzero exit status and a command-specific usage message.

The native Rust CLI talks directly to Skrynia's HTTP store and grant
endpoints. There is no remote shell or static hosting deployment daemon.
Application release operations require --url and a user bearer token in
SKRYNIA_TOKEN (or the environment variable selected by --token-env).

## Application release commands

~~~sh
skrynia-tools app new --name hello --directory ./hello
skrynia-tools app deploy --url URL --name hello --repo GITHUB_URL   --commit FULL_40_HEX_SHA --subdir apps/hello [--output build]
skrynia-tools app deploy --url URL --name hello --directory ./hello/build   [--repo GITHUB_URL --commit FULL_SHA --subdir apps/hello]
skrynia-tools app list --url URL
skrynia-tools app current --url URL --name hello
skrynia-tools app versions --url URL --name hello
skrynia-tools app show --url URL --name hello --release RELEASE_ID
skrynia-tools app compare --url URL --name hello --from RELEASE_A --to RELEASE_B
skrynia-tools app diff --url URL --name hello --from RELEASE_A --to RELEASE_B
skrynia-tools app verify --url URL --name hello --release RELEASE_ID
skrynia-tools app verify-all --url URL --name hello
skrynia-tools app rollback --url URL --name hello --release RELEASE_ID
skrynia-tools app activate --url URL --name hello --release RELEASE_ID
~~~

All output is JSON except new and verify. Every release has a random
32-character hexadecimal ID, immutable file keys, a manifest and a
constant-size ready marker. List/history enumeration may require many
scoped HTTP listing pages; compare examines the two manifests, reporting
added, removed and changed relative paths. Verification streams every
stored asset through its expected SHA-256 commitment.

Rollback does not fetch a manifest or any asset. It reads the ready
marker, reads the small active pointer, and CAS-updates that one key.
Runtime and additional RAM are constant in artifact size and history
length (subject to parameterized HTTP/filesystem costs). A stale pointer
produces a conflict; unknown network outcomes must be reconciled by
reading app current before any deliberate retry.

GitHub source mode clones the exact commit locally and runs make build
in the chosen subdirectory, with the full monorepo checkout available.
Directory mode uploads an already built tree without running a build.
Output paths are preserved under apps/NAME/releases/ID/ and must
satisfy Skrynia's safe hierarchical key grammar. The build must
contain index.html. Do not use absolute-path asset URLs in builds.

Code releases do not imply any application-data format, log, grants or
migrations. The deployer token must have narrow create/modify/list
authority for apps/NAME/* and sufficient quota.

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

The CLI implements HTTP release publication and optional application
data protocols. It does not deploy application servers, install hosting
software, run remote builds, sweep historical assets automatically,
manage push subscriptions, or provide a data-migration framework.
There is no SSH transport or deployment-specific HTTP endpoint.

### Revoking optional app credentials

Use `admin revoke-user` to invalidate the **one user token recorded in the credentials file**, while keeping the child grant and its issuer credential available. Use `admin revoke-grant` to revoke the **entire child grant subtree**. Both commands require an exact `--confirm-grant-id` matching the private journal, serialize access with its owner-only lock, and write a durable `revoking_user` or `revoking_grant` state **before** sending DELETE.

After a confirmed success or an expected Skrynia 404 error code (`user_not_found` or `grant_not_found`), the journal transitions to `user_revoked` (removes the user token) or `grant_revoked` (removes all issuer/user credentials). Repeating a completed operation is idempotent. On a failed/uncertain response, the journal remains pending and the CLI refuses another DELETE without `--retry-after-uncertain true`. Inspect the state first; the flag is an **explicit operator retry decision**, not automatic recovery.

Revocation only applies to grants represented by a wrapped child-grant journal; it does not accept a raw root-credentials file or independently create grants. Per Skrynia's API contract, **revoking a grant does not delete previously stored object bytes**. Revocation does not roll back application code, migrate data, free historical release artifacts or clean up clients that copied credentials earlier.

## Generic grant creation

Skrynia Tools can create ordinary grants independently of application
deployment. The CLI supports `SKRYNIA_URL` as the default URL for **every**
command; an explicit `--url` overrides the environment variable.

**Normal case:** pass the parent grant's **user bearer token** directly.
The parent must have `delegate` authority and enough scope and quota.
`--parent-grant` takes a token, **not** a credentials filename.

~~~sh
export SKRYNIA_URL=https://vau.place/_skrynia
skrynia-tools admin grant --parent-grant "$PARENT_GRANT_USER_TOKEN" \
  --locations 'apps/long/*' --max-bytes 50000000 --max-objects 500
~~~

You can also supply the same token through `SKRYNIA_PARENT_GRANT`;
`--parent-grant` overrides that variable. Note that command-line
tokens may be visible in shell history or operating-system process
listings. Keep the parent token private.

**Root case:** on the machine with access to the Skrynia server's
`SKRYNIA_DATA_DIR`, use the exact same environment variable as the
server. The CLI finds the generated root grant in
`$SKRYNIA_DATA_DIR/state/grants/` and temporarily issues a root user
token over HTTP. It needs **no separate root user credential**.
If the variable is unset, the directory inaccessible, or its root
record missing or ambiguous, it exits with an error.

~~~sh
export SKRYNIA_URL=https://vau.place/_skrynia
export SKRYNIA_DATA_DIR=/absolute/path/to/skrynia
skrynia-tools admin grant --parent-grant-is-root \
  --locations 'apps/*' --max-bytes 500000000 --max-objects 15000
~~~

The default operations are `create,modify,list`; use
`--operations create,modify,list,delegate` to delegate further,
or any comma-separated subset of `create`, `modify`, `delete`,
`list`, `delegate` and `subscribe`. Both quota limits are required,
and `max-bytes` must be at least 2048. Server authority checks
remain authoritative.

The CLI saves issuer and user credentials with mode 0600 in a random
private file under `$XDG_STATE_HOME/skrynia-tools/grants/` (or
`~/.local/state/skrynia-tools/grants/`). Standard output contains
only the new grant ID and the path to its private credential file,
**never tokens**. To choose the output use
`--credentials-out /private/path/grant.json`. Existing files are
not overwritten.

You can also issue a new user token for an existing grant **without
creating another grant**:

~~~sh
skrynia-tools admin grant --existing-grant /private/path/grant.json \
  --credentials-out /private/path/additional-user.json
~~~

The new token is stored at `.user.userToken` in the private JSON file.
To install it for all application releases:

~~~sh
jq -r '.user.userToken' /private/path/grant.json |
  gh secret set SKRYNIA_DEPLOY_TOKEN --repo ottojung/skrynia-apps
~~~

The issuer credential remains private. A lost create response may leave
a `pending_grant_create` record; the CLI does not automatically retry
because the request may already have committed. Use `admin inspect`
to examine the record. When the grant is saved but user issuance
failed, `admin resume` is supported; ambiguous user issuance
requires explicit duplicate-risk acknowledgment.
