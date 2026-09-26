# Nexium 言語リファレンス

`nx` コンパイラが現在実装しているものを記述します。節番号は `nexium-spec.txt`（設計）を指し、「アーカイブ」と記した箇所は `nexium-systems-spec.txt` の第 4〜9 節（構文リファレンス）を指します。

## ファイルとモジュール

ファイル一つがモジュール一つです。`import foo.bar` はルートファイルの隣にある `foo/bar.nx` を読み込み、その `pub` 項目は `bar.item` で参照します。`import std.strings` は標準ライブラリのモジュール（`std.json`、`std.fs`、`std.http` など）を読み込みます。標準ライブラリは Nexium で書かれ、コンパイラに埋め込まれています。[`std.md`](../../std.md)（英語）を参照してください。組み込み名前空間 `math`、`io`、`os`、`time`、`random`、`mem`、`process`、`net`、`thread`、`sync` は常にスコープ内にあり、`import` は不要です。

## 字句構造（アーカイブ 4）

- `//` コメント。`///` ドキュメントコメントは次の宣言に付きます。
- 識別子は ASCII です。文字列とコメントの外にある非 ASCII 文字はエラーです。
- 整数：`42`、`0xFF`、`0o755`、`0b1010_1100`。浮動小数点数：`3.14`、`1e-9`、`0x1.8p3`。文字列 `"text"`（UTF-8 必須）、生文字列 `r"..."`、バイト列 `b"\x00\xff"`、文字 `'a'`（Unicode スカラー値、型は `char`）。
- 文は改行で終わります。`|>`、`.method(`、`catch`、`orelse`、`and`、`or` で始まる行は前の行の続きになり、二項演算子や閉じていない括弧で終わる行も同様です。
- 命名規則：型は `PascalCase`、関数と変数は `snake_case`、定数は `SCREAMING_SNAKE_CASE`。

## 宣言

```
fn name(a: T, b: U) -> R effects { ... }         // エフェクト：例 !allocates !panics
pub fn f(x: i32) -> i32 export(c) { ... }        // C ABI でエクスポート
extern fn puts(s: *u8) -> i32                    // 外部関数。呼び出しには `unsafe` が必要
struct Point derive(Eq, Ord) { x: f64, y: f64 }
struct Frame layout(c) { id: u32, len: u16 }     // C レイアウト。境界を越えて使える
struct Pair(T) { a: T, b: T }                    // ジェネリック
record Dose { mg: f64 where value > 0.0 }        // 検証付きデータ
ref class Node { value: i32, next: ?Node }       // 参照カウント
enum Shape { Circle(f64), Rect { w: f64, h: f64 }, Empty }
type Meters = distinct f64                       // 暗黙変換なし
type Bytes = []u8                                // 別名
error ParseError { Empty, NotANumber }
trait Shape { fn area(self: *Self) -> f64 }
impl Shape for Circle { fn area(self: *Self) -> f64 { ... } }
impl Point { fn origin() -> Point { ... } }
impl(T) Pair(T) { fn swap(self: *mut Self) { ... } }
const TABLE: [256]u8 = comptime build_table()
var counter: u32 = 0                             // 可変グローバル。アクセスには `unsafe` が必要
test "name" { ... }
bench "name" { ... }
artifact cabi { name = "lib", exports = [f] }
```

`{ }` 内のフィールドはコンマか改行で区切ります。フィールドには既定値を付けられます（`verbose: u8 = 0`）。

## 型（アーカイブ 5）

| 構文 | 意味 |
| --- | --- |
| `i8 … i128`、`u8 … u128`、`isize`、`usize` | 整数。幅の異なる型の間に暗黙変換はない |
| `f32`、`f64`、`bool`、`char`、`void`、`never` | プリミティブ |
| `[N]T` | 配列。`N` はコンパイル時定数 |
| `[]T`、`[]mut T` | スライス：ポインタと長さ。`[]u8` はテキスト |
| `*T`、`*mut T` | 一つの値へのポインタ（`&x`、`&mut x`、`p.*`） |
| `?T` | オプショナル。`null` が空の値 |
| `!T`、`Set!T` | エラー共用体 |
| `error` | 任意のエラー値（無名のエラー集合） |
| `List(T)`、`String`、`Map(K, V)` | 所有するコレクション（値、5.3 節） |
| `fn(A, B) -> R !effects` | 関数値（クロージャと関数はこれに変換される） |
| `(A, B)` | タプル。フィールドは `.0`、`.1`。型引数にも使える（`List((A, B))`） |
| `weak T` | `ref class` への弱参照 |

