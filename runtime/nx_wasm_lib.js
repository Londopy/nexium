// The fixed part of every `artifact wasm` loader: the error classes, the WASI
// a library needs (a clock, randomness and the console; no files, arguments
// or environment), and the copying between JavaScript values and the
// module's memory. `nx ship` puts it at the top of <name>.js, followed by the
// library's own `load`.

export class NexiumError extends Error {
  constructor(errorName, code) {
    super(errorName);
    this.name = 'NexiumError';
    this.code = code;
    this.errorName = errorName;
  }
}

export class NexiumPanic extends NexiumError {
  constructor(errorName, code, message) {
    super(errorName, code);
    this.name = 'NexiumPanic';
    this.message = message;
  }
}

const nodeFs = typeof process === 'object' && process.versions && process.versions.node;

// the module's bytes: the file beside this one unless `source` names or holds another
async function nxModule(source, beside) {
  if (source instanceof WebAssembly.Module) return source;
  if (source instanceof ArrayBuffer || ArrayBuffer.isView(source)) return WebAssembly.compile(source);
  const url = new URL(source ?? beside, beside);
  if (nodeFs && url.protocol === 'file:') {
    const { readFile } = await import('node:fs/promises');
    return WebAssembly.compile(await readFile(url));
  }
  const r = await fetch(url);
  if (!r.ok) throw new Error(`cannot fetch ${url}: ${r.status}`);
  return WebAssembly.compile(await r.arrayBuffer());
}

// WASI preview 1 for a library: what it prints goes to the console a line at
// a time, and every call it has no use for answers ENOSYS
function nxWasi(state) {
  const SUCCESS = 0, EBADF = 8, ESPIPE = 70, ENOSYS = 52;
  const view = () => new DataView(state.memory.buffer);
  const decoders = [new TextDecoder(), new TextDecoder()];
  const pending = ['', ''];
  const print = [(s) => console.log(s), (s) => console.error(s)];
  const none = (count, size) => { const v = view(); v.setUint32(count, 0, true); v.setUint32(size, 0, true); return SUCCESS; };
  const calls = {
    args_get: () => SUCCESS,
    args_sizes_get: none,
    environ_get: () => SUCCESS,
    environ_sizes_get: none,
    clock_res_get: (_id, out) => { view().setBigUint64(out, 1000n, true); return SUCCESS; },
    clock_time_get: (id, _precision, out) => {
      const ms = id === 0 ? Date.now() : performance.now();
      view().setBigUint64(out, BigInt(Math.round(ms * 1e6)), true);
      return SUCCESS;
    },
    fd_write: (fd, iovs, n, written) => {
      if (fd !== 1 && fd !== 2) return EBADF;
      const v = view();
      let total = 0;
      for (let i = 0; i < n; i++) {
        const ptr = v.getUint32(iovs + i * 8, true), len = v.getUint32(iovs + i * 8 + 4, true);
        pending[fd - 1] += decoders[fd - 1].decode(new Uint8Array(state.memory.buffer, ptr, len), { stream: true });
        total += len;
      }
      const lines = pending[fd - 1].split('\n');
      pending[fd - 1] = lines.pop();
      for (const line of lines) print[fd - 1](line);
      v.setUint32(written, total, true);
      return SUCCESS;
    },
    // stdout and stderr are terminals, so the C library flushes them a line at a time
    fd_fdstat_get: (fd, ptr) => {
      if (fd > 2) return EBADF;
      const v = view();
      v.setUint8(ptr, 2);
      v.setUint16(ptr + 2, 0, true);
      v.setBigUint64(ptr + 8, 0xFFFFFFFFFFFFFFFFn & ~0x24n, true);
      v.setBigUint64(ptr + 16, 0n, true);
      return SUCCESS;
    },
    fd_close: () => SUCCESS,
    fd_seek: () => ESPIPE,
    fd_prestat_get: () => EBADF,
    fd_prestat_dir_name: () => EBADF,
    proc_exit: (code) => { throw new Error(`the WebAssembly library exited with code ${code}`); },
    random_get: (ptr, len) => {
      for (let at = 0; at < len; at += 65536) {
        crypto.getRandomValues(new Uint8Array(state.memory.buffer, ptr + at, Math.min(65536, len - at)));
      }
      return SUCCESS;
    },
    sched_yield: () => SUCCESS,
    // time.sleep: the wait is spun, as a page cannot block
    poll_oneoff: (inPtr, outPtr, nsubs, neventsPtr) => {
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
      const until = performance.now() + Number(longest) / 1e6;
      while (performance.now() < until) { /* spin */ }
      v.setUint32(neventsPtr, nsubs, true);
      return SUCCESS;
    },
  };
  return new Proxy(calls, { get: (t, k) => (k in t ? t[k] : () => ENOSYS) });
}

// The helpers a library's functions call with, over the instance's exports.
function nxBind(x, errorName, lastPanic) {
  const bytes = () => new Uint8Array(x.memory.buffer);
  const view = () => new DataView(x.memory.buffer);
  const utf8 = new TextEncoder();
  const text = new TextDecoder();
  const cstr = (ptr) => { const b = bytes(); let end = ptr; while (b[end] !== 0) end++; return text.decode(b.subarray(ptr, end)); };
  const alloc = (size) => {
    const ptr = x.malloc(Math.max(size, 1));
    if (!ptr) throw new RangeError('the WebAssembly library is out of memory');
    return ptr;
  };
  const wide = (Ctor) => Ctor === BigInt64Array || Ctor === BigUint64Array;
  // a slice argument, copied into the module's memory: [pointer, length, the array copied]
  const slice = (value, Ctor) => {
    let a;
    if (value instanceof Ctor) a = value;
    else if (typeof value === 'string' && Ctor === Uint8Array) a = utf8.encode(value);
    else a = wide(Ctor) ? Ctor.from(value, (e) => BigInt(e)) : Ctor.from(value);
    const ptr = alloc(a.byteLength);
    bytes().set(new Uint8Array(a.buffer, a.byteOffset, a.byteLength), ptr);
    return [ptr, a.length, a];
  };
  return {
    view,
    raise(code) {
      const name = cstr(errorName(code));
      if (name === 'Panic') throw new NexiumPanic(name, code, cstr(lastPanic()));
      throw new NexiumError(name, code);
    },
    // the memory one call takes, freed together when the call is over
    temps() {
      const held = [];
      return {
        alloc: (size) => { const ptr = alloc(size); held.push(ptr); return ptr; },
        slice: (value, Ctor) => { const s = slice(value, Ctor); held.push(s[0]); return s; },
        free: () => { for (const ptr of held) x.free(ptr); },
      };
    },
    // a `[]mut` argument after the call: what the function wrote goes back
    back(value, ptr, a, Ctor) {
      const got = new Ctor(bytes().slice(ptr, ptr + a.byteLength).buffer);
      if (value instanceof Ctor) value.set(got);
      else if (Array.isArray(value)) for (let i = 0; i < got.length; i++) value[i] = wide(Ctor) ? got[i] : Number(got[i]);
    },
  };
}
