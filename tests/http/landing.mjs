import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { Script } from 'node:vm';

const html = readFileSync(new URL('../../src/http_app/landing.html', import.meta.url), 'utf8');
const script = new Script(html.match(/<script>([\s\S]*?)<\/script>/)[1]);
const release = '1234567890abcdef1234567890abcdef';

async function visit(address, pointer = { schema: 1, release }) {
  const page = new URL(address);
  const requested = [];
  const redirects = [];
  let error = '';
  const location = {
    pathname: page.pathname, search: page.search, hash: page.hash,
    origin: page.origin, href: page.href,
    replace(url) { redirects.push(String(url)); },
  };
  script.runInNewContext({
    URL, location,
    fetch: async (url, options) => {
      requested.push(String(url));
      assert.equal(options.cache, 'no-store');
      return { ok: !!pointer, json: async () => pointer };
    },
    document: { getElementById: () => ({
      set textContent(value) { error = value; },
    }) },
  });
  await new Promise(resolve => setImmediate(resolve));
  return { requested, redirects, error };
}

const tests = [
  [
    'https://kawun.a.vau.place/?some=query&argument=list#section',
    'https://kawun.a.vau.place/active.json',
    'https://kawun.a.vau.place/releases/' + release + '/index.html?some=query&argument=list#section',
  ],
  [
    'https://kawun.a.vau.place/index.html?a=1#top',
    'https://kawun.a.vau.place/active.json',
    'https://kawun.a.vau.place/releases/' + release + '/index.html?a=1#top',
  ],
  [
    'https://kawun.a.vau.place/profile/settings?tab=2#field',
    'https://kawun.a.vau.place/active.json',
    'https://kawun.a.vau.place/releases/' + release + '/profile/settings?tab=2#field',
  ],
  [
    'http://127.0.0.1:1337/store/apps/kawun/index.html?q=3#legacy',
    'http://127.0.0.1:1337/store/apps/kawun/active.json',
    'http://127.0.0.1:1337/store/apps/kawun/releases/' + release + '/index.html?q=3#legacy',
  ],
  [
    'https://vau.place/_skrynia/store/apps/kawun/index.html?q=2#details',
    'https://vau.place/_skrynia/store/apps/kawun/active.json',
    'https://vau.place/_skrynia/store/apps/kawun/releases/' + release + '/index.html?q=2#details',
  ],
];
for (const [address, pointer, target] of tests) {
  const result = await visit(address);
  assert.deepEqual(result.requested, [pointer]);
  assert.deepEqual(result.redirects, [target]);
  assert.equal(result.error, '');
}
for (const pointer of [null, {}, {schema: 1, release: '../evil'}]) {
  const result = await visit('https://kawun.a.vau.place/', pointer);
  assert.deepEqual(result.redirects, []);
  assert.match(result.error, /Application unavailable/);
}
console.log('Stable app-origin bootstrap URLs and error handling: PASS');