整数リテラルは文脈が求める型になり、既定は `i64` です。浮動小数点数リテラルの既定は `f64` です。文字リテラルは、それを収められる任意の整数型に入ります（`c: u8` で `c == 'a'`）。

キャスト：`x as T` は数値の間（範囲が証明されない限り、縮小は実行時に検査されます）、`distinct` 型とその表現の間、`char` と整数の間、`bool` から整数、単位 enum から整数に使えます。`@truncate(T, x)` は検査せずに回り込みます。

## 値、束縛、所有権

`let x = e` は不変の束縛、`var x = e` は可変の束縛です。可変性は浅く（アーカイブ 5.10）、`*mut T` を持つ `let` はそれを通して変更できます。

コレクションはヒープ上のバッファを所有します（5.3）。`let b = a` は `a` をムーブし、その後 `a` を使うとコンパイルエラーです。`a.clone()` はコピーします。ムーブは分岐ごとに追跡されます。`if` の一方の分岐や `match` の一つのアームでムーブされた値は他の分岐ではまだ使え、その構文の後ではムーブ済みとして扱われます。引数は借用です。`List` を受け取る関数はそれを読み、変更するには `*mut List(T)` を受け取ります。所有権を受け取るには、引数に `own` を付けます：

```
fn token(kind: u8, own text: String) -> Token {
    return Token{ .kind = kind, .text = text }     // ムーブで受け取りムーブで渡す：clone 不要
}
let t = token(1, name)                             // `name` はムーブされる。再び使うとエラー
```

`own` 引数は可変で、さらに先へムーブされない限り関数が戻るときにドロップされ、所有型に対してだけ意味を持ちます。レシーバは `own` にできず、エクスポートされる関数は `own` 引数を取れず、`own` 引数を持つ関数は関数値として使えません（その型では誰が引数を所有するのかがわからないため）。スライスが期待される場所に渡すと、`List(T)` と `String` は `[]T` / `[]u8` に変換されます。フィールドや要素からのムーブはエラーです。所有された値はスコープの終わりで解放され、それがスコープ終了時の唯一の自動動作です（H7）。

`ref class` の値は参照です。コピーすると保持（retain）され、最後の参照が消えたときにオブジェクトが解放されます（5.1）。循環はリークします。逆向きの辺には `weak` を使います（`@weak(x)` または `x.weak()`、その後 `w.upgrade()`）。

## 式

