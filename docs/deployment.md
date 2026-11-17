# HTTP-only application deployment

Deployment uses only the existing Skrynia object HTTP protocol.
There is no SSH, deployment host, special deployment endpoint, extra
file server, proxy service, filesystem symlink, or server-side builder.
GitHub Actions or a developer machine builds the application.

## Release protocol

Every app has hierarchical public keys under apps/NAME/.
Each deployment creates a unique release ID and uploads all build files
under apps/NAME/releases/ID/ using their **original relative paths**.
The manifest apps/NAME/versions/ID.json records the file tree, each
asset length and SHA-256 digest. The constant-sized
apps/NAME/versions/ID.ready marker is written after all files and
the manifest have been uploaded successfully.

The stable apps/NAME/index.html is created once; it fetches the
current apps/NAME/active.json and navigates to
apps/NAME/releases/ID/index.html. Relative asset links work because
the browser sees the real release directory. No HTML/JS/CSS rewriting
or additional hosting service is necessary.

Activation creates or CAS-replaces active.json with a JSON object
containing the release ID and a unique nonce. Rollback checks the
constant-size ready marker and current pointer and makes one CAS.
It does not scan history, copy assets, rebuild code or migrate data.
A concurrent pointer change returns a conflict. A lost reply requires
an authoritative read before retrying.

Uploads are multiple single-key transactions, **not** an atomic
directory operation. A failed upload may leave inert unreferenced
objects. Releases are intentionally retained because other browsers
may still reference their version-specific URLs.

## Native CLI

~~~sh
export SKRYNIA_TOKEN='RESTRICTED_DEPLOY_GRANT_USER_TOKEN'
skrynia-tools app deploy --name hello --url https://example.org/_skrynia \
  --repo https://github.com/OWNER/REPOSITORY \
  --commit FULL_40_CHARACTER_SHA --subdir apps/hello
skrynia-tools app deploy --name hello --url https://example.org/_skrynia \
  --directory ./apps/hello/build --commit FULL_40_CHARACTER_SHA
skrynia-tools app versions --name hello --url https://example.org/_skrynia
skrynia-tools app rollback --name hello --release VERSION \
  --url https://example.org/_skrynia
~~~

The GitHub source mode clones the entire repository locally, checks out
the exact commit, and runs make build from its subdirectory. Directory
mode accepts a prebuilt tree. The output must include index.html. The
CLI rejects symlinks, nonregular files, path traversal and object paths
longer than the server's key limit. Build scripts execute only on the
caller's machine and must be treated as trusted code.

## GitHub Actions

~~~yaml
name: Deploy web app
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
      - uses: actions/setup-node@v4
        with:
          node-version: '22'
      - uses: ottojung/skrynia-tools/action/deploy@PINNED_COMMIT
        with:
          url: https://example.org/_skrynia
          token: GITHUB_ACTIONS_SECRET_DEPLOY_TOKEN
          namespace: hello
          subdir: apps/hello
          directory: build
~~~

Replace GITHUB_ACTIONS_SECRET_DEPLOY_TOKEN with the GitHub Actions
secret expression for the repository's scoped Skrynia user credential.
The action builds on the runner; its optional builder input runs an
OCI container **locally** on the runner, never on the Skrynia server.
The default uses the runner's native make/build tools. Never pass the
deployment credential into a build container. Pin action commits and
external builder images to reviewed immutable digests. For builds
of another GitHub repository, supply its exact commit.

The action outputs release ID, source Git commit, and the public Skrynia
entrypoint URL. No SSH key, known-hosts file, permanent runner,
separate host, or cutover variable is involved.

## Scope and correctness

The simplest deployment grant has create, modify and list on
apps/NAME/*, with finite maxBytes and maxObjects. It cannot delegate
new grants unless separately authorized. A token holder can nevertheless
rewrite old assets under that scope, so *immutability* is a publisher
protocol assumption, not server-enforced under the one-token mode.

Stronger isolation uses a create-only grant for release assets and
manifests, and a separate exact-key create/modify grant for
active.json. These require multiple credentials, not yet offered by
the single-token action. Keep root and issuer tokens private.

The public app entrypoint can be loaded directly at
https://example.org/_skrynia/store/apps/hello/index.html. It does not
require URL rewriting. An app that hardcodes absolute paths such as
/a/hello/assets.js must switch to relative assets during build.
Service worker scopes, CSP and client-side routers must also account
for the versioned release directory.

Skrynia currently limits an object to 10 MiB by default and an entire
object key to 255 bytes. Large assets require chunking or an explicit
change to Skrynia's limits. Historical releases consume quota until
explicitly reclaimed.
