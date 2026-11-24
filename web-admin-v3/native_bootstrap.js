// Runtime wiring for the independently hosted static admin app.
//
// The trusted origin provisions operator session authority out of band (never
// in a public bundle, URL or log) and exposes it as
// window.__skryniaNativeSession before this module runs. We install a factory
// that main.js resolves into the semantic native capability adapter. With no
// session present the shipped UI keeps its fail-closed default.

import {createNativeCapabilities} from './native_capabilities.js';

const session = globalThis.__skryniaNativeSession;
if (session && typeof session === 'object') {
  if (typeof session.endpoint !== 'string') {
    session.endpoint = new URL('/_skrynia/', globalThis.location.href).href;
  }
  try {
    // Fail closed at construction if the session is incomplete; do not leave a
    // half-configured factory that could emit malformed native requests.
    createNativeCapabilities(session, globalThis.fetch);
    globalThis.__skryniaV3AdminCapabilities = () => createNativeCapabilities(session, globalThis.fetch);
  } catch {
    // Leave the fail-closed adapter in place.
  }
}
