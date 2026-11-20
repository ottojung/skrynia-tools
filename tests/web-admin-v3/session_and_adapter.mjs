import assert from 'node:assert/strict';
import {createAppsController} from '../../web-admin-v3/apps_ui.js';

function makeElement(tag = 'div') {
  const children = [];
  const listeners = new Map();
  const el = {
    tagName: String(tag).toUpperCase(),
    textContent: '',
    className: '',
    hidden: false,
    type: '',
    disabled: false,
    value: '',
    min: '',
    max: '',
    title: '',
    style: {},
    children,
    _attrs: new Map(),
    append(...nodes) {
      for (const n of nodes) {
        if (n == null) continue;
        if (n && n._fragment) {
          for (const c of n.children) children.push(c);
        } else {
          children.push(n);
        }
      }
    },
    replaceChildren() { children.length = 0; },
    setAttribute(k, v) { this._attrs.set(k, String(v)); },
    getAttribute(k) { return this._attrs.get(k); },
    addEventListener(type, fn) {
      if (!listeners.has(type)) listeners.set(type, []);
      listeners.get(type).push(fn);
    },
    async dispatch(type, event = {}) {
      const fns = listeners.get(type) || [];
      for (const fn of [...fns]) await fn(event);
    },
    querySelectorAll() { return []; },
  };
  return el;
}

function makeFragment() {
  const children = [];
  return {
    _fragment: true,
    children,
    append(...nodes) {
      for (const n of nodes) {
        if (n == null) continue;
        if (n && n._fragment) {
          for (const c of n.children) children.push(c);
        } else {
          children.push(n);
        }
      }
    },
  };
}

function collectText(node, out = []) {
  if (!node) return out;
  if (typeof node === 'string') { out.push(node); return out; }
  if (node.textContent) out.push(node.textContent);
  if (Array.isArray(node.children)) {
    for (const c of node.children) collectText(c, out);
  }
  return out;
}

function setupDom() {
  const elements = new Map([
    ['apps', makeElement('div')],
    ['manifest-view', makeElement('pre')],
    ['manifest-empty', makeElement('p')],
  ]);
  const prevDocument = globalThis.document;
  const prevConfirm = globalThis.confirm;
  globalThis.document = {
    getElementById(id) {
      if (!elements.has(id)) elements.set(id, makeElement('div'));
      return elements.get(id);
    },
    createElement(tag) { return makeElement(tag); },
    createDocumentFragment() { return makeFragment(); },
    createTextNode(text) { return {nodeType: 3, textContent: String(text), children: []}; },
  };
  globalThis.confirm = () => true;
  return {
    elements,
    restore() {
      globalThis.document = prevDocument;
      globalThis.confirm = prevConfirm;
    },
  };
}

function manifest(app, id) {
  return {
    schema: 'skrynia-native-static-v1', app, release: id,
    commit: 'c', repo: 'r', files: [{path: 'index.html', bytes: 1, sha256: '0'.repeat(64)}],
  };
}

const r1 = '1'.repeat(64);
const r2 = '2'.repeat(64);
const r3 = '3'.repeat(64);

function makeCaps({activateImpl, setOperatorSecret} = {}) {
  const state = {
    apps: new Map([['alpha', {
      active: r1,
      pointerVersion: 1,
      releases: new Map([
        [r1, {id: r1, ready: true, manifest: manifest('alpha', r1)}],
        [r2, {id: r2, ready: true, manifest: manifest('alpha', r2)}],
        [r3, {id: r3, ready: true, manifest: manifest('alpha', r3)}],
      ]),
    }]]),
  };
  return {
    kind: 'fake-session-test',
    setOperatorSecret,
    async listApps() { return ['alpha']; },
    async getAppState(name) {
      const rec = state.apps.get(name);
      if (!rec) throw Object.assign(Error('not found'), {code: 'not_found'});
      const versions = [...rec.releases.values()].map(rel => ({
        id: rel.id, ready: rel.ready, usable: true, commit: 'c', repo: 'r', files: 1,
      }));
      return {name, active: rec.active, pointerVersion: rec.pointerVersion, versions};
    },
    async getReleaseManifest(name, id) { return manifest(name, id); },
    async activateRelease(name, id, opts) {
      if (activateImpl) return activateImpl({name, id, opts, state});
      const rec = state.apps.get(name);
      if (rec.active !== opts.expectedActive || rec.pointerVersion !== opts.expectedPointerVersion) {
        throw Object.assign(Error('conflict'), {code: 'conflict'});
      }
      rec.active = id;
      rec.pointerVersion += 1;
      return {app: name, active: id, pointerVersion: rec.pointerVersion};
    },
    _state: state,
  };
}

