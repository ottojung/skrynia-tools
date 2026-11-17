import {applications, history, activate} from './apps.js';

const appId = name => 'app-history-' + name;
function element(tag, value) {
  const node = document.createElement(tag);
  if (value !== undefined) node.textContent = value;
  return node;
}
function button(label, handler) {
  const node = element('button', label);
  node.type = 'button';
  node.addEventListener('click', handler);
  return node;
}
function count(versions) {
  return versions.length + (versions.length === 1 ? ' release' : ' releases');
}
async function displayApp(panel, session, name, status, reload) {
  const details = await history(session.client, name, session.data.parentToken);
  panel.replaceChildren();
  const heading = element('h3', name + ' — ' + count(details.versions));
  panel.append(heading);
  panel.append(element('p', 'Active: ' + (details.active || 'none')));
  if (!details.versions.length) panel.append(element('p', 'No ready releases'));
  for (const release of details.versions) {
    const row = element('div');
    row.className = 'version-row';
    const selected = release.id === details.active;
    const metadata = release.usable
      ? (release.commit ? ' · commit ' + release.commit : '')
        + ' · ' + release.files + ' files'
      : ' · unavailable: ' + release.error;
    const title = element('span', release.id
      + (selected ? ' (ACTIVE)' : '') + metadata);
    row.append(title);
    if (!selected && release.usable) {
      row.append(button('Activate this version', async () => {
        const description = 'Activate app ' + name + '\nCurrent: '
          + (details.active || 'none') + '\nTarget: ' + release.id
          + '\nOnly active.json is atomically replaced; no data migration runs.';
        if (!confirm(description)) return;
        status('Activating ' + name + '…');
        try {
          await activate(session.client, name, release.id,
            session.data.parentToken, session.key);
          await reload();
          status('Activated ' + name + ' → ' + release.id);
        } catch (error) {
          status(error.message);
          await reload();
        }
      }));
    }
    panel.append(row);
  }
}
export function bindApps(elementId, sessionProvider, status) {
  const host = document.getElementById(elementId);
  let refreshing = false;
  async function refresh() {
    const session = sessionProvider();
    if (!session || refreshing) return;
    refreshing = true;
    host.replaceChildren(element('p', 'Reading application release history…'));
    try {
      const all = await applications(session.client, session.data.parentToken);
      const container = document.createDocumentFragment();
      if (!all.length) container.append(element('p',
        'No apps found. Listing requires the root grant to have list authority over apps/*.'));
      for (const name of all) {
        const panel = element('section');
        panel.className = 'app-panel';
        await displayApp(panel, session, name, status, refresh);
        container.append(panel);
      }
      host.replaceChildren(container);
      status('Loaded ' + all.length + ' applications.');
    } catch (error) {
      host.replaceChildren(element('p', error.message));
      status('Application listing failed: ' + error.message);
    } finally {
      refreshing = false;
    }
  }
  return {refresh, clear: () => host.replaceChildren()};
}
