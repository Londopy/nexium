# The standard library

Modules written in Nexium and embedded in the compiler. `import std.<module>`
makes it available as `<module>.function(...)`; nothing to install or link.
The core containers (`List`, `String`, `Map`), formatting, and the `math`,
`io`, `os`, `time`, `random`, `mem`, and `process` namespaces are compiler
builtins and are documented in [`language.md`](language.md).

Sources are in [`std/`](../std); each module carries its own `test` blocks,
run by `nx test std/<module>.nx` and by the test harness. This file is generated
by `scripts/std_docs.py` from the doc comments.

| module | what |
| --- | --- |
| [`std.args`](#stdargs) | command-line argument parsing, written in Nexium. |
| [`std.base64`](#stdbase64) | base64 (RFC 4648), written in Nexium: the standard alphabet |
| [`std.bytes`](#stdbytes) | encodings and byte-level utilities, written in Nexium. |
| [`std.csv`](#stdcsv) | comma-separated values (RFC 4180), written in Nexium: reading |
| [`std.deque`](#stddeque) | a double-ended queue, written in Nexium: two Lists back to |
| [`std.env`](#stdenv) | the environment, and where a program keeps its files, written in |
| [`std.fs`](#stdfs) | files, directories and paths, written in Nexium. |
| [`std.hash`](#stdhash) | hash functions, written in Nexium: FNV-1a (64-bit), SipHash-2-4 |
| [`std.heap`](#stdheap) | a priority queue, written in Nexium: a binary heap over a List, |
| [`std.http`](#stdhttp) | an HTTP/1.1 client and a small server, written in Nexium over |
| [`std.json`](#stdjson) | a JSON parser and serializer, written in Nexium. |
| [`std.lists`](#stdlists) | generic helpers over slices and Lists, written in Nexium. |
| [`std.log`](#stdlog) | leveled, structured logging, written in Nexium: a message and |
| [`std.net`](#stdnet) | TCP and UDP with addresses, written in Nexium over the `net.*` |
| [`std.num`](#stdnum) | integer utilities, written in Nexium. |
| [`std.path`](#stdpath) | paths as text, written in Nexium: joining, splitting, comparing |
| [`std.process`](#stdprocess) | run programs and capture what they print, or talk to them |
| [`std.regex`](#stdregex) | regular expressions without backtracking, written in Nexium. |
| [`std.set`](#stdset) | a set of values, written in Nexium over `Map(T, bool)`: its |
| [`std.sort`](#stdsort) | sorting by a comparison of your own, stable sorting, and |
| [`std.stream`](#stdstream) | buffered readers and writers over files and the standard |
| [`std.strings`](#stdstrings) | text utilities on `[]u8` and `String`, written in Nexium. |
| [`std.testing`](#stdtesting) | conveniences for `test` blocks, written in Nexium. |
| [`std.text`](#stdtext) | UTF-8 text by code point and by grapheme cluster, written in |
| [`std.thread`](#stdthread) | threads, channels, select, mutexes and atomics, written in |
| [`std.time`](#stdtime) | dates, durations, time zones and timers, written in Nexium. |
| [`std.toml`](#stdtoml) | TOML 1.0 (toml.io), written in Nexium: reading a document into |
| [`std.uuid`](#stduuid) | UUIDs (RFC 9562), written in Nexium: random ones (version 4), |
| [`std.websocket`](#stdwebsocket) | a WebSocket client (RFC 6455), written in Nexium over |

## std.args

std.args: command-line argument parsing, written in Nexium. `import std.args` then: var p = args.Parser.new(os.args()) let verbose = p.flag("--verbose", "-v") let level = p.option("--level", "-l") orelse "info" let files = p.rest() Long options take `--name value` or `--name=value`; short ones `-n value`. Everything after `--` is positional. Unknown options stay in `rest()` so the caller can report them.

Types: `Parser`

| function | what it does |
| --- | --- |
| `(method) new(argv: [][]u8) -> Parser` | Wrap `os.args()`; the program name in `argv[0]` is skipped. |
| `(method) flag(self: *mut Self, long: []u8, short: []u8) -> bool` | True when the flag is present (any number of times). |
| `(method) count(self: *mut Self, long: []u8, short: []u8) -> usize` | How many times the flag appears (`-vvv` counts as three). |
| `(method) option(self: *mut Self, long: []u8, short: []u8) -> ?[]u8` | The value of `--name value`, `--name=value`, or `-n value`; the last one wins. |
| `(method) options(self: *mut Self, long: []u8, short: []u8) -> List([]u8)` | Every value of a repeatable option, in order. |
| `(method) int_option(self: *mut Self, long: []u8, short: []u8) -> !?i64` | An option parsed as an integer; `error.InvalidInput` when present but not a number. |
| `(method) rest(self: *Self) -> List([]u8)` | Arguments not consumed by any query, plus everything after `--`. |
| `(method) unknown_options(self: *Self) -> List([]u8)` | Unconsumed arguments that look like options: the ones the program did not ask for. |
| `usage(program: []u8, summary: []u8, rows: [][]u8) -> String` | Render a usage line and option table from (flags, description) rows. |
| `env_map() -> Map(String, String)` | The environment as a map, from `os.environ()`. |

## std.base64

std.base64: base64 (RFC 4648), written in Nexium: the standard alphabet with its `=` padding, and the URL-safe one (`-` and `_` for `+` and `/`) without, as JSON Web Tokens and URLs carry it. `import std.base64` then: let s = base64.encode("hi!")                  // "aGkh" let raw = try base64.decode(s) let token = base64.encode_url(key)            // no `=`, safe in a URL let back = try base64.decode_url(token) Decoding is strict: a character outside the alphabet, padding anywhere but at the end or more of it than the length needs, or a length no encoding produces is `error.InvalidInput`. `std.bytes` keeps the lenient reader, `unbase64`, which skips line breaks and stray padding.

| function | what it does |
| --- | --- |
| `encode(data: []u8) -> String` | Standard base64 with `=` padding. |
| `decode(text: []u8) -> !String` | Bytes from standard base64; the padding may be left out. |
| `encode_url(data: []u8) -> String` | URL-safe base64 without padding (RFC 4648 section 5, as JWTs use). |
| `decode_url(text: []u8) -> !String` | Bytes from URL-safe base64, padded or not. |

## std.bytes

std.bytes: encodings and byte-level utilities, written in Nexium. `import std.bytes` then `bytes.hex(data)`, `bytes.base64(data)`, ... Decoders return `error.InvalidInput` on malformed text.

| function | what it does |
| --- | --- |
| `hex(data: []u8) -> String` | Lower-case hexadecimal, two characters per byte. |
| `unhex(text: []u8) -> !String` | Bytes from hexadecimal text (either case, even length). |
| `base64(data: []u8) -> String` | Standard base64 with `=` padding. |
| `unbase64(text: []u8) -> !String` | Bytes from standard base64 (padding optional). |
| `fnv1a(data: []u8) -> u32` | FNV-1a, 32 bits: a fast non-cryptographic hash. |
| `crc32(data: []u8) -> u32` | CRC-32 (IEEE), as used by zip and PNG. |
| `read_u32_be(data: []u8, at: usize) -> u32` | Big-endian 32-bit read. |
| `read_u32_le(data: []u8, at: usize) -> u32` | Little-endian 32-bit read. |
| `write_u32_be(out: *mut String, v: u32)` | Append a big-endian 32-bit value. |
| `write_u32_le(out: *mut String, v: u32)` | Append a little-endian 32-bit value. |
| `first_difference(a: []u8, b: []u8) -> ?usize` | Bytes that differ, for a compact diff of two buffers. |

## std.csv

std.csv: comma-separated values (RFC 4180), written in Nexium: reading rows, reading rows by their header, and writing rows. `import std.csv` then: let rows = try csv.parse(text)                      // List(List(String)) for row in try csv.by_header(text) {                // Map(String, String) let name = row["name"] orelse "?" } let tsv = try csv.parse_with(text, '\t') let out = csv.to_text(&rows)                         // quoted where needed, CRLF A field in double quotes may hold the separator, line breaks and `""` for a quote. A quote inside an unquoted field, anything but the separator or a line end after a closing quote, and a quoted field never closed are `error.InvalidInput`. Lines may end in CRLF or LF, empty lines are skipped, and a leading UTF-8 byte-order mark is dropped.

| function | what it does |
| --- | --- |
| `parse(text: []u8) -> !List(List(String))` | The rows of comma-separated text. |
| `parse_with(text: []u8, sep: u8) -> !List(List(String))` | The rows of text whose fields are separated by `sep` (`'\t'` for TSV, `';'` where the decimal separator is a comma). |
| `by_header(text: []u8) -> !List(Map(String, String))` | The rows after the first, each as a map from the first row's names to its fields. A row shorter than the header leaves the missing names out; one longer is `error.InvalidInput`. |
| `field(value: []u8, sep: u8) -> String` | A field as written between separators: in double quotes, with any quote doubled, when it holds `sep`, a quote or a line break. |
| `to_text(rows: *List(List(String))) -> String` | Rows as comma-separated text, each line ending in CRLF. |
| `to_text_with(rows: *List(List(String)), sep: u8, line_end: []u8) -> String` | Rows as text with `sep` between fields and `line_end` after each row. |

## std.deque

std.deque: a double-ended queue, written in Nexium: two Lists back to back, the first kept reversed, so both ends push and pop in O(1) amortized (when one side runs out, half of the other moves over). `import std.deque` then: var q = deque.of(Job) q.push_back(job) q.push_front(urgent) let next = q.pop_front() orelse return if let last = q.last() { ... }                 // a view, left in place

Types: `Deque(T){`

| function | what it does |
| --- | --- |
| `of(comptime T: type) -> Deque(T)` | An empty deque. |
| `(method) push_back(self: *mut Self, own x: T)` |  |
| `(method) push_front(self: *mut Self, own x: T)` |  |
| `(method) pop_front(self: *mut Self) -> ?T` | Takes out the first element, or null when empty. |
| `(method) pop_back(self: *mut Self) -> ?T` | Takes out the last element, or null when empty. |
| `(method) len(self: *Self) -> usize` |  |
| `(method) is_empty(self: *Self) -> bool` |  |
| `(method) get(self: *Self, i: usize) -> ?*T` | The element `i` places from the front, left in place; null past the end. |
| `(method) first(self: *Self) -> ?*T` |  |
| `(method) last(self: *Self) -> ?*T` |  |
| `(method) clear(self: *mut Self)` |  |

## std.env

std.env: the environment, and where a program keeps its files, written in Nexium: variables, the home directory, the config, data, cache and state directories each platform expects (XDG on Linux and the BSDs, Library on macOS, AppData on Windows), and `.env` files. `import std.env` then: let port = env.get_or("PORT", "8080") let dir = env.config_dir("myapp")              // ?String: ~/.config/myapp, ... let notes = env.expand_home("~/notes.txt") let n = env.load_dotenv(".env", false) catch 0 // sets what is not set yet The directories are where a program should keep its files; nothing here creates them (`fs.make_dirs` does). Each is null when the variables it comes from are not set.

Types: `Var`

| function | what it does |
| --- | --- |
| `get(name: []u8) -> ?String` | The variable's value, or null when it is not set or set to nothing. |
| `get_or(name: []u8, default: []u8) -> String` | The variable's value, or `default` when it is not set or set to nothing. |
| `set(name: []u8, value: []u8)` | Set a variable for this program and the ones it starts. |
| `unset(name: []u8)` | Remove a variable. |
| `all() -> List(Var)` | Every variable, sorted by name. |
| `home() -> ?String` | The user's home directory: `HOME`, or on Windows `USERPROFILE`. |
| `expand_home(p: []u8) -> String` | `~` or `~/...` at the start of a path, with the home directory in its place; any other path as it is. |
| `config_dir(app: []u8) -> ?String` | Where a program keeps its settings, in a directory named `app` (none when `app` is empty): `$XDG_CONFIG_HOME` or `~/.config` on Linux and the BSDs, `~/Library/Application Support` on macOS, `%APPDATA%` on Windows. |
| `data_dir(app: []u8) -> ?String` | Where a program keeps what it makes and needs to keep: `$XDG_DATA_HOME` or `~/.local/share`, `~/Library/Application Support`, `%APPDATA%`. |
| `cache_dir(app: []u8) -> ?String` | Where a program keeps what it can make again: `$XDG_CACHE_HOME` or `~/.cache`, `~/Library/Caches`, `%LOCALAPPDATA%`. |
| `state_dir(app: []u8) -> ?String` | Where a program keeps its state between runs (logs, history, what was open): `$XDG_STATE_HOME` or `~/.local/state`, `~/Library/Application Support`, `%LOCALAPPDATA%`. |
| `parse_dotenv(text: []u8) -> !List(Var)` | The variables a `.env` file sets, in order: `NAME=value` lines, an optional `export ` before the name, `#` comments and blank lines skipped, a value in single quotes taken as written, one in double quotes with `\n`, `\t`, `\"` and `\\` read as escapes, and an unquoted value trimmed and cut at ` #`. A line that is none of these is an error. |
| `load_dotenv(file: []u8, override: bool) -> !usize` | Read a `.env` file and set its variables: those not set already, or every one when `override`. How many were set; `error.InvalidInput` when a line does not parse (nothing is set then). |

## std.fs

std.fs: files, directories and paths, written in Nexium. `import std.fs` then: if fs.exists("notes.txt") { ... } try fs.make_dirs("out/logs") for name in try fs.list("out") { ... } for path in try fs.walk("src") { ... }        // every file, recursively let cfg = fs.join(fs.parent(argv0), "app.toml") The platform calls are the `io.*` builtins (documented in the language reference); this module adds sorted listings, recursive create and remove, and a walker. Its path functions call std.path's, which has more. Paths are byte strings; `/` and `\` both separate components on every platform, and results use `/` unless the input used `\`.

| function | what it does |
| --- | --- |
| `exists(path: []u8) -> bool` | Is there a file or directory at `path`? |
| `is_file(path: []u8) -> bool` | Is `path` an existing regular file (anything that is not a directory)? |
| `is_dir(path: []u8) -> bool` | Is `path` an existing directory? |
| `size(path: []u8) -> !u64` | The size of a file in bytes. |
| `modified(path: []u8) -> !i64` | The modification time in milliseconds since the epoch. |
| `read(path: []u8) -> !String` | The whole file as a String. |
| `read_lines(path: []u8) -> !List(String)` | The lines of a file, without their line endings. |
| `write(path: []u8, data: []u8) -> !void` | Write (replace) a file. |
| `append(path: []u8, data: []u8) -> !void` | Append to a file, creating it when missing. |
| `copy(from: []u8, to: []u8) -> !void` | Copy a file's contents to a new path (the destination is replaced). |
| `list(path: []u8) -> !List(String)` | The names in a directory, sorted, without `.` and `..`. |
| `make_dir(path: []u8) -> !void` | Create one directory; fine when it already exists. |
| `make_dirs(path: []u8) -> !void` | Create a directory and every missing parent. |
| `remove(path: []u8) -> !void` | Remove a file or an empty directory. |
| `remove_all(path: []u8) -> !void` | Remove a file, or a directory with everything in it. |
| `rename(from: []u8, to: []u8) -> !void` | Rename or move a file or directory (an existing destination file is replaced). |
| `walk(root: []u8) -> !List(String)` | Every file under `root`, recursively, as paths joined onto `root`, sorted directory by directory. Directories themselves are not listed. |
| `cwd() -> !String` | The current working directory. |
| `temp_dir() -> String` | The directory for temporary files. |
| `temp_path(prefix: []u8) -> String` | A fresh path in the temporary directory, `<temp>/<prefix><number>`, that does not exist yet. The caller creates it. |
| `is_absolute(p: []u8) -> bool` | Does the path start at a root (`/x`, `C:\x`, `C:/x`, `\\server`)? As `path.is_absolute`. |
| `join(dir: []u8, name: []u8) -> String` | `dir/name`; a separator is added only when needed, and an absolute `name` replaces `dir`. As `path.join`. |
| `parent(p: []u8) -> []u8` | Everything before the last separator: `a/b/c.txt` -> `a/b`, `c.txt` -> ``, `/c.txt` -> `/`. As `path.parent`. |
| `base_name(p: []u8) -> []u8` | The last component: `a/b/c.txt` -> `c.txt`. As `path.base_name`. |
| `extension(p: []u8) -> []u8` | The extension without the dot: `a/b.tar.gz` -> `gz`, `Makefile` -> ``. As `path.extension`. |
| `stem(p: []u8) -> []u8` | The base name without its extension: `a/b.tar.gz` -> `b.tar`. As `path.stem`. |
| `with_extension(p: []u8, ext: []u8) -> String` | The path with its extension replaced (or added): `a/b.txt`, `md` -> `a/b.md`. As `path.with_extension`. |
| `normalize(p: []u8) -> String` | Collapse `.` and `..` components and repeated separators: `a/./b/../c//d` -> `a/c/d`. A leading `..` is kept. As `path.normalize`. |

## std.hash

std.hash: hash functions, written in Nexium: FNV-1a (64-bit), SipHash-2-4 (keyed: a table whose keys an adversary picks), SHA-256 (checksums and content addresses), and SHA-1 for the protocols that still require it. `import std.hash` then: let h = hash.fnv1a64(name)                    // fast, not keyed let k = hash.siphash(key16, name)             // keyed with 16 bytes let sum = hash.sha256_hex(file_text)          // 64 hex digits var s = hash.Sha256.new()                     // or piece by piece s.update(part1) s.update(part2) let digest = s.finish()                       // 32 bytes `std.bytes` keeps the 32-bit `fnv1a` and `crc32`.

Types: `Sha256`

| function | what it does |
| --- | --- |
| `fnv1a64(data: []u8) -> u64` | FNV-1a over 64 bits. |
| `siphash(key: []u8, data: []u8) -> u64` | SipHash-2-4 of `data` under a 16-byte key (panics on another length): what a hash table keyed by untrusted input should use. |
| `(method) new() -> Sha256` |  |
| `(method) update(self: *mut Self, data: []u8)` |  |
| `(method) finish(self: *mut Self) -> String` | The 32-byte digest of everything given to `update`. |
| `sha256(data: []u8) -> String` | The SHA-256 digest of `data`: 32 bytes. |
| `sha256_hex(data: []u8) -> String` | The SHA-256 digest of `data` as 64 lower-case hex digits. |
| `sha1(data: []u8) -> String` | The SHA-1 digest of `data`: 20 bytes. SHA-1 is broken for anything an adversary can choose, so it is here for the protocols that still require it (the WebSocket handshake, `std.websocket`), never to check or sign data; use `sha256` for that. |
| `sha1_hex(data: []u8) -> String` | The SHA-1 digest of `data` as 40 lower-case hex digits. |

## std.heap

std.heap: a priority queue, written in Nexium: a binary heap over a List, ordered by a comparison of your own. `import std.heap` then: var q = heap.by(Job, |a: *Job, b: *Job| -> bool { return a.due < b.due }) q.push(job) while true { let next = q.pop() orelse break       // the least by the comparison first run(next) } `push` and `pop` are O(log n), `peek` O(1). A heap that pops the greatest first is one whose comparison says `a > b`.

Types: `Heap(T){`

| function | what it does |
| --- | --- |
| `by(comptime T: type, less: fn(*T, *T) -> bool) -> Heap(T)` | An empty heap ordered by `less`. |
| `(method) push(self: *mut Self, own x: T)` | Adds `x`. |
| `(method) pop(self: *mut Self) -> ?T` | Takes out the least element, or null when the heap is empty. |
| `(method) peek(self: *Self) -> ?*T` | The least element, left in place; null when the heap is empty. |
| `(method) len(self: *Self) -> usize` |  |
| `(method) is_empty(self: *Self) -> bool` |  |
| `(method) clear(self: *mut Self)` |  |

## std.http

std.http: an HTTP/1.1 client and a small server, written in Nexium over std.net and std.stream. `import std.http` then: let r = try http.get("https://example.com/")  // TLS by the system's own library println("{} {}", .{r.status, r.body.len}) if let ct = r.header("content-type") { ... } var client = http.client_with(NxTls, &mut layer)   // or a TLS layer of a package's client.timeout_ms = 10000                   // and timeouts, redirects, limits var s = try client.open("GET", "https://example.com/big", &headers, "") while true {                                // the body as it arrives let piece = (try s.next()) orelse break ... } s.close() fn hello(req: *http.Request) -> http.Response { return http.text(200, "hello from Nexium") } var router = http.Router.new() router.get("/", hello) var server = try http.Server.bind("127.0.0.1", 8080) try server.serve(&router)                  // forever, one request at a time The client speaks HTTP/1.1 with `Connection: close` over a `Transport`: TCP for `http://`, and for `https://` the TLS layer a program hands its client (the slot of decision 120, which nxtls, the TLS 1.3 client written in Nexium, fills), or without one `SystemTls`, the platform's own: SChannel on Windows, Security.framework on macOS, OpenSSL on Linux and the BSDs, each checking the server's certificate against the system's roots and the host's name (`SystemTls.problem()` says why one failed). It reads a body by Content-Length, chunked encoding, or until the connection ends (refused when a TLS connection was cut rather than closed, as nothing then shows the body is whole), whole or as it arrives, follows up to five redirects, and gives up on a connect or a wait after `timeout_ms`. The server handles one connection at a time, which is what a tool, a local dashboard or a test needs; threads come later in the roadmap.

Types: `Header`, `Url`, `Response`, `Plain`, `SystemTls`, `Streaming(T){`, `Client(T){`, `Request`, `Route`, `Router`, `Server`

| function | what it does |
| --- | --- |
| `parse_url(s: []u8) -> ?Url` | Parse `http://host[:port][/path]`; null for anything else. |
| `(method) header(self: *Self, name: []u8) -> ?[]u8` | A header value, case-insensitive; null when absent. |
| `(method) with_header(self: *mut Self, name: []u8, value: []u8)` | Add or replace a header (builder style). |
| `(method) ok(self: *Self) -> bool` |  |
| `reason_for(status: u16) -> []u8` | The standard reason phrase for a status. |
| `respond(status: u16, content_type: []u8, body: []u8) -> Response` | A response with a body and a content type. |
| `text(status: u16, body: []u8) -> Response` |  |
| `html(status: u16, body: []u8) -> Response` |  |
| `json(status: u16, body: []u8) -> Response` |  |
| `not_found() -> Response` |  |
| `redirect(location: []u8) -> Response` | A redirect to `location`. |
| `content_type_for(path: []u8) -> []u8` | The content type for a file name, by extension. |
| `read_response(r: *mut stream.Reader) -> !Response` | Read a full response from a reader over the connection. |
| `send_request(w: *mut stream.Writer, method: []u8, url: *Url, headers: *List(Header), body: []u8) -> !void` | Write a request; `headers` may add or override the defaults. |
| `(method) new() -> Plain` |  |
| `(method) new() -> SystemTls` |  |
| `(method) available() -> bool` | Does this system have a TLS library? Windows and macOS always do; Linux and the BSDs when OpenSSL's libssl (3 or 1.1) is installed. |
| `(method) problem() -> String` | Why the last TLS connection or call on this thread failed, in words: "the server's certificate has expired", "no such host". |
| `(method) header(self: *Self, name: []u8) -> ?[]u8` | A header value, case-insensitive; null when absent. |
| `(method) ok(self: *Self) -> bool` |  |
| `(method) next(self: *mut Self) -> !?String` | The next piece of the body; null once all of it has come. `error.Truncated` when the connection ends before the body does. |
| `(method) read_all(self: *mut Self) -> !String` | The rest of the body at once. |
| `(method) close(self: *mut Self)` | Closes the connection. |
| `client() -> Client(Plain)` | A client for `http://`, and for `https://` over the system's TLS (`SystemTls`). |
| `client_with(comptime T: type where T: Transport, tls: *mut T) -> Client(T)` | A client whose `https://` goes over `tls`, a TLS layer: a `Transport`, such as nxtls's. |
| `(method) open(self: *mut Self, method: []u8, url_text: []u8, headers: *List(Header), body: []u8) -> !Streaming(T)` | Sends a request and reads the response's status and headers, following redirects: a 303, and a 301 or 302 to a POST, turn into a GET without the body, and a redirect to another host goes without the Authorization and Cookie headers. The body is read from the `Streaming` as it arrives; close it when done. |
| `(method) send(self: *mut Self, method: []u8, url: []u8, headers: *List(Header), body: []u8) -> !Response` | A request, its response read whole, following redirects. |
| `(method) get(self: *mut Self, url: []u8) -> !Response` |  |
| `(method) post(self: *mut Self, url: []u8, content_type: []u8, body: []u8) -> !Response` |  |
| `request(method: []u8, url_text: []u8, headers: *List(Header), body: []u8) -> !Response` | Perform a request with a `client()`, following redirects: `https://` over the system's TLS. `error.InvalidInput` for a URL that is not http or https, `error.Unsupported` for `https://` on a system without a TLS library. |
| `get(url: []u8) -> !Response` |  |
| `post(url: []u8, content_type: []u8, body: []u8) -> !Response` |  |
| `(method) header(self: *Self, name: []u8) -> ?[]u8` |  |
| `(method) param(self: *Self, name: []u8) -> ?[]u8` | The value of a query parameter (`?a=1&b=2`), not decoded. |
| `read_request(r: *mut stream.Reader, peer: []u8) -> !?Request` | Read a request from a reader over the connection; null when the connection was closed before a request line. |
| `read_request_max(r: *mut stream.Reader, peer: []u8, max_body: usize) -> !?Request` | `read_request` with a body limit of `max_body` bytes (0: none). |
| `write_response(w: *mut stream.Writer, resp: *Response) -> !void` | Write a response with `Content-Length` and `Connection: close`. |
| `(method) new() -> Router` |  |
| `(method) route(self: *mut Self, method: []u8, path: []u8, handler: fn(*Request) -> Response)` |  |
| `(method) get(self: *mut Self, path: []u8, handler: fn(*Request) -> Response)` |  |
| `(method) post(self: *mut Self, path: []u8, handler: fn(*Request) -> Response)` |  |
| `(method) serve_static(self: *mut Self, root: []u8)` | Serve files under `root` for paths no route claims. |
| `(method) handle(self: *Self, req: *Request) -> Response` | The response for a request. |
| `static_file(root: []u8, path: []u8) -> Response` | A file under `root` for a request path, refusing `..`; `index.html` for directories. |
| `(method) bind(host: []u8, port: u16) -> !Server` |  |
| `(method) port(self: *Self) -> !u16` |  |
| `(method) serve_one(self: *Self, router: *Router, timeout_ms: i64) -> !void` | Accept one connection, answer one request, close. `error.Timeout` when nobody connects within `timeout_ms` (0 waits forever). |
| `(method) serve(self: *Self, router: *Router) -> !void` | Serve forever, one request at a time. |
| `(method) close(self: *mut Self)` |  |

## std.json

std.json: a JSON parser and serializer, written in Nexium. `import std.json` then `json.parse(text)`, `json.stringify(&value)`. Values are the `Json` enum below; arrays and objects own their children. Numbers are f64 (JSON has one number type); integers up to 2^53 round trip.

Types: `Member`, `Json`

| function | what it does |
| --- | --- |
| `null_value() -> Json` |  |
| `boolean(b: bool) -> Json` |  |
| `number(n: f64) -> Json` |  |
| `string(s: []u8) -> Json` |  |
| `array() -> Json` |  |
| `object() -> Json` |  |
| `push(v: *mut Json, own item: Json)` | Append to an array; does nothing when `v` is not an array. |
| `set(v: *mut Json, key: []u8, own value: Json)` | Set a key on an object (replacing an existing one); does nothing otherwise. |
| `get(v: *Json, key: []u8) -> ?*Json` | The member `key` of an object, or null. |
| `get_mut(v: *mut Json, key: []u8) -> ?*mut Json` | The member `key` of an object, mutable, or null. |
| `at_mut(v: *mut Json, i: usize) -> ?*mut Json` | Element `i` of an array, mutable, or null. |
| `at(v: *Json, i: usize) -> ?*Json` | Element `i` of an array, or null. |
| `len(v: *Json) -> usize` | Number of elements or members; 0 for scalars. |
| `is_null(v: *Json) -> bool` |  |
| `as_bool(v: *Json) -> ?bool` |  |
| `as_num(v: *Json) -> ?f64` |  |
| `as_str(v: *Json) -> ?[]u8` |  |
| `keys(v: *Json) -> List([]u8)` | Keys of an object in order; empty for anything else. |
| `parse(text: []u8) -> !Json` | Parse a JSON document. Trailing whitespace is allowed, anything else is an error. |
| `stringify(v: *Json) -> String` | Compact text: no whitespace. |
| `pretty(v: *Json, indent: usize) -> String` | Indented text, `indent` spaces per level. |

## std.lists

std.lists: generic helpers over slices and Lists, written in Nexium. `import std.lists` then `lists.sum(xs)`, `lists.map(f, xs)`, ... Functions take slices, so arrays, Lists, and slices all work; results that are new collections are returned as owned `List`s.

| function | what it does |
| --- | --- |
| `sum(comptime T: type, xs: []T) -> T` | Sum of the elements. |
| `min(comptime T: type where T: Ord, xs: []T) -> ?T` | Smallest element, or null when empty. |
| `max(comptime T: type where T: Ord, xs: []T) -> ?T` | Largest element, or null when empty. |
| `arg_max(comptime T: type where T: Ord, xs: []T) -> ?usize` | Position of the largest element, or null when empty. |
| `all(comptime T: type, xs: []T, pred: fn(T) -> bool) -> bool` | True when every element satisfies `pred`. |
| `any(comptime T: type, xs: []T, pred: fn(T) -> bool) -> bool` | True when some element satisfies `pred`. |
| `count_if(comptime T: type, xs: []T, pred: fn(T) -> bool) -> usize` | How many elements satisfy `pred`. |
| `filter(comptime T: type, xs: []T, pred: fn(T) -> bool) -> List(T)` | The elements that satisfy `pred`, in order. |
| `map(comptime T: type, comptime U: type, xs: []T, f: fn(T) -> U) -> List(U)` | `f` applied to every element. |
| `fold(comptime T: type, comptime A: type, xs: []T, init: A, f: fn(A, T) -> A) -> A` | Left fold: `f(f(f(init, x0), x1), x2)`. |
| `find(comptime T: type, xs: []T, pred: fn(T) -> bool) -> ?T` | First element satisfying `pred`, or null. |
| `position(comptime T: type, xs: []T, pred: fn(T) -> bool) -> ?usize` | Position of the first element satisfying `pred`, or null. |
| `reversed(comptime T: type, xs: []T) -> List(T)` | A reversed copy. |
| `dedup(comptime T: type where T: Eq, xs: []T) -> List(T)` | Adjacent duplicates removed (sort first for global dedup). |
| `take(comptime T: type, xs: []T, n: usize) -> []T` | The first `n` elements (or all when shorter). |
| `drop(comptime T: type, xs: []T, n: usize) -> []T` | Everything after the first `n` elements. |
| `window_starts(len: usize, size: usize) -> List(usize)` | Consecutive windows of `size`, as start indices; `for i in windows(xs, 3)` then `xs[i..i+3]`. |
| `zip(comptime A: type, comptime B: type, a: []A, b: []B) -> List((A, B))` | Pairs (a[i], b[i]) up to the shorter length. |
| `repeat(comptime T: type, xs: []T, times: usize) -> List(T)` | Elements repeated `times` times in sequence. |
| `starts_with(comptime T: type where T: Eq, xs: []T, prefix: []T) -> bool` | True when `xs` starts with `prefix`. |

## std.log

std.log: leveled, structured logging, written in Nexium: a message and named fields, as a line of text a person reads or a line of JSON a machine reads, on stderr. `import std.log` then: var l = log.Logger.new(log.Level.Info) l.info("listening", [log.int("port", 8080), log.str("host", host)][..]) l.warn("slow answer", [log.float("seconds", 2.5)][..]) l.debug("not shown: below the logger's level", []) var j = log.Logger.json(log.Level.Debug)      // one JSON object a line let level = log.parse_level(env.get_or("LOG", "info")) orelse log.Level.Info var request = l.with([log.str("id", id)][..]) // every line carries the id A text line is `2026-09-25T18:04:05.120Z INFO listening port=8080 host=0.0.0.0`, a value with a space, `=` or a quote in quotes; a JSON line is `{"time":"2026-09-25T18:04:05.120Z","level":"info","msg":"listening", "port":8080,"host":"0.0.0.0"}`.

Types: `Level`, `Field`, `Logger`

| function | what it does |
| --- | --- |
| `parse_level(text: []u8) -> ?Level` | The level named by `debug`, `info`, `warn` (or `warning`) or `error`, in any case; null for anything else. |
| `str(key: []u8, value: []u8) -> Field` | A text field. |
| `int(key: []u8, value: i64) -> Field` | A whole-number field. |
| `float(key: []u8, value: f64) -> Field` | A floating-point field; a value JSON cannot hold (NaN, the infinities) is written as text. |
| `flag(key: []u8, value: bool) -> Field` | A true-or-false field. |
| `(method) new(level: Level) -> Logger` | Text lines on stderr, from `level` up. |
| `(method) json(level: Level) -> Logger` | JSON lines on stderr, from `level` up. |
| `(method) keeping(level: Level, json: bool) -> Logger` | A logger that keeps its lines in `kept` rather than writing them: for tests of what a program logs. |
| `(method) with(self: *Self, fields: []Field) -> Logger` | A logger like this one whose lines carry `fields` too. |
| `(method) enabled(self: *Self, level: Level) -> bool` | Does this logger write lines of `level`? |
| `(method) log(self: *mut Self, level: Level, msg: []u8, fields: []Field)` | A line at `level`, if the logger writes that level. |
| `(method) debug(self: *mut Self, msg: []u8, fields: []Field)` |  |
| `(method) info(self: *mut Self, msg: []u8, fields: []Field)` |  |
| `(method) warn(self: *mut Self, msg: []u8, fields: []Field)` |  |
| `(method) err(self: *mut Self, msg: []u8, fields: []Field)` | A line at `Level.Error` (`error` itself is a keyword). |

## std.net

std.net: TCP and UDP with addresses, written in Nexium over the `net.*` primitives. `import std.net` then: var c = try net.TcpStream.connect("example.com", 80) try c.send("GET / HTTP/1.0\r\nHost: example.com\r\n\r\n") let reply = try c.recv_all()                 // until the peer closes c.close() var l = try net.TcpListener.bind("127.0.0.1", 8080) while true { var conn = try l.accept() var r = conn.reader()                     // a std.stream Reader let line = try r.read_line() try conn.send("ok\n") conn.close() } var u = try net.UdpSocket.bind("0.0.0.0", 0) try u.send_to("127.0.0.1", 9000, "ping") let d = try u.recv_from(1500)                 // d.data, d.from Every call blocks; timeouts are per socket (`set_timeout`, milliseconds, 0 waits forever) and expire with `error.Timeout`. `recv` returns an empty String when the peer has closed. Errors: `NotFound` (name lookup), `ConnectionRefused`, `Timeout`, `IoError`.

Types: `Addr`, `TcpStream`, `TcpListener`, `Datagram`, `UdpSocket`

| function | what it does |
| --- | --- |
| `parse_addr(s: []u8) -> ?Addr` | Split `host:port` or `[v6]:port`; null when there is no valid port. |
| `port_of(s: []u8) -> ?u16` | The port at the end of `host:port`, or null. |
| `resolve(host: []u8) -> !List(String)` | The addresses a name resolves to, numeric, in resolver order. |
| `(method) connect(host: []u8, port: u16) -> !TcpStream` | Connect with a 10 second timeout. |
| `(method) connect_timeout(host: []u8, port: u16, timeout_ms: i64) -> !TcpStream` | Connect; `timeout_ms` 0 waits as long as the OS does. |
| `(method) from_socket(sock: i64) -> TcpStream` | Wrap a socket from `net.accept` or `net.connect`. |
| `(method) set_timeout(self: *mut Self, ms: i64)` | The receive timeout in milliseconds; 0 waits forever. |
| `(method) send(self: *Self, data: []u8) -> !void` |  |
| `(method) recv(self: *Self, n: usize) -> !String` | Up to `n` bytes; empty when the peer has closed. |
| `(method) recv_all(self: *Self) -> !String` | Everything until the peer closes. |
| `(method) peer(self: *Self) -> !String` | The remote address as `ip:port`. |
| `(method) local(self: *Self) -> !String` | The local address as `ip:port`. |
| `(method) reader(self: *Self) -> stream.Reader` | A buffered reader over the socket (lines, chunks), with its receive timeout; it does not own the socket. |
| `(method) writer(self: *Self) -> stream.Writer` | A buffered writer over the socket; flush it before waiting for a reply. |
| `(method) close(self: *mut Self)` |  |
| `(method) bind(host: []u8, port: u16) -> !TcpListener` | Bind and listen; port 0 picks a free port (see `local`). |
| `(method) local(self: *Self) -> !String` | The bound address as `ip:port`. |
| `(method) port(self: *Self) -> !u16` | The bound port. |
| `(method) accept(self: *Self) -> !TcpStream` | Wait for a connection. |
| `(method) accept_timeout(self: *Self, timeout_ms: i64) -> !TcpStream` | Wait up to `timeout_ms` for a connection (`error.Timeout` otherwise). |
| `(method) close(self: *mut Self)` |  |
| `(method) bind(host: []u8, port: u16) -> !UdpSocket` | Bind; port 0 picks a free port. |
| `(method) set_timeout(self: *mut Self, ms: i64)` |  |
| `(method) local(self: *Self) -> !String` |  |
| `(method) port(self: *Self) -> !u16` |  |
| `(method) send_to(self: *Self, host: []u8, port: u16, data: []u8) -> !void` |  |
| `(method) recv_from(self: *Self, n: usize) -> !Datagram` | One datagram of at most `n` bytes. |
| `(method) close(self: *mut Self)` |  |

## std.num

std.num: integer utilities, written in Nexium. `import std.num` then `num.gcd(12, 18)`, `num.clamp(x, 0, 10)`, ... The `math` namespace (sqrt, sin, pow on floats, ...) is a compiler builtin; this module covers what is naturally integer work.

| function | what it does |
| --- | --- |
| `gcd(a: u64, b: u64) -> u64` | Greatest common divisor (Euclid); gcd(0, 0) is 0. |
| `lcm(a: u64, b: u64) -> u64` | Least common multiple; lcm(0, n) is 0. |
| `clamp(x: i64, lo: i64, hi: i64) -> i64` | `x` limited to `lo..=hi`. |
| `abs_diff(a: i64, b: i64) -> u64` | |a - b| without overflow on the way. |
| `pow(base: u64, exp: u32) -> ?u64` | `base` to the `exp`, by squaring; null on overflow. |
| `isqrt(n: u64) -> u64` | Integer square root: the largest `r` with `r * r <= n`. |
| `is_prime(n: u64) -> bool` | Trial division; fine for the sizes people type by hand. |
| `factors(n: u64) -> List(u64)` | The prime factors of `n` with multiplicity, ascending. |
| `digits(n: u64) -> List(u8)` | Decimal digits of `n`, most significant first. |
| `digit_sum(n: u64) -> u64` | Sum of the decimal digits. |
| `to_base(n: u64, base: u64) -> String` | `n` in base 2..36, upper-case digits. |
| `from_base(text: []u8, base: u64) -> ?u64` | Parse in base 2..36 (either case); null on an invalid digit or overflow. |
| `round_up(n: u64, m: u64) -> u64` | Round up to a multiple of `m` (m > 0). |
| `is_power_of_two(n: u64) -> bool` | True for 1, 2, 4, 8, ... |
| `next_power_of_two(n: u64) -> u64` | The smallest power of two >= n (n <= 2^63). |
| `popcount(n: u64) -> u32` | Number of set bits. |

## std.path

std.path: paths as text, written in Nexium: joining, splitting, comparing and normalizing them, without touching the file system (std.fs does that). `import std.path` then: let cfg = path.join(path.parent(argv0), "app.toml") let ext = path.extension("notes.tar.gz")           // "gz" let n = path.normalize("a/./b/../c")               // "a/c" let r = path.relative("src/app", "src/lib/x.nx")   // "../lib/x.nx" for part in path.components("/usr/local/bin") { }  // "/", "usr", "local", "bin" Paths are byte strings. `/` and `\` both separate components on every platform, and a result uses `/` unless its input used `\`. A path is absolute when it starts at a root: `/`, `\`, a drive (`C:\`, `C:/`) or a share (`\\server`). std.fs keeps its path functions, which call these.

| function | what it does |
| --- | --- |
| `root(p: []u8) -> []u8` | The root a path starts with (`/`, `C:\`, `\\`), or `` for a relative path. |
| `is_absolute(p: []u8) -> bool` | Does the path start at a root (`/x`, `C:\x`, `C:/x`, `\\server`)? |
| `is_relative(p: []u8) -> bool` | Is the path relative to some directory: not `is_absolute`? |
| `components(p: []u8) -> List([]u8)` | The root, if any, then each component, separators left out: `/usr/local/` -> `/`, `usr`, `local`; `a/./b` -> `a`, `.`, `b`. |
| `join(dir: []u8, name: []u8) -> String` | `dir/name`; a separator is added only when needed, and an absolute `name` replaces `dir`. |
| `join_all(parts: [][]u8) -> String` | Every part joined in turn, as `join` joins two: `join_all(parts[..])`. |
| `parent(p: []u8) -> []u8` | Everything before the last separator: `a/b/c.txt` -> `a/b`, `c.txt` -> ``, `/c.txt` -> `/`. |
| `base_name(p: []u8) -> []u8` | The last component: `a/b/c.txt` -> `c.txt`. |
| `extension(p: []u8) -> []u8` | The extension without the dot: `a/b.tar.gz` -> `gz`, `Makefile` -> ``. |
| `stem(p: []u8) -> []u8` | The base name without its extension: `a/b.tar.gz` -> `b.tar`. |
| `with_extension(p: []u8, ext: []u8) -> String` | The path with its extension replaced (or added): `a/b.txt`, `md` -> `a/b.md`. |
| `normalize(p: []u8) -> String` | Collapse `.` and `..` components and repeated separators: `a/./b/../c//d` -> `a/c/d`. A leading `..` is kept; `..` at a root is dropped; an empty result is `.`. |
| `strip_prefix(p: []u8, prefix: []u8) -> ?[]u8` | What follows `prefix` in `p`, compared component by component (so `a/bc` does not start with `a/b`): `a/b/c/d`, `a/b` -> `c/d`; null when `p` does not start with `prefix`. Neither is normalized first. |
| `starts_with(p: []u8, prefix: []u8) -> bool` | Does `p` start with `prefix`, component by component? |
| `relative(from: []u8, to: []u8) -> ?String` | The path that leads from the directory `from` to `to`, both normalized: `src/app`, `src/lib/x.nx` -> `../lib/x.nx`, and `.` for the same place. Null when no such path can be written: one is absolute and the other not, their roots differ, or `from` climbs out through a `..`. |
| `to_slash(p: []u8) -> String` | Every `\` as `/`. |
| `to_native(p: []u8) -> String` | The separators the platform spells paths with: `\` on Windows, `/` elsewhere. |

## std.process

std.process: run programs and capture what they print, or talk to them while they run, written in Nexium over the `process.*` primitives. `import std.process` then: let out = try process.run(["git", "status", "--short"]) if out.ok() { print("{}", .{out.stdout}) } let r = try process.run_with(["sort"], process.Options{ .stdin = "b\na\n", .cwd = "" }) let sh = try process.shell("echo hi")          // cmd /C on Windows, sh -c elsewhere The child inherits the environment. `error.IoError` when the program cannot be started; a non-zero exit is reported in `code`, not as an error. The input is written as the child takes it while its output is read as it comes, so a program that writes before it reads its input, however much, finishes as it would in a terminal. A program can also run alongside, talked to while it runs: var py = try process.start(["python", "-i", "-q"]) try py.stdin.write_line("print(6 * 7)") let answer = try py.stdout.read_line()         // "42" let status = try py.wait()                     // closes its input first var build = try process.start_with(["make"], process.Start{ .stderr = process.Stdio.Merge }) while true { let line = (try build.stdout.read_line()) orelse break println("{}", .{line}) } Output a program writes while this one waits for something else (its input to go in, the other stream, its end) is kept until read, so it never stalls on a full pipe. `set_timeout` bounds every wait (`error.Timeout`); `kill` and `terminate` end it early. Exit codes and signals have names (`EXIT_USAGE`, `SIGTERM`, `exit_name`, `signal_name`), and `trap_signals` lets this program catch Ctrl-C and SIGTERM to finish cleanly (`caught`, `wait_signal`).

Types: `Output`, `Options`, `Status`, `Stdio`, `Start`, `PipeReader`, `PipeWriter`, `Child`

| function | what it does |
| --- | --- |
| `(method) ok(self: *Self) -> bool` |  |
| `(method) text(self: *Self) -> []u8` | stdout without a trailing newline. |
| `run_with(argv: [][]u8, opts: Options) -> !Output` |  |
| `run(argv: [][]u8) -> !Output` | Run and capture, inheriting the working directory, with no stdin. |
| `shell(command: []u8) -> !Output` | Run a command line through the platform shell. |
| `exit_name(code: i32) -> []u8` | What an exit code says by convention (sysexits and the shells): "usage" for 64, "not found" for 127; empty for a code with no common meaning. |
| `signal_name(sig: i32) -> []u8` | "SIGTERM" for 15; empty for a number without a portable name. |
| `(method) ok(self: *Self) -> bool` |  |
| `(method) text(self: *Self) -> String` | `exit 0 (success)`, `exit 3`, `signal 15 (SIGTERM)`. |
| `(method) read_line(self: *mut Self) -> !?String` | The next line without its `\n` (or `\r\n`), waiting for it; null at the end of the output. After a timeout, what came of the line so far is kept for the next call. |
| `(method) read(self: *mut Self, n: usize) -> !String` | What the program has written and was not read yet, up to `n` bytes, waiting for some; empty at the end of the output. |
| `(method) read_all(self: *mut Self) -> !String` | Everything to the end of the output: until the program closes it, usually by ending. |
| `(method) write(self: *mut Self, data: []u8) -> !void` | Write all of `data` as the program takes it. `error.IoError` once it no longer reads (it closed its input or ended). |
| `(method) write_line(self: *mut Self, line: []u8) -> !void` | `line` and a `\n`, in one write. |
| `(method) close(self: *mut Self)` | End the input: the program reads to its end. |
| `(method) pid(self: *Self) -> i64` | The operating system's number for it. |
| `(method) set_timeout(self: *mut Self, ms: i64)` | Every read and write waits at most `ms`, then fails with `error.Timeout`; 0 waits for ever (the default). |
| `(method) wait(self: *mut Self) -> !Status` | Close its input and wait for it to end. What it writes meanwhile is kept for the readers. |
| `(method) wait_for(self: *mut Self, ms: i64) -> !?Status` | How it ended, waiting at most `ms` for it (0: for ever); null when it still runs then. Its input stays open. |
| `(method) try_wait(self: *mut Self) -> !?Status` | How it ended, without waiting; null while it runs. |
| `(method) signal(self: *mut Self, sig: i32) -> !void` | Send it a signal (`SIGTERM`, `SIGINT` and the others). On Windows, which has none, every signal ends it at once, with exit code 128 + the signal, and `wait` reports the signal. A program that already ended is left alone. |
| `(method) terminate(self: *mut Self) -> !void` | Ask it to end: SIGTERM. |
| `(method) kill(self: *mut Self) -> !void` | End it at once: SIGKILL. |
| `(method) finish(self: *mut Self) -> !Output` | Close its input, read all it writes and wait for it: what `run` gives, for a program already talked to. |
| `(method) close(self: *mut Self)` | Let it go: its pipes close and what was not read is dropped. A program still running goes on by itself. |
| `start(argv: [][]u8) -> !Child` | Start a program alongside this one, with its input and output as pipes to this one and its errors where this program's go. `error.NotFound` when there is no such program. |
| `start_with(argv: [][]u8, how: Start) -> !Child` | Start a program as `how` says. |
| `trap_signals()` | Keep SIGINT (Ctrl-C), SIGTERM and SIGHUP from ending this program: each is queued instead, for `caught` and `wait_signal`, so the program can finish what it was doing. On Windows: Ctrl-C (`SIGINT`), Ctrl-Break (`SIGBREAK`), the console closing (`SIGHUP`) and logoff or shutdown (`SIGTERM`); after those three the system ends the program within seconds. |
| `caught() -> i32` | The next signal caught and not taken yet, or 0; never waits. |
| `wait_signal(ms: i64) -> i32` | Wait for a caught signal; 0 when `ms` pass first (0: waits for ever). |

## std.regex

std.regex: regular expressions without backtracking, written in Nexium. `import std.regex` then: let re = try regex.compile("(\\w+)@(\\w+)\\.com") if re.is_match(text) { ... } if let m = re.find(text) { println("{} at {}", .{m.text(), m.start}) } for m in re.find_all(text) { println("{}", .{m.group(1).?}) } let out = re.replace_all(text, "$2:$1") let parts = try regex.compile(",\\s*") for p in parts.split("a, b,c") { ... } Syntax: literals, `.` (any byte but newline), classes `[a-z]` `[^...]`, `\d \w \s \D \W \S \b \B`, escapes `\. \\ \n \t \r`, anchors `^ $`, groups `(...)` and `(?:...)`, alternation `|`, repeats `* + ? {n} {n,} {n,m}` and their lazy forms `*? +? ??`. Matching is a Pike VM (Thompson's NFA simulation), so every search is linear in the text and the pattern; there are no back-references. Patterns and text are bytes.

Types: `Regex`, `Match`

| function | what it does |
| --- | --- |
| `compile(pattern: []u8) -> !Regex` | Compile a pattern; `error.InvalidInput` when it is malformed. |
| `is_match(pattern: []u8, text: []u8) -> !bool` | Does `pattern` match anywhere in `text`? (Compiles every call.) |
| `(method) text(self: *Self) -> []u8` | The matched bytes. |
| `(method) group(self: *Self, i: usize) -> ?[]u8` | Group `i` (0 is the whole match); null when the group did not participate. |
| `(method) group_count(self: *Self) -> usize` | The number of groups, counting group 0. |
| `(method) find_at(self: *Self, text: []u8, from: usize) -> ?Match` | The first match at or after byte `from`. |
| `(method) find(self: *Self, text: []u8) -> ?Match` | The first match in `text`. |
| `(method) is_match(self: *Self, text: []u8) -> bool` | Does the pattern match anywhere in `text`? |
| `(method) find_all(self: *Self, text: []u8) -> List(Match)` | Every non-overlapping match, left to right. |
| `(method) replace_all(self: *Self, text: []u8, repl: []u8) -> String` | Replace every match. In `repl`, `$0`..`$9` insert groups and `$$` is a dollar sign. |
| `(method) split(self: *Self, text: []u8) -> List([]u8)` | The pieces of `text` between matches. |

## std.set

std.set: a set of values, written in Nexium over `Map(T, bool)`: its elements are the types a Map takes as keys (integers, bool, char, `[]u8`, `String`). `import std.set` then: var seen = set.of(i64) if seen.add(id) { println("new: {}", .{id}) } let common = set.intersection(i64, &a, &b) `add`, `contains` and `remove` take a value like a Map's key: a copy, or for a `String` set an owned string (`name.clone()` keeps yours).

Types: `Set(T){`

| function | what it does |
| --- | --- |
| `of(comptime T: type) -> Set(T)` | An empty set. |
| `(method) add(self: *mut Self, own x: T) -> bool` | Adds `x`; true when it was not already there. |
| `(method) contains(self: *Self, x: T) -> bool` |  |
| `(method) remove(self: *mut Self, x: T) -> bool` | Takes `x` out; true when it was there. |
| `(method) len(self: *Self) -> usize` |  |
| `(method) is_empty(self: *Self) -> bool` |  |
| `(method) clear(self: *mut Self)` |  |
| `(method) items(self: *Self) -> List(T)` | The elements, in the order each was first added. |
| `union(comptime T: type, a: *Set(T), b: *Set(T)) -> Set(T)` | The values in `a` or `b`. |
| `intersection(comptime T: type, a: *Set(T), b: *Set(T)) -> Set(T)` | The values in both `a` and `b`. |
| `difference(comptime T: type, a: *Set(T), b: *Set(T)) -> Set(T)` | The values in `a` that are not in `b`. |
| `is_subset(comptime T: type, a: *Set(T), b: *Set(T)) -> bool` | Whether every value of `a` is in `b`. |

## std.sort

std.sort: sorting by a comparison of your own, stable sorting, and searching sorted slices, written in Nexium over the slice's `swap`. `import std.sort` then: sort.by(Point, points[..], |a: *Point, b: *Point| -> bool { return a.x < b.x }) sort.stable_by(Task, tasks[..], by_priority)     // equal elements keep their order sort.by_key(Point, i64, points[..], |p: *Point| -> i64 { return p.y }) let at = sort.binary_search(i64, xs[..], 42)      // a position of 42, or null `by` is a heapsort: in place, O(n log n) comparisons whatever the input, and not stable. The stable sorts merge-sort the positions and then move each element into place along the cycles of the permutation, so no element is ever copied and any element type sorts. `xs.sort()` sorts by `<` without a comparison.

| function | what it does |
| --- | --- |
| `by(comptime T: type, xs: []mut T, less: fn(*T, *T) -> bool)` | Sorts `xs` so that `less(b, a)` holds for no `a` before `b`: ascending by `less`. The order of equal elements is not kept. |
| `stable_by(comptime T: type, xs: []mut T, less: fn(*T, *T) -> bool)` | Sorts `xs` ascending by `less`, keeping equal elements in the order they had. |
| `by_key(comptime T: type, comptime K: type where K: Ord, xs: []mut T, key: fn(*T) -> K)` | Sorts `xs` ascending by `key` of each element, keeping elements with equal keys in the order they had; `key` is called once per element. |
| `is_sorted(comptime T: type where T: Ord, xs: []T) -> bool` | Whether `xs` is ascending by `<`. |
| `is_sorted_by(comptime T: type, xs: []T, less: fn(*T, *T) -> bool) -> bool` | Whether `xs` is ascending by `less`. |
| `lower_bound(comptime T: type where T: Ord, xs: []T, x: T) -> usize` | In an ascending `xs`, the first position whose element is not less than `x`: where `x` would go before any equal to it. |
| `upper_bound(comptime T: type where T: Ord, xs: []T, x: T) -> usize` | In an ascending `xs`, the first position whose element is greater than `x`: where `x` would go after any equal to it. |
| `binary_search(comptime T: type where T: Ord, xs: []T, x: T) -> ?usize` | In an ascending `xs`, a position holding `x`, or null. |

## std.stream

std.stream: buffered readers and writers over files and the standard streams, written in Nexium. `import std.stream` then: var r = try stream.Reader.open("big.log") while true {                                    // no whole-file allocation let line = (try r.read_line()) orelse break ... } r.close() var w = try stream.Writer.open("out.txt") try w.write_line("hello") try w.close()                                    // flushes, then closes var input = stream.Reader.stdin() var out = stream.Writer.stdout() try stream.copy(&mut input, &mut out) try out.flush() Readers buffer 64 KB at a time; writers gather output and flush when the buffer fills, on `flush`, and on `close`. A Writer must be flushed or closed before the program ends, or buffered output is lost.

Types: `Reader`, `Writer`

| function | what it does |
| --- | --- |
| `(method) open(path: []u8) -> !Reader` | Open a file for reading. |
| `(method) stdin() -> Reader` | Standard input. |
| `(method) from_handle(handle: i64) -> Reader` | Wrap a handle from `io.open`; `close` will not close it. |
| `(method) from_socket(sock: i64) -> Reader` | Wrap a connected socket from `net.connect` or `net.accept`; `close` will not close it. |
| `(method) from_socket_timeout(sock: i64, timeout_ms: i64) -> Reader` | `from_socket`, each read waiting at most `timeout_ms` for data (`error.Timeout` then; 0 waits forever). |
| `(method) read_line(self: *mut Self) -> !?String` | The next line without its `\n` (or `\r\n`); null at end of input. |
| `(method) read_line_max(self: *mut Self, max: usize) -> !?String` | `read_line`, refusing a line longer than `max` bytes with `error.TooLarge` rather than buffering it (0: no limit). Input from a peer that need not end its lines calls for one. |
| `(method) read(self: *mut Self, n: usize) -> !String` | Up to `n` bytes; empty at end of input. |
| `(method) read_all(self: *mut Self) -> !String` | Everything that is left. |
| `(method) close(self: *mut Self)` | Release the file (the standard streams stay open). |
| `(method) open(path: []u8) -> !Writer` | Create or replace a file. |
| `(method) append(path: []u8) -> !Writer` | Open a file for appending. |
| `(method) stdout() -> Writer` |  |
| `(method) stderr() -> Writer` |  |
| `(method) from_handle(handle: i64) -> Writer` | Wrap a handle from `io.open`; `close` will not close it. |
| `(method) from_socket(sock: i64) -> Writer` | Wrap a connected socket; `close` will not close it. |
| `(method) write(self: *mut Self, data: []u8) -> !void` |  |
| `(method) write_line(self: *mut Self, data: []u8) -> !void` |  |
| `(method) flush(self: *mut Self) -> !void` | Hand buffered output to the handle. |
| `(method) close(self: *mut Self) -> !void` | Flush, then release the file (the standard streams stay open). |
| `copy(r: *mut Reader, w: *mut Writer) -> !usize` | Copy everything from a reader to a writer; the number of bytes moved. |

## std.strings

std.strings: text utilities on `[]u8` and `String`, written in Nexium. `import std.strings` then `strings.join(parts, ", ")`. The core methods (`len`, `split`, `trim`, `find`, `starts_with`, `parse_int`, ...) are compiler builtins; this module adds what is naturally written in the language itself. Slices returned here point into the argument they were cut from; `String` results are owned by the caller.

| function | what it does |
| --- | --- |
| `join(parts: [][]u8, sep: []u8) -> String` | Concatenate `parts` with `sep` between them. |
| `repeat(s: []u8, n: usize) -> String` | `s` repeated `n` times. |
| `pad_left(s: []u8, width: usize, fill: u8) -> String` | Left-pad with `fill` to at least `width` bytes. |
| `pad_right(s: []u8, width: usize, fill: u8) -> String` | Right-pad with `fill` to at least `width` bytes. |
| `center(s: []u8, width: usize, fill: u8) -> String` | Center in `width` bytes, extra fill on the right. |
| `count(s: []u8, needle: []u8) -> usize` | How many non-overlapping times `needle` occurs in `s`. |
| `replace(s: []u8, from: []u8, to: []u8) -> String` | Every occurrence of `from` replaced by `to`. |
| `index_from(s: []u8, needle: []u8, start: usize) -> ?usize` | Position of `needle` at or after `start`. |
| `last_index(s: []u8, needle: []u8) -> ?usize` | Position of the last occurrence of `needle`. |
| `strip_prefix(s: []u8, prefix: []u8) -> ?[]u8` | `s` without a leading `prefix`, or null when it does not start with it. |
| `strip_suffix(s: []u8, suffix: []u8) -> ?[]u8` | `s` without a trailing `suffix`, or null when it does not end with it. |
| `trim_left(s: []u8) -> []u8` | Leading ASCII whitespace removed. |
| `trim_right(s: []u8) -> []u8` | Trailing ASCII whitespace removed. |
| `is_blank(s: []u8) -> bool` | True when `s` is empty or only ASCII whitespace. |
| `split_whitespace(s: []u8) -> List([]u8)` | Split on runs of ASCII whitespace; no empty pieces. |
| `to_upper(s: []u8) -> String` | ASCII letters upper-cased; other bytes unchanged. |
| `to_lower(s: []u8) -> String` | ASCII letters lower-cased; other bytes unchanged. |
| `capitalize(s: []u8) -> String` | First ASCII letter upper-cased. |
| `reverse(s: []u8) -> String` | Bytes in reverse order (bytes, not code points). |
| `split_once(s: []u8, sep: []u8) -> ?([]u8, []u8)` | Cut at the first `sep`: (before, after), or null when `sep` is absent. |
| `ellipsize(s: []u8, max: usize) -> String` | Truncate to `max` bytes, appending `...` when something was cut. |

## std.testing

std.testing: conveniences for `test` blocks, written in Nexium. `import std.testing` then, inside a test: testing.expect_approx(area, 3.14159, 0.001) testing.expect_err(i32, parse("nope")) testing.expect_contains(output, "42 items") testing.expect_lines(rendered, expected)    // reports the first differing line testing.expect_snapshot("report", rendered)  // compares to snapshots/report.txt try testing.snapshot("report", rendered)     // the same, as an error union fn lists(r: *mut testing.Rng) -> List(i64) { ... r.size(20) ... r.int(-50, 50) ... } testing.check(List(i64), lists, |xs: *List(i64)| -> bool { ... })   // a property Snapshots live in `snapshots/<name>.txt` under the current directory. A missing file is written and the test passes; a mismatch fails with the first differing line and how to accept the new output. Set `NX_UPDATE_SNAPSHOTS=1` to rewrite them all. A property is checked on 100 random values (`NX_CASES` for more, `NX_SEED` for another seed). Generators draw from a `Rng`, which keeps every choice, so a failing case is shrunk by making it again from fewer and lower choices while it still fails: the report is about the smallest case found, with no shrinking code of the generator's own. `search` is the driver without the panic; the compiler's fuzzer runs on it.

Types: `Rng`, `Search`, `Failure`

| function | what it does |
| --- | --- |
| `approx(a: f64, b: f64, eps: f64) -> bool` | Are two floats within `eps` of each other? |
| `expect_approx(a: f64, b: f64, eps: f64)` | Panics unless `a` and `b` are within `eps`. |
| `is_err(comptime T: type, own r: !T) -> bool` | Did the call fail? (Any error counts.) |
| `expect_err(comptime T: type, own r: !T)` | Panics unless the result is an error. |
| `expect_error(comptime T: type, own r: !T, err: error)` | Panics unless the result is exactly `err`. |
| `expect_contains(hay: []u8, needle: []u8)` | Panics unless `hay` contains `needle`. |
| `expect_lines(actual: []u8, expected: []u8)` | Compares line by line; panics naming the first line that differs. |
| `snapshot_in(dir: []u8, name: []u8, actual: []u8) -> !void` | Compare `actual` to `<dir>/<name>.txt`; write it when missing or when NX_UPDATE_SNAPSHOTS is set. |
| `snapshot(name: []u8, actual: []u8) -> !void` | `snapshot_in("snapshots", name, actual)`. |
| `expect_snapshot_in(dir: []u8, name: []u8, actual: []u8)` | `snapshot_in`, in the `expect_` form: a mismatch fails the test naming the file, the first differing line and how to accept the new output; a file that cannot be read or written fails it too, instead of returning an error for the test to handle. |
| `expect_snapshot(name: []u8, actual: []u8)` | `expect_snapshot_in("snapshots", name, actual)`. |
| `rng(seed: u64) -> Rng` |  |
| `(method) next(self: *mut Self) -> u64` | The next choice: 64 random bits (splitmix64), or the kept one when replaying (0 past the end of them). Shrinks toward 0. |
| `(method) below(self: *mut Self, n: u64) -> u64` | A number below `n` (0 when `n` is 0); shrinks toward 0. |
| `(method) pick(self: *mut Self, n: usize) -> usize` | An index into `n` things; shrinks toward the first. |
| `(method) flip(self: *mut Self) -> bool` | True or false alike; shrinks toward false. |
| `(method) int(self: *mut Self, lo: i64, hi: i64) -> i64` | An integer from `lo` to `hi`, both included; shrinks toward 0, or toward the end nearer to it when the range does not hold 0. |
| `(method) float(self: *mut Self) -> f64` | A float from 0 up to 1, 1 left out; shrinks toward 0. |
| `(method) size(self: *mut Self, max: usize) -> usize` | A length up to `max`, short ones likelier; shrinks toward 0. |
| `(method) bytes(self: *mut Self, max: usize) -> String` | Bytes of any value, up to `max` of them. |
| `(method) ascii(self: *mut Self, max: usize) -> String` | Printable ASCII, up to `max` bytes; shrinks toward `a`s. |
| `(method) text(self: *mut Self, max: usize) -> String` | UTF-8 text of up to `max` characters: ASCII mostly, with accented Latin, Greek, Cyrillic, CJK, emoji and combining marks among it. |
| `replay(comptime T: type, choices: []u64, gen: fn(*mut Rng) -> T) -> T` | The value `gen` makes from kept choices. |
| `search(comptime T: type, how: Search, gen: fn(*mut Rng) -> T, holds: fn(*T) -> bool) -> ?Failure` | Look for a case where `holds` is false among the values `gen` makes, and shrink the first found; null when every case held. `holds` answers with false; a panic in it ends the test as it stands, unshrunk. |
| `seed() -> u64` | The seed `check` starts from: `NX_SEED` when it is set, else 1, so a run repeats. |
| `check(comptime T: type, gen: fn(*mut Rng) -> T, holds: fn(*T) -> bool)` | Check that `holds` is true of every value `gen` makes: 100 of them (`NX_CASES` sets how many, `NX_SEED` the seed). A failure is shrunk to the smallest failing case found and fails the test with the seed that repeats it; `check_show` prints the value too. |
| `check_show(comptime T: type, gen: fn(*mut Rng) -> T, holds: fn(*T) -> bool, show: fn(*T) -> String)` | `check`, with the smallest failing value written by `show` in the failure. |

## std.text

std.text: UTF-8 text by code point and by grapheme cluster, written in Nexium, with the Unicode tables it needs. `import std.text` then: let n = text.char_count("héllo")            // 5 code points, not 6 bytes for c in text.scalars("héllo") { ... }       // chars (Unicode scalars) let g = text.grapheme_count("🇯🇵 e\u{301}")   // 3: a flag, a space, an é for g in text.graphemes(s) { ... }           // what a reader counts as characters let w = text.width("日本語")                   // 6 columns on a terminal let cell = text.pad_right(text.truncate_width(name, 20), 20) let s = text.to_upper("straße")               // "STRASSE" let same = text.eq_ignore_case("Straße", "STRASSE") let t = text.truncate("héllo wörld", 5)       // "héllo", never mid-character Strings are bytes; this module reads them as UTF-8, tolerating bad input (an invalid byte decodes as U+FFFD and advances one byte). Grapheme clusters are Unicode's extended grapheme clusters (UAX #29): a letter and its marks, a Hangul syllable, an emoji sequence joined by ZWJ, a flag, an Indic conjunct. Case mapping is Unicode's full mapping for every script, without the rules that depend on a language (Turkish and Lithuanian i); `to_lower` writes a final sigma where a word ends. `width` counts terminal columns as terminals draw them: East Asian wide and fullwidth characters and emoji sequences take two, marks and zero-width characters none. The tables at the end are generated by scripts/unicode_tables.py (`UNICODE_VERSION` says from which version).

Types: `Decoded`

| function | what it does |
| --- | --- |
| `decode_at(s: []u8, i: usize) -> Decoded` | Decode the code point starting at byte `i`. Invalid input yields U+FFFD with length 1 so callers always make progress. |
| `push(out: *mut String, cp: u32)` | Append a code point as UTF-8. |
| `encode(cp: u32) -> String` | A code point as a String. |
| `is_valid(s: []u8) -> bool` | Is the text well-formed UTF-8? |
| `chars(s: []u8) -> List(u32)` | All code points. |
| `char_count(s: []u8) -> usize` | The number of code points. |
| `byte_offset(s: []u8, n: usize) -> usize` | The byte offset of the `n`th code point (or `s.len` when past the end). |
| `char_at(s: []u8, n: usize) -> ?u32` | The `n`th code point, or null. |
| `slice(s: []u8, from: usize, to: usize) -> []u8` | Code points `from` (inclusive) to `to` (exclusive), as a slice of `s`. |
| `truncate(s: []u8, n: usize) -> []u8` | The first `n` code points; never cuts a character in half. |
| `reverse(s: []u8) -> String` | The code points in reverse order. |
| `scalars(s: []u8) -> List(char)` | The Unicode scalars, as `char`s (an invalid byte reads as U+FFFD). |
| `grapheme_end(s: []u8, i: usize) -> usize` | The byte offset where the grapheme cluster that starts at byte `i` ends. A grapheme cluster is what a reader takes for one character: a letter and its marks, a Hangul syllable, an emoji sequence joined by ZWJ, a flag, an Indic conjunct, CR LF (Unicode's extended grapheme clusters, UAX #29). |
| `graphemes(s: []u8) -> List([]u8)` | The grapheme clusters, as slices of the text. |
| `grapheme_count(s: []u8) -> usize` | The number of grapheme clusters: the characters a reader counts. |
| `truncate_graphemes(s: []u8, n: usize) -> []u8` | The first `n` grapheme clusters; never splits one. |
| `is_zero_width(cp: u32) -> bool` | Does the code point take no columns (combining and enclosing marks, format and zero-width characters, controls, Hangul vowels and finals)? |
| `is_wide(cp: u32) -> bool` | Does the code point take two columns (East Asian wide and fullwidth, emoji among them)? |
| `char_width(cp: u32) -> usize` | Columns a code point takes on a terminal: 0, 1 or 2. |
| `width(s: []u8) -> usize` | Columns the text takes on a terminal, grapheme cluster by cluster, as terminals draw it: `e` with an accent is one, a family emoji two. |
| `pad_right(s: []u8, columns: usize) -> String` | Pad on the right to `columns` terminal columns (by width, not bytes). |
| `pad_left(s: []u8, columns: usize) -> String` | Pad on the left to `columns` terminal columns: the text right-aligned. |
| `truncate_width(s: []u8, columns: usize) -> []u8` | The longest start of the text that fits in `columns` terminal columns, cut between grapheme clusters. |
| `upper_char(cp: u32) -> u32` | Upper-case a code point by Unicode's simple mapping, one code point to one: `ß` stays `ß` here, where `to_upper` writes `SS`. |
| `lower_char(cp: u32) -> u32` | Lower-case a code point by Unicode's simple mapping. |
| `fold_char(cp: u32) -> u32` | Case-fold a code point by Unicode's simple folding (`ς` and `σ` fold alike); `fold` does the full folding of a text. |
| `to_upper(s: []u8) -> String` | Upper-case by Unicode's full mapping, every script: `straße` becomes `STRASSE`, `ﬁ` becomes `FI`. |
| `to_lower(s: []u8) -> String` | Lower-case by Unicode's full mapping, with `ς` for a sigma that ends a word: `ΟΔΟΣ` becomes `οδος`. |
| `fold(s: []u8) -> String` | Case-fold by Unicode's full folding, to compare or look up text without regard to case: `Straße`, `STRASSE` and `strasse` fold alike. |
| `eq_ignore_case(a: []u8, b: []u8) -> bool` | Compare ignoring case, by full case folding. |

## std.thread

std.thread: threads, channels, select, mutexes and atomics, written in Nexium over the `thread.*` and `sync.*` primitives. `import std.thread` then: fn work(job: *mut Job) -> i64 { ... } var t = thread.spawn(Job, i64, work, Job{ .from = 0, .to = 1000 }) let total = t.join()                        // the function's result fn produce(p: *mut Producer) { ... }        // no result: a Worker var ch = thread.channel(String)             // shared by pointer var producer = thread.run(Producer, produce, Producer{ .out = &mut ch }) let msg = ch.recv() orelse break            // null once closed and drained producer.join() var counter = thread.mutex(i64, 0) let n = counter.lock()                      // *mut i64 while held n.* += 1 counter.unlock() var hits = thread.atomic(0)                 // an i64 changed without a lock _ = hits.add(1) // two channels at once: which has a value (or closed), null after 1 s let which = thread.select2(Job, bool, &mut jobs, &mut quit, 1000) orelse continue // threads that end before the call does, so they may point into locals thread.each(Stage, stages[..], run_stage)   // a thread for every item thread.both(Producer, Consumer, &mut p, produce, &mut c, consume) A thread function takes a pointer to its argument, which the `Thread` owns until `join` returns the result. Channels, mutexes and atomics are values that threads share by pointer; the owner must join every thread using them before letting them go out of scope (`each` and `both` do it themselves), and call `free` when done. Panics inside a thread surface from `join`. Timeouts are in milliseconds, and 0 waits for ever.

Types: `Task(T,`, `Thread(T,`, `WorkerTask(T){`, `Worker(T){`, `Channel(T){`, `Mutex(T){`, `Atomic`

| function | what it does |
| --- | --- |
| `spawn(comptime T: type, comptime R: type, f: fn(*mut T) -> R, own arg: T) -> Thread(T, R)` | Run `f(&mut arg)` on a new thread. |
| `(method) join(self: *mut Self) -> R` | Wait for the thread and take its result. Joining twice panics. |
| `(method) arg(self: *Self) -> *T` | The argument after the thread finished (for results written in place). |
| `run(comptime T: type, f: fn(*mut T) -> void, own arg: T) -> Worker(T)` | Run `f(&mut arg)` on a new thread, for functions without a result. |
| `(method) join(self: *mut Self)` | Wait for the thread. Joining twice panics. |
| `(method) arg(self: *Self) -> *T` | The argument after the thread finished (for results written in place). |
| `count() -> usize` | The number of hardware threads. |
| `channel(comptime T: type) -> Channel(T)` |  |
| `(method) send(self: *mut Self, own value: T)` |  |
| `(method) recv(self: *mut Self) -> ?T` | The next value, waiting for one; null when closed and empty. |
| `(method) recv_for(self: *mut Self, ms: i64) -> !?T` | The next value, waiting at most `ms` for one (0: for ever): `error.Timeout` when none came in time, null when the channel is closed and empty. |
| `(method) try_recv(self: *mut Self) -> ?T` | The next value if one is queued, without waiting. |
| `(method) close(self: *mut Self)` | No more values will be sent; receivers drain what is left, then see null. |
| `(method) is_closed(self: *mut Self) -> bool` |  |
| `(method) len(self: *mut Self) -> usize` |  |
| `(method) free(self: *mut Self)` | Release the lock and condition variable; after every user has stopped. |
| `mutex(comptime T: type, own value: T) -> Mutex(T)` |  |
| `(method) lock(self: *mut Self) -> *mut T` | Take the lock; the pointer is valid until `unlock`. |
| `(method) unlock(self: *mut Self)` |  |
| `(method) free(self: *mut Self)` | Release the lock; after every user has stopped. |
| `select2(comptime A: type, comptime B: type, a: *mut Channel(A), b: *mut Channel(B), ms: i64) -> ?usize` | Wait until one of two channels has a value or is closed: 0 for `a`, 1 for `b` (`a` first when both are), or null when `ms` pass first (0: waits for ever). Then take the value with `try_recv`: another receiver may have taken it first, and a closed channel stays ready, so leave one out once it is closed and drained. |
| `select(comptime T: type, chans: []*mut Channel(T), ms: i64) -> ?usize` | `select2` over any number of channels of one type: the index of the first with a value or closed, or null when `ms` pass first (0: waits for ever). |
| `each(comptime T: type, items: []mut T, f: fn(*mut T) -> void)` | Run `f` on every item, each on a thread of its own, all at once, and return when every one has finished. Nothing started here outlives the call, so the items may point into the caller's locals, and the threads may wait on each other (the stages of a pipeline over channels). A panic in one is raised here once all have ended. To split work over the cores, `for parallel` is the tool. |
| `both(comptime A: type, comptime B: type, a: *mut A, fa: fn(*mut A) -> void, b: *mut B, fb: fn(*mut B) -> void)` | Run `fa(a)` and `fb(b)` on two threads at once and return when both have finished, as `each` does. |
| `atomic(value: i64) -> Atomic` |  |
| `(method) load(self: *Self) -> i64` |  |
| `(method) store(self: *mut Self, value: i64)` |  |
| `(method) add(self: *mut Self, n: i64) -> i64` | Add `n`; the value before. |
| `(method) sub(self: *mut Self, n: i64) -> i64` | Subtract `n`; the value before. |
| `(method) swap(self: *mut Self, value: i64) -> i64` | Put `value` in; the value before. |
| `(method) compare_swap(self: *mut Self, expected: i64, new: i64) -> bool` | Put `new` in if the value is `expected`; whether it was. |

## std.time

std.time: dates, durations, time zones and timers, written in Nexium. `import std.time` then: let now = time.now_utc()                    // a DateTime println("{}", .{now.iso()})                 // 2026-09-19T04:15:14.123Z println("{}", .{now.format("%Y-%m-%d %H:%M")}) let local = time.now_local()                // with the machine's UTC offset let ny = try time.zone("America/New_York")  // from the platform's database println("{}", .{ny.format(time.now(), "%H:%M %Z")})    // 00:15 EDT let d = time.Duration.minutes(90) println("{} {}", .{d.text(), d.iso()})      // 1h 30m PT1H30M var sw = time.Stopwatch.start() ... work ... println("took {}", .{sw.elapsed().text()}) Instants are milliseconds since 1970-01-01T00:00:00Z (`time.now()`), as `i64`; negative values are before the epoch. Calendar arithmetic is the proleptic Gregorian calendar. Time zones come from the platform's database: the zoneinfo files on Linux, macOS and the BSDs (under /usr/share/zoneinfo, or `$TZDIR`), ICU on Windows (10, version 1903 and later). `now_local` and `local` take only the local offset, from the C library; `local_zone` is the machine's zone with its history.

Types: `DateTime`, `Duration`, `Period`, `RuleDay`, `Rule`, `Zone`, `Stopwatch`

| function | what it does |
| --- | --- |
| `is_leap(year: i32) -> bool` |  |
| `days_in_month(year: i32, month: u8) -> u8` |  |
| `weeks_in_year(year: i32) -> u8` | 52 or 53: a year has 53 ISO weeks when it begins on a Thursday, or on a Wednesday in a leap year. |
| `utc(ms: i64) -> DateTime` | Break an instant down in UTC. |
| `local(ms: i64) -> DateTime` | Break an instant down in the machine's local time zone. |
| `with_offset(ms: i64, offset_min: i32) -> DateTime` | Break an instant down at a fixed offset in minutes east of UTC. |
| `now_utc() -> DateTime` | The current instant, in UTC. |
| `now_local() -> DateTime` | The current instant, in local time. |
| `date(year: i32, month: u8, day: u8) -> ?DateTime` | A date at midnight UTC; `null` when the fields do not name a real day. |
| `from_week(year: i32, week: i64, day: i64) -> ?DateTime` | The date of an ISO week date (`day` 1 is Monday); `null` for a week the year does not have. |
| `from_ordinal(year: i32, n: i64) -> ?DateTime` | The date of an ordinal date, the `n`th day of the year (1-based); `null` past the year's last day. |
| `(method) to_ms(self: *Self) -> i64` | Milliseconds since the epoch (the offset is subtracted back out). |
| `(method) to_utc(self: *Self) -> DateTime` | The same instant expressed in UTC. |
| `(method) weekday(self: *Self) -> u8` | Day of the week, 0 = Monday ... 6 = Sunday. |
| `(method) day_of_year(self: *Self) -> u16` | Day of the year, 1-based. |
| `(method) iso_week(self: *Self) -> (i32, u8)` | The ISO 8601 week: the week-numbering year and the week, 1 to 53. Weeks begin on Monday and week 1 holds the year's first Thursday, so January 1 can be in the last week of the year before. |
| `(method) date_text(self: *Self) -> String` | `YYYY-MM-DD`. |
| `(method) week_date(self: *Self) -> String` | The ISO 8601 week date, `2026-W38-6` (the day is 1 for Monday). |
| `(method) ordinal_date(self: *Self) -> String` | The ISO 8601 ordinal date, `2026-262`. |
| `(method) time_text(self: *Self) -> String` | `HH:MM:SS`. |
| `(method) offset_text(self: *Self) -> String` | The offset as `Z`, or `+HH:MM` / `-HH:MM`. |
| `(method) iso(self: *Self) -> String` | ISO 8601 / RFC 3339: `2026-09-19T04:15:14.123Z`, `...+02:00`. |
| `(method) iso_basic(self: *Self) -> String` | ISO 8601's basic format, with no separators: `20260919T041514.123Z`, `...+0200`. |
| `(method) format(self: *Self, spec: []u8) -> String` | strftime-style formatting: `%Y %m %d %H %M %S`, `%3` (millis), `%y` (two-digit year), `%I %p` (12-hour clock, AM/PM), `%a %A %b %B` (day and month names, short and full), `%j` (day of year), `%G %V %u` (ISO week year, week and weekday, 1 for Monday), `%s` (seconds since the epoch), `%z` (offset, `+02:00`), `%Z` (`UTC`, or the offset as the zoneinfo files abbreviate it, `+02`; `Zone.format` gives the zone's own, `CEST`) and `%%`. Unknown letters are copied through. |
| `(method) plus(self: *Self, d: Duration) -> DateTime` | The instant `d` later, at the same offset. |
| `(method) minus(self: *Self, d: Duration) -> DateTime` | The instant `d` earlier, at the same offset. |
| `(method) until(self: *Self, other: *DateTime) -> Duration` | The time from this instant to `other`, negative when `other` is earlier. |
| `(method) add_days(self: *Self, n: i64) -> DateTime` | `n` calendar days later (earlier when negative), at the same clock time and offset. In a zone whose offset changes in between, `z.at(z.instant(&later))` keeps the clock time. |
| `(method) add_months(self: *Self, n: i64) -> DateTime` | `n` months later; a day the month lacks becomes its last, so January 31 plus a month is February 28 or 29. |
| `(method) add_years(self: *Self, n: i64) -> DateTime` | `n` years later; February 29 becomes February 28 in a common year. |
| `parse_iso(s: []u8) -> ?DateTime` | Parse ISO 8601 / RFC 3339 text. The date is `2026-09-19` or `20260919`, a week date `2026-W38-6` or `2026W386`, or an ordinal date `2026-262` or `2026262`. A time may follow after `T` (or a space): `04:15:14.123` or `041514.123`, to the hour, the minute or the second, with `.` or `,` before a fraction of a second (read to the millisecond). An offset may end it: `Z`, `+02:00`, `+0200` or `+02`. Missing parts are zero; `null` when the text is not a date. |
| `(method) millis(n: i64) -> Duration` |  |
| `(method) seconds(n: i64) -> Duration` |  |
| `(method) minutes(n: i64) -> Duration` |  |
| `(method) hours(n: i64) -> Duration` |  |
| `(method) days(n: i64) -> Duration` |  |
| `(method) between(a: i64, b: i64) -> Duration` | The span from `a` to `b` (instants in ms). |
| `(method) since(ms: i64) -> Duration` | The span from an instant to now. |
| `(method) total_seconds(self: *Self) -> f64` |  |
| `(method) total_minutes(self: *Self) -> f64` |  |
| `(method) total_hours(self: *Self) -> f64` |  |
| `(method) whole_seconds(self: *Self) -> i64` | Whole units, rounded toward zero. |
| `(method) whole_minutes(self: *Self) -> i64` |  |
| `(method) whole_hours(self: *Self) -> i64` |  |
| `(method) whole_days(self: *Self) -> i64` |  |
| `(method) plus(self: *Self, other: Duration) -> Duration` |  |
| `(method) minus(self: *Self, other: Duration) -> Duration` |  |
| `(method) times(self: *Self, n: i64) -> Duration` | `n` times as long. |
| `(method) div(self: *Self, n: i64) -> Duration` | An `n`th of it, rounded toward zero. |
| `(method) ratio(self: *Self, other: Duration) -> f64` | How many times `other` goes into it: `hours(3).ratio(minutes(90))` is 2.0. |
| `(method) neg(self: *Self) -> Duration` |  |
| `(method) abs(self: *Self) -> Duration` |  |
| `(method) is_zero(self: *Self) -> bool` |  |
| `(method) is_negative(self: *Self) -> bool` |  |
| `(method) truncate(self: *Self, unit: Duration) -> Duration` | Cut to a whole number of `unit`s, toward zero: `millis(1999).truncate(seconds(1))` is one second. |
| `(method) round(self: *Self, unit: Duration) -> Duration` | The nearest whole number of `unit`s; a half goes away from zero. |
| `(method) text(self: *Self) -> String` | Human text: `250ms`, `3.5s`, `2m 05s`, `1h 02m`, `3d 04h`. |
| `(method) iso(self: *Self) -> String` | ISO 8601: `PT1H30M`, `PT0.25S`, `PT76H`, `-PT1.5S`, `PT0S`. Hours are the largest unit, since a day in a zone is not always 24 of them. |
| `(method) parse_iso(s: []u8) -> ?Duration` | Parse ISO 8601 duration text: `PT1H30M`, `PT0.25S`, `P2DT3H`, `P1W`, `-PT5M`. A day is 24 hours here and a week 7 days; years and months (`P1Y`, `P2M`), whose length depends on the calendar, are `null`, as is anything that is not a duration. The last number may have a fraction (`PT1.5H`), read to the millisecond. |
| `add(ms: i64, d: Duration) -> i64` | `instant + duration`. |
| `utc_zone() -> Zone` | UTC as a zone. |
| `zone(name: []u8) -> !Zone` | A time zone by name: an IANA name from the platform's database (`Europe/Berlin`, `America/New_York`, `Asia/Kolkata`), `UTC`, a fixed offset (`+05:30`, `-0800`), or a POSIX rule (`EST5EDT,M3.2.0,M11.1.0`). `error.NotFound` when the database has no such zone, or there is no database (the playground); `error.InvalidInput` for text that cannot name a zone, or a database file that is not one. |
| `local_zone() -> Zone` | The machine's time zone: `$TZ` when it is set (a zone's name, the path of a TZif file, or a POSIX rule such as `EST5EDT,M3.2.0,M11.1.0`; set but empty is UTC), else the system's: /etc/localtime on Linux, macOS and the BSDs (named by /etc/timezone where there is one, else `Local`), the zone Windows is set to through ICU. When none is found, a zone with the C library's offset now, named `Local`. |
| `from_tzif(name: []u8, b: []u8) -> !Zone` | A zone from the bytes of a TZif file (RFC 8536), the form the zoneinfo database is kept in; `error.InvalidInput` when they are not one. |
| `(method) offset_seconds_at(self: *Self, ms: i64) -> i32` | Seconds east of UTC at an instant. A few offsets, the local mean time places kept before they took a standard one (mostly before 1900), are not whole minutes. |
| `(method) offset_at(self: *Self, ms: i64) -> i32` | Minutes east of UTC at an instant, to the nearest minute: the offset of the `DateTime` that `at` gives. |
| `(method) is_dst_at(self: *Self, ms: i64) -> bool` | Whether daylight saving time is in force at an instant. |
| `(method) abbrev_at(self: *Self, ms: i64) -> String` | The zone's abbreviation at an instant: `CEST`, `PST`, or an offset such as `+0530` where the database has no name. |
| `(method) at(self: *Self, ms: i64) -> DateTime` | An instant broken down in this zone. |
| `(method) now(self: *Self) -> DateTime` | The current instant in this zone. |
| `(method) instant(self: *Self, dt: *DateTime) -> i64` | The instant a wall-clock time names in this zone; the offset in `dt` is ignored. A time that happens twice, when clocks are turned back, is the first; one that is skipped, when they are turned forward, is read with the offset before the change, so 02:30 on the night clocks jump from 02:00 to 03:00 is 03:30. |
| `(method) format(self: *Self, ms: i64, spec: []u8) -> String` | strftime-style text of an instant in this zone: `DateTime.format`'s letters, with `%Z` the zone's abbreviation (`CEST`). |
| `parse_rule(s: []u8) -> ?Rule` | Parse a POSIX TZ rule: `EST5EDT,M3.2.0,M11.1.0`, `<+0530>-5:30`, `AEST-10AEDT,M10.1.0,M4.1.0/3`. Its offsets are written west of UTC, the other way round from everywhere else; a rule with a daylight name and no dates follows the United States' dates. `null` when the text is not a rule. |
| `(method) start() -> Stopwatch` |  |
| `(method) elapsed_ms(self: *Self) -> f64` | Elapsed time in milliseconds, fractional. |
| `(method) elapsed(self: *Self) -> Duration` | Elapsed time as a Duration (whole milliseconds). |
| `(method) lap(self: *mut Self) -> Duration` | Restart and return what had elapsed. |

## std.toml

std.toml: TOML 1.0 (toml.io), written in Nexium: reading a document into values, finding a value by its key, and writing values back as TOML. `import std.toml` then: let doc = try toml.parse(text) let name = toml.as_str(toml.lookup(&doc, "package.name") orelse return) orelse "" for dep in toml.keys(toml.get(&doc, "dependencies") orelse return) { } if let why = toml.problem(text) { eprintln("{}", .{why}) }   // "line 3: ..." let out = toml.stringify(&doc) Every value TOML has: strings (basic and literal, each also multi-line), 64-bit integers (decimal, hex, octal, binary), floats (with `inf` and `nan`), booleans, dates and times (kept as written: offset date-time, local date-time, local date, local time), arrays, tables, inline tables and arrays of tables, under TOML's rules that a key and a table are defined once. A document that breaks one is `error.InvalidInput`; `problem` says where and why.

Types: `Toml`, `Entry`, `StampKind`, `Stamp`

| function | what it does |
| --- | --- |
| `parse(text: []u8) -> !Toml` | The document's values: a `Table` of its keys. |
| `problem(text: []u8) -> ?String` | Why `text` is not a TOML document (`line N: ...`), or null when it is. |
| `get(v: *Toml, key: []u8) -> ?*Toml` | The value of a table's `key`, or null. |
| `lookup(v: *Toml, dotted: []u8) -> ?*Toml` | The value at a dotted path of bare keys, `package.name`, or null. |
| `at(v: *Toml, i: usize) -> ?*Toml` | An array's item `i`, or null. |
| `len(v: *Toml) -> usize` | How many items an array holds or keys a table has; 0 for anything else. |
| `is_table(v: *Toml) -> bool` | Is it a table (a document, a `[header]`'s, or an inline one)? |
| `is_array(v: *Toml) -> bool` | Is it an array (an array of tables included)? |
| `keys(v: *Toml) -> List([]u8)` | A table's keys in the document's order; empty for anything else. |
| `as_str(v: *Toml) -> ?[]u8` |  |
| `as_int(v: *Toml) -> ?i64` |  |
| `as_float(v: *Toml) -> ?f64` | A float's value, or an integer's as a float. |
| `as_bool(v: *Toml) -> ?bool` |  |
| `as_time(v: *Toml) -> ?Stamp` | A date or time as the document wrote it, and which form it is. |
| `stringify(v: *Toml) -> String` | A table as a TOML document: its plain keys first, then each table under a `[header]` and each array of tables under `[[headers]]`. Anything that is not a table is written as a value. |

## std.uuid

std.uuid: UUIDs (RFC 9562), written in Nexium: random ones (version 4), ones that sort by the time they were made (version 7), and their text. `import std.uuid` then: let id = try uuid.v4()                        // 122 random bits println("{}", .{id.text()})                    // "9f1c2e7a-4b3d-4e8f-a1b2-c3d4e5f60718" let row = try uuid.v7()                       // the time first: sorts by creation let back = uuid.parse(text) orelse return error.BadId The random bits come from `random.secure`, the operating system's generator, so an id cannot be guessed from the ones before it; `v4` and `v7` fail only where the system has no generator (`IoError`).

Types: `Uuid`

| function | what it does |
| --- | --- |
| `(method) text(self: *Self) -> String` | The canonical text: 32 lowercase hex digits in groups of 8, 4, 4, 4 and 12, joined by hyphens. |
| `(method) version(self: *Self) -> u8` | The version: 4 for random, 7 for time-ordered, 0 for the nil UUID. |
| `(method) is_nil(self: *Self) -> bool` | Is every bit zero? |
| `(method) time_ms(self: *Self) -> ?i64` | When a version 7 UUID was made, in milliseconds since the epoch; null for any other version. |
| `(method) eq(self: *Self, other: *Uuid) -> bool` | Is it the same UUID? |
| `v4() -> !Uuid` | A random UUID (version 4). |
| `v7() -> !Uuid` | A UUID that begins with the time it was made (version 7): 48 bits of milliseconds since the epoch, then 74 random bits, so ids made later sort after earlier ones, as text and as bytes, to the millisecond. |
| `nil() -> Uuid` | The nil UUID, every bit zero. |
| `max() -> Uuid` | The max UUID, every bit one. |
| `from_bytes(data: []u8) -> ?Uuid` | A UUID from its 16 bytes; null for any other length. |
| `parse(text: []u8) -> ?Uuid` | A UUID from its text: the canonical form with hyphens, in either case, or the 32 hex digits alone, either one also in braces or after `urn:uuid:`. Null for anything else. |

## std.websocket

std.websocket: a WebSocket client (RFC 6455), written in Nexium over std.http's transports. `import std.websocket` then: var ws = try websocket.connect("ws://localhost:8080/chat", 10000) try ws.send_text("hello") while true { let m = try ws.recv()                   // a whole message if m.op == websocket.CLOSE { break }    // the server's close, or the end println("{}", .{m.data}) } ws.close(1000, "done") var gw = try websocket.connect_with(NxTls, &mut layer, "wss://gateway.discord.gg/?v=10", 10000) var api = websocket.socket(NxTls, &mut layer)   // with headers of its own api.header("Authorization", token) try api.open("wss://example.com/stream", 10000) `wss://` goes over a TLS layer, the slot std.http's client uses (`http.Transport`, which nxtls fills). No extensions are asked for (no compression). A message arrives whole, its fragments joined, up to `max_message` bytes; `recv` answers a ping with a pong on its way. The first protocol mistake by the server ends the connection, with the reason in `problem`. The handshake's key and every frame's mask come from `random.secure`. A `recv` that waits longer than the timeout is `error.Timeout` and may be called again: nothing that came is lost.

Types: `Message`, `Decoder`, `Socket(T){`

| function | what it does |
| --- | --- |
| `accept_for(key: []u8) -> String` | The Sec-WebSocket-Accept a server answers `key` with: the base64 of the SHA-1 of the key and RFC 6455's GUID. |
| `request(host: []u8, path: []u8, key: []u8, headers: []http.Header) -> String` | The opening request for `host` (the Host header, as the server knows itself) and `path` (with its query); `headers` are added. |
| `check_response(head: []u8, key: []u8) -> ?String` | What is wrong with the server's answer to the handshake (`head`, up to but not including the blank line), or null when it accepts `key`. |
| `frame(op: u8, payload: []u8, mask: []u8) -> String` | A client frame: final, opcode `op`, masked with the 4 bytes of `mask`. |
| `close_payload(code: u16, reason: []u8) -> String` | The payload of a close frame: the code, big-endian, then the reason, cut to 123 bytes (a control frame carries 125) at a character's start. |
| `(method) new(max: usize) -> Decoder` |  |
| `(method) feed(self: *mut Self, data: []u8)` |  |
| `(method) next(self: *mut Self) -> ?Message` | The next whole message, a control frame as it comes (PING, PONG, CLOSE), or null when more bytes are needed or `problem` is set. |
| `connect(url: []u8, timeout_ms: i64) -> !Socket(http.Plain)` | Connects to a `ws://` URL and completes the handshake; the connect and each wait for data take at most `timeout_ms` (0: no limit). |
| `connect_with(comptime T: type where T: http.Transport, tls: *mut T, url: []u8, timeout_ms: i64) -> !Socket(T)` | Connects to a `wss://` (or `ws://`) URL, TLS over `tls`, a TLS layer: an `http.Transport`, such as nxtls's. |
| `socket(comptime T: type, tls: ?*mut T) -> Socket(T)` | A socket to open, after `header` has added what the handshake should carry (Authorization, Origin, Sec-WebSocket-Protocol). `tls` may be null for `ws://`. |
| `(method) header(self: *mut Self, name: []u8, value: []u8)` | A header for the handshake to carry. |
| `(method) open(self: *mut Self, url: []u8, timeout_ms: i64) -> !void` | Connects to a `ws://` or `wss://` URL and completes the handshake; `wss://` without a TLS layer is `error.Unsupported`, a URL of another scheme and an answer that is not a WebSocket's are `error.InvalidInput` (`problem` says how). |
| `(method) set_timeout(self: *mut Self, ms: i64)` | How long each wait for data takes from now on, in ms (0: no limit): `recv` is `error.Timeout` past it, and can be called again. |
| `(method) is_open(self: *Self) -> bool` | Whether messages can still be sent. |
| `(method) send_text(self: *mut Self, data: []u8) -> !void` | Sends a text message; `data` should be UTF-8. |
| `(method) send_binary(self: *mut Self, data: []u8) -> !void` | Sends a binary message. |
| `(method) ping(self: *mut Self, data: []u8) -> !void` | Sends a ping (at most 125 bytes); its pong comes back through `recv`. |
| `(method) recv(self: *mut Self) -> !Message` | The next message: TEXT or BINARY whole, a PONG, or CLOSE, either the server's (its code and reason) or the end of the connection (1006, with `problem` saying so). A ping is answered on the way. `error.Timeout` when nothing whole came in time (call again: nothing is lost), `error.Closed` once a CLOSE was returned, and `error.InvalidInput` for the server's protocol mistake (`problem` says which), which ends the connection. |
| `(method) close(self: *mut Self, code: u16, reason: []u8)` | Closes the connection with a status (1000: normal) and a reason: sends the close frame, reads until the server's close answers it, the connection ends or a wait times out, then ends the connection. Messages that come meanwhile are dropped. |
