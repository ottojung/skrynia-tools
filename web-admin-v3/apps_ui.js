import {activationPlan, inspectManifest, loadAllAppStates} from './apps.js';

export const PAGE_SIZE = 50;

export function isUncertainActivationError(error) {
  if (!error) return false;
  if (error.code === 'transport_unknown' || error.code === 'uncertain' || error.code === 'lost_ack') return true;
  const msg = String(error.message || '');
  return /outcome unknown|lost ack|response.*unknown|unknown outcome/i.test(msg);
}

export function paginateVersions(versions, page, pageSize = PAGE_SIZE) {
  const list = Array.isArray(versions) ? versions : [];
  const totalPages = Math.max(1, Math.ceil(list.length / pageSize));
  const safePage = Math.min(Math.max(1, Number.isInteger(page) ? page : 1), totalPages);
  const start = (safePage - 1) * pageSize;
  return {
    page: safePage,
    totalPages,
    pageSize,
    total: list.length,
    start,
    end: Math.min(start + pageSize, list.length),
    items: list.slice(start, start + pageSize),
  };
}

function el(tag, text, className) {
  const node = document.createElement(tag);
  if (text !== undefined && text !== null) node.textContent = text;
  if (className) node.className = className;
  return node;
}

function clear(node) {
  node.replaceChildren();
}

function hideManifest() {
  const view = document.getElementById('manifest-view');
  const empty = document.getElementById('manifest-empty');
  if (view) {
    view.hidden = true;
    view.textContent = '';
  }
  if (empty) empty.hidden = false;
}

