# Skrynia Tools CLI

**Implemented commands**, executed by the `skrynia-tools` native Rust binary (or an installed `skrynia` alias). See `skrynia-tools help` for a summary. Static application commands do not require a Skrynia token, an append-only log, or access to the Skrynia object store.

## Static web applications

```sh
skrynia-tools app new --name hello --directory ./apps/hello
skrynia-tools app deploy --name hello --repo https://github.com/OWNER/REPO --subdir apps/hello --apps-dir /srv/skrynia-apps [--commit FULL_SHA] [--builder-image IMAGE]
skrynia-tools app inspect --name hello --apps-dir /srv/skrynia-apps
skrynia-tools app releases --name hello --apps-dir /srv/skrynia-apps
skrynia-tools app rollback --name hello --release RELEASE_ID --apps-dir /srv/skrynia-apps
```

`app new` fails if the destination already exists. `app deploy` clones the whole Git repository (HTTPS or `git@github.com:owner/repo.git`), checks out the specified or resolved commit, runs the Makefile's `build` target in the given subdirectory inside rootless Podman, validates the hierarchical `build/` output and switches one symlink atomically. It requires an `index.html`, preserves nested assets and records source/commit/subdirectory/builder metadata. Errors leave any previous active release intact. Deploy and rollback serialize with a per-app file lock. The CLI host must have writable access to the static server's app directory; no server-side deployment endpoint is called.

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
