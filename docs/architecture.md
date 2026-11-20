> **Native v3 scope update (2026-11-19):** Program 0 is a transparent
> proxy only. Earlier prototype descriptions of Program 0 as the database,
> registry, code publisher or administrator are obsolete. Native services
> belong to nonzero user programs; see v3-program0.md.

# Skrynia Tools architecture

**One Skrynia server, HTTP operations only.** Skrynia stores public bytes,
applies grants, and serializes conditional mutations. Tools is unprivileged
client software: builds and release publication run on a developer machine
or ephemeral CI runner, not on an application/deployment server.

## Components

1. Native HTTP client: GET, create-if-absent, replacement, deletion,
   byte-SHA256 CAS, scoped listing, and grant administration.
2. Optional client data protocols: append-only logs, CAS cells, event sets,
   and content-addressed snapshots. Deployment does not depend on them.
3. Release manager: uploads a build tree to release-qualified hierarchical
   keys, publishes a manifest and ready marker, then CAS-updates a small
   active-release pointer.
4. Browser entrypoint: stable HTML fetches the active pointer and
   navigates to the selected release's index.html. No dynamic server
   routing, backend execution, HTML rewriting or symlink activation.
5. GitHub Actions: build on the runner, authenticate with scoped user
   tokens, and publish directly using JavaScript HTTP modules.

Skrynia's hierarchical store URLs preserve directory layouts and MIME
types. Every publish is a series of single-object operations; only
the final pointer change is atomic. Incomplete pre-activation uploads
remain inert and may need explicit retention management.

The deployment protocol is cooperative: another holder of modify
authority could rewrite an old release. Separate create-only asset
and exact-key pointer grants can enforce stronger immutability, but
the current single-token action does not claim that protection.

## Costs and failure boundaries

Uploading scales with bytes and files; listing versions scales with
history; verification re-reads assets. Rollback reads only a fixed-size
ready marker and bounded active pointer and performs one CAS.
CAS conflict fails rather than overwriting concurrent activation.
Lost mutation replies remain ambiguous until reconciled by reading
the authoritative active pointer.

## Web administration

The interactive administrative interface is a static app deployed as
ordinary Skrynia objects on its **own browser origin**. The one-time
bootstrap prints the URL and a cryptographically random access secret;
there is no privileged admin daemon. The admin app fetches its
publicly readable, **encrypted** credential vault after the user
unlocks it locally. Skrynia itself authenticates every operation
using the grant user or issuer token held in the unlocked browser.

The browser lists all apps and their ready release manifests through
Skrynia's paginated list API. Any backward/forward activation replaces
only the fixed-size `active.json` pointer with a byte-SHA256 CAS;
release assets and application data are untouched. The interface
never promises a multi-object atomic grant workflow, and persists
an encrypted intent journal before credential mutations.

There is **no CLI binary**. CI uses composite GitHub Actions; interactive
administration is through the web interface. See
[Web administration](web-admin.md).
