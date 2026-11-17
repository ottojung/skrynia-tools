# Skrynia app releases

The GitHub deployment Action builds application source on the
runner, then publishes directly to Skrynia's HTTP object API.
There is no SSH, CLI binary, special deployment server or
application-specific server endpoint.

## Release protocol

All keys live under `apps/NAME/`. Immutable release assets are stored
under `releases/ID/PATH`, preserving original relative paths. The
`versions/ID.json` manifest lists file paths, byte counts, SHA-256
digests, repository, subdirectory and exact source commit.
The `versions/ID.ready` marker is created only after the assets and
manifest have been uploaded.

The stable `index.html` fetches `active.json` without caching and
navigates to the chosen release within the app's browser origin.
The pointer includes a random nonce. Activation uses create-if-absent
or a conditional replacement on the old pointer's byte-SHA256.
Any backward or forward version switch therefore changes **one
fixed-size object**: O(1) in history and asset size. It does not
copy assets, rebuild code, migrate data or change app grants.
Concurrent changes fail with a CAS conflict; uncertain network
outcomes require an authoritative read before any retry.

## Browser origins and assets

For example, nginx maps `https://hello.a.vau.place/` internally
to `apps/hello/index.html`, `/active.json` to the corresponding
Skrynia pointer, and `/releases/ID/file.js` to the matching file.
The browser is never redirected to the shared `vau.place` origin.

Publish applications with relative asset URLs. The bootstrap
preserves the incoming query and hash. Unversioned HTML deep links
can use an explicit bootstrap fallback; never return fallback HTML
for missing scripts or stylesheets. A SPA router may need a separate
versioned-route policy.

Stable app-root files such as `/sw.js`,
`/manifest.webmanifest` and `/icon.svg` are published independently
through [the stable asset Action](../action/publish-stable/action.yml).
They are not changed by rollback, which is important for persistent
service-worker scope and push subscriptions. Stable entrypoints and
pointers use `Cache-Control: no-store`; successful release objects
may be cached `immutable`, provided deployers do not rewrite history.

## GitHub deployment Action

~~~yaml
name: Publish app
on:
  push:
    branches: [main]
permissions:
  contents: read
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: ottojung/skrynia-tools/action/deploy@PINNED_COMMIT
        with:
          url: https://vau.place/_skrynia
          token: ${{ secrets.SKRYNIA_DEPLOY_TOKEN }}
          namespace: hello
          subdir: apps/hello
          directory: build
~~~

The Action confirms the exact Git commit before building, validates
artifact paths, rejects symlinks and nonregular files, publishes
assets and records a ready release before activation. Build tools
run locally on the runner and do not receive the upload token.
Pin the Action commit and optional builder OCI image.

The Action outputs the release ID, commit and stable entrypoint URL.
It calls Node HTTP modules internally, **not a CLI**.
A separately scoped user-token grant with create, modify and list
under `apps/NAME/*` authorizes its operations. A broader modify
grant can rewrite history, so stronger immutability requires more
restrictive grants. Release retention and orphan garbage collection
remain separate data management tasks.