- 算術 `+ - * / %` はオーバーフローでトラップします（定義されたパニック）。回り込み `+% -% *%` と飽和 `+| -| *|` は決して失敗しません。ビット演算 `& | ^ ~ << >>`。比較 `== != < <= > >=` は数値、文字、bool、`[]u8`、`String`、単位 enum、`derive(Eq)` / `derive(Ord)` した型に使えます。論理演算 `and`、`or`、`!`。
- `derive(Clone)` は struct や enum に `.clone()`（フィールドごとの深いコピー）を与えます。すべてのフィールドがクローンできることが条件です（ポインタのフィールドはできません）。クローンできるものからなるタプル、オプショナル、配列はこれなしでクローンでき、`where T: Clone` は型パラメータを制約します。
- `x |> f(a)` は `f(x, a)` です。
- `if c { a } else { b }` は式です。`if let v = opt { } else { }` はアンラップします（変数やフィールドのような場所に対しては、束縛はループ変数と同じくビューになります。clone するか、`opt.?` で値を取り出してください）。条件に括弧は付けず、本体は常に波括弧で囲むので、一行で書くと `if c { return v }` です。`else` は次の行から始めても構いません。
- `match v { pat => expr, ... }` は整数（リテラル、範囲 `1..=9`）、文字列、bool、文字、enum（`.Variant(p)`）、オプショナル（`null`、束縛）、エラー共用体（`error.Name`、束縛）、タプル、形によるスライス（`[]`、`[first, rest..]`、`[.., last]`。残りは `[]T` のビュー）、`whole @ pat`、バイトスライス（バイナリパターン）に使えます。enum と bool の match は網羅的でなければならず、スライスの match は長さについて網羅的でなければなりません（`[]` と `[x, rest..]`）。それ以外は `_ =>` が必要です。
- ブロックは式で、その値は最後の式です。ラベル付きブロックは `break :label value` で値を返します。
- `try e` はエラーを伝播します。`e catch |err| handler`、`opt orelse default`。`opt.?` はアンラップします（null ならパニック）。`orelse` と `catch` の右辺にはジャンプも書けます：`let v = opt orelse return null`、`let v = r catch |e| return -1`。
- `opt?.field` と `opt?.m(x)` はオプショナルを通して読みます。オプショナルが空なら `null`、そうでなければメンバーをオプショナルとして返します。チェーンの残りは中身に適用され（`a?.name.len orelse 0`）、オプショナルの結果が二重に包まれることはありません（`a?.b?.c`）。
- `let (a, b) = pair` と `for (k, v) in pairs` はタプルの要素ごとに名前を束縛します（`_` で一つ飛ばせます）。所有された値に対しては名前が要素を所有し、場所に対しては `if let` の束縛と同じくそのビューになります。
- 整数や浮動小数点数のリテラルは `?T` に変換されます：`f(x: ?i32)` に対する `f(1)`。
- `defer stmt` はスコープ終了時に実行され、`errdefer stmt` はエラーでスコープを抜けるときだけ実行されます。どちらも登録の逆順です。
- クロージャ：`|[captures] params| -> R { body }`。キャプチャは明示的で、`[x]` はコピー、`[&x]` と `[&mut x]` は参照を取ります。
- 可変グローバル、外部呼び出し、ポインタのキャスト、スライスの `.ptr` には `unsafe { }` が必要です。
- `comptime expr` はコンパイル時に評価されます。`@embedFile("path")` は宣言されたビルド入力を `[]u8` として埋め込みます。

## 文とループ

```
while cond { }
for x in items { }               // 配列、スライス、リスト、文字列、マップのキー
for (k, v) in m { }              // マップのエントリ
for x in it { }                  // next(self: *mut Self) -> ?T を持つ任意の値
for x, i in items { }            // インデックス付き
for x, y in a, b { }             // 同時走査。長さは一致していること
for i in 0..10 step 2 { }        // 0 2 4 6 8。`for i in 10..0 step -1` は逆順に数える（符号付き）
while cond { } else { }          // else は cond が偽になったときに実行され、break の後には実行されない
for i in 0..n { }
outer: for a in ... { for b in ... { continue :outer } }
break, continue, return
_ = expr                          // 明示的な破棄。未使用の値はエラー
```

## バイナリパターン（アーカイブ 6）

```
match packet {
    <<version:4, ihl:4, total_len:16/big, rest:bytes>> => ...
    <<0x1b, '[', 'A', rest:bytes>> => Key.Up
    <<len:16/little, payload:len*8, rest:bytes>> => payload
    _ => ...
}
let written = try <<4:4, 5:4, 0:8, 1500:16/big, "ab">> into buf[..]
```

セグメントは `name:size/modifiers` かリテラルです。サイズはビット単位（既定 8）で、`bytes` は残りをすべて消費します。修飾子：`big`（既定）、`little`、`native`、`signed`、`unsigned`、`float`、`utf8`。定数サイズが 64 ビット以下の束縛は収まる最小幅の整数になり、それより大きいか動的なサイズは `[]u8` のビューを束縛します。サイズの計算はポインタ幅で行われ、常に検査されます。構築は `[]mut u8` のバッファに書き込んで `![]u8`（書き込まれた先頭部分）を返し、入りきらなければ `error.BufferTooSmall` で失敗します。

## エフェクト（第 8 節）

`allocates refcounts blocks shared_mutable nondeterministic panics ffi`

