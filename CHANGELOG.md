# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
The file is validated in CI with [patchnotes](https://pypi.org/project/patchnotes/).
The line under each version header is the release's name, a place on a
mountain; [docs/release-names.md](docs/release-names.md) has the scheme.

## [Unreleased]

### Added

- `xs.swap(i, j)` on a mutable slice (and a `List`): the two elements
  exchanged, any element type, a panic on an index out of bounds. It is what
  the new collections move owned values with.
- `std.sort`: `by(T, xs, less)` (a heapsort: in place, O(n log n), not
  stable), `stable_by(T, xs, less)` and `by_key(T, K, xs, key)` (a merge sort
  of the positions, then each element moved into place along the
  permutation's cycles: no element copied, any type), `is_sorted`,
  `is_sorted_by`, and on ascending slices `lower_bound`, `upper_bound` and
  `binary_search`.
- `std.heap`: `Heap(T)`, a priority queue by a comparison
  (`heap.by(T, less)`): `push`, `pop` (the least first), `peek`, `len`.
- `std.set`: `Set(T)` over `Map(T, bool)`: `add` (true when new),
  `contains`, `remove`, `items`, and `union`, `intersection`, `difference`,
  `is_subset`.
- `std.deque`: `Deque(T)`, both ends in O(1) amortized (two Lists back to
  back): `push_front`, `push_back`, `pop_front`, `pop_back`, `get`, `first`,
  `last`.

## [1.3.0] - 2026-09-24

*Annapurna: North Face* — the face of the first ascent, the original line followed through: the toolchain grown up. `nx fix` makes the checker's own edits, `nx bench` measures `bench` blocks, `nx debug` stops gdb or lldb at `.nx` lines with formatters for the language's values, `--sanitize` brings AddressSanitizer and UBSan to any build, `if comptime` builds only the branch the target picks, the language server answers from the checker when the program checks, and a large program's debug build is a C file per module, compiled again only where it changed. The view rules are errors, trait impls are held to their traits, and the checker is split by responsibility. The seed is regenerated from the final sources.

### Changed

- A program that uses `std.thread` or `for parallel` builds for
  `wasm32-wasi`, which has no threads: starting one fails, so its work runs
  in place, as it already did wherever a thread could not be started, and a
  lock has no one else to wait for. (The playground's compiler needs it:
  its driver compiles C on threads, which in the page it never does.)
- The view rules V1 to V5 (specification 5.6 and 5.7) are errors, as 1.2
  said they would be: a warning in one release, an error in the next
  (docs/stability.md). No program in this repository had one left.
  `--strict` and `NX_STRICT=1` stay and make any warning an error; nothing
  warns now, and a deprecation will. The compile-fail cases that needed
  `--strict` fail without it.
- The view rules' advice names a fix that works. "Clone the view" and
  "clone it" could not: a view's clone is another view of the same
  storage. The errors say to take the view later, keep an owned value (a
  `String`, a `List`; `.clone()` of the value itself), move `s.clone()`
  instead of `s`, or return an owned value, as the case is; the
  specification says the same (5.6).
- The rules for methods are written down (decision 110): a method is
  visible wherever its type is, as a field is, and `pub` on it marks the
  documented interface; a type's `impl` blocks may sit in several modules.
  The compiler behaved so already.
- `nx doc` lists a public struct's or enum's `pub` methods under it, from
  every `impl` block of the type in any module, each with its signature,
  inferred effects and doc comment; for a generic type, the methods of an
  impl of one instance (`impl Pair(i32)`) are headed by it. Methods were
  missing from the page before, doc comments and all.
- The checker is no longer one file of 15,000 lines. `self/check.nx` keeps
  its state and core (types, definitions, loading, instances, the passes
  over the whole program, ownership, unification, ranges; 3,700 lines),
  and the rest of its methods sit in ten modules, each an `impl Checker`
  block: `check_stmts`, `check_views`, `check_exprs`, `check_fields`,
  `check_calls`, `check_match`, `check_builtins`, `check_closures`,
  `check_print` and `check_interp`. The code moved unchanged: the compiler
  before and after emits the same C and the same typed IR for every
  example, spec case, compile-fail case and itself.

### Added

- The package catalogue (`docs/packages.md`) lists nxtls, cryptography
  in pure Nexium, and "In the wild" (the README, the site) lists it and
  QNI, the Discord helper built on it. Both pass their tests with the
  compiler on main.
- `nx fix` makes the edits the checker offers for its errors (decision
  111): rule V4's `.clone()` where a value moves out from under a view of
  it, rule V5's `@escape(...)` around a value kept past its `using arena`
  block, and `_ = ` in front of a value nothing uses (not an error union,
  and not inside an `if` with no `else` whose value is wanted). Each edit
  is tried on the program in memory, in whichever of the program's own
  files it falls, and kept only when the checker then reports fewer
  errors and none it did not report before; `--check` reports without
  writing. Wrapping and saturating arithmetic are not edits it makes:
  which one a program means is the programmer's choice (decision 113). An error with an edit on offer ends with
  "note: `nx fix FILE` can try an edit for N of these". The loader takes
  stand-in texts for any of the program's files (`load_with_texts`).
- Incremental builds (decision 116): the debug build of a large program
  (about 200 KB of source or more; `NX_UNITS=1` or `0` decides for any)
  is a C file per module, each object kept under
  `<out-dir>/<name>.units/` by a hash of its C and its command line, so a
  build compiles again only the modules whose C changed, on up to eight
  threads, and links; `nx build` says how many it compiled
  (`1 of 36 C files compiled`). A one-line change to the compiler rebuilds
  it in 0.99 seconds on Linux (gcc) and 1.75 on Windows (zig cc), where
  its one C file took 7.8. The runtime's state is shared between the files
  (`NX_STATE`), one copy defined by the root module's.
- The language server answers from the checker when the program checks
  (decision 115): go to definition follows what each local, function,
  method and field resolved to, so `s.scale(2)` goes to the `scale` of
  `s`'s own type; hover shows a local's or field's type (`var total:
  i64`) or a function's signature and effects; rename edits the
  declaration and every use in each of the program's own files. Where the
  program does not check the syntactic answers stand
  (`self/lsp_index.nx`).
- Conditional compilation (decision 114): `if comptime C { } else { }`
  evaluates the whole condition while checking and checks and builds only
  the branch it picks, so the other may name functions and externs that
  exist only on another platform (`if comptime @target().0 == "windows"`).
  `@target()` is a value of the compile-time interpreter: `--target`'s
  triple when cross-compiling, else the machine compiling, in the runtime's
  words. `else if comptime` chains, and a value's type is the taken branch's.
- `#line` directives in the C of a debug or sanitized build, so a
  debugger steps through `.nx` lines, a sanitizer's report names them, and
  `break file.nx:12` works. Each function is followed by a directive giving
  the generated code after it back to the C file's own lines; `nx emit-c`,
  whose C is the seed's, has none.
- `nx debug FILE [-- args]`: the program built for debugging and run under
  gdb, or lldb on macOS (`NX_DEBUGGER` names another; a name with `lldb`
  in it takes lldb's flags), with formatters written beside the binary
  (`runtime/nx_gdb.py`, `runtime/nx_lldb.py`): a `String` or `[]u8` as
  text, a `List`, slice or array as its elements, an optional as its value
  or `null`, an error union as its value or the error's name, a struct's
  fields by their names, and a `Map` as its entries, a debug build naming
  each map type and keeping its key and value types for them. The CI job
  `debugger` stops gdb and lldb at a `.nx` line of `tests/debug/values.nx`
  and checks each value they show.
- `nx bench` and `bench "name" { ... }` blocks beside the tests. The file
  is built optimized (`safe` unless `--mode` says otherwise) and each block
  measured: the iterations calibrated to a 10 ms sample, which is the
  warmup, then 21 samples, the median time per iteration printed with the
  fastest and slowest (`bench  sum to 1000  187 ns/iter  (min 186 ns, max
  193 ns; 21 samples of 64398)`). The value of the block's last expression
  is kept (`nx_bench_keep`), so the work that makes it is not optimized
  away; an error union must be `try`'d. A panic or an error fails the
  benchmark and the others run on; a filter narrows the run and `--quick`
  takes 1 ms samples. `nx test` leaves benchmarks to `nx bench`. `bench`
  is a keyword only before a string, so it stays a name elsewhere; the
  tree-sitter grammar and the editor files know it.
- `--sanitize address,undefined` for `nx build`, `run` and `test`: the C
  compiler's AddressSanitizer and UBSan, with debug information, the first
  finding stopping the program. In `fast` mode they put back the overflow
  and bounds checks the compiler dropped. `zig cc` has no AddressSanitizer
  runtime, so `address` asks for gcc or clang (`NX_CC`) and says so before
  anything is built. zig's UBSan reports only in a build not optimised for
  speed (at `-O2` a finding is a bare trap, the program dying without a
  word), so through zig a sanitized `safe` or `fast` build is compiled at
  `-Og`, and it checks a fixed array's bounds too (`-fstrict-flex-arrays=3`:
  the array is the last member of its struct, which C otherwise takes for a
  flexible one). The CI job `sanitizers` runs every std module's tests with
  `nx test --sanitize address,undefined`.
- The playground: the Topo's exercises and every example on the site run in
  the page. The compiler is built as WebAssembly (`site/play_build.sh`: the
  C of the current sources for `wasm32-wasi`, 2.8 MB, under 1 MB
  compressed) and fetched on a reader's first *Run*; `site/play.js` runs it
  in a worker with its own WASI layer, so nothing is sent anywhere and a
  long program never freezes the page. An exercise's code is editable, with
  *Run*, *Check* (graded against the recorded output, as `nx topo check`
  grades it, ticking the exercise's box) and *Reset*; a prediction takes
  the reader's guess before showing the output; every other code block
  with a `main` gets *Run*. The runtime compiles as `NX_WASM` on WASI: no
  processes, sockets, terminal or `setjmp`, each failing as the operating
  system would refuse.
- `nx play [FILE]`: a whole program, from the file or from stdin, checked
  as `nx check` checks it and run from `main` by the interpreter, nothing
  compiled. It is the playground's command and works the same in a
  terminal; the exit code is `main`'s own, 1 after diagnostics or an error
  from `main`, 2 after a panic or a program the interpreter cannot run,
  which it says at the line of the program that led there.
- The interpreter runs closures (captures by copy and by reference), trait
  objects (`dyn`), binary construction (`<<...>> into buf`), `@bitCast`,
  ordering comparisons of byte slices, and slice builtins on `xs[..]`
  (`sort`, `fill`, `reverse`). The Topo's chapters 2 to 15 run in the page.
- The rest of the Topo course: sixteen new exercises for chapters 10, 11,
  17 to 21 and 23 (forty-one in all, from a calculator's `%` to a network
  handler tested without a network, a framebuffer, a package tag's version,
  an export through a caller's buffer and a neuron's gradient), and quizzes
  for chapters 10, 11, 17 and 19 to 22 (eleven in all).
- Quiz questions the compiler grades (decision 109): a question can carry
  its program and ask about it (`ask: effects NAME`, `ask: output`, `ask:
  check`), and a wrong answer is met with the compiler's own words (`says:`,
  from `nx explain` and the diagnostics). `nx topo verify` asks the compiler
  every such question and fails when a quiz has fallen behind; the harness
  runs it. On the site a click on a choice is graded at once and ticks the
  quiz's box when every question is right.
- The `play` suite of the harness runs every spec case, example and Topo
  program through `nx play` and holds it to the compiled output, and runs
  every fill and fix exercise of chapters 2 to 15; the CI job `playground`
  runs them through the WebAssembly build and the page's script.

### Fixed

- A socket's receive timeout never reached its reader: `TcpStream.reader()`
  read with no timeout whatever `set_timeout` said, so a client that
  connected and sent nothing held a `std.http` server (whose `serve_one`
  sets ten seconds) forever. The reader keeps the stream's timeout
  (`stream.Reader.from_socket_timeout`) and a read past it is
  `error.Timeout`.
- `std.http` sized its buffers from what the peer declared: a request
  announcing a huge `Content-Length` made the server allocate it at once
  and die out of memory before a byte of body arrived, a chunk size of 17
  or more hex digits overflowed and panicked, and a line that never ended
  grew without bound. A request body past `http.MAX_BODY` (16 MiB;
  `read_request_max` takes another) is `error.TooLarge`, answered 413 by
  `serve_one`; request, header and chunk-size lines past 64 KiB are too
  (`stream.Reader.read_line_max`); a chunk size past `usize` is
  `error.InvalidInput`; and a body buffer grows as its bytes arrive. Found
  reviewing QNI, a server built on `std.http`.
- `docs/std.md` showed only the last line of a multi-line doc comment
  (`join`: "replaces `dir`."); `scripts/std_docs.py` takes the whole one.
- `nx play` wrote a float in exponent form as Rust does (`1e-7`, `1e21`)
  where the compiled program writes C's (`1e-07`, `1e+21`), and computed
  `f32` arithmetic in `f64` precision; a compiled program wrote an `f32`
  as the `f64` it widens to (`0.1` as `0.10000000149011612`). An `f32` is
  now written as the shortest text that reads back as that `f32` (`0.3`),
  and the interpreter rounds every `f32` result to `f32` and writes floats
  with the compiled writer itself (spec `s6_float_format`, `s4_f32`; the
  play suite compares both).
- Rule R1 rejected a slice into a loop item's buffer over the caller's
  storage: `for s in xs { return s[..] }` with `xs: []String` a parameter,
  `return s.title` in `for s in self.sections` in a method, an `if let`
  over a field, a `match` binding over a borrowed parameter, nested loops.
  A binding that copies an element is now judged by where the element
  lives (its origins, traced through pointers and slices), so those are
  accepted, while `&s`, a slice of an array field of `s`, and a loop over
  the function's own `List` or an `own` parameter stay errors (a map's
  `for (k, v)` walks copies it frees, so a view of `v` stays one too).
  R1 also judged only one branch of a returned `if` and no arm of a
  `match`: `return match n { 0 => a[..], _ => "x" }` with `a` a local
  array returned a dangling slice, and so did an `else` whose `then` was
  safe; every branch is judged now (spec `s5_views_through_bindings`,
  compile-fail `view_into_loop_copy`, `view_into_binding_array`,
  `view_through_local_list`, `view_through_own_param`,
  `view_of_map_entry`, `view_in_match_arm`, `view_in_else_branch`).
  Found while writing QNI.
- `nx fmt` took a function's body brace after a return type that is a
  slice, an array or a `*mut` of a named type (`-> []T {`, `-> *mut Json
  {`) for a struct literal and wrote `-> []T{`; `std.lists`, `std.json`,
  `std.thread`, an example and two compiler files carried it. It looks
  back over `[]`, `[N]` and `mut` to the arrow now (fmt suite case).
- A `u128` past `i128`'s maximum was written as a negative number by every
  formatting path (`{}`, `{x}`, widths, `format`, `to_string()`, a
  `?u128`), and `0 -% 1` printed -1: the runtime wrote every integer as
  signed. Unsigned values are written by `nx_w_uint` now. The literals
  behind them failed too: 2^128-1 was read as "literal `-1`", and 2^127
  became 0 with no error, for an `i128` as well; a literal only a `u128`
  holds is read in full, one no type holds is an error that says so, and
  a literal past `i128`'s range is an error for `i128`. `parse_int` into
  an unsigned type reads `u128`'s whole range (`nx_parse_uint`). The
  interpreter holds integers as `i128`: `nx play` refuses a `u128` past
  its maximum instead of panicking with a false overflow, and its
  `parse_int` now fails as the compiled one does (`Overflow` past the
  target's range, where it returned `300` for `u8`). Spec `s4_u128`,
  `s11_parse_int_ranges`, compile-fail `literal_past_i128` and
  `literal_pattern_past_u64`. Found while writing QNI.
- An integer literal under `as` was typed `i64` first and then cast:
  `0xffffffffffffffff as i128` gave -1, `(0xffffffffffffffff as u128) *
  (0xffffffffffffffff as u128)` panicked with an overflow, and
  `0x1ffffffffffffffff as u128` failed in the C compiler. The operand of
  `as` now gives a literal (or `-` on one) its numeric target as its type
  (specification 4), so those are the literal's values, and `300 as u8`
  is a compile error as `let z: u8 = 300` is; it always panicked. A
  literal that does not fit the type it finally has is an error, the
  `i64` it defaults to included: `let x = 0xffffffffffffffff` printed -1,
  and says now how to give it a type. `-9223372036854775808` with no type
  given is one literal, where it panicked at run time (spec
  `s4_literal_types`, compile-fail `literal_default_i64`,
  `literal_cast_out_of_range`, `literal_cast_wider_than_target`). Found
  while writing QNI.
- A function over `dyn Trait` did not build when nothing in the program
  coerced a value to that trait (`fn g(d: dyn Area) -> i64 { return
  d.area() }`): the vtable type took its slots' types from a vtable
  instance and, with none, declared every slot `void (*)(nx_ctx*, void*)`,
  so the C compiler refused the call. The slots now come from the trait's
  declared signatures (`vt_slot`), and a trait impl's methods must match
  that declaration (decision 112, specification 8.3): the receiver, each
  parameter's type and `own`, the return type, `Self` read as the
  implementing type; an impl returning `i32` for a trait's `i64` was
  accepted and called through the trait with the wrong type. The errors
  name the method, what it has and what the trait declares (compile-fail
  `trait_impl_signature`). Found while writing QNI.
- A call through `dyn Trait` ignored `own` parameters: `d.put(s)` and
  `d.put(String.from("x"))` handed the value to the callee and still
  dropped it in the caller, a double free (0xC0000374 on Windows, no
  message); `s` was not marked moved, so reading it after the call
  compiled; and a `ref class` argument was not retained. The call now
  takes ownership as a direct call does (moves a local or a `.?`
  payload, passes a temporary without dropping it, retains a reference),
  and reading `s` afterwards is "use of `s` after it was moved" (spec
  `s8_dyn_own`, compile-fail `dyn_own_moves`). Found while writing QNI.
- Moving an owning payload out of an optional or error union straight
  into an `own` parameter (`take(x.?)`, `take(x orelse d)`, `take(try r)`,
  a method's or a generic function's `own` parameter too) handed the
  payload over but left the source set, so its scope-end drop freed it
  again: a double free, which on Windows ended the program with heap
  corruption and no message. A call now moves those forms as it moves a
  bare local, zeroing the source, as `let s = x.?` already did
  (`examples/optional_move.nx` gains the call forms; `nx leaks` finds
  nothing). Found while writing QNI.
- `panic(..)` standing for a value whose C type is a struct (a struct, an
  enum, a slice, `String`, `List`, `?T`, `!T`, a tuple), as a tail, in
  `return`, in a typed `let` or as a `match` arm, passed `nx check` and
  failed in the C compiler (`nx_Kind _t1 = 0;`), and a call to a
  `-> never` function in value position failed for every type, scalars
  too (its C function returns `void`). Both now stand for a zero of the
  expression's type after the call, as `unreachable` already did, and a
  call through a `fn(..) -> never` value does the same (spec
  `s10_panic_as_value`, and `s10_never_call_panics`, which takes the
  panicking path). Found while writing QNI.
- A `while` condition that made an owned temporary (`while
  short(format("n{}", .{k}))`, `while format(...).len < 4`, a `List`
  returned by a call) passed `nx check` and failed in the C compiler: the
  temporary's release was emitted after the loop, out of its C scope. The
  condition now has a scope of its own on every pass, released before the
  test, so a `continue`, a `break` and the last pass each free it once
  (spec `s6_while_condition_temporaries`, also run under `nx leaks`).
  Found while writing QNI.
- A method defined twice for one type, in two `impl` blocks of a module or
  in two modules, was accepted, and every call reached the first; it is an
  error at the second definition (compile-fail cases `duplicate_method`,
  `duplicate_method_generic`). A generic `impl(T) Pair(T)` conflicts with
  every instance of `Pair`; `impl Pair(i32)` and `impl Pair(f64)` may both
  define a method.
- A struct reached only through pointers, such as the `self` of a trait's
  methods in a library with no `main`, was declared in the generated C but
  never defined, and the C compiler refused it as an incomplete type. The
  fuzzer found it; the ship suite now builds such a library.
- `nx explain FILE f effect` on a function without the effect printed
  `fn f(...)  does not blocks`; it prints the absence as a signature
  bounds it, `fn f(...)  !blocks`.
- The interpreter (the REPL, `nx -e`, and now `nx play`) printed some
  programs differently from their compiled selves; running every recorded
  program through it found them all. Format specs were ignored (`{:.2}`,
  `{x}`, widths); an enum printed as `variant 0` and an error as `?`
  instead of their names; writes through a `[]mut T` argument or a local
  `[]mut` view of an array were lost; `errdefer` never ran; `lines()` kept
  an empty last line after a final newline; `@refCount` answered 1; and a
  `ref class` value was copied rather than shared, so an alias did not see
  a write through another. Each is fixed (spec cases `s6_format_widths`,
  `s6_mut_slices`, `s5_ref_class_shared`), and the REPL keeps its `ref
  class` objects shared across lines. A `[]u8` taken from a list of bytes
  printed as `?`; it prints as text, as compiled code writes it.
- `nx check` accepted an `export(c)` function that `nx build` then refused:
  which types may cross the C boundary (scalars, slices of scalars as
  parameters, `layout(c)` structs of scalars, no returned slice) was decided
  only when C was generated, so the editor and `nx play` said nothing. The
  checker decides it now, with the same words, and also refuses a parameter
  named `p_len` beside a slice `p`, which C would see twice (compile-fail
  cases `export_returns_slice`, `export_list`, `export_struct_layout`,
  `export_len_name`).
- An exported function with a parameter named `out`, `ctx`, `r` or `prev`
  did not compile: the C wrapper named its own things so. They are
  `_nx_`-prefixed now, and the result pointer of a status-returning export
  is `out` unless a parameter has the name, then `result` (spec case
  `s15_export_names`; the header says the same).
- `nx fmt` took `.*` and `.?` for openers and wrote nothing after them:
  `i.*+= 1`, `if n.*== 3`, `match p.*{`, `{ o.?}`. They are postfix
  operators, so what follows gets the space it gets after any operand
  (`i.* += 1`, `match p.* {`), and member access, indexing and calls stay
  tight (`p.*.x`, `xs.*[0]`). And a `*` before `(` was always taken for a
  pointer sigil, so `a * (b - c)` came out as `a *(b - c)`; after an
  operand it multiplies and keeps its spaces. The tree is reformatted.
- A format width did not pad a `String`, a boolean, or an enum or error
  name in a compiled program (`{>8}` padded numbers and `[]u8` only); it
  pads every value written as text now.


## [1.2.1] - 2026-09-22

*Annapurna: de Noyelle* — the expedition's liaison officer, whose whole job was permits and diplomacy: this patch is the roads in. `pip install nexium-lang` and `npm install nexium-lang` are real, the extension is on Open VSX, the package is on Chocolatey and submitted to winget, every release attaches Debian and RPM packages and signed provenance for each asset, a Nix flake and a dev container open the repository anywhere, `nx completions` and `nx man` come from the compiler, the site has an API, and the Topo is a course of twenty-five exercises graded by the compiler. The workflows are pinned to commits and their tokens are read-only outside the publishing jobs. The seed is regenerated from the final sources.

### Security

- Every GitHub Action the workflows use is pinned to a commit, with its
  version in a comment that Dependabot keeps current, and the Docker base
  images to a digest, so a moved tag cannot change what a workflow runs.
  Each workflow's token is read-only except in the one job that publishes
  (the release, the images, the numbers branch, the CLA signatures).
  `install.sh` checks a downloaded Zig against checksums it carries rather
  than ones fetched beside the archive. The VS Code extension's packaging
  tool moved to `@vscode/vsce` 3.9, whose `markdown-it` and `linkify-it`
  carry the fixes for their quadratic-time advisories (a build-time
  dependency; nothing shipped was affected).

### Fixed

- The Release workflow pushed to Chocolatey from inside the Windows build and
  published the extension from the release job, so a registry's refusal
  failed the build and no release was made (Chocolatey returns 403 for every
  version after a package's first until that one is approved). The
  registries are their own workflows now, run when a release finishes:
  `chocolatey.yml` and `extension.yml` beside `pypi.yml` and `npm.yml`, each
  failing alone and rerunnable by hand for a tag.
- Rule V3 reported a view into one field of a struct as stale when another
  field of the same struct grew (`for row in lv.rows { lv.origins.append(..) }`),
  found in Point of Origin's level compiler. An origin remembers the first
  field on its path and a change the field it touched; they meet only when
  one is the whole value or both are the same field (spec case
  `s5_views_by_field`).

### Added

- `nx completions bash|zsh|fish|powershell` and `nx man`, generated from
  one table of the commands and the options. The release archives carry
  `nx.1` and `completions/`, the install script and the Homebrew formula
  put them in place, the Debian, RPM and Nix packages install them.
- Debian and RPM packages (`nexium_<version>_<arch>.deb`, `.rpm`, amd64
  and arm64) attached to every release, built by nfpm and installed in
  the release job as their test; a Nix flake (`nix run
  github:Londopy/nexium`) built from the one C file, checked in CI; a dev
  container on the Docker image, so the repository opens in a Codespace
  with `nx` ready; mise through its `ubi` backend, documented.
- The site's API: `api/latest.json` (the current release and every
  asset's URL), `api/releases.json`, `api/std.json` (every module and
  public function with its doc), `api/topo.json`, `api/commands.json` and
  `api/index.json`, written by the site builder at each deploy
  (`docs/api.md`).
- A Pygments lexer for Nexium inside the wheel (`pip install nexium-lang`
  makes `nexium` a language Pygments knows: `pygmentize -l nexium`,
  Sphinx, MkDocs, Jupyter); the AUR package `nexium-bin`
  (`installers/aur/`, written at each release); a tldr page
  (`docs/tldr/nx.md`, submitted to tldr-pages).
- Signed provenance for every release asset (`gh attestation verify
  <file> --owner Londopy`), the OpenSSF Scorecard workflow and badge, the
  site's `security.txt`, `robots.txt` and `sitemap.xml`, `CITATION.cff`,
  and `.pre-commit-hooks.yaml` with `nx fmt` and `nx check` hooks.
- `pip install nexium-lang` is real: the wheels every release builds reach
  PyPI through trusted publishing (`.github/workflows/pypi.yml`, no token),
  when the Release workflow finishes or by hand for a tag. The npm packages
  have the same workflow (`npm.yml`, with provenance), a token for a
  package's first publish and npm's trusted publishing after. The VS Code
  extension is on Open VSX and the Chocolatey package in its moderation
  queue (`extension.yml`, `chocolatey.yml` publish a release's by hand),
  and `Londopy.Nexium` is submitted to winget. The install doc's "Where
  to get it" table links every registry; the README carries their badges.
- The Topo is a course: twenty-five exercises of five kinds (fill in the
  blank, fix the error, write it, make the tests pass, predict the output)
  and four quizzes at the end of chapters 2 to 16, graded by the compiler.
  `nx topo` runs it in the terminal (the route map with progress, `nx topo
  <chapter>` writing the exercises to work on, `check`, `hint`, `solution`,
  `quiz`, `reset`; progress kept beside the update cache, or where
  `NX_TOPO_PROGRESS` says), and the site shows the same exercises with the
  solutions folded away, a "done" box per exercise kept in the browser and
  the count on every chapter link. The harness runs every solution, every
  starter's failure and a whole chapter through `nx topo`. `nx fmt` spaces
  the hole of an exercise (`???`) like a word, so formatting a starter in
  the editor leaves it as written.

## [1.2.0] - 2026-09-21

*Annapurna: South Face* — the great wall, climbed by siege in 1970: memory safety without a garbage collector. The checker knows where every view points and enforces five rules over it: a view may not be returned inside a value or into an `own` parameter, stored past the storage it points into, read after its container grew or its value moved, and a value made inside `using arena` may not leave the block; `@escape(v)` is the copy that may. The rules are warnings in this release and errors in 1.3, and `--strict` makes them errors now. They found three bugs in the compiler on their first run (a struct literal evaluating out of written order, an arena leaked by an early `return`, two views held by luck), all fixed. The fuzzer runs mutants in debug mode, `nx` says when a newer release exists, and the Windows installer's welcome page carries the topographic contours. The seed is regenerated from the final sources.

### Added

- The view rules of 1.2 (`SPEC.md` 5.6 and 5.7): a view may not be stored
  past the storage it points into (V2), read after its container grew or
  was cleared (V3) or after its value moved away (V4), or returned inside
  a value (V1, extended from rule R1 to structs, closures and lists that
  hold views, and to `own` parameters), and a value made inside `using
  arena` may not be kept past the block (V5). The checker reports at the
  use that would read released storage, naming the storage and the moment
  it was released. They are warnings in this release and become errors in
  1.3; `--strict` (or `NX_STRICT=1`) makes them errors now.
- A warning channel: the checker's warnings print like errors with
  `warning:` in front, the language server reports them with warning
  severity, and the harness runs compile-fail cases marked `// STRICT`
  under `--strict`.
- `@escape(v)`: a copy of a value made by the allocator outside the
  innermost `using arena` block, the one way a value leaves the block.
- The fuzzer builds every tenth accepted mutant in debug mode (released
  storage filled with `0xDD`) and runs it: a signal or an abort is a
  finding (`nx run tests/fuzz.nx -- --run 10`, as CI does).
- Every `unsafe` block in `self/` and `std/` carries a reason comment.
- `nx` says once a day when a newer release exists, the way npm and Deno
  do: after a command has done its work, one line on stderr names the
  release and `nx upgrade`, and the REPL's banner names it too. The
  question to GitHub is asked at most once a day (kept in
  `~/.nexium/update-check`, `%LocalAppData%\Nexium\update-check` on
  Windows, or beside a portable copy) and never in CI or with
  `NX_OFFLINE=1`; `NX_NO_UPDATE_CHECK=1` silences it. The harness checks
  the notice, its once-a-day silence and the switch.

### Changed

- The Windows installer's welcome page shows the mark over the social
  preview's topographic contours (`scripts/make_wizard_images.py` draws
  them).
- The six translated READMEs (es, zh-CN, ja, ko, fr, de) caught up with
  the English one: the install roads (PowerShell, pip and npm, Docker,
  Homebrew and Scoop), the commands that arrived in 1.0.3 (`--watch`,
  `explain`, `audit --lock`, `layout`, `upgrade`, `install`), the `--cpu`
  flag, the status line, the languages table and the layout.
- The Windows installer's "More from Londopy" page lists statusmith first
  (Discord Rich Presence from the tray, with its SDK as a Nexium package)
  and fits its four rows on the page: shorter blurbs, a "GitHub" and a
  "Download" button each.

### Fixed

- A struct literal evaluated its initializers in declaration order while
  the checker had checked moves in the order written, so `Pair{ .first =
  name[..], .name = name }` with `name` declared first read a zeroed
  `name`. Initializers run in the order written (spec case
  `s6_literal_written_order`, decision 105).
- A `return`, `break` or `continue` out of a `using arena` block leaked
  the arena. It ends with the block's scope on every exit (the harness
  runs `s5_arena_escape` under `nx leaks`).
- `nx` with no arguments at a terminal read its padded argument list
  through a view into a list released at the end of an inner block, and
  the loader read a module's name after moving the string it was a view
  into. Both worked by luck; the new rules found them.

## [1.1.0] - 2026-09-21

*Annapurna: Dutch Rib* — the 1977 route on the north face that became the everyday line because it is safer than the Sickle: the language the compiler wanted. `?.` reads through an optional, `let (a, b) = pair` and `for (k, v) in m` take tuples apart, `derive(Clone)` gives a struct its deep copy (the compiler's own hand-written copies are gone), any value with `next(self: *mut Self) -> ?T` iterates, slices match by shape with `[first, rest..]` and `whole @ pattern`, guard facts flow through `while` and `else` and prove `!= 0`, `!= null`, `s.len >= n` and `i < s.len`, format arguments have names and a width from a value, and `@alignOf`, `@bitCast`, `@min`, `@max` and `@target()` arrive. Found on the way and fixed: `place.?` and `place orelse d` dropped the payload twice in a read, and a guard fact outlived the loop that changed its variable, a bounds check elided since 0.6.0. The seed is regenerated from the final sources.

### Added

- `?.` reads through an optional: `opt?.field` and `opt?.method(args)`
  are `null` when `opt` is and the member as an optional otherwise; the
  rest of a postfix chain applies to the payload (`a?.name.len orelse
  0`), an optional result is not wrapped twice (`a?.b?.c`), and a chain
  over a temporary owns and drops the payload. It is the `if let` it
  stands for in the typed IR, so the emitter and the interpreter learned
  nothing new; the formatter, the tree-sitter grammar, the spec and the
  Topo know the operator. The first item of 1.1.
- Tuple destructuring: `let (a, b) = pair`, `var (x, y) = f()`, and
  `for (k, v) in pairs` (an index may follow, `for (k, v), i in pairs`),
  with `_` skipping an element. Over an owned value the names own the
  elements; over a place they are views of it, as an `if let` binding is
  (decision 92). Each name is one `let` in the typed IR. Spec case
  `s5_tuple_destructuring`; the second item of 1.1.
- `derive(Clone)` gives a struct or enum `.clone()`, a deep copy field by
  field, once every field and payload clones (numbers, `String`, `List`,
  `Map`, slices, optionals, tuples, arrays, weak references, other `Clone`
  types, reference classes by retaining); a pointer, a trait object or a
  function value does not, and the error names the field. Tuples,
  optionals and arrays of clonable things clone without a derive, and
  `where T: Clone` bounds a type parameter. Closes the `KNOWN_ISSUES.md`
  entry: the compiler's hand-written `cv_clone` and `frames_clone` are
  gone (decision 93). Spec case `s8_derive_clone`; the third item of 1.1.
- Iterators: `for x in it { }` drives any value with a
  `next(self: *mut Self) -> ?T` method until it yields `null`, owning the
  iterator and each yielded value, with `break`, `continue` and labels
  as in any loop (it is the `while`/`if let` it stands for in the typed
  IR, decision 94). `for (k, v) in m { }` walks a map's entries. Spec
  case `s6_iterators`; the fourth item of 1.1.
- Slice patterns in `match`: `[]`, `[x]`, `[a, b]`, `[first, rest..]`,
  `[.., last]` on slices and arrays, the rest a `[]T` view of the middle,
  exhaustive by length (`[]` with `[x, rest..]` needs no `_`); and
  `whole @ pattern`, the value under a name while its parts match
  (decision 95). Spec case `s7_slice_patterns`; the fifth item of 1.1.
- Guard facts flow into the body of a `while` (at the top of every pass)
  and, negated, into the `else` of an `if` or a `while`; a guard also
  proves `y != 0` for `x / y`, `x != null` for `x.?`, `i < s.len` for
  `s[i]`, and `s.len >= n` for indexing and slicing within `n`, so fewer
  functions carry `panics` (decision 96). Closes the `KNOWN_ISSUES.md`
  entry on range facts. Spec case `s9_guard_proofs`; the sixth item of
  1.1.
- Named format arguments: `println("{name} is {age}", .{ .name = n, .age = a })`,
  `{name:spec}` for a spec, an argument written more than once, and a
  width from an integer argument, `{v:>w}` with `.w = 8` (decision 97).
  Format arguments are now evaluated once each, in order, before the
  first write. Spec case `s6_formatting` grew; the seventh item of 1.1.
- Intrinsics `@alignOf(T)`, `@bitCast(T, x)` (scalars of one size,
  through `memcpy`), `@min(a, b)` and `@max(a, b)` (which carry the range
  of their operands, so `@min(i, n - 1)` proves an index), and
  `@target()`, the `(os, arch, pointer_bits)` of the C build (decision
  98). Spec case `s12_intrinsics`; the eighth item of 1.1.

### Fixed

- A guard's fact about a `var` survived into a loop that changed the
  variable (`if i < 3 { while i < 6 { xs[i]; i += 1 } }` elided the bounds
  check on `xs[i]` and read past the array, with `nx effects` reporting no
  `panics`), since 0.6.0. Facts about mutable locals now end at an
  assignment, at a mutable borrow, and at the entry of any loop. Spec case
  `s9_facts_end_at_loops`.
- `place.?`, `place orelse d` and `try place` used where nothing takes the
  value (a print argument, a `.len`) dropped the payload twice: once as the
  read's temporary and once with the place. The read is now a view of the
  place's payload, and only an `orelse` default is a temporary of the
  read's own. Spec case `s6_optional_reads`.
- The Release and Bench workflows could not commit the package-manager
  manifests and the numbers to a protected `main` (both jobs of v1.0.3
  failed on the push, the release itself was fine). A refused push now
  puts the commit on a branch (`packaging/vx.y.z`, `numbers`) and the job
  says how to land it; the site reads the numbers page from that branch,
  the next Bench run measures against it, and the release steps in
  `CONTRIBUTING.md` name the landing.

## [1.0.3] - 2026-09-20

*Annapurna: Schatz* — the one who found the summit party in the crevasse the morning after: the long-hidden bugs, and then the roads in. An outside review of 1.0.1 read the code and found five bugs that had been there since their features shipped: a contained panic leaked what the call acquired, the lockfile pinned nothing, the Python wrapper dropped writes to a mutable slice, effect notes pointed at the function instead of the line, and `SECURITY.md` promised more than the specification did. All five are fixed here, the compile-time interpreter is freed from its 32-call limit, and the roadmap's quick wins that followed are in too: every way to install (a one-liner on each platform, pip and npm, Homebrew, Scoop, winget and Chocolatey, Docker, torrents), `nx upgrade`, `nx install`, `nx layout`, `nx explain`, the effects lockfile, code lenses with the panic proof, the REPL's line editor and one-liners, watch mode, the numbers page, and the first package from outside the tree.

### Added

- The Windows installers, the compiler's own and the ones `nx ship`
  writes, refuse to start while another installer or uninstaller of the
  same program is open and name it, close the program if it is running
  before replacing its files (Restart Manager; it is not started again),
  and keep a log of the run as `install.log` next to the program. The
  uninstaller takes `/LOG="path"` for a log of its own.
- `nx run FILE --watch` and `nx test FILE --watch` run again whenever a
  file of the program changes: the modules the checker loads (the
  program's own and its packages', not the standard library), polled
  twice a second by their modification times. `NX_WATCH_ROUNDS=N` stops
  after N reruns, which is how the harness checks it.
- The numbers page: `bench/` holds four programs (`fib`, `nbody`, `sieve`,
  `words`) written the same way in Nexium, C, Rust, Go and Python, and
  `bench/run.py` builds and times them, checks the answers agree, and
  writes the medians with the machine and the tool versions to JSON and
  to `docs/numbers.md`; the Bench workflow runs it on a GitHub runner
  weekly and at every tag, commits the numbers, and fails when Nexium
  got a quarter slower than the last run.
- `nx explain FILE f effect` prints why `f` has an effect as a tree: the
  reason the checker recorded in each function (an allocation, an
  overflow check, an explicit panic, with its position) and the calls
  that carry the effect in, each with the callee's own tree, down to the
  primitive that brings it.
- The REPL has a line editor at a terminal: the line colours as you
  type, Tab completes from the language server's completion at the
  cursor, the arrows and Home/End move, Up/Down walk the history, the
  usual control keys cut and clear, Ctrl-C drops the line. Raw terminal
  input comes from three builtins, `io.raw_mode(on)`, `io.read_key()` and
  `io.pending_input()`; the console is raw only while a line is typed.
  `NX_PLAIN=1` asks for the plain reader, `NO_COLOR` for no colours.
- The language server puts a code lens above every function with its
  panic proof, `cannot panic` or `panics: <why> at :<line>` from the
  witness the checker recorded, followed by the rest of its effects.
- The effects lockfile: `nx audit FILE --lock` writes `FILE.effects.lock`,
  every function's effects (the program's own modules and its packages,
  not the standard library), and `nx audit FILE --check` fails when a
  function gained an effect the lock does not name, noting what else
  moved. The shipped example's lock is committed and CI checks it.
- Portable mode: a file named `portable` beside the executable keeps
  Zig's global cache beside it (`cache/`), so a copy on a USB stick leaves
  nothing on the host, and `nx doctor` says so. `nx install [DIR]` puts
  the copy in the user's place (`%LocalAppData%\Programs\Nexium`, or
  `~/.nexium` with `bin/` and `share/`) with the zig, examples, std and
  docs beside it, and adds the directory to the user's PATH unless
  `NEXIUM_NO_MODIFY_PATH=1`: the portable zip installing itself.
