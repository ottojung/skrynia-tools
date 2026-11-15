# Proposed `skrynia` CLI

**Status: design, not implemented.** A single native Rust executable should serve both app developers and administrators, using the same Rust protocol crate. Do not ship a command stub that misleadingly looks functional.

## Ordinary clients and developers

```text
skrynia config profile add <name> --url HTTPS_URL
skrynia config profile use <name>
skrynia object get KEY [--output FILE]
skrynia object create KEY --file FILE
skrynia object replace KEY --file FILE --if-sha256 DIGEST
skrynia object delete KEY --if-sha256 DIGEST
skrynia object list --locations 'a.id.events.*' [--all-pages]
skrynia log init PREFIX
skrynia log append PREFIX --file FILE --operation-id ID
skrynia log read PREFIX [--from POSITION]
skrynia log follow PREFIX [--from POSITION]
skrynia app new DIRECTORY --template rust|web
skrynia doctor
```

Public GET needs no credential. Listing requires a scoped `list` user token. Following a log must reconcile by polling regardless of Web Push hints. Scans, waits and retries need work limits, cancellation and progress. Binary data to stdout must never be interleaved with human diagnostic text on stderr.

## Admin workflows

```text
skrynia admin app plan --profile events --name demo --max-bytes N --max-objects N
skrynia admin app create --profile events --name demo --max-bytes N --max-objects N
skrynia admin app inspect APP_ID
skrynia admin app issue-user APP_ID --role publisher --output-private FILE
skrynia admin app revoke-user APP_ID --role publisher --user-id USER_ID
skrynia admin app diagnose APP_ID
skrynia admin app revoke APP_ID --confirm-app APP_ID
skrynia admin grant inspect GRANT_ID
skrynia admin grant revoke GRANT_ID --confirm-grant GRANT_ID
```

`admin app create` requires a **parent delegate user token**, not root issuer token. `app new` creates local source files; `admin app create` creates remote grants/objects. Dry-run is side-effect-free and prints the planned prefix, each grant's exact matcher/operations, quota costs and risks. Secrets are redacted by default, stored atomically in private owner-only files, and exported only with an explicit private output destination.

## Configuration and observability

Use XDG conventions for profiles and private provisioning journals. Profiles contain URLs and nonsecret choices plus references to secrets; no credentials in generated app code. Support automation via `--json` with stable versioned output, `--dry-run`, explicit token sources and distinct exit codes for configuration failure, forbidden, conflict, CAS 412, quota, transport, outcome unknown and invalid remote data. Never claim automatic rollback after a lost grant-create response.

## Distinctive proposed capabilities

- `skrynia protocol plan <recipe>` (pure): derive the **minimum grant bundles** and object layout for a protocol, including quotas and authority assumptions, without creating anything.
- `skrynia protocol verify <descriptor>` (read-only): check observed log contiguity, head digests, protocol versions, manifest reachability and epoch seals. Report a provable violation separately from insufficient evidence; never claim server certification.
- `skrynia admin app diagnose`: reconcile local provisioning journal with known remote state and identify orphaned or ambiguous steps without blindly issuing new grants.
- `skrynia object snapshot`: safe, explicitly bounded export of matching known objects; not a promise of cross-key atomic backup while writers are active.
- `skrynia doctor`: validate endpoint, API version, local permissions, credentials and minimal requested operations without destructive probes.

These are useful directions, not an initial-release promise. Decommissioning must explicitly separate credential revocation from stored byte deletion, and issuer-token rotation remains unsupported by Skrynia's current API.
