import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

// Negative boundary checks for Theorem Lab native v3 feedback migration.
// Ensures no Program0/root token, no write/admin cap, no legacy public-write store path,
// POST-only /_skrynia/ transport, and fail-closed behavior.

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const read = (p) => readFile(resolve(root, p), 'utf8');

const feedbackRaw = await read('source/theorem-lab/feedback.js');
const commentsDoc = await read('source/theorem-lab/docs/comments.md');
const readme = await read('source/theorem-lab/README.md');

// Strip comments for token/secret pattern checks (comments may describe what is forbidden).
const feedback = feedbackRaw
  .replace(/\/\*[\s\S]*?\*\//g, '')
  .replace(/(^|[^:])\/\/.*$/gm, '$1');

for (const [label, re] of [
  ['Program 0/root/operator/bearer token', /program0.?token|operator.?token|root.?token|bearer\s+[a-f0-9]/i],
  ['Authorization header', /\bAuthorization['"\s:]/],
  ['write/admin capability exposure', /writeCap|adminCap|write_cap|admin_cap|capabilityWrite/i],
  ['legacy public-write mode header', /X-Skrynia-Mode/i],
  ['legacy /store path', /\/_skrynia\/store\//],
  ['GET/PUT/DELETE feedback writes to store', /method:\s*['"`](GET|PUT|DELETE)['"`]/i],
  ['capability persistence in web storage', /localStorage|sessionStorage|IndexedDB/i],
  ['secret fragment/capability parsing', /location\.hash|url\.hash|#cap|capability.*fragment/i],
]) {
  assert.doesNotMatch(feedback, re, `feedback.js must not contain ${label}`);
}

// Must use POST-only /_skrynia/ endpoint.
assert.match(feedback, /\/_skrynia\/?/, 'feedback must target /_skrynia/');
assert.match(feedback, /method:\s*['"`]POST['"`]/, 'feedback must use POST');
assert.doesNotMatch(feedback, /method:\s*['"`]GET['"`]/, 'feedback must not use GET');
assert.doesNotMatch(feedback, /method:\s*['"`]PUT['"`]/, 'feedback must not use PUT');

// Native v3 opcodes / mediated handler.
assert.match(feedback, /OP_INVOKE\s*=\s*3|op.*invoke.*3/i, 'feedback must reference OP_INVOKE=3');
assert.match(feedback, /OP_APP_GET\s*=\s*7|op.*app.?get.*7/i, 'feedback must reference OP_APP_GET=7');

// Fail closed on missing config/service.
assert.match(feedback, /failClosed|Missing feedback|unavailable|fail closed/i, 'feedback must fail closed');
assert.match(feedback, /ensureConfigured|Missing feedback capability configuration|Missing feedback invoke capability/i, 'feedback must require explicit config');

// No insecure fallback.
assert.doesNotMatch(feedback, /insecure fallback|fallback to GET|legacy store/i, 'feedback must not fall back to insecure legacy paths');

// Docs must not advertise legacy public-write store.
assert.doesNotMatch(readme, /X-Skrynia-Mode:\s*public-write/i, 'README must not advertise public-write mode as current');
assert.doesNotMatch(readme, /\/_skrynia\/store\/theorem-lab\/feedback-v1/, 'README must not advertise legacy store URL');
assert.doesNotMatch(commentsDoc, /\/_skrynia\/store\/theorem-lab\/comments-v1/, 'comments doc must not advertise legacy store URL');
assert.match(commentsDoc, /native v3|OP_INVOKE|POST-only/i, 'comments doc must describe native v3 path');

console.log('ok: Theorem Lab feedback native v3 boundary checks passed');
