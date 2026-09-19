# Nexium 言語リファレンス

`nx` コンパイラが今日実装しているものを記述します。節番号は `nexium-spec.txt`
（設計）を、「アーカイブ」と記した箇所は `nexium-systems-spec.txt` の第 4〜9 節
（構文リファレンス）を指します。

## ファイルとモジュール

ファイル一つがモジュール一つです。`import foo.bar` はルートファイルの隣の
`foo/bar.nx` を読み込み、その `pub` 項目は `bar.item` で参照します。
`import std.math`（または任意の std モジュール）は書けますが必須ではありません。
組み込み名前空間 `math`、`io`、`os`、`time`、`random`、`mem` は常にスコープ内に
あります。

## 字句構造（アーカイブ 4）

- `//` コメント。`///` ドキュメントコメントは次の宣言に付きます。
- 識別子は ASCII です。文字列とコメントの外にある非 ASCII はエラーです。
- 整数：`42`、`0xFF`、`0o755`、`0b1010_1100`。浮動小数点：`3.14`、`1e-9`、
  `0x1.8p3`。文字列 `"text"`（UTF-8 必須）、生文字列 `r"..."`、バイト列
  `b"\x00\xff"`、文字 `'a'`（Unicode スカラー、型は `char`）。
- 文は改行で終わります。`|>`、`.method(`、`catch`、`orelse`、`and`、`or` で
  始まる行は前の行の続きになり、二項演算子や開き括弧で終わる行も同様です。
- 慣習：型は `PascalCase`、関数と変数は `snake_case`、定数は
  `SCREAMING_SNAKE_CASE`。

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
artifact cabi { name = "lib", exports = [f] }
```

`{ }` 内のフィールドはコンマか改行で区切ります。フィールドには既定値を
付けられます（`verbose: u8 = 0`）。

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
| `List(T)`、`String`、`Map(K, V)` | 所有するコレクション（値、5.3 節） |
| `fn(A, B) -> R !effects` | 関数値（クロージャと関数はこれに変換される） |
| `(A, B)` | タプル。フィールドは `.0`、`.1` |
| `weak T` | `ref class` への弱参照 |

整数リテラルは文脈が求める型になり、既定は `i64`。浮動小数点の既定は `f64`。
文字リテラルはそれを収められる任意の整数型に入ります（`c: u8` で `c == 'a'`）。

キャスト：`x as T` は数値間（範囲が証明されない限り縮小は実行時に検査）、
`distinct` 型とその表現の間、`char` と整数の間、`bool` から整数、単位 enum
から整数。`@truncate(T, x)` は回り込みます。

## 値、束縛、所有権

`let x = e` は不変束縛、`var x = e` は可変束縛。可変性は浅い（アーカイブ
5.10）：`*mut T` を持つ `let` はそれを通じて変更できます。

コレクションはヒープ上のバッファを所有します（5.3）。`let b = a` は `a` を
ムーブし、その後 `a` を使うとコンパイルエラーです。`a.clone()` はコピーします。
引数は借用です。`List` を受け取る関数はそれを読むだけで、変更するには
`*mut List(T)` を取ります。スライスが期待される場所に渡すと `List(T)` と
`String` は `[]T` / `[]u8` に変換されます。フィールドや要素からのムーブは
エラーです。所有された値はスコープの終わりで解放され、それがスコープ終了時の
唯一の自動動作です（H7）。

`ref class` の値は参照です。コピーすると保持され、最後の参照が消えたときに
オブジェクトが解放されます（5.1）。循環はリークします。逆向きの辺には `weak`
を使います（`@weak(x)` または `x.weak()`、その後 `w.upgrade()`）。

## 式

- 算術 `+ - * / %` はオーバーフローでトラップします（定義されたパニック）。
  回り込み `+% -% *%` と飽和 `+| -| *|` は決して失敗しません。ビット演算
  `& | ^ ~ << >>`。比較 `== != < <= > >=` は数値、文字、bool、`[]u8`、
  `String`、単位 enum、`derive(Eq)` / `derive(Ord)` した型に使えます。論理
  `and`、`or`、`!`。
- `x |> f(a)` は `f(x, a)`。
- `if (c) a else b` は式。`if (opt) |v| { } else { }` はアンラップします。
- `match v { pat => expr, ... }` は整数（リテラル、範囲 `1..=9`）、文字列、
  bool、文字、enum（`.Variant(p)`）、オプショナル（`null`、束縛）、エラー
  共用体（`error.Name`、束縛）、タプル、バイトスライス（バイナリパターン）に
  使えます。enum と bool の match は網羅的でなければならず、それ以外は `_ =>`
  が必要です。
- ブロックは式で、値は最後の式です。ラベル付きブロックは `break :label value`
  で値を返します。
- `try e` はエラーを伝播。`e catch |err| handler`。`opt orelse default`。
  `opt.?` はアンラップ（null ならパニック）。
- `defer stmt` はスコープ終了時に実行、`errdefer stmt` はエラーで抜けるとき
  だけ実行。どちらも登録の逆順です。
- クロージャ：`|[captures] params| -> R { body }`。キャプチャは明示的で、`[x]`
  はコピー、`[&x]` と `[&mut x]` は参照を取ります。
- 可変グローバル、外部呼び出し、ポインタキャスト、スライスの `.ptr` には
  `unsafe { }` が必要です。
- `comptime expr` はコンパイル時に評価されます。`@embedFile("path")` は宣言
  されたビルド入力を `[]u8` として埋め込みます。

## 文とループ

```
while (cond) { }
for (items) |x| { }               // 配列、スライス、リスト、文字列、マップのキー
for (items) |x, i| { }            // インデックス付き
for (a, b) |x, y| { }             // 同時走査。長さは一致していること
for (0..n) |i| { }
outer: for (...) |a| { for (...) |b| { continue :outer } }
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

