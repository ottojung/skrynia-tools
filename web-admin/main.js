import {createClient, openVault, validateConfig} from './skrynia.js';
import {decrypt} from './crypto.js';
import {bindApps} from './apps-ui.js';
import {createGrant, importGrant, issueUser, inspectGrant,
  revokeGrant, revokeUser, clearPending} from './admin.js';

const el = id => document.getElementById(id);
let session = null;
let emergency = null;
let busy = false;
const status = text => { el('status').textContent = text; };
const appManager = bindApps('applications', () => session, status);
el('refresh-apps').addEventListener('click', () => void appManager.refresh());
function recovery(value) {
  emergency = value;
  el('recovery').hidden = false;
}
function displayToken(token) {
  el('credential-output').value = token;
}
function append(parent, tag, text) {
  const element = document.createElement(tag);
  element.textContent = text;
  parent.append(element);
  return element;
}
function action(parent, title, callback) {
  const button = append(parent, 'button', title);
  button.type = 'button';
  button.disabled = busy || Boolean(session?.data.pending) || Boolean(session?.unresolved);
  button.addEventListener('click', () => run(callback));
}
function render() {
  if (!session) return;
  const pending = session.data.pending;
  el('pending-section').hidden = !pending && !session.unresolved;
  el('resolve').disabled = Boolean(session.unresolved);
  el('pending-text').textContent = session.unresolved
    ? 'Vault save outcome unknown. Reload and unlock to reconcile; do not retry mutations.'
    : pending ? 'An operation may already have committed: '
      + JSON.stringify(pending.operation)
      + '. Do not retry until its outcome has been investigated.'
    : '';
  const choices = el('grant-parent');
  const previous = choices.value;
  choices.replaceChildren();
  choices.add(new Option('Installed root grant', ''));
  for (const grant of session.data.grants) {
    if (!grant.revoked && grant.users.length) {
      choices.add(new Option(grant.id + ' (user token)', grant.id));
    }
  }
  if ([...choices.options].some(option => option.value === previous))
    choices.value = previous;
  const container = el('grants');
  container.replaceChildren();
  for (const grant of session.data.grants) {
    const box = append(container, 'article', '');
    box.className = 'grant';
    append(box, 'h3', grant.id + (grant.revoked ? ' (revoked)' : ''));
    append(box, 'p', (grant.locations || 'Imported issuer')
      + ' · ' + (grant.operations?.join(', ') || 'permissions unknown'));
    if (grant.revoked) continue;
    const buttons = append(box, 'div', '');
    buttons.className = 'actions';
    action(buttons, 'Issue user', async () => {
      const issued = await issueUser(session, grant.id, recovery);
      displayToken(issued.token);
      status('User issued and saved; user ID: ' + issued.id);
    });
    if (grant.users.length) {
      action(buttons, 'Inspect', async () => {
        const inspected = await inspectGrant(session, grant.id);
        status(JSON.stringify(inspected, null, 2));
      });
    }
    action(buttons, 'Show issuer token', async () => {
      if (!confirm('Reveal the issuer capability in this browser?')) return;
      displayToken(grant.issuerToken);
    });
    action(buttons, 'Revoke grant', async () => {
      const id = prompt('Type the exact grant ID to revoke its subtree:');
      if (id !== grant.id) return;
      await revokeGrant(session, grant.id);
      status('Grant revoked: ' + grant.id);
    });
    for (const user of grant.users) {
      const row = append(box, 'div', '');
      row.className = 'actions';
      append(row, 'span', 'User ' + user.id);
      action(row, 'Show user token', async () => displayToken(user.token));
      action(row, 'Revoke user', async () => {
        if (!confirm('Revoke user ' + user.id + '?')) return;
        await revokeUser(session, grant.id, user.id);
        status('User revoked: ' + user.id);
      });
    }
  }
}
async function run(callback) {
  if (busy) return;
  busy = true;
  status('');
  try {
    if (!session) throw Error('Unlock first');
    await callback();
  } catch (error) {
    status(error.message || 'Unknown error');
  } finally {
    busy = false;
    render();
  }
}
el('unlock-form').addEventListener('submit', async event => {
  event.preventDefault();
  const password = el('password').value;
  status('Unlocking…');
  try {
    const response = await fetch('./config.json', {cache: 'no-store',
      redirect: 'error'});
    if (!response.ok) throw Error('Missing deployment configuration');
    const config = validateConfig(await response.json());
    const client = createClient();
    const {bytes, hash} = await openVault(client, config.vault);
    const data = await decrypt(bytes, password, config.vault);
    session = {client, key: config.vault, hash, data, password};
    el('password').value = '';
    el('unlock').hidden = true;
    el('workspace').hidden = false;
    status('Vault unlocked. Grant authority remains in this browser only.');
    render();
    void appManager.refresh();
  } catch (error) {
    status('Unlock failed: ' + (error.message || 'invalid access secret'));
  }
});
el('grant-form').addEventListener('submit', event => {
  event.preventDefault();
  const form = new FormData(event.currentTarget);
  void run(async () => {
    const spec = {locations: String(form.get('locations')).trim(),
      operations: form.getAll('op'),
      maxBytes: Number(form.get('maxBytes')),
      maxObjects: Number(form.get('maxObjects'))};
    const grant = await createGrant(session, spec, recovery,
      el('grant-parent').value || null);
    status('Grant created and issuer safely saved: ' + grant.id);
  });
});
el('import-form').addEventListener('submit', event => {
  event.preventDefault();
  const form = new FormData(event.currentTarget);
  void run(async () => {
    await importGrant(session, String(form.get('id')).trim(),
      String(form.get('issuer')).trim());
    event.currentTarget.reset();
    status('Issuer imported into encrypted vault (not yet verified).');
  });
});
el('resolve').addEventListener('click', () => run(async () => {
  if (prompt('Type INVESTIGATED to clear the pending record:') !==
    'INVESTIGATED') return;
  await clearPending(session);
  status('Pending record acknowledged. No mutation was retried.');
}));
el('download-recovery').addEventListener('click', () => {
  if (!emergency) return;
  const blob = new Blob([JSON.stringify(emergency)], {type: 'application/json'});
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = 'skrynia-private-recovery.json';
  link.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
});
el('clear-output').addEventListener('click', () => displayToken(''));
el('lock').addEventListener('click', () => {
  session = null;
  emergency = null;
  displayToken('');
  el('grants').replaceChildren();
  appManager.clear();
  el('recovery').hidden = true;
  el('workspace').hidden = true;
  el('unlock').hidden = false;
  status('Vault locked.');
});
window.addEventListener('beforeunload', event => {
  if (emergency) event.preventDefault();
});
document.documentElement.dataset.ready = 'true';
