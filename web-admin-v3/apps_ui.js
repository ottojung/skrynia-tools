import {activationPlan, inspectManifest, loadAllAppStates} from './apps.js';

function el(tag, text, className) {
  const node = document.createElement(tag);
  if (text !== undefined) node.textContent = text;
  if (className) node.className = className;
  return node;
}

function clear(node) {
  node.replaceChildren();
}

function showManifest(viewId, emptyId, manifest) {
  const view = document.getElementById(viewId);
  const empty = document.getElementById(emptyId);
  empty.hidden = true;
  view.hidden = false;
  view.textContent = JSON.stringify(manifest.raw ?? manifest, null, 2);
}

function hideManifest(viewId, emptyId) {
  const view = document.getElementById(viewId);
  const empty = document.getElementById(emptyId);
  view.hidden = true;
  empty.hidden = false;
  view.textContent = '';
}

export function createAppsController({hostId, capabilities, status, onInspectManifest}) {
  const host = document.getElementById(hostId);
  let busy = false;

  async function render() {
    if (!capabilities) {
      clear(host);
      host.append(el('p', 'No capability adapter configured.', 'error'));
      return;
    }
    busy = true;
    clear(host);
    host.append(el('p', 'Loading applications and release history…'));
    try {
      const states = await loadAllAppStates(capabilities);
      const fragment = document.createDocumentFragment();
      if (states.length === 0) {
        fragment.append(el('p', 'No applications found.'));
      }
      for (const appState of states) {
        fragment.append(renderApp(appState));
      }
      clear(host);
      host.append(fragment);
      status('Loaded ' + states.length + ' application(s).');
    } catch (error) {
      clear(host);
      host.append(el('p', error.message || 'Failed to load applications', 'error'));
      status('Failed to load applications: ' + (error.message || 'unknown error'));
    } finally {
      busy = false;
    }
  }

  function renderApp(appState) {
    const panel = el('section', undefined, 'app-panel');
    const title = el('h3', appState.name + ' — ' + appState.versions.length + ' release(s)');
    panel.append(title);
    const activeText = appState.active ? appState.active : 'none';
    panel.append(el('p', 'Active pointer: ' + activeText));
    if (!appState.versions.length) {
      panel.append(el('p', 'No releases found.'));
      return panel;
    }
    for (const version of [...appState.versions].sort((a, b) => a.id.localeCompare(b.id))) {
      const row = el('div', undefined, 'version-row');
      const left = el('div', undefined, 'version-meta');
      left.append(document.createTextNode(version.id));
      if (appState.active === version.id) left.append(el('span', 'ACTIVE', 'badge active'));
      if (version.ready) left.append(el('span', 'ready', 'badge ready'));
      if (!version.usable) left.append(el('span', 'unavailable', 'badge error'));
      const details = [];
      if (version.commit) details.push('commit ' + version.commit);
      if (typeof version.files === 'number') details.push(version.files + ' file(s)');
      if (version.error) details.push(version.error);
      if (details.length) left.append(document.createTextNode(' · ' + details.join(' · ')));
      row.append(left);

      const actions = el('div', undefined, 'version-actions');
      const inspect = el('button', 'Inspect manifest');
      inspect.type = 'button';
      inspect.disabled = busy || !version.usable;
      inspect.addEventListener('click', async () => {
        try {
          status('Loading manifest ' + version.id + '…');
          const manifest = await inspectManifest(capabilities, appState.name, version.id);
          onInspectManifest(manifest);
          status('Manifest loaded: ' + version.id);
        } catch (error) {
          status('Manifest inspection failed: ' + (error.message || 'unknown error'));
        }
      });
      actions.append(inspect);

      if (appState.active !== version.id && version.usable) {
        const activate = el('button', appState.active ? 'Activate this version' : 'Activate');
        activate.type = 'button';
        activate.disabled = busy;
        activate.addEventListener('click', async () => {
          try {
            const plan = activationPlan(appState, version.id);
            const prompt = [
              'Activate release by compare-and-swap pointer only (O(1), no data migration).',
              'Application: ' + plan.app,
              'Current active: ' + (plan.current || 'none'),
              'Target release: ' + plan.target,
              'Expected current pointer: ' + (plan.expectedActive || 'none'),
              '',
              'Continue?',
            ].join('\n');
            if (!confirm(prompt)) return;
            status('Activating ' + plan.app + ' → ' + plan.target + '…');
            await capabilities.activateRelease(plan.app, plan.target, {
              expectedActive: plan.expectedActive,
            });
            status('Activated ' + plan.app + ' → ' + plan.target);
            await render();
          } catch (error) {
            const conflict = error && error.code === 'conflict'
              ? 'Conflict: active pointer changed concurrently.'
              : 'Activation failed.';
            status(conflict + ' ' + (error.message || 'unknown error'));
            await render();
          }
        });
        actions.append(activate);
      }
      row.append(actions);
      panel.append(row);
    }
    return panel;
  }

  return {
    refresh: () => { if (!busy) return render(); },
    clear: () => { clear(host); hideManifest('manifest-view', 'manifest-empty'); },
  };
}