- `nx upgrade` installs the latest release in place of the running
  executable: the archive for the machine, verified against the release's
  `SHA256SUMS.txt`, unpacked with `tar`, moved over `nx` (set aside as
  `nx.exe.old` on Windows until the next upgrade); `--check` only reports,
  and `nx doctor` now points at it when a newer release exists.
- statusmith's Discord Rich Presence SDK is the first package from
  outside the tree: `docs/discord.md` has the card's fields and the
  install line (`nx add discord_rpc --git
  https://github.com/Londopy/statusmith --tag sdk-v0.1.0 --dir nexium`),
  the Topo's packages chapter adds it with a program the suite runs when
  online, and the catalogue in `docs/packages.md` lists it.
- A git dependency may name the package's directory inside the
  repository: `sdk = { git = "...", tag = "v2.0.0", dir = "nexium" }`, or
  `nx add sdk --git URL --tag TAG --dir nexium`, for an SDK that lives
  beside the app it belongs to. The packages suite fetches one.
- The Windows installer detects an installed Nexium: the page after the
  welcome names its version and directory and offers the upgrade (a
  repair when it is the same version, a replacement when the installed
  one is newer), with the previous directory, components and tasks as
  the defaults, or the removal of the installed version, which runs its
  uninstaller and exits.
- `pip install nexium-lang` and `npm install -g nexium-lang`: every
  release carries the compiler as a wheel per platform (the binary under
  `nexium_lang/bin/`, a console script `nx`) and as npm packages
  (`nexium-lang` with `bin/nx.js`, `@nexium-lang/<os>-<cpu>` as optional
  dependencies holding the binaries), built by `scripts/pypi_npm.py` from
  the release archives, attached to the release, and uploaded to PyPI and
  npm when the `PYPI_TOKEN` and `NPM_TOKEN` secrets are set.
