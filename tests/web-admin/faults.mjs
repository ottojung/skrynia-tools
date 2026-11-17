import assert from 'node:assert/strict';
import {createGrant, issueUser, revokeGrant, clearPending} from '../../web-admin/admin.js';
const key = 'apps/tools/vault/' + '0'.repeat(32) + '.json';
const spec = {locations: 'apps/example/*', maxBytes: 1000, maxObjects: 10,
  operations: ['create', 'list']};
function makeSession() {
  const calls = {saves: 0, mutations: 0};
  const client = {
    async replace() {
      calls.saves++;
      if (calls.failAt === calls.saves) throw Error('Mutation outcome UNKNOWN');
      return 'a'.repeat(64);
    },
    async json() {
      calls.mutations++;
      if (calls.failMutation) throw Error('Mutation outcome UNKNOWN');
      return {id:'1'.repeat(32), issuerToken:'issuer-token',
        grantId:'1'.repeat(32),userId:'2'.repeat(32),
        userToken:'child-user-token'};
    },
    async send() {
      calls.mutations++;
      if (calls.failMutation) throw Error('Mutation outcome UNKNOWN');
      return {status:200};
    },
  };
  return {session: {client, hash:'b'.repeat(64), key,
    data:{schema:1,parentToken:'parent-secret',grants:[],pending:null},
    password:'random-test-password'}, calls};
}
{
  const {session, calls} = makeSession();
  calls.failAt = 1;
  await assert.rejects(() => createGrant(session, spec, () => {}), /UNKNOWN/);
  assert.equal(calls.mutations, 0, 'no grant request before durable intent');
  assert.equal(session.unresolved, true);
  await assert.rejects(() => createGrant(session, spec, () => {}), /reload/);
}
{
  const {session, calls} = makeSession();
  calls.failAt = 2;
  const recovered = [];
  await assert.rejects(() => createGrant(session, spec, x => recovered.push(x)),
    /UNKNOWN/);
  assert.equal(calls.mutations, 1, 'never repeat the grant mutation');
  assert.equal(recovered[0].issuerToken, 'issuer-token');
  assert.equal(session.unresolved, true);
  assert.equal(session.data.pending.operation.type, 'grant-create');
}
{
  const {session, calls} = makeSession();
  calls.failMutation = true;
  await assert.rejects(() => createGrant(session, spec, () => {}), /UNKNOWN/);
  assert.equal(session.data.pending.operation.type, 'grant-create');
  await assert.rejects(() => issueUser(session, '1'.repeat(32), () => {}),
    /Unknown or revoked/);
  await clearPending(session);
  assert.equal(session.data.pending, null, 'explicit investigation required');
}
{
  const {session, calls} = makeSession();
  const grant = await createGrant(session, spec, () => {});
  calls.failMutation = true;
  await assert.rejects(() => revokeGrant(session, grant.id), /UNKNOWN/);
  assert.equal(session.data.pending.operation.type, 'revoke-grant');
  assert.equal(session.data.grants[0].issuerToken, 'issuer-token',
    'do not discard issuer credential after uncertain revocation');
  const before = calls.mutations;
  await assert.rejects(() => revokeGrant(session, grant.id), /Unresolved/);
  assert.equal(calls.mutations, before);
}
console.log('Grant journaling and unknown-outcome recovery checks passed');
