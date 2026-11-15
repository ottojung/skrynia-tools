# Optional application data provisioning

**Application deployment does not require Skrynia data or append-only logs.** Static applications can deploy without any grant or storage initialization. This optional workflow assigns a dedicated object key prefix to an application that wants client-side durable storage.

## Independent app identity

A human-readable deploy name (e.g. `hello`) identifies the static website and its releases. An independently generated unpredictable data identifier can reserve `a.<id>.*`. No server-side app object exists. No generic `manifest`, `log` or `head` key is required. Apps choose and version their own data protocols.

## Minimal authority workflow

Given a parent **user** token authorized for `delegate`, an administrator can create a grant with explicit `locations`, `operations`, `maxBytes`, `maxObjects`. The response is an exact-grant issuer token; securely persist it before any next remote action. Issue the exact-grant user token using that issuer token. Narrow, independent role grants should be requested only when justified by the application's chosen data protocol.

An ordinary browser should receive only its explicitly delegated, publicly exposable user token; it must not receive root or issuer credentials. A grant has exactly **one** location matcher; disjoint role scopes require separate grants and credentials. Overlapping broader grants can still edit data. Creation is not transactional: if the successful grant-create response is lost, its issuer secret may be irrecoverable.

`skrynia-tools admin plan --name hello --max-bytes N --max-objects N` previews a data prefix and owner grant without side effects. Supply its 32-hex `--id` to `skrynia-tools admin create --name hello --url URL --parent-token-file FILE --credentials-out FILE --max-bytes N --max-objects N --id ID` to provision that exact scope. This owner grant has broad operations and its credentials must never be shipped in a public app. Neither command creates protocol data.

Revoking a grant never deletes stored bytes, and redeploying or rolling back an app never revokes any data grants.