- `std.testing` has `expect_snapshot(name, actual)` and
  `expect_snapshot_in(dir, name, actual)`: `snapshot` in the `expect_`
  form, so a mismatch fails the test naming the file, the first differing
  line and how to accept the new output (`NX_UPDATE_SNAPSHOTS=1`), and a
  file that cannot be read or written fails it too.
- `nx layout FILE [Type...]` prints the C layout of a struct (every
  field's offset and size, the padding, the total, and the order by
  alignment that would shrink it, since the backend keeps declaration
  order) or an enum (the tag, where the payload starts, each variant's
  payload); without a name, every non-generic struct and enum of the file.
- `nx -e CODE` runs lines as the prompt would and `nx -p EXPR` prints
  the value of an expression on its own; the interpreter runs them, so
  nothing is compiled.
- The REPL has `:undo` (the last line taken back, bindings and all),
  `:save FILE` (the session as a program `nx run` runs; values printed at
  the prompt become `_ = ...`) and `:effects EXPR` (the effects of an
  expression, checked in a function of its own with the kept bindings as
  parameters). `:load FILE` was there already.
- `installers/install.ps1`, the Windows one-liner:
  `irm https://raw.githubusercontent.com/Londopy/nexium/main/installers/install.ps1 | iex`
  downloads the portable build for the machine, verifies it against the
  release's checksums, installs it under `%LocalAppData%\Programs\Nexium`
  on the user's PATH and downloads Zig beside it when no C compiler is
  found; `NEXIUM_VERSION`, `NEXIUM_HOME`, `NEXIUM_NO_MODIFY_PATH` and
  `NEXIUM_NO_ZIG` as in the shell script. A Chocolatey package
  (`installers/chocolatey/`, written by `scripts/packaging.py`) wraps the
  installer with its checksum; the Windows build job packs it, attaches
  the `.nupkg` to the release, and pushes it to chocolatey.org when
  `CHOCO_API_KEY` is set.
