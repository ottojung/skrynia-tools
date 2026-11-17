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
5. GitHub Action: builds on the runner, authenticates with a scoped user
   token and invokes the Rust CLI through ordinary Skrynia HTTP.

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
