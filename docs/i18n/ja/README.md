<p align="center">
  <img src="https://raw.githubusercontent.com/Londopy/nexium/main/assets/banner.svg" alt="Nexium" width="880">
</p>

<p align="center">
  <a href="../../../README.md">English</a> ·
  <a href="../es/README.md">Español</a> ·
  <a href="../zh-CN/README.md">简体中文</a> ·
  <b>日本語</b> ·
  <a href="../ko/README.md">한국어</a> ·
  <a href="../fr/README.md">Français</a> ·
  <a href="../de/README.md">Deutsch</a>
</p>

<p align="center">
  <a href="https://github.com/Londopy/nexium/actions/workflows/ci.yml"><img alt="CI" src="https://img.shields.io/github/actions/workflow/status/Londopy/nexium/ci.yml?branch=main&label=CI&logo=githubactions&logoColor=white"></a>
  <a href="https://github.com/Londopy/nexium/releases"><img alt="Release" src="https://img.shields.io/github/v/release/Londopy/nexium?logo=github&color=8b7cf6"></a>
  <a href="https://crates.io/crates/nexium"><img alt="crates.io" src="https://img.shields.io/crates/v/nexium?logo=rust&color=4fd1c5"></a>
  <a href="../../../LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-blue"></a>
  <a href="https://ziglang.org/download/"><img alt="Zig" src="https://img.shields.io/badge/backend-zig%20cc-f7a41d?logo=zig&logoColor=white"></a>
  <img alt="Platforms" src="https://img.shields.io/badge/platforms-windows%20%7C%20linux%20%7C%20macos-2b3a55">
</p>

<p align="center">
  <b>すべてを作れるほど完成された言語でありながら、他の何かの一部分として採用するのに最適な言語。</b>
</p>

Nexium は C を経由してネイティブコードにコンパイルされ、トレース型ガベージ
コレクタなしの自動参照カウントを持ち、関数がメモリを確保するか、ブロックするか、
パニックし得るかを機械的に検査するエフェクトシステムを備えます。コンパイラは
一つのソースツリーから C ライブラリ、Python の wheel、Rust の crate、コマンド
ラインツールを生み出します。

<table>
<tr>
<td width="50%" valign="top">

**一つのファイル**

```
fn checksum(data: []u8) -> u32 export(c) {
    var h: u32 = 2166136261
    for b in data {
        h ^= b as u32
        h *%= 16777619
    }
    return h
}

artifact cabi   { name = "hasher" }
artifact python { name = "hasher" }
```

</td>
<td width="50%" valign="top">

**あらゆるターゲット**

```
$ nx ship hasher.nx
shipped 4 artifact file(s) for x86_64-windows:
  nx-out/hasher/hasher.dll
  nx-out/hasher/hasher.lib
  nx-out/hasher/hasher.h
  nx-out/hasher/hasher-0.1.0-py3-none-win_amd64.whl
```

```python
>>> import hasher
>>> hasher.checksum(b"hello")
1335831723
```

</td>
</tr>
</table>

エフェクトの署名が C の ABI を決めます。`checksum` は `!panics` であることが
証明されているので、素の `uint32_t checksum(const uint8_t*, size_t)` になり
ます。失敗し得る関数はステータスコードを返し、その内部で起きた Nexium の
パニックはホストプロセスを止める代わりに境界で変換されます。

## 特徴

| | |
| --- | --- |
| 🧾 **推論され検査されるエフェクト** | `allocates` `refcounts` `blocks` `shared_mutable` `nondeterministic` `panics` `ffi`。`!allocates` と宣言すれば、コンパイラは呼び出しをたどってそれを破る正確な行を指し示します。 |
| 🧠 **借用検査器なしの所有権** | コレクションはムーブされ、`.clone()` でコピー、`ref class` の値は参照カウント、`weak` が循環を断ちます。ムーブ後の使用はコンパイルエラーです。 |
| 🔬 **バイナリパターン** | `<<version:4, ihl:4, len:16/big, rest:bytes>>` が検査済みのサイズでパケットを照合し、構築します。 |
| 🧵 **並列ループ、アリーナ、トレイトオブジェクト** | `for parallel`、`using arena { }`、`dyn Trait !allocates`。 |
| 🔌 **バインディング不要の C 呼び出し** | `@cImport("header.h")` がヘッダを直接読み、`artifact link` が同梱の C をプログラムに組み込みます。 |
| 📦 **一つのソースから出荷** | `nx ship` が C のヘッダとライブラリ、Python の wheel、安全なラッパー付きの Rust crate を生成します。 |
| 🖼 **Nexium で書かれた GUI** | [`gui/`](../../../gui)：ソフトウェアラスタライザとビットマップフォントによる即時モード GUI（ボタン、スライダー、テキスト入力）。200 行の C ウィンドウ層の上はすべて Nexium です。 |
| 🛠 **同梱のツール群** | `fmt`、`doc`、`lsp`、`size`、`leaks`、`refcounts`、`effects`、`audit`。依存ゼロ。 |

