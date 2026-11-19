import {assertAppName, assertReleaseId} from './capabilities.js';

export async function listAllApps(capabilities) {
  const names = await capabilities.listApps();
  if (!Array.isArray(names)) throw Error('Invalid application listing');
  return [...names].sort();
}

export async function loadAllAppStates(capabilities) {
  const names = await listAllApps(capabilities);
  const states = [];
  for (const name of names) {
    states.push(await capabilities.getAppState(name));
  }
  return states;
}

export async function inspectManifest(capabilities, app, releaseId) {
  assertAppName(app);
  assertReleaseId(releaseId);
  return capabilities.getReleaseManifest(app, releaseId);
}

export async function activateOrRollback(capabilities, app, releaseId, expectedActive, expectedPointerVersion) {
  assertAppName(app);
  assertReleaseId(releaseId);
  return capabilities.activateRelease(app, releaseId, {expectedActive, expectedPointerVersion});
}

export function activationPrecondition(state) {
  return state && typeof state.active === 'string' ? state.active : null;
}

export function activationPlan(appState, targetId) {
  if (!appState || appState.name == null) throw Error('Missing application state');
  const target = (appState.versions || []).find(v => v.id === targetId);
  if (!target) throw Error('Selected release does not exist in loaded history');
  if (!target.usable) throw Error('Cannot activate release: ' + (target.error || 'unavailable'));
  const current = appState.active ?? null;
  const direction = current === null ? 'initial'
    : targetId === current ? 'noop'
    : 'switch';
  return {
    app: appState.name,
    current,
    target: targetId,
    direction,
    expectedActive: current,
    expectedPointerVersion: appState.pointerVersion,
  };
}
