import {saveVault, validateGrant} from './skrynia.js';

const copy = value => structuredClone(value);
function assertIdle(session) {
  if (session.unresolved) throw Error('Vault save outcome unknown; reload to reconcile');
  if (session.data.pending)
    throw Error('Unresolved previous operation. Investigate before proceeding');
}
async function journal(session, operation) {
  assertIdle(session);
  const updated = copy(session.data);
  updated.pending = {operation, at: new Date().toISOString()};
  await saveVault(session, updated);
}
async function commit(session, change) {
  const updated = copy(session.data);
  change(updated);
  updated.pending = null;
  await saveVault(session, updated);
}
function verifyIssued(value, fields) {
  if (!value || fields.some(name =>
    typeof value[name] !== 'string' || !value[name]))
    throw Error('Unexpected server credential response; outcome uncertain');
  return value;
}
function grantById(session, id) {
  const grant = session.data.grants.find(g => g.id === id);
  if (!grant || grant.revoked) throw Error('Unknown or revoked grant');
  return grant;
}
export async function createGrant(session, specification, emergency, parentId = null) {
  const spec = validateGrant(specification);
  const parent = parentId ? grantById(session, parentId) : null;
  const parentToken = parent ? parent.users[0]?.token : session.data.parentToken;
  if (!parentToken) throw Error('Selected parent grant has no user token');
  await journal(session, {type: 'grant-create', scope: spec.locations,
    parentId: parentId || undefined});
  const value = verifyIssued(await session.client.json(
    'POST', '/grants', spec, parentToken),
  ['id', 'issuerToken']);
  const grant = {...spec, id: value.id, issuerToken: value.issuerToken,
    parentId, users: [], revoked: false};
  try {
    await commit(session, state => state.grants.push(grant));
  } catch (error) {
    emergency(grant);
    throw error;
  }
  return grant;
}
export async function importGrant(session, id, issuerToken) {
  if (!/^[a-f0-9]{32}$/.test(id) || !issuerToken || issuerToken.length > 1024)
    throw Error('Invalid grant credentials');
  assertIdle(session);
  if (session.data.grants.some(g => g.id === id))
    throw Error('Grant already exists in this vault');
  const updated = copy(session.data);
  updated.grants.push({id, issuerToken, users: [], imported: true,
    revoked: false});
  await saveVault(session, updated);
}
export async function issueUser(session, id, emergency) {
  const grant = grantById(session, id);
  await journal(session, {type: 'issue-user', grantId: id});
  const value = verifyIssued(await session.client.json('POST',
    '/grants/' + id + '/users', undefined, grant.issuerToken),
  ['grantId', 'userId', 'userToken']);
  if (value.grantId !== id) throw Error('Issuer returned unexpected grant ID');
  const issued = {id: value.userId, token: value.userToken};
  try {
    await commit(session, state => state.grants
      .find(g => g.id === id).users.push(issued));
  } catch (error) {
    emergency({grantId: id, userId: issued.id, userToken: issued.token});
    throw error;
  }
  return issued;
}
export async function inspectGrant(session, id) {
  const grant = grantById(session, id);
  const user = grant.users[0];
  if (!user) throw Error('Issue a user token to inspect this grant');
  return session.client.json('GET', '/grants/' + id, undefined, user.token);
}
async function revoke(session, id, userId) {
  const grant = grantById(session, id);
  const path = '/grants/' + id + (userId ? '/users/' + userId : '');
  await journal(session, {type: userId ? 'revoke-user' : 'revoke-grant',
    grantId: id, userId: userId || undefined});
  const response = await session.client.send('DELETE', path,
    undefined, grant.issuerToken);
  let confirmed = response.status === 200;
  if (response.status === 404) {
    const value = await response.json().catch(() => ({}));
    confirmed = value.error === (userId ? 'user_not_found' : 'grant_not_found');
  }
  if (!confirmed) throw Error('Revocation unresolved (HTTP ' + response.status
    + '); do not blindly retry');
  await commit(session, state => {
    const target = state.grants.find(g => g.id === id);
    if (userId) target.users = target.users.filter(u => u.id !== userId);
    else {
      const invalid = new Set([id]);
      for (let i = 0; i < state.grants.length; i++) {
        for (const entry of state.grants) {
          if (invalid.has(entry.parentId)) invalid.add(entry.id);
        }
      }
      for (const entry of state.grants) {
        if (invalid.has(entry.id)) {
          entry.revoked = true;
          delete entry.issuerToken;
          entry.users = [];
        }
      }
    }
  });
}
export const revokeUser = (session, id, userId) => revoke(session, id, userId);
export const revokeGrant = (session, id) => revoke(session, id);
export async function clearPending(session) {
  if (!session.data.pending) return;
  const updated = copy(session.data);
  updated.pending = null;
  await saveVault(session, updated);
}
