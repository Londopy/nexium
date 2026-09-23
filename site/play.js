// The playground: the Nexium compiler, built as WebAssembly (play/nx.wasm),
// runs `nx play` in the page. The program goes in on stdin; the compiler
// checks it and runs `main` in its interpreter; stdout and stderr come back.
// Nothing is sent anywhere. The compiler is fetched on the first Run and
// runs in a worker, so a long program never freezes the page.
//
// `runWasi` is the whole WASI layer (the 23 calls nx.wasm imports), shared
// by the worker and by site/play_test.mjs, which checks every exercise
// through it in Node.

function runWasi(module, args, stdinText) {
  const enc = new TextEncoder();
  const stdin = enc.encode(stdinText);
  let stdinPos = 0;
  const out = [];
  const err = [];
  let memory = null;
  const view = () => new DataView(memory.buffer);
  const bytes = () => new Uint8Array(memory.buffer);
  const argv = args.map((a) => enc.encode(a + "\0"));
  // the interpreter runs on the engine's stack, which holds about 750 nested
  // calls in a browser; the limit leaves room for smaller worker stacks
  const envv = ["NX_OFFLINE=1", "NX_PLAY_DEPTH=400"].map((a) => enc.encode(a + "\0"));
  class Exit { constructor(code) { this.code = code; } }
  const EBADF = 8, ENOENT = 44, ESPIPE = 70, SUCCESS = 0;

  function putStrings(list, ptrs, buf) {
    const v = view(), b = bytes();
    let at = buf;
    list.forEach((s, i) => { v.setUint32(ptrs + i * 4, at, true); b.set(s, at); at += s.length; });
    return SUCCESS;
  }
  function sizes(list, countPtr, sizePtr) {
    const v = view();
    v.setUint32(countPtr, list.length, true);
    v.setUint32(sizePtr, list.reduce((n, s) => n + s.length, 0), true);
    return SUCCESS;
  }
  function iovecs(iovs, n) {
    const v = view(), list = [];
    for (let i = 0; i < n; i++) list.push([v.getUint32(iovs + i * 8, true), v.getUint32(iovs + i * 8 + 4, true)]);
    return list;
  }
  const nowNs = (id) => BigInt(Math.round((id === 0 ? Date.now() : performance.now()) * 1e6));

  const wasi = {
    args_get: (ptrs, buf) => putStrings(argv, ptrs, buf),
    args_sizes_get: (c, s) => sizes(argv, c, s),
    environ_get: (ptrs, buf) => putStrings(envv, ptrs, buf),
    environ_sizes_get: (c, s) => sizes(envv, c, s),
    clock_time_get: (id, _precision, outPtr) => { view().setBigUint64(outPtr, nowNs(id), true); return SUCCESS; },
    fd_close: () => SUCCESS,
    fd_fdstat_get: (fd, ptr) => {
      if (fd > 2) return EBADF;
      // not a terminal: no colours, no raw mode
      const v = view();
      v.setUint8(ptr, 0);
      v.setUint16(ptr + 2, 0, true);
      v.setBigUint64(ptr + 8, 0xFFFFFFFFFFFFFFFFn, true);
      v.setBigUint64(ptr + 16, 0xFFFFFFFFFFFFFFFFn, true);
      return SUCCESS;
    },
    fd_fdstat_set_flags: () => SUCCESS,
    fd_prestat_get: () => EBADF,          // no directories: files are not found
    fd_prestat_dir_name: () => EBADF,
    fd_read: (fd, iovs, n, nreadPtr) => {
      if (fd !== 0) return EBADF;
      let total = 0;
      for (const [ptr, len] of iovecs(iovs, n)) {
        const take = Math.min(len, stdin.length - stdinPos);
        bytes().set(stdin.subarray(stdinPos, stdinPos + take), ptr);
        stdinPos += take; total += take;
        if (take < len) break;
      }
      view().setUint32(nreadPtr, total, true);
      return SUCCESS;
    },
    fd_readdir: () => EBADF,
    fd_seek: () => ESPIPE,
    fd_write: (fd, iovs, n, nwrittenPtr) => {
      if (fd !== 1 && fd !== 2) return EBADF;
      let total = 0;
      for (const [ptr, len] of iovecs(iovs, n)) {
        (fd === 1 ? out : err).push(bytes().slice(ptr, ptr + len));
        total += len;
      }
      view().setUint32(nwrittenPtr, total, true);
      return SUCCESS;
    },
    path_create_directory: () => ENOENT,
    path_filestat_get: () => ENOENT,
    path_open: () => ENOENT,
    path_readlink: () => ENOENT,
    path_remove_directory: () => ENOENT,
    path_rename: () => ENOENT,
    path_unlink_file: () => ENOENT,
    poll_oneoff: (inPtr, outPtr, nsubs, neventsPtr) => {
      // time.sleep: a page cannot block, so the wait is spun, a second at most
      const v = view();
      let longest = 0n;
      for (let i = 0; i < nsubs; i++) {
        const sub = inPtr + i * 48;
        if (v.getUint8(sub + 8) === 0) { const t = v.getBigUint64(sub + 24, true); if (t > longest) longest = t; }
        const ev = outPtr + i * 32;
        v.setBigUint64(ev, v.getBigUint64(sub, true), true);
        v.setUint16(ev + 8, 0, true);
        v.setUint8(ev + 10, v.getUint8(sub + 8));
      }
      const until = performance.now() + Math.min(Number(longest) / 1e6, 1000);
      while (performance.now() < until) { /* spin */ }
      v.setUint32(neventsPtr, nsubs, true);
      return SUCCESS;
    },
    proc_exit: (code) => { throw new Exit(code); },
  };

  const instance = new WebAssembly.Instance(module, { wasi_snapshot_preview1: wasi });
  memory = instance.exports.memory;
  let code = 0;
  try {
    instance.exports._start();
  } catch (e) {
    if (e instanceof Exit) code = e.code;
    else if (e instanceof RangeError) {
      err.push(enc.encode("panic: the program ran out of the browser's stack; a recursion this deep needs a compiled run (nx run)\n"));
      code = 2;
    } else { err.push(enc.encode("the compiler stopped: " + e + "\n")); code = 3; }
  }
  const join = (parts) => {
    const n = parts.reduce((s, p) => s + p.length, 0), all = new Uint8Array(n);
    let at = 0;
    for (const p of parts) { all.set(p, at); at += p.length; }
    return new TextDecoder().decode(all);
  };
  return { code, stdout: join(out), stderr: join(err) };
}

