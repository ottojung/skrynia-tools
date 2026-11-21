import assert from 'node:assert/strict';
import {spawn} from 'node:child_process';
const origin = process.env.SKRYNIA_TOOLS_ORIGIN;
const password = process.env.TEST_ACCESS_SECRET;
const browser = process.env.CHROME_BIN || 'chromium';
const port = 18000 + Math.floor(Math.random() * 10000);
const chrome = spawn(browser, ['--headless=new', '--no-sandbox',
  '--disable-gpu', '--disable-dev-shm-usage', '--no-first-run',
  '--remote-allow-origins=*',
  '--remote-debugging-port=' + port,
  '--user-data-dir=/tmp/skrynia-admin-chromium-' + process.pid,
  origin + '/'], {stdio:['ignore','ignore','pipe']});
let chromeErr = '';
chrome.stderr.on('data', d => {
  chromeErr += String(d);
  if (chromeErr.length > 8000) chromeErr = chromeErr.slice(-8000);
});
const sleep = ms => new Promise(resolve => setTimeout(resolve, ms));
async function target() {
  let lastTargets = null;
  for (let i = 0; i < 200; i++) {
    if (chrome.exitCode !== null) {
      throw Error('Browser exited before CDP target ready (code='
        + chrome.exitCode + '): ' + chromeErr.slice(-1500));
    }
    try {
      const response = await fetch('http://127.0.0.1:' + port + '/json/list');
      const pages = await response.json();
      lastTargets = pages;
      const page = pages.find(p => p.type === 'page' && p.url?.startsWith(origin));
      if (page?.webSocketDebuggerUrl) return page.webSocketDebuggerUrl;
    } catch {}
    await sleep(100);
  }
  throw Error('Browser app target unavailable (origin=' + origin
    + ', targets=' + JSON.stringify(lastTargets)
    + ', chrome=' + chromeErr.slice(-1500) + ')');
}
function evaluator(socket) {
  let next = 0;
  const pending = new Map();
  socket.addEventListener('message', event => {
    const message = JSON.parse(event.data);
    const reply = pending.get(message.id);
    if (!reply) return;
    pending.delete(message.id);
    if (message.error || message.result?.exceptionDetails)
      reply.reject(Error(JSON.stringify(message)));
    else reply.resolve(message.result?.result?.value);
  });
  return expression => new Promise((resolve, reject) => {
    const id = ++next;
    pending.set(id, {resolve, reject});
    socket.send(JSON.stringify({id, method:'Runtime.evaluate', params:{
      expression, awaitPromise:true, returnByValue:true}}));
  });
}
async function until(evaluate, expression) {
  let current = null;
  for (let i = 0; i < 100; i++) {
    try {
      current = await evaluate(expression);
      if (current) return current;
    } catch {}
    await sleep(100);
  }
  const state = await evaluate('({status:document.getElementById("status")?.textContent,'
    + 'pending:document.getElementById("pending-text")?.textContent,'
    + 'grants:document.querySelectorAll(".grant").length})').catch(() => null);
  throw Error('Browser condition timed out: ' + expression
    + '; last result=' + JSON.stringify(current)
    + '; state=' + JSON.stringify(state));
}
async function click(evaluate, label) {
  const escaped = JSON.stringify(label);
  return evaluate('([...document.querySelectorAll("button")]'
    + '.find(x=>x.textContent===' + escaped + ')||{}).click?.()');
}
async function test() {
  const socket = new WebSocket(await target());
  await new Promise((resolve, reject) => {
    socket.addEventListener('open', resolve, {once:true});
    socket.addEventListener('error', reject, {once:true});
  });
  const evaluate = evaluator(socket);
  await until(evaluate,
    'location.pathname.startsWith("/releases/") &&'
    + ' document.documentElement.dataset.ready==="true"');
  assert.equal(await evaluate('document.getElementById("workspace").hidden'), true);
  await evaluate('document.getElementById("password").value="wrong";'
    + 'document.getElementById("unlock-form").requestSubmit()');
  await until(evaluate,
    'document.getElementById("status").textContent.includes("Unlock failed")');
  await evaluate('document.getElementById("password").value='
    + JSON.stringify(password)
    + ';document.getElementById("unlock-form").requestSubmit()');
  await until(evaluate, 'document.getElementById("workspace").hidden===false');
  assert.equal(await evaluate('document.getElementById("password").value'), '');
  await until(evaluate,
    '[...document.querySelectorAll("#applications .app-panel h3")]'
    + '.some(x=>x.textContent.startsWith("demo — 2 releases"))');
  const panel = '[...document.querySelectorAll("#applications .app-panel")]'
    + '.find(x=>x.querySelector("h3")?.textContent.startsWith("demo"))';
  const active = panel + '.querySelector("p").textContent';
  const first = '1'.repeat(32);
  const second = '2'.repeat(32);
  assert.equal(await evaluate(active), 'Active: ' + second);
  await evaluate('window.confirm=()=>true');
  async function switchTo(id) {
    const expression = '[...' + panel
      + '.querySelectorAll(".version-row")]'
      + '.find(x=>x.querySelector("span")?.textContent.startsWith('
      + JSON.stringify(id) + ')).querySelector("button").click()';
    await evaluate(expression);
    await until(evaluate, '(' + active + ') === '
      + JSON.stringify('Active: ' + id));
  }
  await switchTo(first);
  await switchTo(second);
  await evaluate('document.querySelector("[name=locations]").value="apps/browser/*";'
    + 'document.querySelector("[name=maxBytes]").value="1000000";'
    + 'document.querySelector("[name=maxObjects]").value="10";'
    + 'document.getElementById("grant-form").requestSubmit()');
  await until(evaluate,
    'document.getElementById("grants").querySelectorAll("article").length===1'
    + ' && document.getElementById("status").textContent.includes("Grant created")');
  const grantId = await evaluate('document.querySelector(".grant h3").textContent');
  assert.match(grantId, /^[a-f0-9]{32}$/);
  await click(evaluate, 'Issue user');
  await until(evaluate,
    'document.getElementById("status").textContent.includes("User issued")');
  assert.ok((await evaluate('document.getElementById("credential-output").value')).length > 20);
  await click(evaluate, 'Inspect');
  await until(evaluate,
    'document.getElementById("status").textContent.includes("apps/browser/*")');
  await evaluate('window.confirm=()=>true;window.prompt=()=>'
    + JSON.stringify(grantId));
  await click(evaluate, 'Revoke user');
  await until(evaluate,
    'document.getElementById("status").textContent.includes("User revoked")');
  await click(evaluate, 'Revoke grant');
  await until(evaluate,
    'document.getElementById("status").textContent.includes("Grant revoked")');
  await click(evaluate, 'Lock');
  assert.equal(await evaluate('document.getElementById("workspace").hidden'), true);
  assert.equal(await evaluate('document.getElementById("credential-output").value'), '');
  await evaluate('document.getElementById("password").value='
    + JSON.stringify(password)
    + ';document.getElementById("unlock-form").requestSubmit()');
  await until(evaluate, 'document.getElementById("workspace").hidden===false');
  assert.match(await evaluate('document.querySelector(".grant h3").textContent'),
    /revoked/);
  console.log('Chromium web admin: history, atomic rollback/forward, grants and encrypted vault passed');
  socket.close();
}
try {
  await test();
} finally {
  chrome.kill('SIGTERM');
}
