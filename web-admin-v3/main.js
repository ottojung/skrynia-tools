import {createFailClosedCapabilities} from './capabilities.js';
import {createAppsController} from './apps_ui.js';
import {createLocalStorageVaultStorage} from './browser_storage.js';
import {createVaultSession, initializeVault, lockVaultSession} from './vault.js';

const $ = id => document.getElementById(id);
const status = msg => { $('status').textContent = msg; };

function resolveCapabilities() {
  const injected = globalThis.__skryniaV3AdminCapabilities;
  try {
    if (typeof injected === 'function') return injected();
    if (injected && typeof injected === 'object') return injected;
  } catch (error) {
    return createFailClosedCapabilities(
      'Injected capability adapter failed to initialize: ' + (error.message || 'unknown error'));
  }
  return createFailClosedCapabilities();
}

let capabilities = resolveCapabilities();
let session = null;

function setCapabilityStatus() {
  const node = $('capability-status');
  if (capabilities.kind === 'fail-closed') {
    node.textContent = 'Status: fail-closed. ' + (capabilities.reason || 'Native Program 0 protocol adapter is not configured.');
    node.className = 'status-note error';
  } else {
    node.textContent = 'Status: capability adapter loaded (' + (capabilities.kind || 'custom') + ').';
    node.className = 'status-note';
  }
}

const apps = createAppsController({
  hostId: 'apps',
  capabilities,
  status,
  onInspectManifest: manifest => {
    $('manifest-empty').hidden = true;
    $('manifest-view').hidden = false;
    $('manifest-view').textContent = JSON.stringify(manifest.raw ?? manifest, null, 2);
  },
});

function lock() {
  lockVaultSession(session);
  session = null;
  $('workspace').hidden = true;
  $('unlock-section').hidden = false;
  $('vault-secret').value = '';
  apps.clear();
  status('Vault locked.');
}

$('lock').addEventListener('click', lock);

$('unlock-form').addEventListener('submit', async event => {
  event.preventDefault();
  const secret = $('vault-secret').value;
  const context = $('vault-context').value.trim() || 'default';
  if (!secret) {
    status('Access secret is required.');
    return;
  }
  status('Unlocking vault…');
  try {
    const storage = createLocalStorageVaultStorage('skrynia-v3-admin-vault:' + context);
    try {
      session = await createVaultSession(storage, secret, context);
      status('Vault unlocked.');
    } catch (error) {
      if (error && error.code === 'not_found') {
        const ok = confirm('No encrypted vault found for this context. Initialize a new local browser vault?');
        if (!ok) throw error;
        session = await initializeVault(storage, secret, context, {
          schema: 1,
          native: null,
          notes: ['Initialized locally; configure native Program 0 credentials once protocol is frozen.'],
        });
        status('New local browser vault initialized and unlocked.');
      } else {
        throw error;
      }
    }
    $('unlock-section').hidden = true;
    $('workspace').hidden = false;
    $('vault-secret').value = '';
    await apps.refresh();
  } catch (error) {
    status('Unlock failed: ' + (error.message || 'unknown error'));
  }
});

$('refresh-apps').addEventListener('click', async () => {
  await apps.refresh();
});

setCapabilityStatus();
status(capabilities.kind === 'fail-closed'
  ? 'Ready (fail-closed): native protocol adapter required.'
  : 'Ready.');
