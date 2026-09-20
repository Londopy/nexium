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
  <a href="https://londopy.github.io/nexium/"><img alt="Docs" src="https://img.shields.io/badge/docs-londopy.github.io%2Fnexium-5b4bd6"></a>
  <a href="../../../LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-blue"></a>
  <a href="https://ziglang.org/download/"><img alt="Zig" src="https://img.shields.io/badge/backend-zig%20cc-f7a41d?logo=zig&logoColor=white"></a>
  <img alt="Platforms" src="https://img.shields.io/badge/platforms-windows%20%7C%20linux%20%7C%20macos-2b3a55">
</p>

<p align="center">
  <b>すべてを作れるほど完全でありながら、別のものの一部分として採用するにも最良の言語。</b>
</p>

<p align="center">
  <a href="https://londopy.github.io/nexium/"><b>ドキュメントとチュートリアル「the Topo」 &rarr; londopy.github.io/nexium</b></a><br>
  <sub><a href="https://londopy.github.io/nexium/topo/01-base-camp.html">Topo から始める</a> &middot; <a href="https://londopy.github.io/nexium/docs/language.html">言語リファレンス</a> &middot; <a href="https://londopy.github.io/nexium/docs/install.html">インストール</a> &middot; <a href="https://londopy.github.io/nexium/docs/std.html">標準ライブラリ</a> &middot; <a href="https://londopy.github.io/nexium/docs/embedding.html">組み込み</a>（英語）</sub>
</p>

Nexium は C を経由してネイティブコードにコンパイルされ、トレース型ガベージ
コレクタなしの自動参照カウントを持ち、関数がメモリを確保するか、ブロックするか、
パニックしうるかを機械的に検証するエフェクトシステムを備え、ひとつのソース
ツリーから C ライブラリ、Python の wheel、Rust のクレート、あるいはコマンド
ラインツールを生み出すコンパイラを持ちます。

<table>
<tr>
<td width="50%" valign="top">

**ひとつのファイル**

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

エフェクトのシグネチャが C ABI を決めます。`checksum` は `!panics` が証明されて
いるので、素の `uint32_t checksum(const uint8_t*, size_t)` になります。失敗しうる
関数はステータスコードを返し、その中で起きた Nexium のパニックはホストプロセスを
中断させる代わりに境界で変換されます。

## 特長

| | |
| --- | --- |
| 🧾 **推論され検証されるエフェクト** | `allocates` `refcounts` `blocks` `shared_mutable` `nondeterministic` `panics` `ffi`。`!allocates` と宣言すれば、コンパイラは呼び出しをたどって、それを破る正確な行を指し示します。 |
| 🧠 **借用チェッカのない所有権** | コレクションはムーブし、`.clone()` はコピーし、`ref class` の値は参照カウントされ、`weak` が循環を断ちます。ムーブ後の使用はコンパイルエラーです。 |
| 🔬 **バイナリパターン** | `<<version:4, ihl:4, len:16/big, rest:bytes>>` がサイズ検証つきでパケットを照合し、組み立てます。 |
| 🧵 **並列ループ、アリーナ、トレイトオブジェクト** | `for parallel`、`using arena { }`、`dyn Trait !allocates`。 |
| 🔌 **バインディング不要の C** | `@cImport("header.h")` がヘッダを直接読み、`artifact link` が同梱の C をプログラムに組み込みます。 |
| 📦 **ひとつのソースから出荷** | `nx ship` が C のヘッダとライブラリ、Python の wheel、安全なラッパつきの Rust クレートを生成します。 |
| 🖼 **Nexium 製の GUI** | [`gui/`](../../../gui)：ソフトウェアラスタライザとビットマップフォントを持つ即時モード GUI（ボタン、スライダ、テキスト欄）。200 行の C ウィンドウ層の上はすべて Nexium です。 |
| 🛠 **同梱のツール** | `fmt`、`doc`、`lsp`、`size`、`leaks`、`refcounts`、`effects`、`audit`。依存関係ゼロ。 |

## インストール