- `ghcr.io/londopy/nexium`: the compiler with its Zig toolchain, the
  standard library, the examples and the docs, on Debian (`latest`) and
  Alpine (`:alpine`), for amd64 and arm64, built from the release's seed
  at each tag (`docker/`, `.github/workflows/docker.yml`); the entry point
  is `nx`. CI builds both images and runs hello inside on every push.
- The Windows installer offers "Open Nexium REPL here" in a folder's
  right-click menu beside the console entry, and a "Nexium REPL" profile
  for Windows Terminal, written as a JSON fragment (per user or for all
  users with the install mode, removed at uninstall) and offered checked
  when Terminal is installed.
- `docs/packages.md` has the list of packages one can `nx add` today,
  what a repository needs to be one, and how to get a row; it is the
  registry until there is a registry.
- The release workflow publishes the VS Code extension to the Visual
  Studio Marketplace and to Open VSX when the `VSCE_PAT` and `OVSX_PAT`
  secrets are set, and says so when they are not; the extension is
  packaged with the release's version so each release publishes once.
- `nx doctor` says when a newer release exists: it asks the GitHub API
  through `curl` (three seconds at most) and prints the latest tag beside
  its own version; `NX_OFFLINE=1` skips the question, and no `curl` or no
  answer is reported rather than failed.
- Homebrew, Scoop and winget: the repository is its own Homebrew tap
  (`brew tap londopy/tap https://github.com/Londopy/nexium`, then
  `brew install londopy/tap/nexium`; the release build on Apple Silicon
  and Linux, the compiler built from its one C file elsewhere) and its own
  Scoop bucket (`bucket/nexium.json`, also installable by URL), and
  `installers/winget/` holds the manifests for `Londopy.Nexium`, ready
  for microsoft/winget-pkgs and usable with `winget install --manifest`
  today. `scripts/packaging.py` writes all three from a release's
  checksums, and the release workflow commits them to `main`.
- Releases attach a `.torrent` for every file, with its GitHub download
  as the web seed, and the notes list the magnet links with a QR code of
  each (`scripts/release_notes.py`; the release job installs `segno`).
- The install script builds `nx` from the one C file, `bootstrap/nx.c`,
  with the C compiler on the machine when no release is built for it (an
  x86-64 Mac, a BSD, a RISC-V board), when the download fails, or when
  `NEXIUM_FROM_SOURCE=1` asks for it; the one-C-file install is on the
  front page of the docs and in `docs/install.md`.


- `artifact cli { stack = "1G" }`: `main` runs on a thread reserving that
  much stack (`"64K"`, `"256M"`, `"1G"` or a byte count), so a recursion
  deeper than the platform's default completes; the reservation is address
  space, committed as the program reaches it, and a 32-bit process caps it
  at 256 MB. The compiler declares a gigabyte for itself, and compile-time
  evaluation may now nest 1000 calls (it stopped at 32 since the fuzzer
  found the unbounded case; each nested call costs about 165 KiB of the
  compiler's stack). A spec case recurses 300,000 deep at run time and 300
  deep at compile time.
- A Sublime Text build system, `editors/sublime/Nexium.sublime-build`:
  Ctrl+B runs the file, the variants test, check, build and print its
  effects, and a diagnostic's `--> file:line:col` line is clickable.
- The roadmap's "run from the editor, everywhere": the same Ctrl+B in
  every supported editor, a one-line `--format short` diagnostic for
  problem matchers and `--format json` for tools.

### Changed

- `SECURITY.md` states the promise precisely: deterministically
  memory-managed without a garbage collector, memory safety complete in
  1.2 when the view rules land, and a host is not left holding what a
  panicked export call acquired. Page titles say "Nexium language"
  (decision 91: `nexium-lang` for the language's own packages on
  registries where `nexium` is taken).
- `nx doctor` compiles and runs a one-line program before it says
  "everything works": the checker, the C emitter, the C compiler, the
  linker and the executable all have to answer. 1.0.1's doctor reported a
  working installation on a machine where the compiler could not compile
  a line, because it had only asked the C compiler for its version. An
  illegal instruction from the probe is named as such, with the fix. The
  harness runs the check.

### Fixed

- `nx doctor` and `nx upgrade` send `GITHUB_TOKEN` (or `GH_TOKEN`) to
  the GitHub API when the environment has one, so a CI runner is not
  rate-limited into "no answer"; the harness counts a silent API as a
  note, not a failure, and fetches the Topo's Discord example only on
  Windows, where the SDK's pipe lives today.
- A `catch` or `orelse` handler whose tail is a panic, on a value with a
  struct type (a `String`, say), emitted C that did not compile (`_t = 0`
  for a struct): the tail was typed as the value the block should have,
  by coercion, and the generator assigned its placeholder. Found by
  `expect_snapshot`'s own test; a spec case keeps it.
- On a macOS or Linux machine without Zig, `nx` takes `cc`, `gcc` or
  `clang` from the PATH instead of running a `zig` it assumed was there;
  `nx doctor` names the choice. A gcc-only Ubuntu could build `nx` from
  the one C file and then compile nothing. `-lpthread` is linked on Linux
  for glibc before 2.34. CI builds hello with zig removed from the PATH.
- The install script no longer stops with `line: parameter not set` at
  the end when `NEXIUM_NO_MODIFY_PATH=1` is set.

## [1.0.2] - 2026-09-20

*Annapurna: Terray* — the one who carried the frostbitten summiters down: the hotfix. The x86-64 binaries of 1.0.0 and 1.0.1 were built for the CPU of the machine that built them and crashed on any other; every build is for the architecture's baseline now, and the release refuses one that is not. With it, the fuzzer's two fixes, UTF-8 on the Windows console, and the day's documentation.

### Fixed

- A panic caught at an export boundary releases what the call acquired.
  The wrapper's `longjmp` skipped every drop between the panic and the
  boundary, so a library call that built a string and divided by zero
  leaked the string, and one that opened a file leaked the handle, on
  every call. Each export call now runs with a tracker: its allocator
  records live allocations (arena chunks included), and file, socket and
  lock handles register as they are acquired; the panic path releases them
  all before the status goes back. Threads started inside the call keep
  what they allocated, since it can escape through `shared_mutable`. The
  harness ships a library whose export opens a file and panics, calls it
  200 times (the runtime has 64 file slots) and checks the next open
  succeeds. The first half of the roadmap's 1.2, "a panic releases what
  it owned".
- `fs.remove_all` removes read-only files on Windows (every object in a
  git checkout is one), where `remove` refused them and a directory tree
  was left half deleted.
- `nexium.lock` pins dependencies. `nx fetch` wrote the commit each git
  dependency resolved to and never read it back, so a fresh clone got
  whatever the tag pointed at that day and the lock's promise was empty.
  `nx fetch` now checks a locked dependency out at the locked commit (a
  shallow fetch of that commit when the tag has moved), rewrites the lock
  only for dependencies it had to resolve, and refuses a locked commit the
  source no longer has with the command that resolves the tag again. The
  harness builds a git dependency, moves its tag, fetches from the lock on
  a bare tree and checks the old commit is what runs.
- The generated Python package took a list, a read-only buffer, an array
  of another item type or a strided view for a `[]mut T` parameter,
  copied it into a temporary and lost the writes; each is a `TypeError`
  now that says what to pass. A `[]mut T` parameter takes any writable,
  contiguous buffer of the matching item format (`bytearray`,
  `array.array`, a writable `memoryview`, a NumPy array) and writes
  through it; a read-only slice parameter takes any buffer by pointer
  when the format matches, otherwise by copy, and any sequence by copy.
  CI calls the shipped library with all of them.
- The `!effect` diagnostic's notes point at the recorded witness (the
  `append`, the `println`, the call) rather than at the header of the
  function that holds it; the README's "exact line" is true again. The
  compile-fail cases now pin the line and column of the notes, which is
  how the wrong position went unnoticed.