export function createAppsController({hostId, capabilities, status, onInspectManifest, getOperatorSecret}) {
  const host = document.getElementById(hostId);
  let busy = false;
  let appStates = [];
  const pageByApp = new Map();
  const uncertainByApp = new Map();
  const inFlightByApp = new Map();

  function setUncertain(app, info) {
    uncertainByApp.set(app, info);
  }
  function clearUncertain(app) {
    uncertainByApp.delete(app);
  }

  async function renderLoaded() {
    const fragment = document.createDocumentFragment();
    if (appStates.length === 0) {
      fragment.append(el('p', 'No applications found.'));
    }
    for (const appState of appStates) {
      fragment.append(renderApp(appState));
    }
    clear(host);
    host.append(fragment);
  }

  async function refresh() {
    if (!capabilities) {
      clear(host);
      host.append(el('p', 'No capability adapter configured.', 'error'));
      return;
    }
    busy = true;
    clear(host);
    host.append(el('p', 'Loading applications and release history…'));
    try {
      if (capabilities && typeof capabilities.setOperatorSecret === 'function' && getOperatorSecret) {
        try { capabilities.setOperatorSecret(getOperatorSecret() || ''); } catch {}
      }
      const states = await loadAllAppStates(capabilities);
      appStates = states;
      for (const st of states) {
        clearUncertain(st.name);
        const p = paginateVersions(st.versions || [], pageByApp.get(st.name) || 1);
        pageByApp.set(st.name, p.page);
      }
      busy = false;
      await renderLoaded();
      status('Loaded ' + states.length + ' application(s). Reconciled active pointers from current adapter state.');
    } catch (error) {
      busy = false;
      clear(host);
      host.append(el('p', error.message || 'Failed to load applications', 'error'));
      status('Failed to load applications: ' + (error.message || 'unknown error'));
    }
  }

  function renderPagination(appState, pageState) {
    if (pageState.totalPages <= 1) return null;
    const nav = el('nav', undefined, 'pagination');
    nav.setAttribute('aria-label', 'Version pages for ' + appState.name);
    const info = el('span', 'Page ' + pageState.page + ' of ' + pageState.totalPages + ' (' + pageState.total + ' versions)', 'page-info');
    const prev = el('button', 'Previous');
    prev.type = 'button';
    prev.disabled = pageState.page <= 1 || busy;
    prev.addEventListener('click', async () => {
      pageByApp.set(appState.name, Math.max(1, pageState.page - 1));
      await renderLoaded();
    });
    const next = el('button', 'Next');
    next.type = 'button';
    next.disabled = pageState.page >= pageState.totalPages || busy;
    next.addEventListener('click', async () => {
      pageByApp.set(appState.name, Math.min(pageState.totalPages, pageState.page + 1));
      await renderLoaded();
    });
    nav.append(prev, info, next);

    const jump = el('label', undefined, 'page-jump');
    jump.append(document.createTextNode('Go to page '));
    const input = document.createElement('input');
    input.type = 'number';
    input.min = '1';
    input.max = String(pageState.totalPages);
    input.value = String(pageState.page);
    input.className = 'page-input';
    jump.append(input);
    const go = el('button', 'Go');
    go.type = 'button';
    go.disabled = busy;
    go.addEventListener('click', async () => {
      const n = Number.parseInt(input.value, 10);
      if (!Number.isInteger(n)) return;
      pageByApp.set(appState.name, Math.min(pageState.totalPages, Math.max(1, n)));
      await renderLoaded();
    });
    nav.append(jump, go);
    return nav;
  }

  function renderApp(appState) {
    const panel = el('section', undefined, 'app-panel');
    panel.append(el('h3', appState.name + ' — ' + appState.versions.length + ' release(s)'));
    const activeText = appState.active ? appState.active : 'none';
    const pointerText = Number.isSafeInteger(appState.pointerVersion) ? String(appState.pointerVersion) : 'unknown';
    panel.append(el('p', 'Active pointer (release ID/hash): ' + activeText));
    panel.append(el('p', 'Loaded CAS pointer state (optimistic token): ' + pointerText, 'status-note'));

    const uncertain = uncertainByApp.get(appState.name);
    if (uncertain) {
      const warn = el('div', undefined, 'uncertain-banner error');
      warn.append(el('strong', 'Activation outcome UNKNOWN — reconciliation required. '));
      warn.append(document.createTextNode(
        'Request for release ' + uncertain.target + ' (expected active ' + (uncertain.expectedActive || 'none') +
        ', expected pointer state ' + uncertain.expectedPointerVersion + ') did not receive a confirmed outcome. ' +
        'The activation may or may not have been applied. Do not retry automatically. Refresh all to reconcile the current active pointer before any further activation for this app.'));
      if (uncertain.message) warn.append(el('div', uncertain.message, 'status-note'));
      panel.append(warn);
    }

    if (!appState.versions.length) {
      panel.append(el('p', 'No releases found.'));
      return panel;
    }

    const sorted = [...appState.versions].sort((a, b) => a.id.localeCompare(b.id));
    const pageState = paginateVersions(sorted, pageByApp.get(appState.name) || 1);
    pageByApp.set(appState.name, pageState.page);

    const topNav = renderPagination(appState, pageState);
    if (topNav) panel.append(topNav);

    const list = el('div', undefined, 'version-list');
    for (const version of pageState.items) {
      list.append(renderVersionRow(appState, version, Boolean(uncertain)));
    }
    panel.append(list);

    const bottomNav = renderPagination(appState, pageState);
    if (bottomNav) panel.append(bottomNav);
    return panel;
  }

  function renderVersionRow(appState, version, appUncertain) {
    const row = el('div', undefined, 'version-row');
    const left = el('div', undefined, 'version-meta');
    const idLine = el('div', undefined, 'version-id');
    idLine.append(document.createTextNode(version.id));
    if (appState.active === version.id) idLine.append(el('span', 'ACTIVE', 'badge active'));
    if (version.ready) idLine.append(el('span', 'ready', 'badge ready')); else idLine.append(el('span', 'not ready', 'badge error'));
    if (!version.usable) idLine.append(el('span', 'unavailable', 'badge error'));
    left.append(idLine);
    const details = [];
    if (version.commit) details.push('commit ' + version.commit);
    if (version.repo) details.push('repo ' + version.repo);
    if (typeof version.files === 'number') details.push(version.files + ' file(s)');
    if (version.error) details.push('issue: ' + version.error);
    if (details.length) left.append(el('div', details.join(' · '), 'status-note'));
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

    const canActivate = appState.active !== version.id && version.usable && !appUncertain && !inFlightByApp.get(appState.name);
    if (appState.active !== version.id) {
      const activate = el('button', appState.active ? 'Activate / rollback to this version' : 'Activate');
      activate.type = 'button';
      activate.disabled = busy || !canActivate;
      if (appUncertain) activate.title = 'Disabled until uncertain activation is reconciled by refresh';
      else if (!version.usable) activate.title = 'Release is not usable';
      activate.addEventListener('click', async () => {
        if (inFlightByApp.get(appState.name)) return;
        let currentState = appStates.find(s => s.name === appState.name) || appState;
        try {
          const plan = activationPlan(currentState, version.id);
          if (plan.direction === 'noop') {
            status('Selected release is already active.');
            return;
          }
          const prompt = [
            'Activate release by compare-and-swap pointer only (O(1), no data migration).',
            'Application: ' + plan.app,
            'Current active: ' + (plan.current || 'none'),
            'Target release: ' + plan.target,
            'Expected active (optimistic CAS): ' + (plan.expectedActive || 'none'),
            'Expected pointer state/nonce: ' + plan.expectedPointerVersion,
            '',
            'Continue?',
          ].join('\n');
          if (!confirm(prompt)) return;
          inFlightByApp.set(plan.app, true);
          status('Activating ' + plan.app + ' → ' + plan.target + '…');
          await capabilities.activateRelease(plan.app, plan.target, {
            expectedActive: plan.expectedActive,
            expectedPointerVersion: plan.expectedPointerVersion,
          });
          clearUncertain(plan.app);
          status('Activation confirmed by adapter: ' + plan.app + ' → ' + plan.target + '. Refreshing state.');
          await refresh();
        } catch (error) {
          currentState = appStates.find(s => s.name === appState.name) || appState;
          let planInfo = {
            app: appState.name,
            target: version.id,
            expectedActive: currentState.active ?? null,
            expectedPointerVersion: currentState.pointerVersion,
          };
          try {
            const p = activationPlan(currentState, version.id);
            planInfo.expectedActive = p.expectedActive;
            planInfo.expectedPointerVersion = p.expectedPointerVersion;
          } catch {}
          if (error && error.code === 'conflict') {
            clearUncertain(appState.name);
            status('Conflict: active pointer changed concurrently. ' + (error.message || '') + ' Refreshing.');
            await refresh();
          } else if (isUncertainActivationError(error)) {
            setUncertain(appState.name, {
              ...planInfo,
              message: error.message || 'Unknown transport outcome',
            });
            status('ACTIVATION OUTCOME UNKNOWN for ' + appState.name + '. Do not retry. Refresh to reconcile.');
            await renderLoaded();
          } else {
            clearUncertain(appState.name);
            status('Activation failed: ' + (error.message || 'unknown error') + '. Refresh recommended to verify current state.');
            await refresh();
          }
        } finally {
          inFlightByApp.delete(appState.name);
        }
      });
      actions.append(activate);
    } else if (appState.active === version.id) {
      actions.append(el('span', 'Current active version', 'status-note'));
    } else if (appUncertain) {
      actions.append(el('span', 'Reconcile required before activation', 'status-note error'));
    }
    row.append(actions);
    return row;
  }

  return {
    refresh: () => { if (!busy) return refresh(); },
    clear: () => {
      appStates = [];
      pageByApp.clear();
      uncertainByApp.clear();
      inFlightByApp.clear();
      clear(host);
      hideManifest();
    },
  };
}