## インストール

実行時に必要なのは `PATH` 上の [Zig](https://ziglang.org/download/) だけで、
C コンパイラとして使われます（`zig cc` はクロスコンパイルもできます。
`--cc clang` も使えます）。

Windows、Linux、macOS 向けのビルド済み `nx` は
[Releases](https://github.com/Londopy/nexium/releases) ページにあります。
展開して `nx` を `PATH` に置いてください。

または Rust 1.75 以降でソースからビルドします。

```bash
cargo install nexium
```

```bash
cargo install --git https://github.com/Londopy/nexium
```

その後：

```bash
nx run examples/hello.nx
```

## ツアー

```
struct Point derive(Eq) { x: f64, y: f64 }

enum Shape { Circle(f64), Rect { w: f64, h: f64 }, Empty }

fn area(s: Shape) -> f64 {
    match s {
        .Circle(r) => math.PI * r * r,
        .Rect(w, h) => w * h,
        .Empty => 0.0,
    }
}

error ParseError { Empty, NotANumber }

fn parse_num(text: []u8) -> ParseError!i64 {
    if text.len == 0 { return error.Empty }
    var total: i64 = 0
    for c in text {
        if c < '0' or c > '9' { return error.NotANumber }
        total = total * 10 + (c - '0') as i64
    }
    return total
}

fn max(comptime T: type where T: Ord, a: T, b: T) -> T {
    return if a > b { a } else { b }
}

fn main() -> !void {
    let n = try parse_num("1234")
    let bad = parse_num("12x") catch |e| {
        println("caught {}", .{e})
        -1
    }
    var xs = List(i32).new()
    for i in 0..10 { xs.append((i * i) as i32) }
    let found = outer: {
        for x, i in xs { if x > 30 { break :outer i as i64 } }
        -1
    }
    println("{} {} {} {} {}", .{n, bad, max(3, 9), xs.len, found})
}
```

<details>
<summary><b>バイナリパターンマッチ</b></summary>

```
fn parse_ipv4(packet: []u8) -> Net!Ipv4 {
    match packet {
        <<version:4, ihl:4, dscp:6, ecn:2, total_len:16/big,
          id:16, flags:3, frag_off:13, ttl:8, proto:8,
          checksum:16, src:32, dst:32, rest:bytes>> => {
            return Ipv4{ .version = version, .ihl = ihl, .total_len = total_len,
                         .ttl = ttl, .proto = proto, .src = src, .dst = dst }
        }
        _ => return error.Truncated,
    }
}

let written = try <<4:4, 5:4, 0:8, 1500:16/big, "ab">> into buf[..]
```

</details>

<details>
<summary><b>エフェクトは推論され、検査される</b></summary>

```
fn hot(xs: []i32) -> i32 !allocates !panics {
    var list = List(i32).new()
    list.append(1)
    return helper(xs) + list.len as i32
}
```

```
error: function `hot` is declared `!allocates` but has the `allocates` effect
  --> examples/effects_bad.nx:7:26
  note: the effect is introduced here: appending to a List may grow it
  --> examples/effects_bad.nx:9:5
```

</details>

<details>
<summary><b>C の呼び出しはヘッダを一つインポートするだけ</b></summary>

```
const libc = @cImport("string.h")
const cv = @cImport("cvendor.h")
artifact link { c_sources = ["cvendor.c"] }

unsafe { println("{}", .{libc.strlen(@cstr("hello"))}) }
```

バインディング生成器もビルド手順も不要です。ヘッダがメモリレイアウトの唯一の
情報源であり、外部呼び出しは `ffi` エフェクトを持ちます。

</details>

<details>
<summary><b>並列ループとアリーナ</b></summary>

```
for parallel p, i in positions {
    out[i] = integrate(p)          // ここでは shared_mutable は許されない
}

using arena {
    var scratch = List(Frame).new()   // バンプ確保、まとめて一度に解放
    ...
}
```

</details>

**ドキュメント**

- [Nexium の仕組み](architecture.md)：ソースからバイナリまでのパイプライン、エフェクト推論、所有権、ランタイム、出荷。
- [言語リファレンス](language.md)：コンパイラが実装するすべての構文。
- [組み込み](../../embedding.md)（英語）：出荷したライブラリを Python、Rust、C から呼ぶ。
- [nexium-gui](../../gui.md)（英語）：即時モード GUI ライブラリとウィジェットの書き方。
- [プログラムのリリース](../../releasing-your-program.md)（英語）：タグ一つで三つのプラットフォーム向けバイナリ、インストーラは任意。
- [エディタ対応](../../../editors)（英語）：VS Code 拡張、Sublime の構文、LSP。
- [設計判断](../../../DECISIONS.md)（英語）：仕様が未定だった箇所で下したすべての判断。

## コマンド

| コマンド | 内容 |
| --- | --- |
| `nx build file.nx` | 実行ファイルにコンパイル（`main` がなければオブジェクト） |
| `nx run file.nx` | ビルドして実行 |
| `nx test file.nx [filter]` | `test "..."` ブロックを実行 |
| `nx check file.nx` | 型検査とエフェクト違反の報告 |
| `nx effects file.nx` | 各関数の推論されたエフェクトを表示 |
| `nx audit file.nx` | `unsafe` ブロックと可変グローバルを一覧 |
| `nx ship file.nx` | 宣言されたすべての `artifact` を生成 |
| `nx emit-c file.nx` | 生成された C を表示 |
| `nx tokens file.nx` | トークン列を出力（セルフホスティングの基準） |
| `nx fmt file.nx [--check]` | 正規の整形 |
| `nx doc file.nx` | 推論されたエフェクト付きの HTML ドキュメント |
| `nx size file.nx` | バイナリのバイト数を宣言ごとに帰属 |
| `nx refcounts file.nx` | すべての retain と release の箇所 |
| `nx leaks file.nx` | 確保を追跡して実行し、リークを報告 |
| `nx lsp` | stdio 上の言語サーバー |

オプション：`--mode debug|safe|fast|small`、`--target x86_64-linux-gnu`
（`zig cc` が知る任意のターゲット）、`--out-dir`、`--keep-c`、`--cc`、C 連携用の
`-I`、`--link`、`--link-path`、`--c-source`。

## 現状

これは `nexium-spec.txt` の設計の最初の実装です。実際のプログラムを書くのに
十分であり（[`examples/`](../../../examples) 参照）、一つのファイルから Python、
Rust、C のコンポーネントを出荷できます。トレイトオブジェクト、並列ループ、
アリーナスコープ、C ヘッダの直接インポート、ツール群はすべて揃っています。
まだ初期段階です。標準ライブラリは第 16 節のごく一部で、リージョン検査は返さ
れるビューにしか及びません。[`DECISIONS.md`](../../../DECISIONS.md) には仕様が
未定だった箇所で下したすべての判断があり、27 番に未実装の項目が並んでいます。

## セルフホスティング

現在のコンパイラは Rust 製です。Nexium 版は [`self/`](../../../self) で一段階
ずつ育ち、各段階は同じ入力で Rust コンパイラと照合されます。

| 段階 | ファイル | 基準 | 状態 |
| --- | --- | --- | --- |
| 字句解析器 | [`self/lexer.nx`](../../../self/lexer.nx) | `nx tokens` | ✅ すべての例と自分自身で一致 |
| 構文解析器 | [`self/parser.nx`](../../../self/parser.nx) | `nx sexp` | ✅ 46 ソースすべてで一致 |
| 検査器 | [`self/check.nx`](../../../self/check.nx) | `nx tir` | 🚧 宣言とシグネチャが一致（`--sigs`、41 ソース）；本体は作業中 |
| C 生成器 | | `nx emit-c` | |

`cargo test` は Rust コンパイラで `self/lexer.nx` をビルドし、出力を基準と比較
します。

## リポジトリの言語構成

空行を除いたコード行数。ビルド成果物、依存、生成ファイルは除外：

| 言語 | 行数 | 割合 | 用途 |
| --- | --- | --- | --- |
| Rust | 22 393 | 86.9 % | `nx` コンパイラ |
| Nexium | 2 111 | 8.2 % | 例、セルフホスト字句解析器、nexium-gui、テスト |
| C | 1 108 | 4.3 % | ランタイム `nx_rt.h` と GUI のウィンドウ層 |
| JavaScript、TypeScript | 159 | 0.6 % | VS Code 拡張 |

## 構成

```
src/            コンパイラ（字句、構文、検査、comptime、C バックエンド、ドライバ）
runtime/        nx_rt.h、生成されるすべての C ファイルに埋め込まれる
self/           Nexium で書かれたコンパイラ、段階ごと
gui/            nexium-gui：Nexium の即時モード GUI、デモ、C プラットフォーム層
editors/        VS Code 拡張と Sublime Text の構文
examples/       出力を記録したプログラム、`cargo test` が実行
tests/          統合テストと compile-fail ケース
docs/           仕組み、言語リファレンス、組み込みガイド、翻訳
assets/         ロゴとバナー
nexium-spec.txt          設計
nexium-systems-spec.txt  アーカイブされたシステム言語。第 4〜9 節が構文リファレンス
DECISIONS.md    仕様が未定だった箇所での判断
```

## 貢献

[`CONTRIBUTING.md`](../../../CONTRIBUTING.md) を参照。バグと提案は GitHub issues
へ。言語の変更は、それが仕様第 3 節のどの厳格な制約に資するかを示す必要が
あります。

## ライセンス

MIT. Copyright (c) 2026 Londopy.

<p align="center"><img src="https://raw.githubusercontent.com/Londopy/nexium/main/assets/logo.svg" alt="" width="48"></p>
