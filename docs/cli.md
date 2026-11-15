# Skrynia Tools CLI

**Implemented commands**, executed by the `skrynia-tools` native Rust binary (or an installed `skrynia` alias). See `skrynia-tools help` for a summary. Static application commands do not require a Skrynia token, an append-only log, or access to the Skrynia object store.

## Application releases

Set `SKRYNIA_APPS_DIR=/srv/skrynia-apps` to use a default hosting root for interactive `app` commands. An explicit `--apps-dir` overrides the environment. No application data or migration protocol is required.

```sh
skrynia-tools app new --name hello --directory ./apps/hello
skrynia-tools app deploy --name hello --repo https://github.com/OWNER/REPO --subdir apps/hello --apps-dir /srv/skrynia-apps [--commit FULL_SHA] [--builder-image IMAGE]
skrynia-tools app list --apps-dir /srv/skrynia-apps [--format table|json]
skrynia-tools app doctor --apps-dir /srv/skrynia-apps [--format table|json]
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
```

The plan is side-effect-free. Reuse its `--id` with `admin create` to preserve the same proposed data namespace; otherwise each invocation generates a new random ID. The private pending file records the proposed name, prefix, permissions and quotas before the first remote mutation. A successful issuer response is durably saved before its user token is requested. Creation requires a **parent user token with delegate authority**, not the parent issuer token. The CLI securely creates a private pending credential file before remote mutations, then preserves issuer and issued user tokens there. The terminal prints only nonsecret identifiers. An uncertain grant-creation response can leave an unrecoverable orphan; this is not a transactional or automatically reversible operation.

## Current boundaries

GitHub Actions can deploy remotely through the SSH composite action under `action/deploy`; the CLI itself has no managed remote build service, hosted app registry, push-subscription management, sealed-epoch collection or transactional multi-object snapshots. Future commands for these must be implemented and qualified separately.
