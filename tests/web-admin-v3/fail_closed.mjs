import assert from 'node:assert/strict';
import {createFailClosedCapabilities} from '../../web-admin-v3/capabilities.js';
import {createFailClosedNativeProtocol} from '../../web-admin-v3/binary_transport.js';

{
  const caps = createFailClosedCapabilities();
  assert.equal(caps.kind, 'fail-closed');
  for (const method of ['listApps', 'getAppState', 'getReleaseManifest', 'activateRelease']) {
    await assert.rejects(() => caps[method]('app', 'x'), err => {
      assert.match(err.message, /protocol not frozen|not configured|capability/i);
      return true;
    });
  }
}

{
  const caps = createFailClosedCapabilities('no adapter');
  await assert.rejects(() => caps.listApps(), /no adapter/);
}

{
  const proto = createFailClosedNativeProtocol();
  await assert.rejects(() => proto.request(new Uint8Array([1, 2, 3])), /not frozen/i);
}

console.log('v3 fail-closed capability and protocol placeholder tests passed');