セグメントは `name:size/modifiers` かリテラルです。サイズはビット（既定 8）で、
`bytes` は残り全部を消費します。修飾子：`big`（既定）、`little`、`native`、
`signed`、`unsigned`、`float`、`utf8`。定数サイズが 64 ビット以下の束縛は収まる
最小幅の整数になり、それより大きいか動的なサイズは `[]u8` ビューを束縛します。
サイズ計算はポインタ幅で行われ、常に検査されます。構築は `[]mut u8` バッファを
対象にして `![]u8`（書き込まれた先頭部分）を返し、失敗すると
`error.BufferTooSmall` になります。

## エフェクト（第 8 節）

`allocates refcounts blocks shared_mutable nondeterministic panics ffi`

エフェクトはすべての関数について推論されます。署名の負の制約（`!allocates`）は
検査され、診断は呼び出しをたどってエフェクトを持ち込んだ箇所を指します。関数型も
負の制約を持てます（`fn(i32) -> i32 !allocates`）。その型に変換されるクロージャや
関数はそれを満たさなければなりません。`panics` は証明によって免除されます（E6）：
同じスライス上の `for` のループインデックスによる添字、コンパイル時に既知の添字、
オペランドの範囲が収まる算術、ガードされたローカルは寄与しません。
`nx effects file.nx` は関数ごとの推論結果を表示します。

## 標準ライブラリ（組み込み）

- `println(fmt, .{args})`、`print`、`eprintln`、`format(...) -> String`。
  プレースホルダ：`{}`、`{x}`、`{X}`、`{b}`、`{o}`、`{e}`、`{c}`、`{:.N}`、
  `{>N}`、`{<N}`。`{{` と `}}` は文字どおりの波括弧。
- `expect(cond)`、`expect_eq(a, b)`、`panic(msg)`。
- `List(T)`：`new`、`with_capacity`、`from`、`append`、`pop`、`clear`、
  `clone`、`last`、`first`、`insert`、`remove`、`swap_remove`、`extend`、
  `reserve`、`items`、`is_empty`、`len`、およびスライスのメソッド。
- `String`：`new`、`from`、`with_capacity`、`append`、`append_char`、`clone`、
  `clear`、`pop`、`bytes`、`len`、および `[]u8` のメソッド。
- `Map(K, V)`（キー：整数、bool、char、`[]u8`、`String`）：`new`、`put`、
  `get`、`contains`、`remove`、`clear`、`clone`、`keys`、`values`、`len`、
  `m[key]`。`for (m) |k|` はキーを走査。
- スライス：`len`、`fill`、`reverse`、`sort`、`contains`、`index_of`、
  `copy_from`、`to_owned`、`is_empty`。`[]u8` はさらに `starts_with`、
  `ends_with`、`find`、`trim`、`split`、`lines`、`to_string`、`parse_int(T)`、
  `parse_float`、`eq_ignore_case`。
