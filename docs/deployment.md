# GitHub-source application deployment

**Primary application deployment workflow.** Deployments are not data protocols. No append-only log, CAS document, manifest in Skrynia storage or even a grant is required for a static app to exist.

## Historical contract

The original Skrynia deployment workflow accepted a repository, an exact Git commit, a relative subdirectory, and an application name (then called a namespace). The builder ran `make build` from that subdirectory with the **entire repository checkout** accessible, expecting output in `build/`. A successful build became an immutable release; its activation replaced a symlink atomically and retained rollback targets. Earlier implementations hosted this in the Skrynia server; this project moves that functionality into an external administrative CLI, without reinstating deployment endpoints in Skrynia.

## Canonical invocation

```sh
skrynia-tools app deploy --name hello \
  --repo https://github.com/owner/monorepo \
  --subdir apps/hello \
  --apps-dir /srv/skrynia-apps \
  --commit 0123456789012345678901234567890123456789 \
  --builder-image docker.io/library/rust:1.93-bookworm
```

`--repo` also accepts the scp-style GitHub SSH spelling `git@github.com:owner/monorepo.git` for private repository access. With no `--commit`, the CLI resolves and records a concrete default-branch commit; pin a full commit SHA for reproducible CI/CD. Both the explicit `--subdir .` and nested monorepo paths work. Git checkout preserves the whole repo so `../shared` is accessible to the build. No arbitrary non-GitHub source is implicitly trusted.

## Builder and output contract

The chosen app subdirectory must contain a Makefile with a `build` target. The default builder image is `docker.io/library/rust:1.93-bookworm`, which supports Rust, Make and Git; applications needing Node, npm, or other runtimes should specify a builder image with those dependencies (the historical Skrynia Node builder can be used if available). The implemented CLI requires **rootless Podman**; it checks that property before launching a disposable build container as container UID 0 (mapped to the unprivileged invoking host account). Run `make build` in that container with a selected builder image, not on the host, and mount a clone **without operator SSH tokens or private Skrynia credentials**. The output is a real `<subdir>/build/` directory with any regular files and subdirectories the application chooses (there is **no mandatory `index.html`**). Static websites may expose HTML/CSS/JS; other applications may publish different code artifacts. How to execute or route an artifact is outside this code-release manager. Reject symbolic links, special files and path escapes in built output. Validate regular-file and directory permissions before release activation. Untrusted build scripts execute inside a container; choose trustworthy builder images and restrict privileges.

## Local publication contract

The administrative machine owns `--apps-dir`, a static-root managed independently of the Skrynia object store. Store immutable snapshots under `releases/<name>/<release-id>/`; atomically replace `apps/<name>` symlink with a relative symlink pointing to the chosen release. Configure nginx or another static server to serve `<apps-dir>/apps/<name>/` at `/apps/<name>/`. Failed builds and incomplete verification leave the active release unchanged. `skrynia-tools app rollback --name NAME --release ID --apps-dir DIR` atomically activates a retained release. Existing persistent Skrynia bytes and grants are **never** removed by deployment or rollback.

This uses **local host publishing** because Skrynia's current public store has flat keys, not hierarchical web paths or atomic multi-file static-root promotion. The tool must not pretend a set of Skrynia `POST` calls recreates the former server-hosting semantics. Running the deploy CLI on the static-host operator machine (or an authorized remote runner with access to that directory) is required.

## Ownership and data permissions

App source deployment and app runtime grants are separate operations. `skrynia-tools admin create` can provision an independent `a.<id>.*` object prefix and selected role grants; it is **optional**, must never grant all users full authority, and does not invent log keys. Static frontends remain public and any bundled credential is visible to recipients. Isolate mutually untrusted browser applications on separate HTTPS origins, including when their static assets are served from an administrative host.

## Recovery and limits

Commit, resolved source URL, subdir, builder identity and release metadata are recorded outside the public document root. Build, clone and publication errors are reported without secrets. Releases are immutable; do not delete retained releases by default, because browsers may still reference them. Source builds do not have a Skrynia server-side deployment API or privilege. A caller must explicitly arrange the static host's routing and build-container prerequisites.

## Crash recovery and durability

Deploy and rollback serialize per application with a local advisory lock. After taking the lock, the deployer checks for abandoned build containers named for prior attempts, stops them if present and removes their stale workspaces; other application workspaces are left alone. Deployment directories may not be symlinks. Release contents and metadata are synchronized before activating a release, and symlink activation synchronizes the public directory. A warning during temporary workspace removal after a successful release does not turn a committed deployment into a false failure. Untrusted build output is rejected if it contains symlinks or non-regular objects.