### Added

- `nx update [name...]`: resolve the tag of every dependency (or of the
  ones named) again and rewrite `nexium.lock`; the counterpart of
  `nx fetch`, which follows the lock.
- `--cpu baseline|native|<name>` and `NX_CPU`, the CPU a build without
  `--target` is for; `nx doctor` prints it. `os.arch()`, the architecture
  a program runs on.
- Nexium programs print UTF-8 on the Windows console: the generated
  `main` switches the console to code page 65001 for the program's life
  and restores it at exit, so `nx version` shows `Rébuffat` rather than
  `R├⌐buffat`.
- The roadmap's "tools only this language can have": `nx explain` for an
  effect's provenance, an effects lockfile for CI, proofs as editor code
  lenses, profiling by effect, record-and-replay tests, `nx layout`, the
  ownership trace, asking the checker for its facts, a binary-pattern
  debugger and workbench, hot reload, one-liners, REPL sessions as files,
  watch mode, snapshot tests, a Jupyter kernel and the playground, each
  pencilled into a minor.
- The roadmap names what 1.0 is not yet (memory safety, maturity,
  numbers, ecosystem) and where each is answered; the numbers page
  (benchmarks against C, Rust, Go and Python, published on the site)
  moves from 1.6 to 1.1; a second maintainer and a list of packages are
  ecosystem items; the *Always* section commits to the corpus growing
  with the language, the sanitizers and the fuzzer on every release, and
  a patch within a week of a fix.
- The README's Status section describes 1.0 (language-stable, early
  ecosystem, with the roadmap's ledger of what it is not yet) instead of
  the first implementation; the six translated READMEs are regenerated
  from the current English one, which had moved on to the self-hosted
  compiler, the site, the Topo, the editors and the 1.0 numbers while
  they still described `cargo install` and a Rust compiler.
- The `CI` workflow skips a push that changes only prose and pictures;
  the `Pages` workflow builds the site from them and validates the
  changelog, so a roadmap edit no longer runs the compiler on three
  platforms. `CONTRIBUTING.md` says what runs on a push.
- The roadmap's page zero: `KNOWN_ISSUES.md` is where every open bug
  waits with its fix, no theme starts over a bug a user can hit, and
  the four an outside review of 1.0.1 found (a contained panic leaks
  what it acquired, `nexium.lock` is never read, the Python wrapper
  drops writes through a list, effect notes miss their witness) are the
  1.0.2 patch, with the wording fixes for `SECURITY.md`, the README and
  the package names; 1.3 begins by splitting `check.nx`.
- The roadmap opens with the quick wins in order: twenty-nine items
  sorted by how soon each can ship (hours, a day or two, about a week),
  each landing as a patch or in 1.1 when done, ahead of the themes.
- The roadmap's niche roads: the PowerShell one-liner (`irm | iex`),
  the one-C-file install (`curl bootstrap/nx.c | cc`), torrents with web
  seeds and magnet links, IPFS pins, `nx install` from a portable copy,
  portable mode, MSIX with an `.appinstaller`, `gh release download` and
  the configuration-management modules, and a QR code.
- The roadmap's "every road in": the installation channels beyond the
  setup and the script, per platform (winget, Chocolatey, Scoop, MSI,
  MSIX; Homebrew, MacPorts, a notarized `.pkg`; apt and dnf repositories,
  PPA, COPR, OBS, AUR, Alpine, Nix, Guix, Snap; BSD ports), containers
  and dev containers, wheels and npm packages that carry the binary,
  version managers and `nxup`, the extension registries, and the same
  channels as targets of the `installer` artifact for Nexium programs.
- The roadmap's 1.5 gains 32-bit architectures: `i686` Windows and
  Linux, `armv7`, `riscv32` and `thumb`, with the list of places 64 bits
  are assumed today (runtime handles and sizes, 64-bit binary segments
  on a 32-bit word, the compile-time interpreter's `usize`, the C header
  import's `long` and `size_t`), the cross-compiled test run in CI, the
  compiler self-hosted as 32-bit, and the tiers.
- The roadmap's Topo as a course and the docs, more: exercises the
  compiler grades (fill in the blank, fix the error, write it), quizzes
  checked against `nx effects` and the recorded outputs, predict-the-
  output cards, code blocks that run in the page once the compiler runs
  in the browser, progress on the route map, more project chapters; std
  reference pages from `nx doc`, an error index, a cookbook, in-depth
  guides, a style guide, site search and docs per release.
- The roadmap's "installer grown up": a setup mutex against two
  wizards at once, upgrade/repair/remove of an installed version, closing
  a running `nx` before files are replaced, checks for disk space, a
  pending reboot and antivirus quarantine, the toolchain choice, the
  wizard in six languages, Authenticode signing, logs and rollback,
  silent-install parameters and an MSI, an ARM64 setup, and a *Check for
  updates* entry.
- The roadmap's clickable line: the REPL with an inspector window,
  `:plot` and `:watch`, the installer's editor page and `nx upgrade`,
  `nx topo` as an interactive tutorial runner, `std.tui`, the visual
  tools (size treemap, leak explorer, effect graph, ownership timeline),
  nexium-gui grown up, and the Hut, a native IDE in nexium-gui.

### Fixed

- The x86-64 release binaries of 1.0.0 and 1.0.1 crashed with an illegal
  instruction on machines without AVX-512 (`nx version` and `nx doctor`
  worked; anything that compiled did not): `zig cc` compiles for the CPU
  it runs on, and the release runner's had AVX-512. Every build without
  `--target` is now for the architecture's baseline (plain x86-64) unless
  `--cpu native` or `NX_CPU` asks for more; the bootstrap scripts do the
  same; the release workflow and CI refuse an x86-64 compiler that
  contains AVX instructions.
- A module name used as a value (`let x = math`, `var t = thread`) crashed
  the checker with an index out of bounds; it is the diagnostic "`math` is
  a module, not a value" now (found by the fuzzer in CI).
- A compile-time call chain that never bottoms out overflowed the
  compiler's stack (found by the fuzzer, which moved the call to `fib`
  into `fib`); the compile-time interpreter stops at 32 nested calls with
  a diagnostic. The limit is low because each nested call costs about
  165 KiB of the compiler's stack; `KNOWN_ISSUES.md` has the fix that
  raises it.
- The documentation site's menu no longer jumps back to the top on every
  page: it is its own scroll box, and a page load reset it, so reading
  the chapters in order meant scrolling the menu down again each time.
  The menu now keeps its position from page to page and brings the
  current page's entry into view when it is not.
- The links under "Files" in the release notes were relative, so GitHub
  resolved them under the tag page and every one was a 404. The notes
  script writes the download URL now, and the notes of every release
  since 0.2.0 were corrected in place.

## [1.0.1] - 2026-09-20

*Annapurna: Rébuffat* — the guide who roped the snow-blind party together on the descent: a patch that ties the loose ends after the summit. The fixes the tutorial found, the documentation site and the Topo, and support for eleven editors; the language is the same.

### Added

- Editor support beyond VS Code and Sublime Text, under `editors/`: Vim
  (syntax, indent, `:NxRun` and friends, `:make`), Neovim (a plugin that
  registers the tree-sitter parser, its queries and `nx lsp` with the
  built-in client), Helix (language config and queries in Helix's scopes,
  with indents and text objects), Zed (an extension with grammar, outline
  and the language server), Emacs (`nexium-mode` with Eglot and lsp-mode
  registration), Kate and the other KSyntaxHighlighting editors, Notepad++
  (a User Defined Language), nano, and a guide for JetBrains IDEs
  (TextMate bundle plus LSP4IJ). CI loads or compiles every one of them.
