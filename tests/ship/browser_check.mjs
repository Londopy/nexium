// CI's browser check of `artifact wasm`: open a page in a headless Chrome
// (or any Chromium) and print the text of its #out once it is no longer
// "running", which the page sets when its checks are done. A page's
// module finishes after the load event, and WebAssembly compiles off the
// main thread, so this asks the browser (its DevTools protocol) rather than
// dumping the DOM at a guessed moment. Node.js 22 or later (WebSocket).
//
//     node tests/ship/browser_check.mjs <chrome> <url>
//
// Exits 0 when the text starts with RESULT.
import { spawn } from 'node:child_process';
import { mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

const [chrome, url] = process.argv.slice(2);
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const profile = mkdtempSync(join(tmpdir(), 'nx-browser-'));
const browser = spawn(chrome, ['--headless=new', '--no-sandbox', '--disable-gpu', '--no-first-run',
  `--user-data-dir=${profile}`, '--remote-debugging-port=0', 'about:blank'], { stdio: 'ignore' });

let text = '';
try {
  // the browser writes the port it chose into the profile; a cold runner's
  // first start can take well over the 15 seconds this once allowed
  let port = '';
  for (let i = 0; i < 600 && !port; i++) {
    await sleep(100);
    try { port = readFileSync(join(profile, 'DevToolsActivePort'), 'utf8').split('\n')[0].trim(); } catch { }
  }
  if (!port) throw new Error('the browser did not start');
  const tab = await (await fetch(`http://127.0.0.1:${port}/json/new?${encodeURIComponent(url)}`, { method: 'PUT' })).json();
  const ws = new WebSocket(tab.webSocketDebuggerUrl);
  await new Promise((resolve, reject) => { ws.onopen = resolve; ws.onerror = () => reject(new Error('no DevTools connection')); });
  const waiting = new Map();
  let next = 0;
  ws.onmessage = (e) => {
    const m = JSON.parse(e.data);
    if (waiting.has(m.id)) { waiting.get(m.id)(m); waiting.delete(m.id); }
  };
  const ask = (expression) => new Promise((resolve) => {
    const id = ++next;
    waiting.set(id, (m) => resolve(m.result && m.result.result ? m.result.result.value : undefined));
    ws.send(JSON.stringify({ id, method: 'Runtime.evaluate', params: { expression, returnByValue: true } }));
  });
  for (let i = 0; i < 150; i++) {
    await sleep(200);
    text = (await ask("document.getElementById('out') ? document.getElementById('out').textContent : ''")) || '';
    if (text && text !== 'running') break;
  }
  ws.close();
} catch (e) {
  text = `FAILED ${e.message}`;
} finally {
  browser.kill();
  await sleep(500);
  try { rmSync(profile, { recursive: true, force: true }); } catch { }
}
console.log(text || 'FAILED the page did not finish');
process.exit(text.startsWith('RESULT ') ? 0 : 1);