This is not a substitute for power-failure qualification on every filesystem or a hostile-tenant local deployment service. The deployment host's operator controls the private release and metadata directories. A failure during the final directory sync can mean activation occurred but durability could not be confirmed; inspect the active release before retrying.

## GitHub Actions deployment

The reusable [composite action](../action/deploy/action.yml) restores push-triggered remote deployment without restoring a privileged deployment endpoint inside Skrynia. The static host runs the native `skrynia-tools` executable with access to its apps directory and rootless Podman. The action securely SSHes to that host and provides a bounded JSON specification over standard input. No app grant, append-only log, or server-side deployment API is required.

```yaml
name: Deploy web app
on:
  push:
    branches: [main]
jobs:
  deploy:
    runs-on: ubuntu-latest
    environment: production
    steps:
      - uses: ottojung/skrynia-tools/action/deploy@PINNED_COMMIT_SHA
        with:
          host: example.org
          user: deployer
          ssh-private-key: ${{ secrets.DEPLOY_SSH_KEY }}
          known-hosts: ${{ secrets.DEPLOY_KNOWN_HOSTS }}
          name: myapp
          apps-dir: /srv/skrynia-apps
          subdir: apps/myapp
```

When `repo` is omitted the action uses the current GitHub repository and commit. When `repo` is explicitly provided without `commit`, the remote checkout resolves that repository's default branch. For reproducibility pass a full commit SHA and a pinned builder image digest. Releases, commits and public app paths are returned as action outputs.

**Threat boundary:** the SSH account is an administrative deployment capability and can replace static site contents. Restrict it to deployment paths and use branch protection and production-environment approvals. Pass trusted known-hosts records rather than disabling host-key verification; never run deployment on untrusted pull requests. The SSH transport does not grant remote filesystem access beyond what the deployment account already possesses. This action has been smoke-tested with a mock SSH transport, but a live remote SSH host has not been independently qualified.

### Restricting the SSH deployment account

Configure an OpenSSH forced command on a dedicated account, setting `SKRYNIA_DEPLOY_ROOT` to its single authorized static-host root. A deployment specification received over SSH that requests any different root is rejected before cloning or building. For example, the host's `authorized_keys` entry can begin with:

```text
restrict,command="env SKRYNIA_DEPLOY_ROOT=/srv/skrynia-apps /usr/local/bin/skrynia-tools app deploy --spec-stdin true" ssh-ed25519 <deployment-public-key>
```

The directory must already exist and be owned by the deployment account. Do not give the deploy key broader administrative or storage-grant capabilities. The `restrict` SSH option also blocks forwarding and PTY access. The remote command from the composite action is a fixed command, not a user-controlled shell fragment.

## Release management and constant-work rollback

The host stores immutable code under `releases/NAME/RELEASE_ID/`, a bounded version record under `metadata/NAME/RELEASE_ID.json`, and the active app as one symlink `apps/NAME -> ../releases/NAME/RELEASE_ID`. Every published version is addressable by its exact release ID, not by its Git commit alone; deploying the same commit twice creates distinct releases.

`skrynia-tools app list` enumerates managed applications. `app versions --name NAME` lists all available snapshots newest-first with timestamps and the active version marked; `app show --name NAME --release ID` reads one version's metadata. Use `--format json` for programmatic callers. `app current` directly resolves the active symlink. Legacy `app releases` and `app inspect` continue to return JSON.

`app rollback --name NAME --release ID` takes only the short activation lock, directly validates the named version's fixed-size (at most 16 KiB) record and a few fixed filesystem paths, creates a relative candidate symlink and atomically replaces the active symlink. The metadata read is bounded; **no version enumeration, recompilation, clone, artifact copy, application callback or data migration occurs**. Its CPU work, number of filesystem operations and extra memory are **O(1) in the number of historical versions and total artifact bytes**, treating keyed filesystem lookups as constant-time. This is an algorithmic complexity guarantee, not a real-time deadline: disk latency and contention with another short activation can still delay completion.

Builds acquire a separate per-app build lock and copy, synchronize and prepare the new immutable snapshot **before** taking the activation lock. A rollback therefore does not wait for a different deployment's source checkout, compilation or artifact copy. Crash recovery of abandoned build workspaces also uses the build lock, not the activation lock.

Listing all versions necessarily enumerates them (and sorts by recorded timestamp), so the `versions` command is **not** O(1). Rollback remains O(1) regardless of how many versions exist. Versions are retained by default; there is no implicit pruning that would silently make an old rollback target disappear. A crash between staging and activation can leave a complete prepared version that was never active; it is still a valid rollback target and may appear in the available-versions view. The list is a catalog of retained code snapshots, not an authoritative activation-event audit log.

Application-data migration and storage lifecycle management are intentionally outside this code-only deployment facility. A rollback changes the static code pointer only. Developers must separately assess data compatibility when necessary.