**Windows**：[Releases](https://github.com/Londopy/nexium/releases) ページから
インストーラをダウンロードして実行します。`nx`、同梱の Zig ツールチェーン
（`nx` が使う C コンパイラ）、標準ライブラリ、サンプル、ドキュメント、VS Code
拡張をインストールし、`nx` を PATH に加えます。ほかに入れるものはありません。

**macOS と Linux**：

```bash
curl -fsSL https://raw.githubusercontent.com/Londopy/nexium/main/installers/install.sh | sh
```

ダウンロードをリリースのチェックサムと照合し、`~/.nexium` にインストールし、
C コンパイラを用意し（macOS では Xcode のツール、Linux では見つからなければ Zig を
ダウンロード）、`nx` を PATH に加えます。

その後、新しいコンソールで `nx doctor` を実行すると何が使われるかが分かります。
チェックサムの検証やすべての環境変数を含む詳細は
[docs/install.md](../../install.md)（英語）にあります。

あるいは C コンパイラだけ（PATH 上の Zig、または `CC`）でソースからビルドします。
Nexium で書かれたコンパイラをその C のシードから組み立てます：

```bash
git clone https://github.com/Londopy/nexium && cd nexium && sh bootstrap/build.sh
```

結果は `nx-out/bootstrap/nx2` です（Windows では `build.ps1`）。続けて：

```bash
nx run examples/hello.nx
```

## ひとめぐり

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
<summary><b>バイナリパターンマッチング</b></summary>

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
<summary><b>エフェクトは推論され、検証される</b></summary>

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
<summary><b>C の呼び出しはヘッダのインポートひとつ</b></summary>

```
const libc = @cImport("string.h")
const cv = @cImport("cvendor.h")
artifact link { c_sources = ["cvendor.c"] }

unsafe { println("{}", .{libc.strlen(@cstr("hello"))}) }
```

バインディング生成器もビルド手順も不要です。ヘッダがレイアウトの唯一の情報源で、
外部呼び出しは `ffi` エフェクトを帯びます。

</details>

<details>
<summary><b>並列ループとアリーナ</b></summary>

```
for parallel p, i in positions {
    out[i] = integrate(p)          // ここでは shared_mutable は許されない
}

using arena {
    var scratch = List(Frame).new()   // バンプ確保、まとめて解放
    ...
}
```

</details>

**ドキュメント**

- [仕様](../../../SPEC.md)（英語）：実装済みの言語。計画中の部分に印つき。
- [ロードマップ](../../../ROADMAP.md)（英語）：フェーズ、完了基準、計画しないもの。
- [Nexium の仕組み](architecture.md)：ソースからバイナリまでのパイプライン、エフェクト推論、所有権、ランタイム、出荷。
- [言語リファレンス](language.md)：コンパイラが実装するすべての構文。
- [組み込み](../../embedding.md)（英語）：出荷したライブラリを Python、Rust、C から呼ぶ。
- [対話セッション](../../repl.md)（英語）：プロンプトでの `nx`、`python` のように。
- [安定性](../../stability.md)と[プラットフォーム](../../platforms.md)（英語）：バージョンが約束すること、非推奨のサイクル、`nx fix`、ティア。
- [the Topo](https://londopy.github.io/nexium/topo/01-base-camp.html)（英語）：チュートリアル。コンパイラのインストールからニューラルネットワーク、GUI、出荷するライブラリまで。ソースは [`topo/`](../../../topo/)。以上すべてのレンダリング版は [londopy.github.io/nexium](https://londopy.github.io/nexium/) にあります。
- [インストール](../../install.md)（英語）：Windows インストーラ、macOS/Linux スクリプト、ソースビルド、チェックサム、`nx` が C コンパイラを見つける方法。
- [パッケージ](../../packages.md)（英語）：`nexium.toml`、`nx add`、`nx fetch`、git またはパス依存、ロックファイル。
- [標準ライブラリ](../../std.md)（英語）：Nexium で書かれたモジュール（`std.strings`、`std.lists`、`std.bytes`、`std.num`、`std.json`、`std.args`、`std.fs`、`std.time`、`std.regex`、`std.text`、`std.testing`、`std.stream`、`std.net`、`std.http`、`std.thread`、`std.process`）。
- [nexium-gui](../../gui.md)（英語）：即時モード GUI ライブラリとウィジェットの書き方。
- [プログラムのリリース](../../releasing-your-program.md)（英語）：タグから 3 プラットフォームのバイナリを、インストーラは任意で。
- [エディタ対応](../../../editors)（英語）：VS Code、Vim、Neovim、Helix、Zed、Emacs、Kate、JetBrains、Sublime Text、Notepad++、nano、そのほかは `nx lsp` で。
- [Linguist](../../../linguist)（英語）：利用の閾値に達したら GitHub に `.nx` を認識させる、適用準備済みのプルリクエスト。
- [翻訳](../README.md)：この README を 6 言語で。言語リファレンスとアーキテクチャ案内はスペイン語、中国語、日本語で。
- [リリース名](../../release-names.md)（英語）：すべてのリリースは山の上の場所。命名規則、台帳、まだ使っていない名前。
- [決定記録](../../../DECISIONS.md)（英語）：仕様が開いていた箇所で下したすべての判断。
- [既知の問題](../../../KNOWN_ISSUES.md)（英語）：未修正のバグ、欠落、制限。再現手順つき。

## コマンド

| コマンド | 何をするか |
| --- | --- |
| `nx build file.nx` | 実行可能ファイルにコンパイル（`main` がなければオブジェクト） |
| `nx run file.nx` | ビルドして実行 |
| `nx test file.nx [filter]` | `test "..."` ブロックを実行 |
| `nx check file.nx` | 型検査とエフェクト違反の報告 |
| `nx effects file.nx` | すべての関数の推論されたエフェクトを表示 |
| `nx audit file.nx` | `unsafe` ブロックと可変グローバルを列挙 |
| `nx ship file.nx` | 宣言されたすべての `artifact` を生成 |
| `nx emit-c file.nx` | 生成された C を表示 |
| `nx tir file.nx [--sigs]` | 検査済みプログラムを S 式で（コンパイラ自身のテストが読む） |
| `nx fmt file.nx [--check]` | 正規の整形 |
| `nx fix file.nx` | コンパイラが移行できる非推奨の形を書き換える（1.0 ではなし。[docs/stability.md](../../stability.md) 参照） |
| `nx doc file.nx` | 推論されたエフェクトつきの HTML ドキュメント |
| `nx size file.nx` | バイナリのバイト数を宣言ごとに帰属 |
| `nx refcounts file.nx` | すべての retain と release の箇所 |
| `nx leaks file.nx` | 確保を追跡しながら実行し、リークを報告 |
| `nx lsp` | stdio 上の言語サーバ |
| `nx doctor` | どの C コンパイラが使われるか、インストールが動くか |
| `nx repl`、または単に `nx` | 対話セッション：コードを打ち、値を見て、束縛を保つ |

オプション：`--mode debug|safe|fast|small`、`--target x86_64-linux-gnu`
（`zig cc` が知るあらゆるターゲット）、`--out-dir`、`--keep-c`、`--cc`、
C 連携用に `-I`、`--link`、`--link-path`、`--c-source`。

## 現状

**1.0：言語は安定、エコシステムは初期段階。** 言語は
[安定性ポリシー](../../stability.md)のもと追加によってのみ変わります。コンパイラは
Nexium で書かれ、自分自身をビルドします。すべてのサンプル、仕様ケース、チュートリアルの
プログラムが CI で 3 プラットフォーム上、サニタイザとファザーのもとで実行されます。
1.0 がまだ何でないか、そしてそれぞれがどこで答えられるかは
[ロードマップ](../../../ROADMAP.md)の最初の節にあります。メモリ安全性は 1.2 まで
保証されません（`unsafe` のないコードでもビューがその記憶域より長く生きることが
あります）。ベンチマークの数値はまだなく、エコシステムはメンテナ一人と標準ライブラリ
16 モジュールです。[`KNOWN_ISSUES.md`](../../../KNOWN_ISSUES.md) は未修正のバグを
その修正案とともに、[`DECISIONS.md`](../../../DECISIONS.md) は仕様が開いていた箇所での
すべての判断を列挙します。

## リリース名

メジャーバージョンは山で、14 座の 8000 メートル峰が初登頂された順に並びます。その下の
バージョンは登攀です。マイナーバージョンにはキャンプ、ルート、壁を、パッチには初登頂
遠征隊のメンバーを、`X.0.0` には `Summit` を当てます。0.x 系は最初に登られた 8000 メートル峰
アンナプルナ（1950 年）へのアプローチとキャンプなので、1.0.0 は `Annapurna: Summit`、
コンパイラが自分自身をビルドし始めた 0.7.0 は頂上アタック前の最終キャンプ
`Annapurna: Camp V` です。名前は changelog、リリースのタイトル、`nx version` に現れます。
[docs/release-names.md](../../release-names.md)（英語）に規則、台帳、これから登る山が
あります。

## セルフホスティング

コンパイラは Nexium で書かれ、[`self/`](../../../self) にあり、自分自身をビルドします。
`nx` のない機械は、コンパイラが自分自身のために出力した C である
[`bootstrap/nx.c`](../../../bootstrap/nx.c) から、任意の C コンパイラだけで、Rust なしに
ひとつ組み立てます：

```sh
sh bootstrap/build.sh     # nx.c -> nx0; nx0 が self/nx.nx をビルド -> nx1; nx1 が自分を同じ C に再ビルド -> nx2
```

| 段階 | ファイル | 役割 |
| --- | --- | --- |
| 字句解析器 | [`self/lexer.nx`](../../../self/lexer.nx) | トークン |
| 構文解析器 | [`self/parser.nx`](../../../self/parser.nx) | id アリーナ上の構文木 |
| 検査器 | [`self/check.nx`](../../../self/check.nx)、[`self/cimport.nx`](../../../self/cimport.nx) | 型、エフェクト、所有権、ジェネリクス、コンパイル時インタプリタ、C ヘッダのインポート、すべての診断 |
| C 生成器 | [`self/cgen.nx`](../../../self/cgen.nx) | プログラムごとに 1 つの C ファイル |
| ドライバ | [`self/nx.nx`](../../../self/nx.nx) | build、run、test、check、emit-c、tir。標準ライブラリを内蔵 |
| ツール | [`self/fmt.nx`](../../../self/fmt.nx)、[`self/doc.nx`](../../../self/doc.nx)、[`self/tools.nx`](../../../self/tools.nx)、[`self/size.nx`](../../../self/size.nx)、[`self/manifest.nx`](../../../self/manifest.nx)、[`self/ship.nx`](../../../self/ship.nx)、[`self/lsp.nx`](../../../self/lsp.nx)、[`self/repl.nx`](../../../self/repl.nx) | 整形器、ドキュメント生成器、各種レポート、パッケージ、`ship`、言語サーバ、REPL |

すべてのサンプル、仕様ケース、コンパイル失敗ケースが、それ自体 Nexium プログラムである
テストハーネス（`nx run tests/run.nx`）に駆動されて、ブートストラップされたコンパイラを
通り、CI で 3 プラットフォーム上、Rust ツールチェーンをいっさい使わずに実行されます。
Rust で書かれた最初のコンパイラは移植を牽引し、1.0 で削除されました（決定 90）。

## リポジトリ内の言語

ビルド出力、依存関係、生成ファイル（`bootstrap/nx.c`、tree-sitter のパーサ、
`gui/font.bin`、ロックファイル）を除いた、空行以外のコード行数：

| 言語 | 行数 | 割合 | 何か |
| --- | --- | --- | --- |
| Nexium | 36,193 | 91.0% | コンパイラとそのツール（`self/` 配下に 25,400 行）、標準ライブラリ、テストハーネスとファザー、サンプル、チュートリアルのプログラム、nexium-gui、サイト生成器、仕様スイート |
| C | 2,021 | 5.1% | ランタイム `nx_rt.h`、GUI のウィンドウ層、同梱のテスト用 C |
| エディタ用ファイル | 1,014 | 2.5% | tree-sitter クエリ、Emacs Lisp、Vim script、Neovim 用 Lua、そして Zed が拡張に要求する 25 行の Rust |
| JavaScript、TypeScript | 550 | 1.4% | VS Code 拡張と tree-sitter 文法 |

コンパイラに Rust はありません。最初のコンパイラは移植を牽引して 1.0 で削除され
（決定 90）、残る Rust は Zed が WebAssembly にコンパイルする Zed 拡張の接着部分だけです。
Zig が表にないのは、ツリーに Zig のソースがないからです。`zig cc` は `nx` が実行する
C コンパイラであり（Windows インストーラが同梱し、インストールスクリプトがダウンロード
します）、C コンパイラが書くものではなく使うものであるのと同じです。

## 構成

```
bootstrap/      コンパイラの元になる C のシードと、ビルドスクリプト
runtime/        nx_rt.h。生成されるすべての C ファイルに埋め込まれる
std/            Nexium で書かれた標準ライブラリ。コンパイラに内蔵
self/           Nexium で書かれたコンパイラ、段階ごとに
gui/            nexium-gui：Nexium の即時モード GUI、デモ、C のプラットフォーム層
editors/        VS Code 拡張、tree-sitter 文法、さらに 10 のエディタ向けファイル
examples/       出力を記録したプログラム。テストが実行する
topo/           チュートリアル：各章と、そこで示すプログラム（テストが実行する）
site/           ドキュメントサイトの生成器。Nexium プログラム
tests/          ハーネス（run.nx）、仕様の適合スイート（tests/spec）、コンパイル失敗ケース
docs/           仕組み、言語リファレンス、組み込みガイド、i18n/ の翻訳
assets/         ロゴとバナー
nexium-spec.txt          設計
nexium-systems-spec.txt  アーカイブされたシステム言語。第 4〜9 節が構文リファレンス
DECISIONS.md    仕様が開いていた箇所で下した決定
KNOWN_ISSUES.md 未修正のバグと制限。修正は changelog へ移る
```

## 貢献

[`CONTRIBUTING.md`](../../../CONTRIBUTING.md) を参照してください。バグと提案は GitHub の
issue で扱います。言語の変更は、仕様第 3 節のどの厳格な制約に資するかを示さなければ
なりません。プルリクエストはマージ前に、3 プラットフォームでのテスト、整形器、changelog
の検査、[コントリビュータ・ライセンス契約](../../../CLA.md)を通ります。著作権はあなたの
ものです。

## ライセンス

MIT. Copyright (c) 2026 Londopy.

<p align="center"><img src="https://raw.githubusercontent.com/Londopy/nexium/main/assets/logo.svg" alt="" width="48"></p>