if (typeof module !== "undefined" && module.exports) module.exports = { runWasi };

// ----- the page -------------------------------------------------------------

(function () {
  if (typeof document === "undefined") return;
  const script = document.currentScript;
  const wasmUrl = new URL(script.dataset.wasm, script.src).href;
  const TIMEOUT_MS = 20000;
  let worker = null;
  let pending = null;

  function startWorker() {
    const src = runWasi.toString() + `
      let mod = null;
      self.onmessage = async (e) => {
        try {
          if (!mod) {
            const r = await fetch(${JSON.stringify(wasmUrl)});
            if (!r.ok) throw new Error("could not fetch the compiler (" + r.status + ")");
            mod = await WebAssembly.compile(await r.arrayBuffer());
            self.postMessage({ loaded: true });
          }
          self.postMessage({ result: runWasi(mod, ["nx", "play"], e.data.source) });
        } catch (x) {
          self.postMessage({ result: { code: 3, stdout: "", stderr: String(x) + "\\n" } });
        }
      };`;
    worker = new Worker(URL.createObjectURL(new Blob([src], { type: "text/javascript" })));
    worker.onmessage = (e) => {
      if (!pending) return;
      if (e.data.loaded) { pending.onLoaded(); return; }
      const p = pending;
      pending = null;
      clearTimeout(p.timer);
      p.resolve(e.data.result);
    };
  }

  /// Run a program; the promise gives { code, stdout, stderr }.
  function play(source, onLoaded) {
    if (!worker) startWorker();
    return new Promise((resolve) => {
      const timer = setTimeout(() => {
        worker.terminate();
        worker = null;
        pending = null;
        resolve({ code: 3, stdout: "", stderr: "stopped after " + TIMEOUT_MS / 1000 + " seconds\n" });
      }, TIMEOUT_MS);
      pending = { resolve, timer, onLoaded: onLoaded || (() => {}) };
      worker.postMessage({ source });
    });
  }

  const norm = (s) => s.replace(/\r\n/g, "\n").trim();

  function el(tag, cls, text) {
    const e = document.createElement(tag);
    if (cls) e.className = cls;
    if (text !== undefined) e.textContent = text;
    return e;
  }

  /// An editable code block with its buttons and an output panel below it.
  function attach(pre, opts) {
    const code = pre.querySelector("code") || pre;
    const original = code.innerHTML;
    code.setAttribute("contenteditable", "plaintext-only");
    if (code.contentEditable !== "plaintext-only") code.setAttribute("contenteditable", "true");
    code.setAttribute("spellcheck", "false");
    // edited code is plain text: the highlighting was for the text as written
    code.addEventListener("input", () => code.classList.add("edited"));
    code.addEventListener("keydown", (e) => {
      if (e.key === "Tab") { e.preventDefault(); document.execCommand("insertText", false, "    "); }
      if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) { e.preventDefault(); run(opts.check); }
    });
    const bar = el("div", "play-bar");
    const outBox = el("div", "play-out");
    outBox.hidden = true;
    const runBtn = el("button", "play-run", "Run");
    bar.appendChild(runBtn);
    let checkBtn = null;
    if (opts.expected !== undefined) { checkBtn = el("button", "play-check", "Check"); bar.appendChild(checkBtn); }
    const resetBtn = el("button", "play-reset", "Reset");
    bar.appendChild(resetBtn);
    const status = el("span", "play-status");
    bar.appendChild(status);
    pre.after(bar);
    bar.after(outBox);

    async function run(grade) {
      runBtn.disabled = true;
      if (checkBtn) checkBtn.disabled = true;
      status.textContent = worker ? "running…" : "loading the compiler (once)…";
      const r = await play(code.innerText, () => { status.textContent = "running…"; });
      runBtn.disabled = false;
      if (checkBtn) checkBtn.disabled = false;
      outBox.hidden = false;
      outBox.replaceChildren();
      if (r.stdout) outBox.appendChild(el("pre", "play-stdout", r.stdout));
      if (r.stderr) outBox.appendChild(el("pre", "play-stderr", r.stderr));
      if (!r.stdout && !r.stderr) outBox.appendChild(el("pre", "play-stdout play-empty", "(no output)"));
      status.textContent = r.code === 0 ? "" : "exit " + r.code;
      if (grade && opts.expected !== undefined) {
        const ok = r.code === 0 && norm(r.stdout) === norm(opts.expected);
        const verdict = el("p", ok ? "play-verdict play-ok" : "play-verdict play-no");
        if (ok) {
          verdict.textContent = "✓ That's it: the output matches.";
          if (opts.done && !opts.done.checked) { opts.done.checked = true; opts.done.dispatchEvent(new Event("change")); }
        } else if (r.code !== 0) {
          verdict.textContent = "Not yet: the program has to run cleanly first.";
        } else {
          const want = norm(opts.expected).split("\n"), got = norm(r.stdout).split("\n");
          let i = 0;
          while (i < want.length && i < got.length && want[i] === got[i]) i++;
          verdict.textContent = "Not yet: line " + (i + 1) + " should be “" + (want[i] ?? "") + "” but is “" + (got[i] ?? "") + "”.";
        }
        outBox.appendChild(verdict);
      }
    }
    runBtn.addEventListener("click", () => run(false));
    if (checkBtn) checkBtn.addEventListener("click", () => run(true));
    resetBtn.addEventListener("click", () => { code.innerHTML = original; code.classList.remove("edited"); outBox.hidden = true; status.textContent = ""; });
  }

  /// A prediction: the reader's guess, then the real output.
  function attachGuess(ex, expected) {
    const box = el("div", "play-guess");
    const area = el("textarea");
    area.rows = Math.max(2, norm(expected).split("\n").length);
    area.placeholder = "what will it print?";
    const btn = el("button", "play-check", "Check my guess");
    const verdict = el("p", "play-verdict");
    box.append(area, btn, verdict);
    ex.querySelector(".included").after(box);
    btn.addEventListener("click", () => {
      const ok = norm(area.value) === norm(expected);
      verdict.className = "play-verdict " + (ok ? "play-ok" : "play-no");
      verdict.textContent = ok ? "✓ Exactly right." : "Not quite; the real output is below.";
      const details = ex.querySelector("details");
      if (details) details.open = true;
      const done = ex.querySelector("input[data-done]");
      if (ok && done && !done.checked) { done.checked = true; done.dispatchEvent(new Event("change")); }
    });
  }

  document.querySelectorAll(".exercise[data-kind]").forEach((ex) => {
    const kind = ex.dataset.kind;
    const pre = ex.querySelector(".included pre.code");
    const expected = ex.dataset.expected;
    const done = ex.querySelector("input[data-done]");
    if (!pre) return;
    if (kind === "predict" && expected !== undefined) { attachGuess(ex, expected); return; }
    if (kind === "fill" || kind === "fix") { attach(pre, { expected, done, check: true }); return; }
    if (pre.textContent.includes("fn main")) attach(pre, {});
  });
  // a quiz: a click on a choice is graded at once and opens the answer; the
  // quiz's box is ticked when every question has been answered right
  document.querySelectorAll(".quiz").forEach((quiz) => {
    const lists = [...quiz.querySelectorAll("ul[data-right]")];
    const solved = new Set();
    lists.forEach((ul, qi) => {
      const right = +ul.dataset.right;
      [...ul.children].forEach((li, i) => {
        li.tabIndex = 0;
        li.setAttribute("role", "button");
        const pick = () => {
          li.classList.add(i === right ? "q-right" : "q-wrong");
          if (i === right) solved.add(qi);
          let d = ul.nextElementSibling;
          while (d && d.tagName !== "DETAILS") d = d.nextElementSibling;
          if (d) d.open = true;
          const done = quiz.querySelector("input[data-done]");
          if (solved.size === lists.length && done && !done.checked) { done.checked = true; done.dispatchEvent(new Event("change")); }
        };
        li.addEventListener("click", pick);
        li.addEventListener("keydown", (e) => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); pick(); } });
      });
    });
  });
  document.querySelectorAll("pre.code.lang-nexium").forEach((pre) => {
    if (pre.closest(".exercise") || pre.closest("details")) return;
    if (pre.textContent.includes("fn main")) attach(pre, {});
  });
})();