エフェクトはすべての関数について推論されます。シグネチャの負の制約（`!allocates`）は検査され、診断は呼び出しをたどって、そのエフェクトを持ち込んだ箇所を指します。関数型も負の制約を持てます（`fn(i32) -> i32 !allocates`）。その型に変換されるクロージャや関数はそれを満たさなければなりません。`panics` は証明によって免除されます（E6）：同じスライス上のループインデックスによる添字、コンパイル時に既知の添字、オペランドの範囲が収まる算術、ガードされたローカルは寄与しません。`nx audit file.nx --lock` は各関数のエフェクトを `file.effects.lock` に書き出し、`nx audit file.nx --check` は、ロックにないエフェクトを関数が得たときに失敗します（振る舞いのセマンティックバージョニング：アロケートやブロックを始めた依存関係は、意図してロックを作り直すまでビルドを失敗させます）。`nx effects file.nx` は関数ごとの推論結果を表示します。`nx explain file.nx f effect` は、`f` がなぜそのエフェクトを持つのかを木の形で表示します。各関数に記録された理由（アロケーション、オーバーフロー検査、明示的なパニック）と、それを持ち込む呼び出しを、エフェクトの元になるプリミティブまでたどります。

## 標準ライブラリ（組み込み）

- `println(fmt, .{args})`、`print`、`eprintln`、`format(...) -> String`。名前付き引数 `.{ .name = v }` とプレースホルダ `{name}` / `{name:spec}`、引数から取る幅 `{v:>w}` も使えます。プレースホルダ：`{}`、`{x}`、`{X}`、`{b}`、`{o}`、`{e}`、`{c}`、`{:.N}`、`{>N}`、`{<N}`。`{{` と `}}` は波括弧そのものです。
- `expect(cond)`、`expect_eq(a, b)`、`panic(msg)`。
- `List(T)`：`new`、`with_capacity`、`from`、`append`、`pop`、`clear`、`clone`、`last`、`first`、`insert`、`remove`、`swap_remove`、`extend`、`reserve`、`items`、`is_empty`、`len`、およびスライスのメソッド。
- `String`：`new`、`from`、`with_capacity`、`append`、`append_char`（コードポイント一つを UTF-8 で符号化）、`push_byte`（生のバイト一つ）、`clone`、`clear`、`pop`、`bytes`、`len`、および `[]u8` のメソッド。
- `Map(K, V)`（キー：整数、bool、char、`[]u8`、`String`。ハッシュ値に関係なく、キーが最初に入れられた順に走査されます）：`new`、`put`、`get`、`contains`、`remove`、`clear`、`clone`、`keys`、`values`、`len`、`m[key]`。`for k in m` はキーを走査します。
- スライス：`len`、`fill`、`reverse`、`sort`、`swap(i, j)`、`contains`、`index_of`、`copy_from`、`to_owned`、`is_empty`。`[]u8` はさらに `starts_with`、`ends_with`、`find`、`trim`、`split`、`lines`、`to_string`、`parse_int(T)`、`parse_float`、`eq_ignore_case`。
- 整数：`abs`、`min`、`max`、`checked_add/sub/mul`（`?T` を返す）、`to_string`。浮動小数点数：`abs`、`sqrt`、`floor`、`ceil`、`round`、`min`、`max`、`pow`、`to_string`。文字：`is_digit`、`is_alpha`、`is_space`、`to_lower`、`to_upper`、`to_digit`。
- `math`：`PI E TAU INF NAN`、`sqrt abs floor ceil round sin cos tan exp log log2 min max pow atan2 clamp`。
- `io.read_file(path) -> !String`、`io.write_file(path, bytes) -> !void`、`io.append_file(path, bytes) -> !void`、`io.read_line() -> ?String`。
- ファイルシステムのプリミティブ（`std.fs` がパスとディレクトリの走査を加えて包みます）：`io.file_kind(path) -> i32`（0 は存在しない、1 はファイル、2 はディレクトリ）、`io.file_size(path) -> !u64`、`io.file_modified(path) -> !i64`（ミリ秒）、`io.make_dir(path) -> !void`、`io.remove_file(path) -> !void`、`io.remove_dir(path) -> !void`（空のもの）、`io.rename(from, to) -> !void`、`io.list_dir(path) -> !List(String)`、`io.cwd() -> !String`、`io.temp_dir() -> String`。失敗は `error.NotFound` か `error.IoError` です。
- ファイルハンドル（`std.stream` がバッファリングを加えて包みます）：`io.open(path, mode) -> !i64`（モードは `r`、`w`、`a`）、`io.read(h, n) -> !String`（最大 `n` バイト。入力の終わりでは空）、`io.write(h, bytes) -> !void`、`io.flush(h) -> !void`、`io.close(h) -> !void`。ハンドル 1、2、3 は stdin、stdout、stderr で、`io.is_terminal(h) -> bool` はそのどれかが端末かどうかを返します。`io.raw_mode(on) -> bool` はコンソールのバイトを打たれたとおりに、エコーなしで、入出力とも VT シーケンス付きで渡します（REPL の行エディタが使います。終了時にコンソールは元に戻ります）。`io.read_key() -> ?i64` は入力の 1 バイト、`io.pending_input() -> i64` はバッファにあるバイト数です（エスケープシーケンスはまとめて届きます）。REPL では使えません。
- ソケット（`std.net` と `std.http` はこれらの上に作られています。どの呼び出しも `blocks` を持ちます）：`net.connect(host, port, timeout_ms) -> !i64`、`net.listen(host, port) -> !i64`、`net.accept(listener, timeout_ms) -> !i64`、`net.send(sock, bytes) -> !void`、`net.recv(sock, n, timeout_ms) -> !String`（相手が閉じたら空）、`net.close(sock)`、`net.peer(sock)` / `net.local(sock) -> !String`（`ip:port`）、`net.resolve(host) -> !List(String)`、`net.udp_bind(host, port) -> !i64`、`net.send_to(sock, host, port, bytes)`、`net.recv_from(sock, n, timeout_ms) -> !String`（送信元は `net.last_peer()` でわかります）。タイムアウト 0 は無期限に待ちます。エラー：`NotFound`（名前解決）、`ConnectionRefused`、`Timeout`、`IoError`。REPL では使えません。
- TCP 上の TLS。プラットフォーム自身のライブラリを使い、最初に使うときに読み込みます（Windows では SChannel、macOS では Security.framework、それ以外では OpenSSL の libssl 3 または 1.1。`std.http` の `SystemTls` がこれらを包み、どの呼び出しも `blocks` を持ちます）：`net.tls_available() -> bool`、`net.tls_connect(host, port, timeout_ms) -> !i64`（接続とハンドシェイクをタイムアウト内に行い、サーバの証明書をシステムのルート証明書とホスト名に照らして検証します）、`net.tls_send(h, bytes) -> !void`、`net.tls_recv(h, n, timeout_ms) -> !String`（終わりでは空）、`net.tls_truncated(h) -> bool`（close_notify なしで終わったかどうか）、`net.tls_close(h)`、そして `net.tls_problem() -> String`（このスレッドで最後の TLS 呼び出しが失敗した理由）。エラーはソケットと同じで、検証に通らない証明書は `IoError` です。REPL では使えません。
- スレッド（`std.thread` はこれらの上に `Thread`、`Channel`、`Mutex`、`Atomic`、`select`、`each`、`both` を作ります）：`thread.start(f: fn(*mut T) -> void, arg: *mut T) -> i64` は独自のコンテキストを持つ新しいスレッドで `f(arg)` を実行し、`thread.join(h)` はその終了を待ってパニックを再送出し、`thread.join_all(hs: []i64)` はすべてを待ってから、その中の最初のパニックを再送出します。`thread.count() -> usize` はハードウェアスレッド数です。`sync.mutex_new() -> i64`、`sync.lock(m)`、`sync.unlock(m)`、`sync.mutex_free(m)`、`sync.cond_new() -> i64`、`sync.wait(cv, m)`、`sync.wait_for(cv, m, ms) -> bool`（`ms` が過ぎたら false。0 未満なら無期限に待つ）、`sync.signal(cv)`、`sync.broadcast(cv)`、`sync.cond_free(cv)`。ベルはどのスレッドからでも鳴らせ、一つのスレッドが待ちます。待つ前に鳴ったベルは、その待機のために取っておかれます：`sync.bell_new() -> i64`、`sync.bell_ring(b)`、`sync.bell_wait(b, ms) -> bool`、`sync.bell_free(b)`。`i64` に対する逐次一貫性のアトミック操作：`sync.atomic_load(p: *i64) -> i64`、`sync.atomic_store(p: *mut i64, v)`、`sync.atomic_add(p, n) -> i64` と `sync.atomic_swap(p, v) -> i64`（どちらも操作前の値を返す）、`sync.atomic_cas(p, expected, new) -> bool`。スレッドの開始は `nondeterministic` と `shared_mutable` を持ち、join、ロック、待機は `blocks` を持ち、`sync` の呼び出しは `shared_mutable` です。REPL では使えません。
- `os.arch() -> []u8`：プログラムが動くアーキテクチャ（`x86_64`、`aarch64`、`x86`、`arm`、`riscv64`、または `unknown`）。プログラムのコンパイル時に決まります。
- `os.args() -> [][]u8`、`os.exe_path() -> String`（実行中の実行ファイル。プラットフォームが教えない場合は空）、`os.env(name) -> ?[]u8`、`os.set_env(name, value)`（このプロセスと、これから起動するプロセスに対して。空の値は変数を削除）、`os.environ() -> List(String)`（すべての `NAME=value`）、`os.exit(code)`、`process.run(argv: [][]u8) -> !i32`（起動して待ち、終了コードを返す。プログラムを起動できないときは `error.IoError`）、`process.exec(argv, stdin, cwd) -> !i32`（同じだが、標準入力に `stdin` を与え、空でなければ `cwd` で実行し、stdout と stderr を取り込む）と、その後の `process.last_stdout()` / `process.last_stderr() -> String`。`std.process` がこれらを包みます。
- 並行して動くプログラム（`std.process` はこれらの上に `Child` を作ります。`child_pid` 以外の呼び出しはすべて `blocks` を持ちます）：`process.spawn(argv, cwd, flags) -> !i64` はプログラムを一つ起動します（`cwd` で、空ならこのプログラムのディレクトリで）。各標準ストリームには `flags` の 2 ビットずつを stdin、stdout、stderr の順に使い、このプログラムのものを使う（0）、パイプ（1）、どこにもつながない（2）、stderr を stdout へ（3）のどれかを指定します。`process.child_write(h, bytes, timeout_ms) -> !void` は、プログラムが受け取るのに合わせて `bytes` をすべて書き込み、`process.child_close_input(h)` はその入力を閉じ、`process.child_read(h, stream, n, timeout_ms) -> !String` はストリーム 1（stdout）か 2（stderr）の最大 `n` バイトで、終わりでは空です。`process.child_wait(h, timeout_ms) -> !i64` は下位 32 ビットに終了コード、その上に終了させたシグナルを返し、`process.child_signal(h, sig) -> !void` はシグナルを送ります（シグナルのない Windows では、0 以外のどのシグナルも終了コード 128 + `sig` でプログラムを終わらせます）。さらに `process.child_pid(h) -> i64`、そして手放すための `process.child_close(h)` があります。ある呼び出しが待っている間も、プログラムの出力は後の読み取りのために保持されるので、パイプが一杯になって止まることはありません。0 未満のタイムアウトは無期限に待ち、0 は待ちません。エラー：`NotFound`（そのようなプログラムがない）、`Timeout`、`IoError`（入力を読まなくなったプログラムもこれ）。`process.trap_signals()` は SIGINT、SIGTERM、SIGHUP でプログラムが終了しないようにし（Windows では Ctrl-C が 2、Ctrl-Break が 21、コンソールを閉じるのが 1、ログオフとシャットダウンが 15）、`process.next_signal(timeout_ms) -> i32` は捕まえた次のシグナルを取り出します。時間内に来なければ 0 です。REPL では使えません。
- `time.now() -> i64`（エポックからのミリ秒）、`time.monotonic() -> u64`（ナノ秒）、`time.utc_offset(ms) -> i64`（その時点の現地時刻の、UTC から東向きのずれを分で表したもの。`std.time` はこれらの上に日付を作ります）、`time.sleep(ms)`。
- `time.zone_rules(name) -> !string`：IANA データベースのタイムゾーン一つをテキストで返します。データベースを zoneinfo ファイルではなく ICU に持つ Windows（Windows 10 1903 以降。最初の呼び出しで読み込み）で `std.time` が使うためのものです。1 行目はゾーン名で、その後に期間ごとに一行、`start offset dst abbrev` が続きます。空の名前はシステムのゾーンです。ICU にないゾーンでは `NotFound` で、`std.time` が zoneinfo ファイルを自分で読む他のすべてのプラットフォームでも `NotFound` です。代わりに `std.time` の `time.zone(name)` を使ってください。
- `random.int(lo, hi)`、`random.float()`、`random.seed(n)`：`random.seed` で再現可能にできる高速な生成器です。秘密には決して使わないでください。
- `random.secure(buf) -> !void`：`[]mut u8` をオペレーティングシステムの安全な生成器（BCryptGenRandom、getrandom、arc4random）で埋めます。鍵、トークン、UUID 向けです。`random.seed` の影響は受けず、システムに生成器がない場合にだけ `IoError` で失敗します。REPL と `nx play` では実行されません。コンパイルされたプログラムが必要です。
- `mem.copy(dst, src)`。
- `@typeName(T)`、`@sizeOf(T)`、`@alignOf(T)`、`@truncate(T, x)`、`@bitCast(T, x)`（同じサイズのスカラー）、`@min(a, b)`、`@max(a, b)`、`@errorName(e)`、`@embedFile(path)`、`@weak(x)`、`@refCount(x)`、`@cImport(header)`、`@cstr(literal)`、`@target()`（`(os, arch, bits)`。C ビルドの定数）、`@escape(v)`（最も内側の `using arena` ブロックの外で作られた `v` のコピー）。