- 整数：`abs`、`min`、`max`、`checked_add/sub/mul`（`?T` を返す）、
  `to_string`。浮動小数点：`abs`、`sqrt`、`floor`、`ceil`、`round`、`min`、
  `max`、`pow`、`to_string`。文字：`is_digit`、`is_alpha`、`is_space`、
  `to_lower`、`to_upper`、`to_digit`。
- `math`：`PI E TAU INF NAN`、`sqrt abs floor ceil round sin cos tan exp log
  log2 min max pow atan2 clamp`。
- `io.read_file(path) -> !String`、`io.write_file(path, bytes) -> !void`、
  `io.read_line() -> ?String`。
- `os.args() -> [][]u8`、`os.env(name) -> ?[]u8`、`os.exit(code)`、
  `process.run(argv: [][]u8) -> !i32`（起動して待ち、終了コードを返す。
  プログラムを起動できないときは `error.IoError`）。
- `time.now() -> i64`（エポックからのミリ秒）、`time.monotonic() -> u64`
  （ナノ秒）、`time.sleep(ms)`。
- `random.int(lo, hi)`、`random.float()`、`random.seed(n)`。
- `mem.copy(dst, src)`。
- `@typeName(T)`、`@sizeOf(T)`、`@truncate(T, x)`、`@errorName(e)`、
  `@embedFile(path)`、`@weak(x)`、`@refCount(x)`、`@cImport(header)`、
  `@cstr(literal)`。

定義済みエラー：`OutOfMemory Panic InvalidRecord Truncated Overflow InvalidUtf8
NotFound IoError InvalidInput BufferTooSmall`。任意の `error.Name` は新しい
エラーを作ります。

## エントリポイント

`fn main()`、`fn main() -> !void`、または `fn main() -> u8`。`main` からのエラーは
`error: Name` を表示して 1 で終了し、パニックは位置を表示して 101 で終了します。
`test "name" { }` ブロックは `nx test` で実行されます。

## トレイトオブジェクト

`dyn Trait` は、そのトレイトを実装する `T` の `*T` または `*mut T` から作られる
ファットポインタです：`let s: dyn Shape = &circle`、`List(dyn Shape)`、
`[]dyn Shape`。呼び出しは vtable を通じてディスパッチされ、オブジェクトの型が
許すすべてのエフェクトを得ます。`dyn Shape !allocates !blocks` は別の型で、
その制約を満たす実装だけが変換されます。オブジェクトとして使うトレイトは
レシーバの位置でしか `Self` を使えません。

## 並列ループ

`for parallel (items) |x, i| { ... }` はインデックス範囲を本体ごとスレッドプール
で実行します（仕様 7.2）。本体は `shared_mutable` エフェクトを持てず、`return`
や `break` もできず（`continue` を使う）、結果は `i` で添字付けした可変スライス
に書きます。ワーカーでのパニックは全ワーカー終了後に呼び出し側で再送出されます。
このループは `blocks` エフェクトを持ちます（合流するため）。

## アロケーションスコープ

`using arena { ... }` はそのブロックにバンプアロケータを設置します。内部で作られた
値はアリーナから確保され、解放は何もせず、ブロックの終わりでアリーナ全体が
一度に解放されます（仕様 5.1「どのスコープでも置き換え可能」）。ブロックの外で
作られたコンテナは内部で成長してもヒープを使い続けるので、外側の `List`、
`String`、`Map` に結果を集めるのは安全です。内部で作られた値はブロックから
逃がしてはいけません。

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

`@cImport` は C プリプロセッサを実行し、関数、typedef、struct、enum、リテラル
マクロをインポートします。`const T*` は `*T`、他のポインタは `*mut T`、
`void*` は `*mut u8` になります。外部呼び出しには `unsafe` が必要で、`ffi`
エフェクトを持ちます。翻訳できない宣言（union、ビットフィールド、関数
ポインタ、関数形式マクロ）は使ったときにエラーで名指しされます。

## コンパイル時テスト

`comptime test "name" { ... }` は検査中にインタプリタで実行されます。失敗は
その期待の位置を指すコンパイルエラーになります。

## リージョン

関数は自身のローカルへのスライスやポインタを返せません（規則 R1）。引数への
ビューは呼び出し側が所有するので問題ありません。外側の変数に格納されたビューは
追跡されません。

## 未実装

`soa` と `packed` レイアウト、`node` と `installer` アーティファクト、
`nx publish` とレジストリ、`pool`/`stack` アロケーション戦略、規則 R1 を超える
リージョン検査。
