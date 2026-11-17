# Optional application data grants

Static releases do not depend on persistent application data, grants
for client writers or particular data protocols.

For an app that stores data, unlock the [Skrynia web administration
interface](web-admin.md), choose a parent user grant with `delegate`
authority, and issue a narrowly scoped child grant with explicit
locations, operations and byte/object quotas. Save its issuer token
in the encrypted vault before issuing a user token.

Only a user token intended for public use may be embedded in static
client files. Never publish root or issuer credentials or a global
writer grant. Grant issuance and vault persistence are not a single
atomic operation; the browser journals intents and requires explicit
reconciliation after uncertain failures.

A revoked grant invalidates its users and descendants but does not
delete objects. Changing an app's active code version does not
migrate its data or grant tree.
