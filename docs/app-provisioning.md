# Client-managed application provisioning

**Status: proposal, not implemented.** Skrynia has no native application object. An app is a convention of a unique key prefix, a public descriptor, and separately issued, resource-bounded grants. No global registry or new server endpoint is required.

## Namespace and descriptor

Generate a collision-resistant app identifier, e.g. `k7x4...`, and reserve `a.<id>.*`. Names are display metadata, **not** authority boundaries. All keys must match Skrynia's legal flat-key grammar; `/` is illegal. Define `a.<id>.manifest` as a public versioned descriptor of protocol formats and entrypoint keys, containing **no bearer tokens**. It is not an authorization oracle: an owner with a broad modify token can change it, so clients requiring authenticity should pin its hash or validate a signature. Public GET requires no token; **listing** requires a grant with `list`.

## Suggested grants (optional profiles, not mandatory for every app)

| Purpose | Matcher | Operations | Holder |
| --- | --- | --- | --- |
| Owner | `a.<id>.*` | create, modify, delete, list, delegate, subscribe | trusted admin only |
| Public name discovery | `a.<id>.*` | list | optional browser token, deliberately exposed |
| Event publisher | `a.<id>.events.*` | create | contributor |
| Snapshot head maintainer | `a.<id>.state.head` | modify | trusted state coordinator |
| Immutable snapshot writer | `a.<id>.state.versions.*` | create | coordinator |
| Epoch log contributor | `a.<id>.log.e000001.*` | create | delegated writer |
| Sealed epoch collector | `a.<id>.log.e000001.*` | delete | trusted collector after sealing |

Skrynia allows **one matcher per grant**. A role needing disjoint scopes comprises multiple grants and credentials assembled **client-side**, not a multi-matcher server grant. Give each participant the narrowest needed operations and scope. Root and owner credentials are not application runtime credentials. Child quotas don't reserve ancestor quota and grants can overlap; list/inspect cannot guarantee absolute isolation from actors with broader grants.

## Remote provisioning sequence

1. Operator supplies a parent **user token** with `delegate` authority and adequate scope/capacity. The root **issuer** token is not a substitute. Perform local validation and a complete redacted `--dry-run` showing planned keys, operations and quota.
2. Generate an unpredictable app ID and privately journal the plan before remote mutation. Make sure the output credential destination exists with owner-only access.
3. `POST /grants` creates the owner grant for `a.<id>.*` with explicit scope, operations, maxBytes and maxObjects. Its response returns a **one-time issuer token**, not a user token. Persist that secret atomically and durably *before* another remote request.
4. Use the exact issuer to create the owner user token; persist it. Use the owner's `delegate` user token to create each narrow role grant, persist each returned issuer before issuing that role's user token. Never accidentally grant a browser the owner token.
5. Create `a.<id>.manifest` with create-if-absent from an appropriately scoped token. A 409 conflict is not permission to overwrite an existing descriptor. Mark completion only when all required durable components are confirmed.
6. Print app ID, public entrypoint, redacted grant summary, and explicit next steps. Private secrets stay in secure files and never appear in routine JSON/log output.

**Not transactional:** if grant creation succeeds but its response is lost, an orphan grant may be impossible to recover: Skrynia does not expose a grant-create idempotency key or parent-level child issuer recovery. Stop and report an **ambiguous** outcome; never blindly repeat creation. A local journal enables diagnosis and continuation **only where credentials and evidence exist**. Don't promise automatic rollback.

## Profiles and lifecycle

Start with optional declared profiles: `events` (create-only contributions), `document` (one CAS head + immutable data), `log` (epoch writers), and `static` (published entrypoint). Each declares key layout, grant plan, exact initialization/recovery rules and resource overhead. Don't require every app to have a head, a log, Web Push or broad admin control.

Admin tooling can inspect known grants, issue/revoke **user** tokens with the exact issuer, and revoke grant subtrees. **Issuer-token rotation is not a native Skrynia operation**; replacement requires separately specified grant migration. Revocation invalidates credentials/subscriptions but **does not delete object bytes**. Decommissioning needs a separate, explicitly authorized data-retention/cleanup decision.

Distinguish `skrynia app new` (local source template, never embeds tokens) from `skrynia admin app create` (remote grant/manifest provisioning).