定義済みエラー：`OutOfMemory Panic InvalidRecord Truncated Overflow InvalidUtf8 NotFound IoError InvalidInput BufferTooSmall`。任意の `error.Name` は新しいエラーを作ります。

## エントリポイント

`fn main()`、`fn main() -> !void`、または `fn main() -> u8`。`main` からのエラーは `error: Name` を表示して 1 で終了し、パニックは位置を表示して 101 で終了します。`test "name" { }` ブロックは `nx test` で実行されます。

## ベンチマーク

`bench "name" { ... }` はテストと並べて書き、`nx bench` で実行します。`nx bench` はファイルを最適化してビルドし（`--mode` で別のモードを指定しない限り `safe` モード）、各ブロックを計測します。1 サンプルが 10 ms になるよう反復回数を調整し（この調整がウォームアップを兼ねます）、21 個のサンプルを取って、反復あたりの時間の中央値を、最速と最遅のサンプルとともに表示します：

```
bench  sum to 1000      187 ns/iter  (min 186 ns, max 193 ns; 21 samples of 64398)
bench  a list of 100    204 ns/iter  (min 202 ns, max 209 ns; 21 samples of 58241)
```

ブロックの最後の式の値は保持されるので、それを作る処理が最適化で消されることはありません。`bench "sum" { sum_to(1000) }` は合計を計測しますが、`_ = sum_to(1000)` では何も計測されないかもしれません。エラー共用体の値は `try` しなければならず、エラーは計測されるのではなくベンチマークを失敗させます。パニックやエラーはそのベンチマークを失敗させますが、他のベンチマークは引き続き実行されます。ファイルの後の引数は、名前にそれを含むものだけに実行を絞り込み、`--quick` は 1 ms のサンプルを取ります。`nx test` はベンチマークを実行せず、`nx check` はベンチマークも検査します。

