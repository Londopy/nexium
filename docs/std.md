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
| [`std.bytes`](#stdbytes) | encodings and byte-level utilities, written in Nexium. |
| [`std.deque`](#stddeque) | a double-ended queue, written in Nexium: two Lists back to |
| [`std.fs`](#stdfs) | files, directories and paths, written in Nexium. |
| [`std.hash`](#stdhash) | hash functions, written in Nexium: FNV-1a (64-bit), SipHash-2-4 |
| [`std.heap`](#stdheap) | a priority queue, written in Nexium: a binary heap over a List, |
| [`std.http`](#stdhttp) | an HTTP/1.1 client and a small server, written in Nexium over |
| [`std.json`](#stdjson) | a JSON parser and serializer, written in Nexium. |
| [`std.lists`](#stdlists) | generic helpers over slices and Lists, written in Nexium. |
| [`std.net`](#stdnet) | TCP and UDP with addresses, written in Nexium over the `net.*` |
| [`std.num`](#stdnum) | integer utilities, written in Nexium. |
| [`std.process`](#stdprocess) | run programs and capture what they print, written in Nexium |
| [`std.regex`](#stdregex) | regular expressions without backtracking, written in Nexium. |
| [`std.set`](#stdset) | a set of values, written in Nexium over `Map(T, bool)`: its |
| [`std.sort`](#stdsort) | sorting by a comparison of your own, stable sorting, and |
| [`std.stream`](#stdstream) | buffered readers and writers over files and the standard |
| [`std.strings`](#stdstrings) | text utilities on `[]u8` and `String`, written in Nexium. |
| [`std.testing`](#stdtesting) | conveniences for `test` blocks, written in Nexium. |
| [`std.text`](#stdtext) | UTF-8 text by code point, written in Nexium. |
| [`std.thread`](#stdthread) | threads, channels and mutexes, written in Nexium over the |
| [`std.time`](#stdtime) | dates, durations and timers, written in Nexium. |

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

## std.fs

std.fs: files, directories and paths, written in Nexium. `import std.fs` then: if fs.exists("notes.txt") { ... } try fs.make_dirs("out/logs") for name in try fs.list("out") { ... } for path in try fs.walk("src") { ... }        // every file, recursively let cfg = fs.join(fs.parent(argv0), "app.toml") The platform calls are the `io.*` builtins (documented in the language reference); this module adds paths, sorted listings, recursive create and remove, and a walker. Paths are byte strings; `/` and `\` both separate components on every platform, and results use `/` unless the input used `\`.

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
| `is_absolute(path: []u8) -> bool` | Does the path start at a root (`/x`, `C:\x`, `C:/x`, `\\server`)? |
| `join(dir: []u8, name: []u8) -> String` | `dir/name`; a separator is added only when needed, and an absolute `name` replaces `dir`. |
| `parent(path: []u8) -> []u8` | Everything before the last separator: `a/b/c.txt` -> `a/b`, `c.txt` -> ``, `/c.txt` -> `/`. |
| `base_name(path: []u8) -> []u8` | The last component: `a/b/c.txt` -> `c.txt`. |
| `extension(path: []u8) -> []u8` | The extension without the dot: `a/b.tar.gz` -> `gz`, `Makefile` -> ``. |
| `stem(path: []u8) -> []u8` | The base name without its extension: `a/b.tar.gz` -> `b.tar`. |
| `with_extension(path: []u8, ext: []u8) -> String` | The path with its extension replaced (or added): `a/b.txt`, `md` -> `a/b.md`. |
| `normalize(path: []u8) -> String` | Collapse `.` and `..` components and repeated separators: `a/./b/../c//d` -> `a/c/d`. A leading `..` is kept. |

## std.hash

std.hash: hash functions, written in Nexium: FNV-1a (64-bit), SipHash-2-4 (keyed: a table whose keys an adversary picks), and SHA-256 (checksums and content addresses). `import std.hash` then: let h = hash.fnv1a64(name)                    // fast, not keyed let k = hash.siphash(key16, name)             // keyed with 16 bytes let sum = hash.sha256_hex(file_text)          // 64 hex digits var s = hash.Sha256.new()                     // or piece by piece s.update(part1) s.update(part2) let digest = s.finish()                       // 32 bytes `std.bytes` keeps the 32-bit `fnv1a` and `crc32`.

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

std.http: an HTTP/1.1 client and a small server, written in Nexium over std.net and std.stream. `import std.http` then: let r = try http.get("http://example.com/") println("{} {}", .{r.status, r.body.len}) if let ct = r.header("content-type") { ... } fn hello(req: *http.Request) -> http.Response { return http.text(200, "hello from Nexium") } var router = http.Router.new() router.get("/", hello) var server = try http.Server.bind("127.0.0.1", 8080) try server.serve(&router)                  // forever, one request at a time The client speaks HTTP/1.1 with `Connection: close`, reads bodies by Content-Length, chunked encoding, or until close, and follows up to five redirects. Plain `http://` only; TLS needs a C library through `@cImport`. The server handles one connection at a time, which is what a tool, a local dashboard or a test needs; threads come later in the roadmap.

Types: `Header`, `Url`, `Response`, `Request`, `Route`, `Router`, `Server`

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
| `request(method: []u8, url_text: []u8, headers: *List(Header), body: []u8) -> !Response` | Perform a request, following redirects. `error.InvalidInput` for a URL this client cannot speak (including `https://`). |
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
| `(method) reader(self: *Self) -> stream.Reader` | A buffered reader over the socket (lines, chunks); does not own it. A buffered reader over the socket, with its receive timeout. |
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

## std.process

std.process: run programs and capture what they print, written in Nexium over the `process.*` primitives. `import std.process` then: let out = try process.run(["git", "status", "--short"]) if out.ok() { print("{}", .{out.stdout}) } let r = try process.run_with(["sort"], process.Options{ .stdin = "b\na\n", .cwd = "" }) let sh = try process.shell("echo hi")          // cmd /C on Windows, sh -c elsewhere The child inherits the environment. `error.IoError` when the program cannot be started; a non-zero exit is reported in `code`, not as an error. Output is read after stdin is fully written, so a program that produces more than a megabyte of output before reading its input can stall; feed such programs through files.

Types: `Output`, `Options`

| function | what it does |
| --- | --- |
| `(method) ok(self: *Self) -> bool` |  |
| `(method) text(self: *Self) -> []u8` | stdout without a trailing newline. |
| `run_with(argv: [][]u8, opts: Options) -> !Output` |  |
| `run(argv: [][]u8) -> !Output` | Run and capture, inheriting the working directory, with no stdin. |
| `shell(command: []u8) -> !Output` | Run a command line through the platform shell. |

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
| `(method) items(self: *Self) -> List(T)` | The elements, in no particular order. |
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

std.testing: conveniences for `test` blocks, written in Nexium. `import std.testing` then, inside a test: testing.expect_approx(area, 3.14159, 0.001) testing.expect_err(i32, parse("nope")) testing.expect_contains(output, "42 items") testing.expect_lines(rendered, expected)    // reports the first differing line testing.expect_snapshot("report", rendered)  // compares to snapshots/report.txt try testing.snapshot("report", rendered)     // the same, as an error union Snapshots live in `snapshots/<name>.txt` under the current directory. A missing file is written and the test passes; a mismatch fails with the first differing line and how to accept the new output. Set `NX_UPDATE_SNAPSHOTS=1` to rewrite them all.

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

## std.text

std.text: UTF-8 text by code point, written in Nexium. `import std.text` then: let n = text.char_count("héllo")            // 5, not 6 for cp in text.chars("héllo") { ... }        // code points as u32 let w = text.width("日本語")                   // 6 columns on a terminal let s = text.to_upper("straße")               // "STRASSE" is not attempted: "STRAßE" let t = text.truncate("héllo wörld", 5)       // "héllo", never mid-character let ok = text.is_valid("...") Strings are bytes; this module reads them as UTF-8, tolerating bad input (an invalid byte decodes as U+FFFD and advances one byte). Case mapping covers ASCII, Latin-1, Latin Extended-A, Greek and Cyrillic, which is what is cheap to do without tables. `width` follows the usual terminal convention: East Asian wide and fullwidth forms take two columns, combining marks and zero-width characters take none.

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
| `is_zero_width(cp: u32) -> bool` | Does the code point take no columns (combining marks, zero-width, controls)? |
| `is_wide(cp: u32) -> bool` | Does the code point take two columns (East Asian wide and fullwidth)? |
| `char_width(cp: u32) -> usize` | Columns a code point takes on a terminal: 0, 1 or 2. |
| `width(s: []u8) -> usize` | Columns the text takes on a terminal. |
| `pad_right(s: []u8, columns: usize) -> String` | Pad on the right to `columns` terminal columns (by width, not bytes). |
| `upper_char(cp: u32) -> u32` | Upper-case a code point where the mapping is a fixed offset: ASCII, Latin-1, Latin Extended-A (pairs), Greek, Cyrillic. |
| `lower_char(cp: u32) -> u32` | Lower-case a code point; the inverse of `upper_char`. |
| `to_upper(s: []u8) -> String` |  |
| `to_lower(s: []u8) -> String` |  |
| `eq_ignore_case(a: []u8, b: []u8) -> bool` | Compare ignoring case, using the same mapping as `to_lower`. |

## std.thread

std.thread: threads, channels and mutexes, written in Nexium over the `thread.*` and `sync.*` primitives. `import std.thread` then: fn work(job: *mut Job) -> i64 { ... } var t = thread.spawn(Job, i64, work, Job{ .from = 0, .to = 1000 }) let total = t.join()                        // the function's result fn produce(p: *mut Producer) { ... }        // no result: a Worker var ch = thread.channel(String)             // shared by pointer var producer = thread.run(Producer, produce, Producer{ .out = &mut ch }) let msg = ch.recv() orelse break            // null once closed and drained producer.join() var counter = thread.mutex(i64, 0) let n = counter.lock()                      // *mut i64 while held n.* += 1 counter.unlock() A thread function takes a pointer to its argument, which the `Thread` owns until `join` returns the result. Channels and mutexes are values that threads share by pointer; the owner must join every thread using them before letting them go out of scope, and call `free` when done. Panics inside a thread surface from `join`.

Types: `Task(T,`, `Thread(T,`, `WorkerTask(T){`, `Worker(T){`, `Channel(T){`, `Mutex(T){`

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
| `(method) try_recv(self: *mut Self) -> ?T` | The next value if one is queued, without waiting. |
| `(method) close(self: *mut Self)` | No more values will be sent; receivers drain what is left, then see null. |
| `(method) len(self: *mut Self) -> usize` |  |
| `(method) free(self: *mut Self)` | Release the lock and condition variable; after every user has stopped. |
| `mutex(comptime T: type, own value: T) -> Mutex(T)` |  |
| `(method) lock(self: *mut Self) -> *mut T` | Take the lock; the pointer is valid until `unlock`. |
| `(method) unlock(self: *mut Self)` |  |
| `(method) free(self: *mut Self)` | Release the lock; after every user has stopped. |

## std.time

std.time: dates, durations and timers, written in Nexium. `import std.time` then: let now = time.now_utc()                    // a DateTime println("{}", .{now.iso()})                 // 2026-09-19T04:15:14.123Z println("{}", .{now.format("%Y-%m-%d %H:%M")}) let local = time.now_local()                // with the machine's UTC offset let d = time.Duration.minutes(90) println("{}", .{d.text()})                  // 1h 30m var sw = time.Stopwatch.start() ... work ... println("took {}", .{sw.elapsed().text()}) Instants are milliseconds since 1970-01-01T00:00:00Z (`time.now()`), as `i64`; negative values are before the epoch. Calendar arithmetic is the proleptic Gregorian calendar; the only platform call is the local UTC offset, and only `now_local` and `local` use it.

Types: `DateTime`, `Duration`, `Stopwatch`

| function | what it does |
| --- | --- |
| `is_leap(year: i32) -> bool` |  |
| `days_in_month(year: i32, month: u8) -> u8` |  |
| `utc(ms: i64) -> DateTime` | Break an instant down in UTC. |
| `local(ms: i64) -> DateTime` | Break an instant down in the machine's local time zone. |
| `with_offset(ms: i64, offset_min: i32) -> DateTime` | Break an instant down at a fixed offset in minutes east of UTC. |
| `now_utc() -> DateTime` | The current instant, in UTC. |
| `now_local() -> DateTime` | The current instant, in local time. |
| `date(year: i32, month: u8, day: u8) -> ?DateTime` | A date at midnight UTC; `null` when the fields do not name a real day. |
| `(method) to_ms(self: *Self) -> i64` | Milliseconds since the epoch (the offset is subtracted back out). |
| `(method) to_utc(self: *Self) -> DateTime` | The same instant expressed in UTC. |
| `(method) weekday(self: *Self) -> u8` | Day of the week, 0 = Monday ... 6 = Sunday. |
| `(method) day_of_year(self: *Self) -> u16` | Day of the year, 1-based. |
| `(method) date_text(self: *Self) -> String` | `YYYY-MM-DD`. |
| `(method) time_text(self: *Self) -> String` | `HH:MM:SS`. |
| `(method) offset_text(self: *Self) -> String` | The offset as `Z`, or `+HH:MM` / `-HH:MM`. |
| `(method) iso(self: *Self) -> String` | ISO 8601 / RFC 3339: `2026-09-19T04:15:14.123Z`, `...+02:00`. |
| `(method) format(self: *Self, spec: []u8) -> String` | strftime-style formatting: `%Y %m %d %H %M %S %3` (millis) `%z` (offset) `%a %b` (short day and month names) `%j` (day of year) `%%`. Unknown letters are copied through. |
| `parse_iso(s: []u8) -> ?DateTime` | Parse `YYYY-MM-DD`, optionally followed by `THH:MM[:SS[.mmm]]` and an offset `Z` / `+HH:MM` / `-HH:MM`. Missing parts are zero; `null` when the text is not a date. |
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
| `(method) plus(self: *Self, other: Duration) -> Duration` |  |
| `(method) minus(self: *Self, other: Duration) -> Duration` |  |
| `(method) text(self: *Self) -> String` | Human text: `250ms`, `3.5s`, `2m 05s`, `1h 02m`, `3d 04h`. |
| `add(ms: i64, d: Duration) -> i64` | `instant + duration`. |
| `(method) start() -> Stopwatch` |  |
| `(method) elapsed_ms(self: *Self) -> f64` | Elapsed time in milliseconds, fractional. |
| `(method) elapsed(self: *Self) -> Duration` | Elapsed time as a Duration (whole milliseconds). |
| `(method) lap(self: *mut Self) -> Duration` | Restart and return what had elapsed. |
