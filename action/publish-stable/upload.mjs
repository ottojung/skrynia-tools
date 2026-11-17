import {readFile} from 'node:fs/promises';
import {createTransport} from '../../bootstrap/transport.mjs';

const env = process.env;
const required = name => {
  const value = env[name];
  if (!value) throw Error('Missing GitHub Action input: ' + name);
  return value;
};
const app = required('SKRYNIA_APP_NAME');
const asset = required('SKRYNIA_STABLE_ASSET');
if (!/^[a-z0-9][a-z0-9-]{0,63}$/.test(app)
  || !['sw.js','manifest.webmanifest','icon.svg'].includes(asset))
  throw Error('Invalid application name or stable asset');
const bytes = await readFile(required('SKRYNIA_STABLE_FILE'));
if (bytes.length === 0 || bytes.length > 5000000)
  throw Error('Stable PWA file must be nonempty and at most 5 MB');
const client = createTransport(required('SKRYNIA_URL'),
  required('SKRYNIA_TOKEN'));
const key = 'apps/' + app + '/' + asset;
await client.stable(key, bytes);
console.log('Published stable PWA asset:', key);