- The documentation site, [londopy.github.io/nexium](https://londopy.github.io/nexium/),
  built by `site/build.nx`, a Nexium program that renders the Markdown in
  the repository (the docs, the project files, the tutorial) with the
  language's own token rules for highlighting, and published by a Pages
  workflow on every push. The test harness builds it.
- The Topo, `topo/`: a 23-chapter tutorial from installing the compiler
  to a neural network, a GUI, a network service and a library shipped to
  C, Python and Node. Every program in it (`topo/code/`) is run by the
  harness against its recorded output, and the diagnostics it shows are
  recorded too. Writing it found the bugs below.
- The documentation site on a phone: one column with the navigation
  behind a Menu button, tables that scroll sideways instead of widening
  the page, grids and code blocks that stay inside the screen, larger
  touch targets, and a theme colour for the browser chrome. Every page
  is checked at 375 pixels wide.
- `assets/social-preview.svg` and its 1280x640 PNG, the card GitHub and
  the chat apps show for a link to the repository (uploaded under the
  repository's settings); every page of the documentation site carries
  Open Graph tags pointing at it.
- The roadmap's 1.7, the seam both ways: Python (strings, NumPy without
  copies, dataclasses and exception classes from the Nexium types, `ref
  class` handles, callbacks, a CPython extension wheel, Python from
  Nexium), Rust (error sets as enums, strings and handles, `no_std`
  crates, source crates, Rust crates from Nexium), C++, Go, Java and
  Kotlin, C#, Ruby, Lua, Swift and the browser as `artifact` kinds, a
  versioned ABI document, buffers that leave, `nx ship --abi-check`, and
  `@cImport` accepting more of C. Release name Annapurna II; Gangapurna
  moves to 1.8.
- The roadmap's 1.2, memory safety without a garbage collector: the view
  rules V1 to V5 (returning, storing, growth, moving, arenas), `unsafe`
  for what the checker cannot prove, what stays (moves, scope-exit
  destruction, reference counting, `weak`, leak detection) and how the
  rules roll out under the stability policy, which now says that
  undefined behaviour may become an error in a minor. The later themes
  move down one number.

### Fixed

- A function checked on demand while a constant was evaluated at compile
  time could name a global before its type was resolved, and the checker
  crashed on the unresolved type (found by the fuzzer on its second run in
  CI). A global's type now resolves on first use; the program gets the
  diagnostic for the compile-time global access instead.
- A bare `break`, `continue` or `return` ends a `match` arm before the
  comma (`_ => break,` parsed the comma as an expression). A one-element
  array literal can be indexed and sliced (`[x][..]`, `[x][0]`): only a
  `[` that opens `[]T` or `[N]T` with a type after it starts an array
  type in expression position.
- `Set!T` widens into `!T` where `!T` is expected (an initializer, an
  argument, a return); `!T` never narrows into a named set. Type names
  now spell a named set (`Parse!i32`), so the diagnostic no longer reads
  "expected `!i32` but found `!i32`".
- The `!effect` diagnostic's first note points at the call that brings
  the effect in, not at the function's header.
- Two dangling views, found by running the tutorial's programs on Linux
  and the compiler under AddressSanitizer: `std.http.parse_url` kept the
  host as a view of a value that was released when the `if let` ended,
  so every URL with a port could fail with `NotFound` where the freed
  memory was reused; and the checker read a syntax node through a
  pointer after adding nodes to the tree (a binary pattern with a
  computed size). Both are the case the roadmap's 1.2 makes an error; a
  CI job now builds the compiler with the sanitizers, checks every source
  with it, and runs every spec case and tutorial program built with them.
- `for k in m` over a `Map` emitted C that did not compile: the keys are
  collected into an owned `List` the loop walks and releases, the same
  as `for k in m.keys()`. `m[key]` crashed the emitter; it is the lookup
  `m.get(key)` is. Both have a spec case now.
- `@weak(x)` did not parse (`weak` is a keyword); a builtin may be spelled
  with one. A weak reference created in a struct literal was retained a
  second time on its way into the field and its storage never freed; and
  an object whose fields held a weak reference back to it could be freed
  while its own fields were still being released. A `ref class` now holds
  its storage until its fields are gone; both cases are spec cases.

## [1.0.0] - 2026-09-20

*Annapurna: Summit* — the top of the mountain the project has been on since 0.1: the language stops changing under people's feet, the compiler is written in itself, and nothing but Nexium, one C file and a runtime header is left.

### Removed

- The first compiler, written in Rust (`bootstrap/rust/`, `Cargo.toml`,
  `Cargo.lock`), and with it the last Rust toolchain use in CI: the `lint`
  job now runs `nx fmt --check` on the tree and `nx check` on the
  compiler, the harness and the fuzzer. The repository is Nexium, one
  generated C file, and the runtime header (decision 90). The git history
  keeps the crate.

### Changed

- The architecture tour (`docs/architecture.md`) names the files under
  `self/`; the specification's status table and the roadmap record that
  the port is complete.

### Fixed

- The fuzzer wrote its cases under `nx-out/fuzz`, the path of its own
  executable on Linux and macOS, so CI's fuzz job could not start; it
  works under `nx-out/fuzzing`.

## [0.9.0] - 2026-09-20

*Annapurna: Summit Ridge* — the last ridge, nothing left but walking up: every tool in Nexium, the shipped `nx` the Nexium one, a harness and a fuzzer in Nexium, the stability policy and the tiers.

### Added

- `nx fmt` in Nexium (`self/fmt.nx`, a command of `self/nx.nx`): the
  formatter, byte for byte the same as the first one on every source in
  the tree with its layout disturbed, which `cargo test` checks while the
  Rust one exists. The lexer keeps `//` comments as tokens on request.
- `nx effects`, `audit`, `refcounts`, `leaks`, `size`, `version` and
  `doctor` in Nexium (`self/tools.nx`, `self/size.nx`): the reports over
  the checked program and the section-table reader for ELF64 and COFF
  binaries, matching the frozen Rust commands on every example except
  where the Nexium compiler does better (`dyn` and `fn` types printed as
  written, recursive functions showing `unbounded_stack`).
- `nx doc` in Nexium (`self/doc.nx`): the same page as the first
  generator on every source, apart from a function's effects coming from
  its own module and generic functions saying so instead of showing one
  instantiation. The parser keeps each `///` run's text (`Tree.docs`).
- Packages in Nexium (`self/manifest.nx`): the `nexium.toml` subset,
  `nx init`, `nx add`, `nx fetch` and the lock file; the loader resolves
  `import dep` and `import dep.module` through the manifest and scopes a
  package's own imports to its `src`.
- `nx ship` in Nexium (`self/ship.nx`, `self/ship_node.nx`,
  `self/installer.nx`): the C header, shared and static libraries, the
  Python package and wheel, the Rust crate, the npm package and the
  installers, from the export records `cgen.nx` writes as it emits each
  wrapper. The header and typings carry each function's `///` comment.
- `nx lsp` in Nexium (`self/lsp.nx`): the language server over stdio,
  with diagnostics, hover, go to definition, completion and rename, and
  the editor-module loader and IDE queries behind it; `cargo test` drives
  it the way an editor does.
- `nx repl` in Nexium (`self/repl.nx`): the interactive session on the
  compile-time interpreter, which in REPL mode may print, read the
  console, touch files, the clock and randomness (`check.nx` `repl_mode`,
  `run_repl`, values rendered by their types). `nx` alone at a terminal
  opens it. The last tool leaves the Rust crate.
- `io.is_terminal(h)`, `os.set_env(name, value)` and `os.exe_path()`; with
  the last, `nx` finds a Zig bundled next to it (`<dir>/zig/zig` or
  `<dir>/../zig/zig`), as the Rust driver did. The driver gives
  each build its own Zig cache under the output directory when
  `ZIG_LOCAL_CACHE_DIR` is not set, as the Rust driver did, since Zig's
  cache is not safe against several `zig cc` starting at once on Windows.
- The seed `bootstrap/nx.c` is regenerated when `self/` needs a builtin
  the seed lacks, not only at a release.
- The test harness is a Nexium program, `tests/run.nx`: it builds the
  compiler from the seed (when the sources are newer than it), checks the
  fixed point, and runs the examples, the spec cases, the compile-fail
  cases, `fmt --check` on the tree, the standard library's tests, the GUI,
  packages, `ship`, the installer, the REPL and the language server, the
  cases on several threads. `cargo test` and the Rust harness retire; CI's
  test jobs have no Rust toolchain. On Windows a program run through
  `process` may be named with forward slashes.
- The stability policy, `docs/stability.md`: what a patch, a minor and a
  major may change from 1.0 on, the deprecation cycle, what is not
  covered, and the C toolchain policy. `nx fix FILE...` applies the
  deprecations the compiler can migrate mechanically; there are none yet.
- The platform tiers, `docs/platforms.md`: tier 1 is built, tested and
  released (x86_64 Linux and Windows, Apple Silicon); tier 2 is built and
  released, not tested (aarch64 Linux, Windows on ARM), cross-compiled
  from the same C in the release workflow and picked by the install
  script on an ARM Linux; tier 3 is whatever Zig targets.
- Fuzzing, `tests/fuzz.nx`: mutated corpus sources through the front end
  and random bytes through the binary pattern engine (`tests/fuzz_patterns.nx`);
  a crash, a signal or a hang is a finding, saved for the report. CI runs
  it on every push with a fresh seed.
- Releases ship the compiler written in Nexium: the release workflow
  builds it from the C seed on each platform with Zig alone (`cc` on
  macOS), checks the tag against `self/nx.nx`, and packages that binary
  in the archives and the Windows installer. Building from source is
  `sh bootstrap/build.sh`; no Rust is needed.

### Fixed

- `if let v = opt` over a place (a local, a field, an element) bound a
  bitwise copy of the payload that could then be moved on, so the
  optional and the binding both freed it. The binding is now a view of
  the payload, like a loop variable: moving out of it is an error with a
  `.clone()` / `.?` hint, while a binding over an owned temporary (a call
  result) still takes the payload. The comptime interpreter and the
  language server had such copies.
- Reading standard input through `std.stream.Reader` blocked until 64 KiB
  arrived or the pipe closed, since `fread` fills its whole count; the
  runtime now reads the descriptor and hands over what the pipe has, so
  a server on stdin answers each message as it comes. `io.read_line` and
  the reader share one buffer. Standard input is binary on Windows, like
  the other two streams.

## [0.8.0] - 2026-09-20

*Annapurna: the Sickle* — the exposed glacier crossing below the summit: the compiler in Nexium is the compiler, and builds from its own C with no Rust.

### Changed

- The compiler written in Nexium is the compiler (decision 90). The first
  compiler, in Rust, is frozen at 0.7 semantics and moved to
  `bootstrap/rust/`; it still builds and still holds the tools not yet
  ported (`fmt`, `doc`, `lsp`, `ship`, packages, the REPL), and leaves at
  1.0. A machine with no `nx` builds one from `bootstrap/nx.c`, the C the
  compiler emits for itself, with any C compiler and no Rust
  (`bootstrap/build.sh`, `build.ps1`); CI does so on three platforms with
  no Rust toolchain. `cargo test` builds the compiler from the seed and
  runs every example, spec case and compile-fail case through it; the
  oracle diffs that drove the port (`nx tokens`, `sexp`, `tir` and
  `emit-c` compared between the two compilers) retire.

### Added

- `unbounded_stack` is an effect, no longer reserved: a function on a
  cycle of the call graph (it calls itself, or calls something that calls
  it back) carries it, because its stack use depends on its input; calls
  through function values and trait objects acquire it as they acquire
  every permitted effect. `!unbounded_stack` is discharged by rewriting
  the recursion as a loop over an explicit stack (SPEC 9). The first
  language change made in the compiler in Nexium alone.
- The compiler in Nexium renders diagnostics as the tools always have:
  the message, `--> path:line:col`, the source line, a caret, notes under
  their error, and the count at the end; it printed byte offsets.

### Fixed

- The compiler in Nexium searched no directory for `@cImport` headers and
  vendored C when the file was named without a path (`nx build demo.nx`
  from inside `gui/`); the file's directory is `.` in that case, as in
  the Rust compiler.

### Removed

- `layout(packed)` and `soa` on structs, and the `pool` and `stack`
  allocation strategies, are gone from the specification (decision 88):
  `layout(c)` and `using arena` are what the language has. The two struct
  spellings used to be accepted and silently ignored; they are errors now.
  Region rule R1 is the region rule; the cases it does not cover are
  listed in SPEC 5.6, and a debug build now fills freed storage with
  `0xDD` so a view that outlived its storage does not read the old
  contents by luck.

## [0.7.0] - 2026-09-19

*Annapurna: Camp V* — the last camp, at 7,400 m; the summit push starts here: the compiler builds itself.

### Added

- `self/check.nx` checks closures: parameter and return types from the
  expected function type, by-value and by-reference captures, inferred
  return types; and records: constraint checks, compile-time decisions on
  constant fields, `Record.new`; and trait objects: vtables, `dyn`
  coercions, dynamic calls, effect bounds on function values.
  `examples/generics.nx`, `examples/tests.nx`, `examples/tour.nx`,
  `examples/records.nx`, `examples/dyn.nx` and `std/thread.nx` join the
  body comparison; and binary patterns and construction: `examples/binary.nx`
  and `examples/binary_sizes.nx` join too (46 sources).
- `examples/records.nx`: compile-time, `.new` and run-time constraint checks.
- `examples/binary_sizes.nx`: float, signed, little-endian and computed-size
  segments, a remainder written back out.
- `self/check.nx` evaluates at compile time: an interpreter over the typed
  IR runs constants, globals, `comptime` expressions, `comptime test` blocks
  and record constraints; `examples/comptime.nx` (a CRC table, primes,
  strings and structs computed at compile time) joins the comparison (50
  sources: every example and std module but `@cImport` users).
- A constant slice whose elements were computed at compile time is stored
  as a static array; only string literals could back a constant slice.
- `self/check.nx` reports the diagnostics-only passes: declared effect
  bounds against inferred effects, `for parallel` bodies mutating shared
  state, mutable globals in embeddable libraries, `own` on copied types and
  on exported or fn-value functions, moves out of map lookups, `match`
  exhaustiveness, returned views into locals. `cargo test` runs it over
  every compile-fail case and expects every message the Rust checker gives.
- `self/cimport.nx`, the C header importer in Nexium, and `@cImport` in
  `self/check.nx`: every source in the tree now matches the oracle,
  `examples/cimport.nx` and nexium-gui included (54 sources).
- `self/cgen.nx`, the C emitter in Nexium: byte-identical to `nx emit-c` on
  every source in the tree (54, its own 96k-line translation unit
  included); `cargo test` diffs them.
- The bootstrap closes: `nx1` (the emitter in Nexium, built by the Rust
  compiler) emits the C of itself, `zig cc` builds `nx2` from it with no
  Rust involved, and `nx2` emits byte-identical C for itself and other
  programs. `cargo test` performs the three stages.
- `tests/spec`: the specification's conformance cases, one program per
  claim SPEC.md makes with its recorded output and exit code, run by
  `cargo test` and diffed through every self-hosting stage; sections 2 to
  14 (artifacts and the toolchain are covered by the ship tests).
- `self/nx.nx`, the `nx` driver in Nexium: build, run, test, check, emit-c
  and tir over the self-hosted pipeline, invoking the C compiler as the
  Rust driver does; the standard library is embedded in it. It builds
  itself, and the result builds and runs programs (`cargo test` checks
  that). Nothing past the first compiler needs `cargo`.

### Fixed

- `Color.Green as u8` (a unit enum cast to an integer) emitted a C cast of
  the whole struct, which the C compiler rejected; it is the tag now.
- `let d: i8 = -128` is accepted: a negative literal is one literal, not
  the negation of 128 (which does not fit an i8).
- A labeled block that ends in `break :label value` has the value's type;
  it was `never`, and printing the value was rejected.
- A `break` inside an `orelse` default, a call argument or any other nested
  expression now ends a `while true`; a non-void function that ended in such
  a loop was accepted without a return value.
- A binary segment sized by an expression (`payload:len*8`) is in bits like a
  constant size; the generated C and the compile-time interpreter scaled it
  by eight again, so such patterns never matched and such constructions
  wrote past the intended width.
- A `!void` tail expression (a `match` whose arms print, a call) at the end
  of a function returning `!void` is returned; it was reported as unused.
- `%` on floats compiles: the generated C applied the integer operator to
  doubles and the C compiler rejected it; it is `fmod` now.
- A constant of `List`, `String`, `Map`, `ref class` or weak type (or one
  containing them) is rejected with a hint to store a slice; it used to
  fail in the C compiler, or worse, be freed by whoever copied it.
- Slicing an empty slice (`text[..]`, `"".split(",")`, an empty binary
  pattern) added an offset to a null pointer, which is undefined in C: a
  debug build made by zig 0.14 trapped with "applying zero offset to null
  pointer". The generated C goes through `nx_padd`, which skips the add.
- `if t < 1000 and t > -1000 { t + 1 }`: both halves of an `and` guard
  narrow the range; the second fact used to replace the first, so the
  addition was not proven and a `!panics` bound on the function failed.
- An array indexed by the index of a `for x, i in xs` over it is proven in
  bounds; the proof was skipped for arrays.
- `self/check.nx` kept range facts on a `var` only until its next
  assignment was checked, so a guarded `t = t - 10` was not proven; the
  facts now hold while the right side is checked, as in the Rust checker.
- The generated C no longer collides with macOS's `mach` headers, which
  define `ts_32` and friends as macros: a local named `ts` at slot 32
  failed to compile on macOS.
- `@cImport` retries a preprocessor run that failed without a diagnostic
  and reports the exit code when it keeps failing, instead of an empty
  message.
- `self/cimport.nx` imported hexadecimal float macros (macOS's `MAXFLOAT`)
  that the Rust importer skips, so the two disagreed on `<math.h>` there;
  neither imports them now.
- `self/nx.nx` links native macOS builds with the system compiler, as the
  Rust driver does (zig 0.14 cannot link against the current Xcode SDK);
  `cargo test` builds the bootstrap's second stage with that compiler too.
- The tree-sitter grammar parses `x orelse return null` and the other jumps
  after `orelse` and `catch`, and `/little-signed` segment modifiers.
- `List(thread.Worker(Job))`: a generic type of another module takes its
  type arguments from the caller's scope, and is accepted in expression
  position (`List(thread.Worker(Job)).new()`). The arguments were looked
  up in the other module ("cannot find type `Job`"), and the expression
  form was rejected as not a type.

## [0.6.1] - 2026-09-19

*Annapurna: Lachenal* — patch.

### Added

- `nx tir`: the checked program as S-expressions, the oracle for the
  self-hosted checker; `--sigs` prints declarations and signatures only.
- `self/check.nx`, the checker in Nexium: module loading, declarations,
  type interning, signatures (`cargo test` diffs `--sigs` over 41 sources),
  and function bodies: statements, expressions, calls, builtin methods and
  namespaces, matches and patterns, casts, coercions, ownership moves and
  the range analysis. It produces the oracle's exact typed IR for 35
  sources, itself among them; generics, closures, trait objects, binary
  patterns, compile-time calls and C imports remain.

### Fixed

- A C compilation that fails because Zig's shared cache was being written by
  another `nx` at the same time ("failed to check cache") is retried, so
  parallel builds on Windows no longer fail at random.
- The formatter spaces bit-or like the other operators (`a | b`, not
  `a| b`) and no longer glues `-> !List(T) {`; closure bars are classified
  per line so a bit-or inside a closure body is not taken for its closing bar.
- A `String` built for the right side of `and`/`or` produced C that did not
  compile (its release was emitted outside the block that declared it).
- `i128` range bounds overflowed inside the compiler; an `i128` literal
  beyond `i64` was emitted as undefined C for the minimum; `parse_int`
  rejected values above 2^65.
- A range fact from an `if` guard (`if i == 1 { ... }`) outlived its block
  and could prove a later `xs[i]` in range, eliding its bounds check.
- `return xs[i]` into a `?T` or `!T` moved the element without a clone, and
  a diverging `orelse` default or `catch` handler marked its operand moved.
- Floats print as the shortest text that reads back exactly
  (`3.141592653589793`, not `3.1415926535897931`).
- The tree-sitter parser is regenerated for 0.6.0 (its version is embedded).

## [0.6.0] - 2026-09-19

*Annapurna: Camp IV* — the syntax settled for 1.0; the parser in Nexium.

### Changed

- **Breaking: control flow drops its parentheses and bodies always take
  braces** (decision 87). `if c { }`, `while c { }`, `for x, i in items { }`,
  `for i in lo..hi step s { }`, `for parallel x in items { }`, and
  `if let v = opt { }` replace `if (c)`, `while (c)`, `for (items) |x, i|`,
  and `if (opt) |v|`. The one-statement forms become braced one-liners:
  `if c { return v }`, `let m = if a > b { a } else { b }`. A struct literal
  in a condition needs parentheses (`if (Point{ .x = 1 }) == p { }`).
  Match-arm guards are `pat if cond =>`. `catch |e|` and closure
  parameters are unchanged.
- `nx fmt` migrates 0.5 sources to the new syntax as part of formatting;
  `nx fmt --migrate-only` upgrades the syntax and leaves the layout alone.
  The migration edits by token span, keeps comments and blank lines, and
  is idempotent, so running it on a mixed tree is safe.
- The formatter puts a space before the body brace of any control head
  (`if k == Kind.Defer {`), where before an uppercase name would have been
  glued to `{` as a struct literal.
- The language server resolves `for` bindings and `if let` bindings to
  their declaration.
- The tree-sitter grammar, the VS Code and Sublime syntaxes, every example,
  the standard library, the GUI, the self-hosting sources, and all
  documentation (including the translations) use the new syntax.
- Moving a `String` or `List` out of a `for` loop variable is a compile
  error (`use x.clone()`); it silently produced a double free before. A
  loop variable is a view of the element.

### Added

- `self/parser.nx`: the parser written in Nexium now parses the 0.6 syntax
  and prints the same tree as `nx sexp` for every example, std module, GUI
  and self-hosting source; `cargo test` diffs the two (roadmap phase 4).

### Fixed

- `unreachable` as the last statement of a function returning an error
  union or struct produced C that did not compile.
- A program whose imported module also defines `main` (as `self/lexer.nx`
  does, for running the lexer on its own) took the wrong entry point.
- `nx` left one Zig cache directory behind per process under
  `nx-out/.zig-cache`; it is removed when the command finishes, and stale
  ones are swept.

## [0.5.0] - 2026-09-19

*Annapurna: Camp III* — other people can build on it: packages, editors, installers.

### Added

- Packages: a `nexium.toml` manifest with `[dependencies]` from a git tag
  (`{ git = "...", tag = "..." }`) or a directory (`{ path = "..." }`);
  `import dep` loads the dependency's `src/lib.nx` and `import dep.module`
  its `src/module.nx`; a package's own imports stay inside the package.
  `nx init` writes a manifest, `nx add` records and fetches a dependency,
  `nx fetch` clones every git dependency (transitively) into
  `nexium_modules/` and writes `nexium.lock` with the resolved commits.
  See `docs/packages.md`.
- A tree-sitter grammar (`editors/tree-sitter-nexium`) with highlight
  queries for Neovim, Helix and Zed; it parses every example, std module
  and self-hosted source without an error node, and CI keeps it that way.
- The language server gained go to definition (functions, types, constants,
  locals, imported and package modules), completion (module members after
  `alias.`, fields, methods, variants and errors after `.`, names in
  scope), and rename (a local within its function, an item across the
  file). They work from the parsed source, so they answer in files that do
  not type-check yet.
- `artifact installer`: `nx ship` produces an installer for a program. On
  Windows an Inno Setup script (compiled to `<Name>-<version>-setup-x64.exe`
  when Inno Setup 6 is installed) with license page, per-user or
  all-users install, Start menu entry, optional PATH entry and uninstaller;
  on Linux and macOS an `install.sh` with `--prefix` and `--uninstall` plus
  a tarball. Listed `files` are copied next to the program.
- `artifact node { name = "pkg" }`: `nx ship` produces an npm package for an
  exported library: `index.js` calling the shared library through koffi
  (no build step, no node-gyp), `index.d.ts` typings, `package.json`.
  Slices take typed arrays, arrays or strings; error unions throw
  `NexiumError` and panics throw `NexiumPanic` with the message.

### Fixed

- Parallel builds on Windows could fail inside Zig's own cache ("failed to
  check cache ... file_open Unexpected") when several `zig cc` processes
  started at once; each `nx` process now gives Zig its own cache under the
  output directory unless `ZIG_LOCAL_CACHE_DIR` is set.

## [0.4.0] - 2026-09-19

*Annapurna: Camp II* — the language talks to the world: sockets, HTTP, threads.

### Added

- Sockets in the runtime: `net.connect`, `listen`, `accept`, `send`, `recv`,
  `close`, `peer`, `local`, `resolve`, `udp_bind`, `send_to`, `recv_from`,
  `last_peer`; blocking, with per-call timeouts, on Winsock and BSD sockets.
  New errors `Timeout` and `ConnectionRefused`.
- `std.net`: `TcpStream` (connect with timeout, send, recv, recv_all, peer,
  buffered `reader()`/`writer()`), `TcpListener` (bind, accept with
  timeout, port), `UdpSocket` (bind, send_to, recv_from), `parse_addr`,
  `port_of`, `resolve`. `std.stream` readers and writers work over sockets.
- `std.http`: a client (`get`, `post`, `request` with headers; HTTP/1.1,
  Content-Length and chunked bodies, up to five redirects) and a server
  (`Server.bind`, `serve`, `serve_one`, `Router` with exact and `/*` routes,
  `serve_static`, `Request.param`/`header`, response helpers `text`, `html`,
  `json`, `redirect`, `not_found`). Plain `http://`; TLS is left to
  `@cImport`.
- `examples/service.nx`: an HTTP service and a client in one program, the
  phase 2 exit example; `examples/errors_more.nx` covers the fixes below.
- Threads in the runtime (`thread.start`, `thread.join`, `thread.count`,
  `sync.mutex_new`/`lock`/`unlock`/`mutex_free`, `sync.cond_new`/`wait`/
  `signal`/`broadcast`/`cond_free`) and `std.thread` on top: `spawn` returns
  a `Thread(T, R)` whose `join` yields the function's result, `run` returns
  a `Worker(T)` for functions without one, `Channel(T)` (`send`, `recv`,
  `try_recv`, `close`), `Mutex(T)` (`lock` returns `*mut T`, `unlock`). A
  panic inside a thread is re-raised by `join`. Starting a thread carries
  the `nondeterministic` and `shared_mutable` effects.
- `own` is accepted on parameters of methods in generic `impl` blocks.
- `process.exec(argv, stdin, cwd)` runs a program with stdin fed, a working
  directory, and stdout/stderr captured (`process.last_stdout`,
  `process.last_stderr`); `std.process` wraps it as `run`, `run_with`,
  `shell` returning an `Output` with `code`, `stdout`, `stderr`.

### Fixed

- Free functions with the same name in two imported modules (`fs.copy` and
  `stream.copy`) collided in the generated C.
- Returning a caught error value (`catch |e| { return e }`) from a function
  returning `!T` produced a bare error id instead of an error union.
- An untyped integer literal now coerces into `!T` (`return 7` in a
  function returning `!i32`).
- The formatter kept the space in `-> http.Response {` (a dotted type before
  a block is not a struct literal).

## [0.3.0] - 2026-09-19

*Annapurna: Camp I* — the first camp on the mountain: the tools' standard library.

### Added

- `std.fs`: `exists`, `is_file`, `is_dir`, `size`, `modified`, `read`,
  `read_lines`, `write`, `append`, `copy`, `list` (sorted), `make_dir`,
  `make_dirs`, `remove`, `remove_all`, `rename`, `walk`, `cwd`, `temp_dir`,
  `temp_path`, and the path helpers `join`, `parent`, `base_name`, `stem`,
  `extension`, `with_extension`, `normalize`, `is_absolute`. Under it, new
  `io` primitives in the runtime: `append_file`, `file_kind`, `file_size`,
  `file_modified`, `make_dir`, `remove_file`, `remove_dir`, `rename`,
  `list_dir`, `cwd`, `temp_dir`; they also work at the REPL.
- `std.time`: `DateTime` (`utc`, `local`, `with_offset`, `now_utc`,
  `now_local`, `date`, `parse_iso`, `to_ms`, `weekday`, `day_of_year`,
  `iso`, `format` with `%Y %m %d %H %M %S %3 %z %a %b %j`), `Duration`
  (`seconds` ... `days`, `between`, `since`, `text` such as `1h 02m`),
  `Stopwatch` on the monotonic clock, `is_leap`, `days_in_month`. Under it,
  `time.utc_offset(ms)` in the runtime (0 at the REPL).
- `std.regex`: a Pike VM (no backtracking, linear time) with classes,
  `\d \w \s \b`, anchors, groups and `(?:...)`, alternation, greedy and
  lazy repeats including `{n,m}`; `compile`, `find`, `find_at`, `find_all`,
  `is_match`, `replace_all` with `$1` references, `split`, and `Match.group`.
- A local that was moved out can be assigned again; the assignment
  re-initializes it instead of being reported as a use after move.
- `for (a..b step s) |i|` walks a range with a step; a negative step counts
  down (the loop variable must be signed). `while (c) { } else { }` runs the
  else block when the condition turns false, but not after a `break`.
- `match` exhaustiveness now uses the full matrix algorithm, so tuples of
  enums, nested optionals and enum payloads are checked precisely instead of
  demanding a catch-all arm.
- `nx test FILE --filter NAME` runs the tests whose names contain NAME.
- `std.text`: UTF-8 by code point (`decode_at`, `chars`, `char_count`,
  `char_at`, `slice`, `truncate`, `reverse`, `encode`, `is_valid`), terminal
  `width` (wide and zero-width aware), and case mapping for ASCII, Latin-1,
  Latin Extended-A, Greek and Cyrillic (`to_upper`, `to_lower`,
  `eq_ignore_case`).
- `std.testing`: `approx`, `expect_approx`, `is_err`, `expect_err`,
  `expect_error`, `expect_contains`, `expect_lines` (names the first
  differing line) and file snapshots (`snapshot`, `snapshot_in`;
  `NX_UPDATE_SNAPSHOTS=1` rewrites them).
- Embedded std modules can import each other; `error` is a type name (the
  anonymous error set), and `own` is accepted on generic parameters.
- `nx test` no longer runs the tests of imported std modules.
- `std.stream`: buffered `Reader` (`open`, `stdin`, `read_line`, `read`,
  `read_all`) and `Writer` (`open`, `append`, `stdout`, `stderr`, `write`,
  `write_line`, `flush`, `close`) plus `copy`, over new runtime file handles
  (`io.open`, `io.read`, `io.write`, `io.flush`, `io.close`; handles 1 to 3
  are the standard streams).
- `os.environ()` lists the environment; `args.env_map()` turns it into a
  `Map(String, String)`.
- `nx test --verbose` prints timings and the tests a filter skipped;
  `nx doc std.fs` and `nx test std.regex` accept an embedded module by name.
- `examples/tool.nx`: a log scanner (walk a tree, parse timestamps, filter
  by a date window and a regex, tally by level) in 142 lines, the phase 1
  exit example of the roadmap; runs on `examples/data/logs` by default.

### Fixed

- A struct, tuple or enum literal that read a local in one field and moved
  it in a later field saw the already-zeroed value; field values are now
  materialized in source order.

## [0.2.1] - 2026-09-19

*Annapurna: Herzog* — patch.

### Added

- `SPEC.md`, the language specification as implemented, and `ROADMAP.md`.
- `nx repl`, and `nx` with no arguments at a terminal: an interactive session
  on the compiler's interpreter, with bindings kept across lines, the real
  diagnostics, I/O, and `:load`. The Windows installer adds a Start menu
  entry that opens it, and offers to launch it when setup finishes.
- The compile-time interpreter now evaluates `Map`, the mutating `List`,
  `String` and slice methods (`insert`, `remove`, `sort`, `split`, `trim`,
  `parse_int`, ...), `expect_eq`, `assert`, UTF-8 helpers and the reference
  class operations, and pointers stay valid across calls, so `std.json` and
  the other std modules run at the prompt. A statement the interpreter
  cannot evaluate is reported with the position of the failing expression.

### Fixed

- Building a program by its bare file name (`nx run app.nx` from inside its
  directory) dropped the C sources declared in `artifact link`: the empty
  parent directory produced a lone `-I` that swallowed the next argument.
- `opt.?`, `opt orelse d` and `try res` on a local holding an owning value
  now move the local: it is no longer dropped a second time at scope end
  (this crashed GUI programs on exit), and a later use is reported as a use
  after move.

## [0.2.0] - 2026-09-19

*Annapurna: Base Camp* — where the expedition is staged: the installer, the spec, the roadmap.

### Added

- `own` parameters: `fn f(own s: String)` takes ownership of its argument
  (moved at the call site, dropped or moved on by the callee).
- `orelse return v` and `catch |e| return v`: jumps on the right of `orelse`
  and `catch`.
- `String.push_byte(b)` appends one raw byte.
- Integer and float literals coerce into `?T`.
- `examples/own.nx` and five compile-fail cases for the `own` rules.
- The standard library in Nexium: `import std.strings`, `std.lists`,
  `std.bytes`, `std.num`, `std.json`, `std.args` (96 functions with tests),
  embedded in the compiler; `docs/std.md` lists them; `examples/stdlib.nx`
  uses them.
- Tuple types as type arguments: `List((A, B))`.
- A Windows installer (`nexium-<version>-setup-x64.exe`): license, overview,
  per-user or all-users, components (bundled Zig, std and examples, docs, VS
  Code extension), PATH and `.nx` file-type tasks, `nx doctor` on finish.
- An install script for macOS and Linux (`installers/install.sh`) that
  verifies checksums and sets up a C compiler.
- `nx doctor`, and `nx` finds a Zig bundled next to itself (`NX_ZIG` too).
- Releases carry `SHA256SUMS.txt`, per-file checksums, and install
  instructions in the notes; the tarballs include examples, std, and docs.
- Recursive types through `List`: `enum Json { Arr(List(Json)) }`.
- Matching through a pointer (`match p.*`) binds owning payloads by reference,
  so `match v.* { .Arr(items) => items.append(x) }` mutates in place.
- `*String` and `*List(T)` coerce to `[]u8` and `[]T`; `null` coerces into a
  nested optional such as `!?T`.

### Changed

- Moves are tracked per branch: a value moved in one `if` branch or `match`
  arm stays usable in the others.
- An unbraced `if` or `else` body is one statement, so `if (c) x = 1` works.
- `self/lexer.nx` moves token text with `own` instead of cloning it.

### Fixed

- A branch that diverges (`if (c) return x`) no longer marks what it moved as
  moved afterwards.
- Two modules each defining a type of the same name (`Parser` in `std.json`
  and `std.args`) collided in the generated C; type names now carry their
  module. A user function named like a runtime identifier (`fn string`)
  no longer collides either.
- `nx fmt` spacing after `-> List(T)`, after a closing closure bar, and
  between an `if` condition and a parenthesized body.

## [0.1.0] - 2026-09-18

*Annapurna: Miristi Khola* — the gorge the 1950 expedition spent weeks finding a way through; the approach.

First public release.

### Added

- The `nx` compiler: `build`, `run`, `test`, `check`, `effects`, `audit`, `ship`, `emit-c`, `parse`.
- Language: structs, records with `where` constraints, enums with payloads, `ref class`
  with reference counting and `weak`, distinct types, traits with default methods,
  generic functions and types through `comptime T: type`, closures with explicit
  captures, error unions with `try`/`catch`, `defer`/`errdefer`, optionals,
  labeled blocks and loops, the pipe operator, binary pattern matching and
  construction, compile-time evaluation of constants, `test` blocks.
- Effects: `allocates`, `refcounts`, `blocks`, `shared_mutable`, `nondeterministic`,
  `panics`, `ffi`, inferred for every function and enforced against negative bounds,
  with diagnostics naming the site that introduces an effect.
- Standard library builtins: `List`, `String`, `Map`, slices, formatting, `math`,
  `io`, `os`, `time`, `random`, `mem`.
- Artifacts: `cabi` (static library, shared library, C header) and `python`
  (ctypes package and wheel) from one source tree with `nx ship`; `cli` binaries.
- Embedded mode: reachable mutable globals are rejected when an embeddable
  artifact is declared (S2); panics never cross the export boundary (S3).
- Examples under `examples/`, compile-fail cases under `tests/compile_fail/`.
- `dyn Trait` trait objects with effect bounds, `for parallel` over a thread pool,
  `using arena { }` allocation scopes, `comptime test`.
- `@cImport("header.h")` direct C header import with variadic calls, `@cstr`, and
  vendored C through `artifact link { c_sources = [...] }`.
- `rustlib` artifact: a generated Cargo crate with safe wrappers.
- Tools: `nx fmt`, `nx doc`, `nx size`, `nx refcounts`, `nx leaks`, `nx lsp`.
- A conservative region check: returning a view into a local is an error.
- `process.run(argv) -> !i32`, `nx tokens`, and `self/lexer.nx`: the first stage
  of the self-hosted compiler, verified against the Rust lexer in `cargo test`.
- Logo and banner under `assets/`; `docs/architecture.md` explains how the compiler works.
- `gui/`: nexium-gui, an immediate-mode GUI written in Nexium (software rasterizer,
  bitmap font, buttons, checkboxes, sliders, text fields) over a 200-line C
  platform layer; Win32 window backend, offscreen rendering everywhere.
- `artifact link` accepts `libs_windows`, `libs_linux`, `libs_macos`; paths in a
  link artifact resolve relative to the module that declares it.
- Editor support: a VS Code extension (grammar, `nx lsp` client, run command,
  packaged as a `.vsix` on every release) and a Sublime Text syntax.
- `setup-nexium` GitHub Action and a release workflow template for programs
  written in Nexium (`docs/releasing-your-program.md`).
- `linguist/`: the prepared GitHub Linguist entry, samples, heuristic, and apply script.
- `@cImport`: structs with untranslatable fields are opaque types instead of
  errors, so `FILE*` works with Apple's libc.
- Native macOS builds use the system `cc` (zig 0.14 cannot link against the
  Xcode 26 SDK); `NX_CC` selects the C compiler without a flag.
- Translations under `docs/i18n/`: the README in Spanish, Chinese, Japanese,
  Korean, French, and German; the language reference and architecture tour in
  Spanish, Chinese, and Japanese.

[Unreleased]: https://github.com/Londopy/nexium/compare/v1.3.0...HEAD
[1.3.0]: https://github.com/Londopy/nexium/compare/v1.2.1...v1.3.0
[1.2.1]: https://github.com/Londopy/nexium/compare/v1.2.0...v1.2.1
[1.2.0]: https://github.com/Londopy/nexium/compare/v1.1.0...v1.2.0
[1.1.0]: https://github.com/Londopy/nexium/compare/v1.0.3...v1.1.0
[1.0.3]: https://github.com/Londopy/nexium/compare/v1.0.2...v1.0.3
[1.0.2]: https://github.com/Londopy/nexium/compare/v1.0.1...v1.0.2
[1.0.1]: https://github.com/Londopy/nexium/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/Londopy/nexium/compare/v0.9.0...v1.0.0
[0.9.0]: https://github.com/Londopy/nexium/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/Londopy/nexium/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/Londopy/nexium/compare/v0.6.1...v0.7.0
[0.6.1]: https://github.com/Londopy/nexium/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/Londopy/nexium/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/Londopy/nexium/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/Londopy/nexium/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/Londopy/nexium/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/Londopy/nexium/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/Londopy/nexium/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/Londopy/nexium/releases/tag/v0.1.0