{
  const dom = setupDom();
  const seen = [];
  const statuses = [];
  try {
    const caps = makeCaps({setOperatorSecret: s => seen.push(s)});
    const ctrl = createAppsController({
      hostId: 'apps',
      capabilities: caps,
      status: m => statuses.push(m),
      getOperatorSecret: () => 'session-only-secret',
      onInspectManifest: () => {},
    });
    await ctrl.refresh();
    assert.ok(seen.includes('session-only-secret'), 'adapter must receive operator secret from session');
    ctrl.clear();
  } finally {
    dom.restore();
  }
}

{
  const dom = setupDom();
  const statuses = [];
  try {
    let calls = 0;
    const caps = makeCaps({
      activateImpl: async () => {
        calls++;
        throw Object.assign(Error('Program 0 request failed; outcome unknown'), {code: 'transport_unknown'});
      },
    });
    const ctrl = createAppsController({
      hostId: 'apps',
      capabilities: caps,
      status: m => statuses.push(m),
      getOperatorSecret: () => '',
      onInspectManifest: () => {},
    });
    await ctrl.refresh();
    const host = dom.elements.get('apps');
    function findButtons(node) {
      const out = [];
      if (!node) return out;
      if (node.tagName === 'BUTTON') out.push(node);
      for (const c of (node.children || [])) out.push(...findButtons(c));
      return out;
    }
    let buttons = findButtons(host);
    let activateBtn = buttons.find(b => /Activate/i.test(b.textContent) && !/Inspect/i.test(b.textContent));
    assert.ok(activateBtn, 'expected an activation button');
    assert.equal(activateBtn.disabled, false);
    await activateBtn.dispatch('click');
    assert.equal(calls, 1, 'must not auto-retry uncertain activation');
    const text = collectText(host).join('\n');
    assert.match(text, /outcome UNKNOWN|reconciliation required/i);
    assert.match(text, /Do not retry automatically|Refresh.*reconcile|Refresh all to reconcile/i);
    buttons = findButtons(host);
    const activateBtnsAfter = buttons.filter(b => /Activate/i.test(b.textContent) && !/Inspect/i.test(b.textContent));
    assert.ok(activateBtnsAfter.length >= 1);
    assert.ok(activateBtnsAfter.every(b => b.disabled), 'all activation actions must be disabled until reconciliation');
    await ctrl.refresh();
    const textAfter = collectText(host).join('\n');
    assert.doesNotMatch(textAfter, /outcome UNKNOWN — reconciliation required/i);
    assert.equal(caps._state.apps.get('alpha').active, r1, 'uncertain outcome must not assume pointer changed');
    ctrl.clear();
  } finally {
    dom.restore();
  }
}

{
  const dom = setupDom();
  const statuses = [];
  try {
    let calls = 0;
    const caps = makeCaps({
      activateImpl: async () => {
        calls++;
        throw Object.assign(Error('Activation conflict: active release changed concurrently'), {code: 'conflict'});
      },
    });
    const ctrl = createAppsController({
      hostId: 'apps', capabilities: caps, status: m => statuses.push(m),
      getOperatorSecret: () => '', onInspectManifest: () => {},
    });
    await ctrl.refresh();
    function findButtons(node) {
      const out = [];
      if (!node) return out;
      if (node.tagName === 'BUTTON') out.push(node);
      for (const c of (node.children || [])) out.push(...findButtons(c));
      return out;
    }
    const host = dom.elements.get('apps');
    let activateBtn = findButtons(host).find(b => /Activate/i.test(b.textContent) && !/Inspect/i.test(b.textContent));
    assert.ok(activateBtn);
    await activateBtn.dispatch('click');
    assert.equal(calls, 1);
    assert.ok(statuses.some(s => /Conflict/i.test(s)));
    const text = collectText(host).join('\n');
    assert.doesNotMatch(text, /outcome UNKNOWN — reconciliation required/i);
    ctrl.clear();
  } finally {
    dom.restore();
  }
}

console.log('v3 session/operator-secret contract and lost-ack/conflict controller tests passed');
