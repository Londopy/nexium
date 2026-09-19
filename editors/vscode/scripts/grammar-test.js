// Tokenizes Nexium sources with VS Code's TextMate engine and checks that the
// scopes we care about show up. Run: node scripts/grammar-test.js ../../examples/tour.nx
const fs = require("fs");
const path = require("path");
const vsctm = require("vscode-textmate");
const oniguruma = require("vscode-oniguruma");

const wasmBin = fs.readFileSync(path.join(require.resolve("vscode-oniguruma"), "..", "onig.wasm")).buffer;
const onigLib = oniguruma.loadWASM(wasmBin).then(() => ({
  createOnigScanner: (patterns) => new oniguruma.OnigScanner(patterns),
  createOnigString: (s) => new oniguruma.OnigString(s),
}));

const registry = new vsctm.Registry({
  onigLib,
  loadGrammar: async (scope) => {
    if (scope !== "source.nexium") return null;
    const raw = fs.readFileSync(path.join(__dirname, "..", "syntaxes", "nexium.tmLanguage.json"), "utf8");
    return vsctm.parseRawGrammar(raw, "nexium.tmLanguage.json");
  },
});

async function main() {
  const files = process.argv.slice(2);
  const grammar = await registry.loadGrammar("source.nexium");
  const seen = new Map();
  let failures = 0;
  for (const file of files) {
    const text = fs.readFileSync(file, "utf8");
    let ruleStack = vsctm.INITIAL;
    const lines = text.split("\n");
    for (let i = 0; i < lines.length; i++) {
      const r = grammar.tokenizeLine(lines[i], ruleStack);
      for (const t of r.tokens) {
        const scope = t.scopes[t.scopes.length - 1];
        if (!seen.has(scope)) seen.set(scope, `${path.basename(file)}:${i + 1} "${lines[i].slice(t.startIndex, t.endIndex)}"`);
      }
      ruleStack = r.ruleStack;
    }
    // a line that starts a string must not leak it into the next line
    ruleStack = vsctm.INITIAL;
    for (let i = 0; i < lines.length; i++) {
      const r = grammar.tokenizeLine(lines[i], ruleStack);
      ruleStack = r.ruleStack;
      if (ruleStack.depth > 3 && !lines[i].includes("<<")) {
        // depth grows only inside multi-line constructs; strings never span lines
        const top = r.tokens[r.tokens.length - 1]?.scopes.join(" ") ?? "";
        if (top.includes("string.quoted")) {
          console.log(`unterminated string scope leaks past ${file}:${i + 1}`);
          failures++;
        }
      }
    }
  }
  const required = [
    "keyword.control.nexium",
    "keyword.other.nexium",
    "storage.type.function.nexium",
    "entity.name.function.nexium",
    "entity.name.type.nexium",
    "support.type.primitive.nexium",
    "string.quoted.double.nexium",
    "constant.other.placeholder.nexium",
    "constant.numeric.hex.nexium",
    "comment.line.double-slash.nexium",
    "comment.line.documentation.nexium",
    "entity.name.tag.effect.negative.nexium",
    "support.function.intrinsic.nexium",
    "support.function.builtin.nexium",
    "entity.name.tag.error.nexium",
    "variable.other.enummember.nexium",
    "meta.binary-pattern.nexium",
    "keyword.other.binary-modifier.nexium",
    "keyword.operator.pipe.nexium",
    "entity.name.label.nexium",
  ];
  for (const scope of required) {
    if (seen.has(scope)) {
      console.log(`ok   ${scope.padEnd(44)} ${seen.get(scope)}`);
    } else {
      console.log(`MISSING ${scope}`);
      failures++;
    }
  }
  console.log(`${seen.size} distinct scopes, ${failures} problem(s)`);
  process.exit(failures ? 1 : 0);
}
main().catch((e) => {
  console.error(e);
  process.exit(1);
});
