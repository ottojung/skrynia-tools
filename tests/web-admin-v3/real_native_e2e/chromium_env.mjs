// On Linux Chromium's singleton/IPC sockets are constrained by sockaddr_un's
// short path limit. A deeply nested evidence TMPDIR can make Chromium exit
// before CDP is available. Keep the test's profile and evidence locations
// unchanged, but use the standard short temporary directory for the browser.
export function chromiumEnvironment(env = process.env, platform = process.platform) {
  return platform === 'linux' ? { ...env, TMPDIR: '/tmp' } : { ...env };
}