## 再帰型とポインタ越しの match

`List` は定義中の型を保持できるので、木や JSON 値は普通の enum で書けます：`enum Json { Null, Arr(List(Json)), Obj(List(Member)) }`。ポインタを通して match すると、所有するペイロードは参照で束縛されます：

```
fn push(v: *mut Json, own item: Json) {
    match v.* {
        .Arr(items) => items.append(item),   // items: *mut List(Json)。ペイロードの別名
        _ => {},
    }
}
```

`v: *Json` なら束縛は `*List(Json)` です。スカラー（`.Num(n)`）はコピーされます。`*String` や `*List(T)` は、スライスが期待される場所で `[]u8` や `[]T` に変換されます。

## トレイトオブジェクト

`dyn Trait` は、そのトレイトを実装する `T` の `*T` または `*mut T` から作られるファットポインタです：`let s: dyn Shape = &circle`、`List(dyn Shape)`、`[]dyn Shape`。呼び出しは vtable を通じてディスパッチされ、オブジェクトの型が許すすべてのエフェクトを持ちます。`dyn Shape !allocates !blocks` は別の型で、その制約を満たす実装だけがそこへ変換できます。オブジェクトとして使うトレイトは、レシーバの位置でしか `Self` に言及できません。

## 並列ループ

`for parallel x, i in items { ... }` は、本体をインデックス範囲にわたってスレッドプールで実行します（仕様 7.2）。本体は `shared_mutable` エフェクトを持てず、`return` も `break` もできず（`continue` を使います）、結果は `i` で添字付けした可変スライスを通して書きます。ワーカーでのパニックは、全ワーカーの終了後に呼び出し側で再送出されます。このループは `blocks` エフェクトを持ちます（ワーカーの合流を待つため）。

