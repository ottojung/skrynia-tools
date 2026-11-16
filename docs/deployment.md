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
          port: '22' # optional
          ssh-private-key: ${{ secrets.DEPLOY_SSH_KEY }}
          known-hosts: ${{ secrets.DEPLOY_KNOWN_HOSTS }}
          name: myapp
          apps-dir: /srv/skrynia-apps
          subdir: apps/myapp
```

When `repo` is omitted the action uses the current GitHub repository and commit. When `repo` is explicitly provided without `commit`, the remote checkout resolves that repository's default branch. For reproducibility pass a full commit SHA and a pinned builder image digest. Releases, commits and public app paths are returned as action outputs.

**Threat boundary:** the SSH account is an administrative deployment capability and can replace static site contents. Restrict it to deployment paths and use branch protection and production-environment approvals. Pass trusted known-hosts records rather than disabling host-key verification; never run deployment on untrusted pull requests. The SSH transport does not grant remote filesystem access beyond what the deployment account already possesses. The complete action has been exercised against an isolated **real OpenSSH server** with a restricted deployment key, verifying host-key refusal, root-scope refusal, Git checkout and published artifacts. An additional test used real rootless Podman over that SSH connection. A separately operated production SSH hosting system has not been qualified.

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

## Activation conflict while another build runs

When a new build starts, the deployer records the currently active release ID. After preparing the new version and acquiring the brief activation lock, it checks that the active release ID has not changed. If an administrator has activated a different version in the meantime, the in-progress build **must not undo that decision**: automatic activation fails with a conflict, leaving the newly prepared snapshot available for deliberate `app activate --release ID`. This check does not implement a durable activation-event counter, so reactivation of the exact same release during a build is not distinguishable from no intervening change.

Release metadata is limited to **16 KiB at both write and read time**, and each record is created once with private permissions and synchronized before use. The CLI refuses to overwrite an existing release record. If the atomic symlink change succeeds but syncing its parent directory fails, the command reports **uncertain durability** and directs the operator to inspect the active release; it must not pretend that activation certainly failed.

### SSH integration qualification

Run `bash tests/ssh/integration.sh` to start an isolated localhost OpenSSH server and test the complete GitHub-action script with a restricted deployment key. The test uses a simulated builder to keep CI portable while executing the real GitHub clone, native Rust deployment and SSH transport. Set `SKRYNIA_SSH_TEST_REAL_PODMAN=1` on a host with rootless Podman and the selected builder image to repeat the same test with a real container build. This still does not simulate a physical machine failure or an unrelated remote production host.

The action defaults to SSH port 22; the optional `port` input supports other SSH ports and is validated before connection. SSH ignores local user configuration (`-F /dev/null`) and refuses host-key changes, credential forwarding and proxy commands. The deployment host must already have the Rust CLI and build prerequisites installed.


## Remote CLI release management

The **same** `skrynia-tools app` release commands can run from a developer machine without a shell session on the hosting machine. Add `--ssh-host`, `--ssh-user`, `--ssh-key` and `--ssh-known-hosts` (and optional `--ssh-port`) to `app deploy`, `list`, `versions`, `current`, `show`, `compare`, `rollback` or `activate`. The local CLI serializes a bounded operation request as JSON over SSH; the remote handler uses the same release-management functions. All app code and metadata remain on the hosting machine.

To grant that SSH key both deployment and management rights, restrict its `authorized_keys` entry to the **root-scoped RPC handler**, instead of a general interactive shell:

```text
restrict,command="env SKRYNIA_DEPLOY_ROOT=/srv/skrynia-apps /usr/local/bin/skrynia-tools app rpc --spec-stdin true" ssh-ed25519 <management-public-key>
```

The handler requires `SKRYNIA_DEPLOY_ROOT`, verifies that the supplied `apps_dir` resolves to that exact directory, accepts a fixed whitelist of actions and refuses unknown JSON fields. A raw `DeploySpec` object is accepted for compatibility with the GitHub composite action, while the typed management envelope supports version inventory and constant-work rollback. The existing deploy-only forced command remains valid for deployments; management commands need a key restricted to the RPC handler. Restrict management keys and SSH accounts to operators with authority over application **code**. Do not give them Skrynia issuer or root tokens.

`app rollback` remains constant-work relative to artifact size and release count; SSH establishment adds a fixed network handshake overhead and no artifact transfer. Arbitrarily slow network or filesystem operations are not subject to a real-time limit. As with local management, **no migration hooks or application-data modifications occur**.

The isolated OpenSSH integration test now exercises both application deployment and remote version inventory, provenance comparison and rollback through the same restricted RPC interface. A separate run has used real rootless Podman for two Git builds over SSH. These tests validate the transport and release lifecycle, but not a separately managed production host or power-loss recovery.

## Explicit release integrity verification

New releases record a **SHA-256 directory-tree digest** alongside their immutable code. The portable tagged serialization begins with `skrynia-release-tree-v1` followed by a NUL byte; it processes each directory's entries in sorted name order, depth-first and encodes their type (`D` directory, `F` regular file, `X` executable file), relative path length, raw relative path bytes, and file size (unsigned 64-bit big-endian). For file entries it then hashes the raw contents. Empty directories are distinguished from absent directories, and arbitrary non-UTF-8 filenames are supported on the native host.

Publication normalizes artifact file modes to **0644** for non-executables and **0755** for executables, stripping privileged setuid/setgid modes. Directory modes are 0755. The hash detects executable-bit changes as well as file byte/path modifications. Symlinks, sockets and other special files remain rejected to prevent path escapes. The input and output artifact trees need not contain any HTML files.

`skrynia-tools app verify --name NAME [--release RELEASE_ID]` explicitly recomputes the hash and fails on mismatch or an unverified legacy snapshot. `--format json` emits `expected_sha256`, `actual_sha256` and `verified` fields (`null` for missing legacy commitments). This check reads **every artifact byte** and sorts entries within each directory (rather than promising constant work); it is deliberately separate from O(1) rollback, which only validates a bounded release record and switches the code pointer without reading old artifact contents.

An administrator who can modify both release artifacts and metadata can forge a matching digest, so verification detects accidental corruption or uncontrolled changes—not a malicious owner of the deployment root. Retained release storage and operating-system permissions must still be trusted.

## Host diagnostics

Use `skrynia-tools app doctor --apps-dir DIR [--format json]`, locally or with the remote SSH options, to check that the hosting root exists as a real directory, Git is callable, rootless Podman is available and the expected private publication subdirectories have been initialized. This is a prerequisite diagnostic, **not** a synthetic deployment or guarantee of production hosting readiness. The check does not touch application data or depend on any client-side Skrynia protocol.

## Build subprocess timeouts

Git checkout and app builds run as isolated Unix subprocess groups. On a configured deadline, the administrative CLI kills the whole group and reaps the direct child rather than leaving shell-spawned descendants running. It also bounds captured subprocess stdout to 64 KiB and consumes it concurrently so verbose tools cannot deadlock on full pipes. The Podman runner separately removes the named container after build failure. These controls do not replace container resource quotas, external service isolation, or the operator's responsibility to vet executable builder images.
