(() => {
  'use strict';

  // Native v3: POST-only /_skrynia/ to P1, mediated by registered P2 feedback handler.
  // No Program0/root token, no write_cap, no admin_cap, no public-write store mode.
  // Read: OP_APP_GET (7) with an explicitly configured public read-only cap.
  // Write: OP_INVOKE (3) carrying the actual P2 v1 input verbatim (numeric op:1,
  //        public invokeId, bounded entry/fragments). The trusted edge only
  //        frames the bearer; it never rewrites Theorem business JSON.
  //        P2 performs scoped CAS merge in its own appdata.
  const CFG = Object.assign(
    { endpoint: `${location.origin}/_skrynia/`, invokeIdHex: '', readCapHex: '', timeoutMs: 15000 },
    window.__THEOREM_LAB_FEEDBACK_V3__ || {},
  );
  const ENDPOINT = CFG.endpoint;
  const INVOKE_ID = hexToBytes32(CFG.invokeIdHex);
  const READ_CAP = CFG.readCapHex ? hexToBytes32(CFG.readCapHex) : null;
  const EMPTY_STATE = () => ({ version: 1, updatedAt: null, htmlFragments: {}, feedback: [] });

  const OP_INVOKE = 3;
  const OP_APP_GET = 7;
  const VERSION = 1;
  const STATUS_OK = 0;

  let writeChain = Promise.resolve();
  let active = false;
  let pointer = { x: 0, y: 0 };
  let anchorElement = null;

  injectStyles();
  const ui = buildUi();
  document.body.append(ui.button, ui.panel, ui.pointer);

  ui.button.addEventListener('click', startFeedback);
  ui.copy.addEventListener('click', copyFeedbackJson);
  ui.cancel.addEventListener('click', cancelFeedback);
  ui.save.addEventListener('click', saveFeedback);
  ui.text.addEventListener('keydown', event => {
    if ((event.metaKey || event.ctrlKey) && event.key === 'Enter') {
      event.preventDefault();
      saveFeedback();
    }
  });

  ui.pointer.addEventListener('pointerdown', event => {
    if (!active) return;
    event.preventDefault();
    ui.pointer.setPointerCapture(event.pointerId);
    movePointerFromEvent(event);
  });
  ui.pointer.addEventListener('pointermove', event => {
    if (!active || !ui.pointer.hasPointerCapture(event.pointerId)) return;
    event.preventDefault();
    movePointerFromEvent(event);
  });
  ui.pointer.addEventListener('pointerup', event => {
    if (ui.pointer.hasPointerCapture(event.pointerId)) ui.pointer.releasePointerCapture(event.pointerId);
    refreshAnchor();
  });
  ui.pointer.addEventListener('pointercancel', event => {
    if (ui.pointer.hasPointerCapture(event.pointerId)) ui.pointer.releasePointerCapture(event.pointerId);
  });

  function hexToBytes32(hex) {
    if (typeof hex !== 'string') return null;
    const h = hex.trim().toLowerCase().replace(/^0x/, '');
    if (!/^[0-9a-f]{64}$/.test(h)) return null;
    const out = new Uint8Array(32);
    for (let i = 0; i < 32; i++) out[i] = parseInt(h.slice(i * 2, i * 2 + 2), 16);
    return out;
  }

  function failClosed(reason) {
    const err = new Error(reason || 'Feedback service unavailable');
    err.name = 'FeedbackUnavailable';
    throw err;
  }

  function ensureConfigured() {
    if (!ENDPOINT || typeof ENDPOINT !== 'string') failClosed('Missing feedback endpoint');
    let url;
    try { url = new URL(ENDPOINT, location.origin); } catch { failClosed('Invalid feedback endpoint'); }
    if (url.origin !== location.origin) failClosed('Feedback endpoint must be same-origin');
    if (url.pathname !== '/_skrynia/' && url.pathname !== '/_skrynia') failClosed('Feedback endpoint must be /_skrynia/');
    if (!INVOKE_ID && !READ_CAP) failClosed('Missing feedback capability configuration');
  }

  function u16le(n) { return [n & 0xff, (n >> 8) & 0xff]; }
  function u32le(n) { return [n & 0xff, (n >> 8) & 0xff, (n >> 16) & 0xff, (n >>> 24) & 0xff]; }
  function concatBytes(parts) {
    let len = 0;
    for (const p of parts) len += p.length;
    const out = new Uint8Array(len);
    let o = 0;
    for (const p of parts) { out.set(p, o); o += p.length; }
    return out;
  }
  function encId(bytes32) {
    if (!(bytes32 instanceof Uint8Array) || bytes32.length !== 32) failClosed('Invalid capability id');
    return new Uint8Array([...u16le(32), ...bytes32]);
  }
  function encBlob(bytes) {
    const b = bytes instanceof Uint8Array ? bytes : new TextEncoder().encode(String(bytes));
    if (b.length > 0xffffffff) failClosed('Feedback payload too large');
    return concatBytes([new Uint8Array(u32le(b.length >>> 0)), b]);
  }

  function encodeInvoke(invokeId, inputBytes) {
    return concatBytes([new Uint8Array([VERSION, OP_INVOKE]), encId(invokeId), encBlob(inputBytes)]);
  }
  function encodeAppGet(readCap, keyBytes) {
    return concatBytes([new Uint8Array([VERSION, OP_APP_GET]), encId(readCap), encBlob(keyBytes)]);
  }

  function decodeP1Response(buf) {
    const b = buf instanceof Uint8Array ? buf : new Uint8Array(buf);
    if (b.length < 6) failClosed('Malformed feedback response');
    const version = b[0];
    const status = b[1];
    const bodyLen = (b[2] | (b[3] << 8) | (b[4] << 16) | (b[5] << 24)) >>> 0;
    if (version !== VERSION) failClosed('Unsupported feedback response version');
    if (6 + bodyLen > b.length) failClosed('Malformed feedback response body');
    const body = b.slice(6, 6 + bodyLen);
    if (status === STATUS_OK) return { ok: true, body };
    // STATUS_ERR: body[0] is error code when present
    const code = body.length > 0 ? body[0] : -1;
    const msg = `Feedback service error (${status}:${code})`;
    const err = new Error(msg);
    err.code = code;
    err.status = status;
    throw err;
  }

  async function postSkrynia(requestBytes) {
    ensureConfigured();
    const ctrl = typeof AbortController !== 'undefined' ? new AbortController() : null;
    const t = ctrl ? setTimeout(() => ctrl.abort(), CFG.timeoutMs) : null;
    let res;
    try {
      res = await fetch(ENDPOINT, {
        method: 'POST',
        headers: { 'Content-Type': 'application/octet-stream' },
        body: requestBytes,
        cache: 'no-store',
        credentials: 'same-origin',
        signal: ctrl ? ctrl.signal : undefined,
      });
    } catch (e) {
      if (e && e.name === 'AbortError') failClosed('Feedback request timed out');
      failClosed('Feedback endpoint unreachable');
    } finally {
      if (t) clearTimeout(t);
    }
    if (!res.ok) failClosed(`Feedback endpoint unavailable (${res.status})`);
    const ab = await res.arrayBuffer();
    return decodeP1Response(ab);
  }

  async function invokeP2(command) {
    if (!INVOKE_ID) failClosed('Missing feedback invoke capability');
    const input = new TextEncoder().encode(JSON.stringify(command));
    const out = await postSkrynia(encodeInvoke(INVOKE_ID, input));
    const text = new TextDecoder().decode(out.body);
    let parsed;
    try { parsed = JSON.parse(text); } catch { failClosed('Malformed feedback payload'); }
    if (!parsed || typeof parsed !== 'object') failClosed('Malformed feedback payload');
    if (parsed.ok === false) {
      const err = new Error(parsed.error || 'Feedback operation failed');
      err.code = parsed.code;
      throw err;
    }
    return parsed;
  }

  async function readLatest() {
    // Prefer OP_APP_GET for public read when a public read-only cap is explicitly configured.
    if (READ_CAP) {
      const key = new TextEncoder().encode('theorem-lab/feedback-v1');
      const out = await postSkrynia(encodeAppGet(READ_CAP, key));
      if (!out.body || out.body.length === 0) return { exists: false, state: EMPTY_STATE() };
      const text = new TextDecoder().decode(out.body);
      let state;
      try { state = JSON.parse(text); } catch { failClosed('Stored feedback has an unsupported format'); }
      validateState(state);
      return { exists: true, state };
    }
    // Fallback: mediated read via P2 invoke (no appdata cap exposed to browser).
    const parsed = await invokeP2({ op: 'read' });
    if (!parsed.exists) return { exists: false, state: EMPTY_STATE() };
    validateState(parsed.state);
    return { exists: true, state: parsed.state };
  }

  function validateState(state) {
    if (!state || state.version !== 1 || !state.htmlFragments || typeof state.htmlFragments !== 'object' || !Array.isArray(state.feedback)) {
      throw new Error('Stored feedback has an unsupported format');
    }
  }

  function injectStyles() {
    const style = document.createElement('style');
    style.dataset.skryniaFeedbackUi = 'true';
    style.textContent = `
      .tl-feedback-button {
        position: fixed; right: 20px; bottom: 20px; z-index: 10020;
        width: 52px; height: 52px; border-radius: 50%; border: 1px solid rgba(167,139,250,.5);
        display: grid; place-items: center; cursor: pointer;
        background: rgba(24,18,42,.96); color: var(--purple,#a78bfa);
        box-shadow: 0 14px 38px rgba(0,0,0,.38), inset 0 0 0 1px rgba(255,255,255,.035);
        font: 800 22px/1 sans-serif; backdrop-filter: blur(12px);
      }
      .tl-feedback-button:hover { transform: translateY(-1px); background: rgba(38,28,65,.98); }
      .tl-feedback-button[aria-expanded="true"] { border-color: rgba(103,232,249,.6); color: var(--cyan,#67e8f9); }
      .tl-feedback-panel {
        position: fixed; right: 20px; bottom: 84px; z-index: 10019;
        width: min(380px, calc(100vw - 32px)); padding: 14px;
        border: 1px solid rgba(167,139,250,.32); border-radius: 16px;
        background: rgba(18,14,32,.97); box-shadow: 0 20px 55px rgba(0,0,0,.48);
        backdrop-filter: blur(16px); display: none;
      }
      .tl-feedback-panel.open { display: block; }
      .tl-feedback-panel label { display: block; margin-bottom: 8px; color: var(--muted,#b5aec8); font: 700 11px/1.2 monospace; letter-spacing: .08em; text-transform: uppercase; }
      .tl-feedback-text { width: 100%; min-height: 96px; resize: vertical; box-sizing: border-box; border: 1px solid rgba(255,255,255,.1); outline: none; border-radius: 11px; padding: 10px 11px; background: rgba(0,0,0,.22); color: var(--text,#f5f1ff); font: inherit; line-height: 1.45; }
      .tl-feedback-text:focus { border-color: rgba(167,139,250,.48); }
      .tl-feedback-help { margin: 8px 1px 10px; color: var(--muted,#9e96b0); font-size: 11px; line-height: 1.4; }
      .tl-feedback-actions { display: flex; align-items: center; justify-content: flex-end; gap: 8px; flex-wrap: wrap; }
      .tl-feedback-actions button { border: 1px solid rgba(255,255,255,.12); border-radius: 9px; padding: 8px 11px; cursor: pointer; background: rgba(255,255,255,.045); color: var(--text,#f5f1ff); font: 700 12px/1 inherit; }
      .tl-feedback-actions .copy { border-color: rgba(103,232,249,.28); color: var(--cyan,#67e8f9); background: rgba(103,232,249,.06); }
      .tl-feedback-actions .save { border-color: rgba(167,139,250,.35); color: var(--purple,#a78bfa); background: rgba(167,139,250,.08); }
      .tl-feedback-actions button:disabled { opacity: .5; cursor: wait; }
      .tl-feedback-status { min-height: 16px; margin-right: auto; color: var(--muted,#9e96b0); font-size: 11px; }
      .tl-feedback-status.error { color: var(--red,#fca5a5); }
      .tl-feedback-status.ok { color: var(--green,#86efac); }
      .tl-feedback-pointer {
        position: absolute; z-index: 10018; width: 30px; height: 30px; border-radius: 50%;
        transform: translate(-50%,-50%); display: none; cursor: grab; touch-action: none;
        border: 2px solid white; background: rgba(124,58,237,.82);
        box-shadow: 0 0 0 5px rgba(167,139,250,.22), 0 6px 20px rgba(0,0,0,.45);
      }
      .tl-feedback-pointer::after { content: ''; position: absolute; left: 50%; top: 50%; width: 5px; height: 5px; border-radius: 50%; background: white; transform: translate(-50%,-50%); }
      .tl-feedback-pointer.active { display: block; }
      .tl-feedback-pointer:active { cursor: grabbing; }
      @media (max-width: 560px) {
        .tl-feedback-button { right: 14px; bottom: 14px; width: 50px; height: 50px; }
        .tl-feedback-panel { right: 14px; bottom: 76px; }
      }
    `;
    document.head.appendChild(style);
  }

  function buildUi() {
    const button = document.createElement('button');
    button.type = 'button';
    button.className = 'tl-feedback-button';
    button.dataset.skryniaFeedbackUi = 'true';
    button.setAttribute('aria-label', 'Add feedback');
    button.setAttribute('aria-expanded', 'false');
    button.textContent = '✎';

    const panel = document.createElement('div');
    panel.className = 'tl-feedback-panel';
    panel.dataset.skryniaFeedbackUi = 'true';
    panel.innerHTML = `
      <label for="tl-feedback-text">Comment at the pointer</label>
      <textarea id="tl-feedback-text" class="tl-feedback-text" placeholder="Add feedback…"></textarea>
      <div class="tl-feedback-help">Drag the purple pointer onto the exact thing you mean. Its page location and enclosing HTML will be saved with your comment.</div>
      <div class="tl-feedback-actions">
        <span class="tl-feedback-status" aria-live="polite"></span>
        <button class="copy" type="button">Copy JSON</button>
        <button class="cancel" type="button">Cancel</button>
        <button class="save" type="button">Save</button>
      </div>
    `;

    const pointerNode = document.createElement('div');
    pointerNode.className = 'tl-feedback-pointer';
    pointerNode.dataset.skryniaFeedbackUi = 'true';
    pointerNode.setAttribute('role', 'img');
    pointerNode.setAttribute('aria-label', 'Feedback pointer. Drag to reposition.');

    return {
      button,
      panel,
      pointer: pointerNode,
      text: panel.querySelector('.tl-feedback-text'),
      status: panel.querySelector('.tl-feedback-status'),
      copy: panel.querySelector('.copy'),
      save: panel.querySelector('.save'),
      cancel: panel.querySelector('.cancel'),
    };
  }

  function startFeedback() {
    if (!active) {
      active = true;
      pointer.x = window.scrollX + window.innerWidth / 2;
      pointer.y = window.scrollY + window.innerHeight / 2;
      positionPointer();
      ui.pointer.classList.add('active');
      ui.panel.classList.add('open');
      ui.button.setAttribute('aria-expanded', 'true');
      refreshAnchor();
    }
    ui.text.focus();
  }

  function cancelFeedback() {
    active = false;
    anchorElement = null;
    ui.pointer.classList.remove('active');
    ui.panel.classList.remove('open');
    ui.button.setAttribute('aria-expanded', 'false');
    ui.text.value = '';
    setStatus('');
  }

  function movePointerFromEvent(event) {
    pointer.x = clamp(event.clientX + window.scrollX, 0, documentWidth());
    pointer.y = clamp(event.clientY + window.scrollY, 0, documentHeight());
    positionPointer();
    refreshAnchor();
  }

  function positionPointer() {
    ui.pointer.style.left = `${pointer.x}px`;
    ui.pointer.style.top = `${pointer.y}px`;
  }

  function clamp(value, min, max) {
    return Math.min(Math.max(value, min), max);
  }

  function documentWidth() {
    return Math.max(document.documentElement.scrollWidth, document.body.scrollWidth, document.documentElement.clientWidth);
  }

  function documentHeight() {
    return Math.max(document.documentElement.scrollHeight, document.body.scrollHeight, document.documentElement.clientHeight);
  }

  function refreshAnchor() {
    const clientX = pointer.x - window.scrollX;
    const clientY = pointer.y - window.scrollY;
    if (clientX < 0 || clientY < 0 || clientX > window.innerWidth || clientY > window.innerHeight) return;
    const stack = document.elementsFromPoint(clientX, clientY);
    const hit = stack.find(element => !element.closest('[data-skrynia-feedback-ui="true"]'));
    if (hit) anchorElement = hit;
  }

  function enclosingContextElements() {
    refreshAnchor();
    const chain = [];
    const seen = new Set();
    let div = anchorElement && anchorElement.closest ? anchorElement.closest('div') : null;
    while (div) {
      if (!seen.has(div)) {
        chain.push(div);
        seen.add(div);
      }
      const parent = div.parentElement;
      div = parent && parent.closest ? parent.closest('div') : null;
    }
    if (!seen.has(document.body)) chain.push(document.body);
    return chain;
  }

  function cloneWithLiveState(element) {
    const clone = element.cloneNode(true);
    const originals = [element, ...element.querySelectorAll('*')];
    const clones = [clone, ...clone.querySelectorAll('*')];
    for (let i = 0; i < Math.min(originals.length, clones.length); i += 1) {
      const source = originals[i];
      const target = clones[i];
      if (source instanceof HTMLInputElement) {
        target.setAttribute('value', source.value);
        if (source.checked) target.setAttribute('checked', '');
        else target.removeAttribute('checked');
      } else if (source instanceof HTMLTextAreaElement) {
        target.textContent = source.value;
      } else if (source instanceof HTMLOptionElement) {
        if (source.selected) target.setAttribute('selected', '');
        else target.removeAttribute('selected');
      } else if (source instanceof HTMLDetailsElement) {
        if (source.open) target.setAttribute('open', '');
        else target.removeAttribute('open');
      }
    }
    if (clone.matches && clone.matches('[data-skrynia-feedback-ui="true"]')) return null;
    clone.querySelectorAll('[data-skrynia-feedback-ui="true"]').forEach(node => node.remove());
    return clone;
  }

  async function captureContext() {
    const fragments = {};
    const chain = [];
    for (const element of enclosingContextElements()) {
      const clone = cloneWithLiveState(element);
      if (!clone) continue;
      const html = clone.outerHTML;
      const ref = await sha256Ref(html);
      if (!(ref in fragments)) fragments[ref] = html;
      chain.push(ref);
    }
    return { fragments, chain };
  }

  async function sha256Ref(text) {
    if (!crypto || !crypto.subtle) throw new Error('This browser cannot hash feedback context');
    const digest = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(text));
    const hex = Array.from(new Uint8Array(digest), byte => byte.toString(16).padStart(2, '0')).join('');
    return `sha256:${hex}`;
  }

  function makeId() {
    if (crypto && typeof crypto.randomUUID === 'function') return crypto.randomUUID();
    return `${Date.now()}-${Math.random().toString(16).slice(2)}`;
  }

  async function readLatest() {
    // Prefer OP_APP_GET for public read when a public read-only cap is explicitly configured.
    if (READ_CAP) {
      const key = new TextEncoder().encode('theorem-lab/feedback-v1');
      const out = await postSkrynia(encodeAppGet(READ_CAP, key));
      if (!out.body || out.body.length === 0) return { exists: false, state: EMPTY_STATE() };
      const text = new TextDecoder().decode(out.body);
      let state;
      try { state = JSON.parse(text); } catch { failClosed('Stored feedback has an unsupported format'); }
      validateState(state);
      return { exists: true, state };
    }
    // No privileged write/read cap is ever embedded here: without a scoped
    // public read cap the native P2 contract has no read op, so fail closed.
    failClosed('Missing public feedback read capability');
  }

  function validateState(state) {
    if (!state || state.version !== 1 || !state.htmlFragments || typeof state.htmlFragments !== 'object' || !Array.isArray(state.feedback)) {
      throw new Error('Stored feedback has an unsupported format');
    }
  }

  async function copyFeedbackJson() {
    ui.copy.disabled = true;
    setStatus('Copying…');
    try {
      const latest = await readLatest();
      const text = JSON.stringify(latest.state, null, 2);
      await copyTextToClipboard(text);
      setStatus('Copied JSON', 'ok');
      setTimeout(() => {
        if (ui.status.textContent === 'Copied JSON') setStatus('');
      }, 1600);
    } catch (error) {
      setStatus(error.message || 'Copy failed', 'error');
    } finally {
      ui.copy.disabled = false;
    }
  }

  async function copyTextToClipboard(text) {
    if (navigator.clipboard && typeof navigator.clipboard.writeText === 'function') {
      await navigator.clipboard.writeText(text);
      return;
    }

    const fallback = document.createElement('textarea');
    fallback.dataset.skryniaFeedbackUi = 'true';
    fallback.value = text;
    fallback.setAttribute('readonly', '');
    fallback.style.position = 'fixed';
    fallback.style.left = '-9999px';
    fallback.style.top = '0';
    document.body.appendChild(fallback);
    fallback.select();
    fallback.setSelectionRange(0, fallback.value.length);
    const copied = document.execCommand('copy');
    fallback.remove();
    if (!copied) throw new Error('Clipboard copy failed');
  }

  function validRef(s) {
    return typeof s === 'string' && /^sha256:[0-9a-fA-F]{64}$/.test(s);
  }
  function toP2Entry(e) {
    return {
      id: String(e.id || '').slice(0, 128),
      comment: String(e.text || e.comment || '').slice(0, 4096),
      pageUrl: ('https://' + location.host + String(e.pagePath || '/')).slice(0, 2048),
      htmlChain: (Array.isArray(e.context && e.context.chain) ? e.context.chain : []).filter(validRef).slice(0, 16),
    };
  }
  function toP2Fragments(frags) {
    const out = {};
    for (const [k, v] of Object.entries(frags || {})) {
      if (validRef(k) && typeof v === 'string' && new TextEncoder().encode(v).length <= 16384 && Object.keys(out).length < 16) out[k] = v;
    }
    return out;
  }
  async function writeMerged(entry, fragments, retry = 2) {
    // Native P2 v1 input, constructed here in the browser and sent verbatim
    // through OP_INVOKE. No gateway business-level translation is required.
    const cmd = {
      v: 1,
      op: 1,
      invokeId: CFG.invokeIdHex.toLowerCase(),
      entry: toP2Entry(entry),
      fragments: toP2Fragments(fragments),
      updatedAt: String(entry.createdAt || '').slice(0, 64),
    };
    try {
      const parsed = await invokeP2(cmd);
      if (parsed.state) validateState(parsed.state);
      return parsed;
    } catch (err) {
      // Bounded retry on the real P2 CAS outcome (409/busy/conflict).
      const msg = String(err && err.message || '');
      const conflict = err && (err.code === 409 || /conflict|cas|precondition|busy/i.test(msg));
      if (conflict && retry > 0) return writeMerged(entry, fragments, retry - 1);
      throw err;
    }
  }

  async function saveFeedback() {
    const text = ui.text.value.trim();
    if (!text || !active) return;
    ui.save.disabled = true;
    ui.cancel.disabled = true;
    setStatus('Saving…');

    try {
      const captured = await captureContext();
      const entry = {
        id: makeId(),
        text,
        createdAt: new Date().toISOString(),
        pagePath: location.pathname,
        pointer: {
          documentX: pointer.x,
          documentY: pointer.y,
          documentWidth: documentWidth(),
          documentHeight: documentHeight(),
          viewportWidth: window.innerWidth,
          viewportHeight: window.innerHeight,
          scrollX: window.scrollX,
          scrollY: window.scrollY,
        },
        context: { chain: captured.chain },
      };

      writeChain = writeChain.catch(() => {}).then(() => writeMerged(entry, captured.fragments));
      await writeChain;
      setStatus('Saved', 'ok');
      ui.text.value = '';
      active = false;
      anchorElement = null;
      ui.pointer.classList.remove('active');
      ui.panel.classList.remove('open');
      ui.button.setAttribute('aria-expanded', 'false');
      setTimeout(() => setStatus(''), 1600);
    } catch (error) {
      setStatus(error.message || 'Save failed', 'error');
    } finally {
      ui.save.disabled = false;
      ui.cancel.disabled = false;
    }
  }

  function setStatus(text, kind = '') {
    ui.status.textContent = text;
    ui.status.className = `tl-feedback-status${kind ? ` ${kind}` : ''}`;
  }
})();