## アロケーションスコープ

`using arena { ... }` はそのブロックにバンプアロケータを設置します。内部で作られた値はアリーナから確保され、その解放は何もせず、ブロックの終わりでアリーナ全体が解放されます（仕様 5.1「どのスコープでも置き換え可能」）。ブロックの外で作られたコンテナは内部で伸びてもヒープを使い続けるので、外側の `List`、`String`、`Map` に結果を集めるのは安全です。内部で作られた値をブロックの外へ逃がしてはいけません。

## C の呼び出し

```
const libc = @cImport("string.h")
const cv = @cImport("cvendor.h")               // ソースファイルの隣で探される
artifact link { c_sources = ["cvendor.c"], libs = [], include = [], libs_windows = ["gdi32"] }

unsafe {
    let n = libc.strlen(@cstr("hello"))
    let p = cv.cv_point{ .x = 1.0, .y = 2.0 }
    _ = cstdio.printf(@cstr("%d\n"), 42)      // 可変長引数はスカラーとポインタを取る
}
```

`@cImport` は C プリプロセッサを実行し、関数、typedef、struct、enum、リテラルのマクロをインポートします。`const T*` は `*T`、その他のポインタは `*mut T`、`void*` は `*mut u8` になります。外部呼び出しには `unsafe` が必要で、`ffi` エフェクトを持ちます。フィールドを翻訳できない struct（関数ポインタ、ビットフィールド、入れ子の定義）は不透明型としてインポートされ、前方宣言と同じようにポインタを通して使えます。どの libc でも `FILE` はこうして使えます。まったく翻訳できない宣言（union、関数ポインタの typedef、関数形式マクロ）は、使ったときにエラーで名指しされます。

