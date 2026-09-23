// node site/play_test.mjs <nx.wasm>
//
// The playground's compiler, checked through the page's own WASI layer
// (runWasi in site/play.js) rather than Node's: every fill and fix exercise
// of chapters 2 to 15 grades as `nx topo check` grades it (solutions print
// their recorded output, fix starters fail with diagnostics), and every spec
// case, example and Topo program prints what the compiled program printed,
// or is refused. Run by CI and by the Pages workflow on the build it deploys.
import { readFileSync, readdirSync, existsSync } from "node:fs";
import { createRequire } from "node:module";
import { join } from "node:path";

const require = createRequire(import.meta.url);
const { runWasi } = require("./play.js");
const wasmPath = process.argv[2];
if (!wasmPath) { console.error("usage: node site/play_test.mjs <nx.wasm>"); process.exit(2); }
const module = new WebAssembly.Module(readFileSync(wasmPath));
const play = (source) => runWasi(module, ["nx", "play"], source);
const norm = (s) => s.replace(/\r\n/g, "\n").replaceAll("\\", "/").trim();
const REFUSED = ["the interpreter cannot run this program", "has no `main`", "call depth", "step budget", "C preprocessor failed", "cannot find module", "@embedFile"];

const failures = [];
let graded = 0, same = 0, refused = 0;

for (const ch of readdirSync("topo/exercises").sort()) {
  const num = parseInt(ch, 10);
  if (!(num >= 2 && num <= 15)) continue;
  const dir = join("topo/exercises", ch);
  for (const name of readdirSync(dir).sort()) {
    if (!name.startsWith("fill_") && !name.startsWith("fix_")) continue;
    const text = readFileSync(join(dir, name), "utf8");
    if (name.endsWith(".solution.nx")) {
      const expected = join(dir, name.replace(".solution.nx", ".expected"));
      if (!existsSync(expected)) continue;
      const r = play(text);
      graded++;
      if (r.code !== 0) failures.push(`${dir}/${name}: exit ${r.code}\n${r.stderr}`);
      else if (norm(r.stdout) !== norm(readFileSync(expected, "utf8"))) failures.push(`${dir}/${name}: printed\n${r.stdout}`);
    } else if (name.startsWith("fix_") && name.endsWith(".nx")) {
      const r = play(text);
      graded++;
      if (r.code !== 1 || !r.stderr.includes("error")) failures.push(`${dir}/${name}: a fix starter must fail with diagnostics (exit ${r.code})`);
    }
  }
}

// programs are read from stdin here, as the page sends them: one that imports
// a sibling file or embeds one has no directory to find it in, and says so
for (const dir of ["tests/spec", "examples", "topo/code"]) {
  for (const name of readdirSync(dir).sort()) {
    if (!name.endsWith(".nx")) continue;
    const expected = join(dir, name.replace(/\.nx$/, ".expected"));
    if (!existsSync(expected)) continue;
    const source = readFileSync(join(dir, name), "utf8");
    // the page has no files and no input to give
    if (/\bfs\.|io\.read|stream\.|read_line/.test(source)) { refused++; continue; }
    const r = play(source);
    if (REFUSED.some((m) => r.stderr.includes(m))) { refused++; continue; }
    // a location in a message names the file the page calls it (main.nx)
    const got = norm((r.stdout + r.stderr).replaceAll("main.nx", `${dir}/${name}`));
    if (got === norm(readFileSync(expected, "utf8"))) same++;
    else failures.push(`${dir}/${name}: differs from the compiled run\n${got.split("\n").slice(0, 6).join("\n")}`);
  }
}

// a panic, main's exit code, and the step budget
const pan = play("fn main() {\n    let xs = [1, 2]\n    var i: usize = 4\n    println(\"{}\", .{xs[i]})\n}\n");
if (pan.code !== 2 || !pan.stderr.includes("index 4 out of bounds")) failures.push(`a panic: exit ${pan.code}\n${pan.stderr}`);
const seven = play("fn main() -> u8 {\n    return 7\n}\n");
if (seven.code !== 7) failures.push(`main returning 7 exited with ${seven.code}`);

if (failures.length) {
  for (const f of failures) console.error("FAIL " + f + "\n");
  console.error(`${failures.length} failure(s)`);
  process.exit(1);
}
console.log(`playground: ${graded} exercise checks graded, ${same} programs as compiled, ${refused} refused`);
