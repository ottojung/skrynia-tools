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

The chosen app subdirectory must contain a Makefile with a `build` target. The default builder image is `docker.io/library/rust:1.93-bookworm`, which supports Rust, Make and Git; applications needing Node, npm, or other runtimes should specify a builder image with those dependencies (the historical Skrynia Node builder can be used if available). The implemented CLI requires **rootless Podman**; it checks that property before launching a disposable build container as container UID 0 (mapped to the unprivileged invoking host account). Run `make build` in that container with a selected builder image, not on the host, and mount a clone **without operator SSH tokens or private Skrynia credentials**. The output is `<subdir>/build/`, including `index.html`; nested CSS/JS/assets keep their relative paths. Reject symbolic links, special files and path escapes in built output. Validate regular-file and directory permissions before release activation. Untrusted build scripts execute inside a container; choose trustworthy builder images and restrict privileges.

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