## 条件付きコンパイル

`if comptime C { ... } else { ... }` はプログラムの検査中に `C` を評価し、選ばれた分岐だけをビルドします。もう一方は検査されないので、別のプラットフォームにしかないものを呼び出せます：

```nexium
fn line_ending() -> []u8 {
    if comptime @target().0 == "windows" {
        return "\r\n"
    } else {
        return "\n"
    }
}
```

`@target()` はビルドの `(os, arch, pointer_bits)` で、クロスコンパイルのときは `--target` のものです。値を持つ `if comptime` は、選ばれた分岐の型を持ちます。

## コンパイル時テスト

`comptime test "name" { ... }` は検査中にインタプリタで実行されます。失敗は、その期待の位置を指すコンパイルエラーになります。

## ビューとその記憶域

関数は自身のローカルへのスライスやポインタを返せません（規則 R1、1.0 からエラー）。借用した引数へのビューは、呼び出し側が所有しているので問題ありません。1.2 はビュー規則 V1〜V5（`SPEC.md` 5.6 と 5.7）を加えました：指している記憶域より長く保持されたビュー、コンテナが伸びた後や値がムーブされた後に使われたビュー、ビューを含む値の返却、`using arena` ブロックより長く保持された値です。1.2 では警告、1.3 からはエラーです。各エラーは修正方法を示し、機械的な修正（値がムーブされる箇所の `.clone()`、`@escape`）は `nx fix` が行います。`@escape(v)` は値をアリーナブロックの外へコピーします。

## 未実装

`nx publish` とレジストリ。`soa` と `packed` のレイアウト、`pool` と `stack` のアロケーション戦略、アーカイブされたリージョン規則 R2〜R4 は言語の一部ではなく（決定 88）、コンパイラはそれらの書き方を拒否します。
