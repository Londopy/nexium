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
  <b>Nexium は、すべてを作れるほど完全でありながら、別のものの一部分として採用するにも最良の言語。</b>
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
| 🧾 **推論され検証されるエフェクト** | `allocates` `refcounts` `blocks` `shared_mutable` `nondeterministic` `panics` `ffi`。`!allocates` と宣言すれば、それを破る行を呼び出しをたどってコンパイラが正確に示します。 |
| 🛡 **ガベージコレクタなしのメモリ安全性** | コレクションはムーブし、`.clone()` はコピーし、`ref class` の値は参照カウントされ、`weak` が循環を断ちます。スライスやポインタが指す先の記憶域より長く生きることはありません。ビュー規則は呼び出し、ループ、分岐をまたいで検査され、ライフタイムを書く必要はありません。修正が機械的なら `nx fix` が行います。 |
| 🔬 **バイナリパターン** | `<<version:4, ihl:4, len:16/big, rest:bytes>>` でパケットを照合・構築し、サイズは検査されます。 |
| 🧵 **並列ループ、アリーナ、トレイトオブジェクト** | `for parallel`、`using arena { }`、`dyn Trait !allocates`。 |
| 🔌 **バインディング不要の C** | `@cImport("header.h")` がヘッダを直接読み、`artifact link` が同梱の C をプログラムにコンパイルし、`if comptime @target().0 == "windows"` はプラットフォームが通る分岐だけをビルドします。 |
| 📦 **一つのソースから出荷** | `nx ship` が C のヘッダとライブラリ、Python の wheel、安全なラッパー付きの Rust crate を生成します。 |
| 🐞 **デバッグと計測** | `nx debug` は gdb か lldb を `.nx` の行で止め、文字列、リスト、マップ、オプショナルを値として表示します。`bench "name" { }` ブロックはテストの隣に置けます。`--sanitize address,undefined` はどのビルドにも AddressSanitizer と UBSan をかけます。 |
| 🧭 **ブラウザで学ぶ** | チュートリアル [Topo](https://londopy.github.io/nexium/topo/01-base-camp.html) は WebAssembly にコンパイルしたコンパイラでプログラムをページ上で実行し、ターミナルの `nx topo` と同じように演習を採点します。`nx repl` はプロンプトです。 |
| 🪞 **自分自身で書かれている** | コンパイラは Nexium で、どの C コンパイラでも一つの C ファイルからビルドできます。大きなプログラムのデバッグビルドは変わったモジュールだけを再コンパイルします。 |
| 🖼 **Nexium の GUI** | [`gui/`](../../../gui)：ソフトウェアラスタライザとビットマップフォントを備えたイミディエイトモード GUI（ボタン、スライダー、テキスト欄）。200 行の C のウィンドウ層の上はすべて Nexium です。 |
| 🛠 **同梱のツール** | `fmt`、`fix`、`doc`、`lsp`（定義、ホバー、名前変更をチェッカーから）、`debug`、`bench`、`size`、`layout`、`leaks`、`refcounts`、`effects`、`explain`、`audit`、`repl`。依存なし。 |

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

**Windows、PowerShell から**：`irm https://raw.githubusercontent.com/Londopy/nexium/main/installers/install.ps1 | iex`
（ポータブル版、検証済み、PATH に追加。ウィザードなし）。

**pip または npm**：`pip install nexium-lang`（[PyPI](https://pypi.org/project/nexium-lang/)）または `npm install -g nexium-lang`（[npm](https://www.npmjs.com/package/nexium-lang)）。プラットフォームごとのバイナリで、C コンパイラはいつも通り必要です。

**Docker**：`docker run --rm -v "$PWD":/work ghcr.io/londopy/nexium run hello.nx`
（Debian。`:alpine` もあり。amd64 と arm64）。

**Chocolatey と winget**：`choco install nexium`（[パッケージ](https://community.chocolatey.org/packages/nexium)）は Chocolatey のモデレーターが承認したバージョンから使えます（最新リリースより数日遅れることがあります）。`winget install Londopy.Nexium` は winget が最初のバージョンを取り込んでから使えます（[状況](../../install.md#where-to-get-it)、英語）。

**Debian、RPM、Nix、mise**：各リリースに `.deb` と `.rpm` のパッケージが付きます（`sudo dpkg -i nexium_*_amd64.deb`）。`nix run github:Londopy/nexium` は一つの C ファイルからビルドし、`mise use -g "ubi:Londopy/nexium[exe=nx]"` はリリースのバイナリをインストールします。すべての成果物に署名付きの来歴があります：`gh attestation verify nx --owner Londopy`。[すべての道](../../install.md#where-to-get-it)（英語）。

**ブラウザで**：[リポジトリを Codespace で開けば](https://codespaces.new/Londopy/nexium)、何もインストールせずに 1 分で `nx run examples/hello.nx` が動きます。

**ノートブックで**：[Google Colab](https://colab.research.google.com) や Jupyter では `!pip install -q nexium-lang` のあと、`%%writefile` のセルが書いたファイルに `!nx run hello.nx`（[3 つのセル](../../install.md#in-a-notebook-colab-and-jupyter)、英語）。

**Homebrew と Scoop**：このリポジトリ自体が tap で、Scoop の bucket は [Londopy/scoop-bucket](https://github.com/Londopy/scoop-bucket) です（Scoop 自身の更新機能で最新に保たれます）。

```bash
brew tap londopy/tap https://github.com/Londopy/nexium && brew install londopy/tap/nexium
```

```powershell
scoop bucket add londopy https://github.com/Londopy/scoop-bucket && scoop install nexium
```

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
<summary><b>ビューは記憶域より長く生きない</b></summary>

```
fn main() {
    var names = List(String).new()
    names.append(String.from("ada"))
    let first = names[0][..]
    names.append(String.from("grace"))
    println("{}", .{first})
}
```

```
error: `first` is a view into `names`, which changed on line 5 after the view
was taken; its storage may have moved (rule V3); take the view after the
change, or keep an owned copy of the container (`.clone()`) taken before it
  --> views.nx:6:21
```

スライスやポインタは指す先の記憶域に対して検査されます（SPEC 5.6、規則 V1〜V5）。
ガベージコレクタもなく、書くべきライフタイムもありません。

</details>

<details>
<summary><b>テストとベンチマークを並べて</b></summary>

```
fn sum_to(n: i64) -> i64 {
    var s: i64 = 0
    for i in 0..n { s += i }
    return s
}

test "sums" {
    expect(sum_to(4) == 6)
}

bench "sum to 1000" {
    sum_to(1000)
}
```

```
$ nx bench sums.nx
bench  sum to 1000  189 ns/iter  (min 188 ns, max 197 ns; 21 samples of 63856)

1 benchmark(s), safe mode
```

`nx test` はテストを実行し、`nx bench` はファイルを最適化してビルドし、反復回数を
較正して、ブロックの値を最適化で消されないように保ちます。

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
- [標準ライブラリ](../../std.md)（英語）：Nexium で書かれたモジュール（`std.strings`、`std.lists`、`std.bytes`、`std.num`、`std.json`、`std.args`、`std.fs`、`std.time`、`std.regex`、`std.text`、`std.testing`、`std.stream`、`std.net`、`std.http`、`std.thread`、`std.process`、`std.sort`、`std.heap`、`std.set`、`std.deque`、`std.hash`、`std.path`、`std.env`、`std.uuid`、`std.log`、`std.csv`、`std.toml`、`std.base64`）。
- [数値](https://londopy.github.io/nexium/docs/numbers.html)（英語）：5 言語で書いた 4 つのプログラムを、同じランナーで毎週計測。
- [nexium-gui](../../gui.md)（英語）：即時モード GUI ライブラリとウィジェットの書き方。
- [プログラムのリリース](../../releasing-your-program.md)（英語）：タグから 3 プラットフォームのバイナリを、インストーラは任意で。
- [エディタ対応](../../../editors)（英語）：VS Code、Vim、Neovim、Helix、Zed、Emacs、Kate、JetBrains、Sublime Text、Notepad++、nano、そのほかは `nx lsp` で。
- [Linguist](../../../linguist)（英語）：利用の閾値に達したら GitHub に `.nx` を認識させる、適用準備済みのプルリクエスト。
- [翻訳](../README.md)：この README を 6 言語で。言語リファレンスとアーキテクチャ案内はスペイン語、中国語、日本語で。
- [リリース名](../../release-names.md)（英語）：すべてのリリースは山の上の場所。命名規則、台帳、まだ使っていない名前。
- [決定記録](../../../DECISIONS.md)（英語）：仕様が開いていた箇所で下したすべての判断。
- [既知の問題](../../../KNOWN_ISSUES.md)（英語）：未修正のバグ、欠落、制限。再現手順つき。

## 速度

5 つの言語で同じように書いた 4 つのプログラムを、コンパイル言語ではそれぞれ約 1 秒
かかる大きさにして、GitHub のランナーで計測（2026-09-25、7 回の中央値、5 秒を超えるものは 3 回、
秒、小さいほど速い）：

| プログラム | Nexium safe | Nexium fast | C | Rust | Go | Python |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `fib` (関数呼び出し) | 1.32 | 0.78 | 0.39 | 0.78 | 1.34 | 28.87 |
| `nbody` (浮動小数点) | 0.66 | 0.66 | 0.59 | 0.68 | 0.71 | 48.54 |
| `sieve` (配列) | 0.62 | 0.60 | 0.48 | 0.53 | 0.53 | 4.29 |
| `words` (マップと文字列) | 1.16 | 1.11 | 0.57 | 0.97 | 1.12 | 3.60 |

浮動小数点と配列では Nexium は C の 3 分の 1 増し以内の時間で、浮動小数点では Rust や Go
よりやや速く、配列ではやや遅くなります。関数呼び出しでは `fast` が Rust と同じく C の 2 倍の
時間で、`safe` のオーバーフロー検査はそこに 70% を加えます。マップと文字列は Go と同じ速さで、
C のおよそ 2 倍の時間です。Python は Nexium `fast` の 3〜74 倍の時間がかかります。

`safe` はオーバーフローと範囲の検査を残し、`nx ship` と `nx bench` の既定です。
`fast`（`--mode fast`）は検査を省きます。[数値のページ](https://londopy.github.io/nexium/docs/numbers.html)（英語）には C の時間に
対する倍率、各バージョン、規則も載っています。Bench ワークフローは毎週とリリースごとに
計測し直し、Nexium の時間の C に対する倍率が前回から 4 分の 1 増えると失敗します。

## 実際の利用

このリポジトリの外で Nexium で書かれたプログラムとパッケージ：

| プロジェクト | そこで Nexium がしていること |
| --- | --- |
| [statusmith](https://github.com/Londopy/statusmith)、タスクトレイからの Discord Rich Presence | その SDK は Nexium のパッケージ：`nx add discord_rpc --git https://github.com/Londopy/statusmith --tag sdk-v0.1.0 --dir nexium` でどの Nexium プログラムからもプレゼンスを設定できる（[解説](../../discord.md)） |
| [Point of Origin](https://github.com/Londopy/point-of-origin)、地面そのものがパズルのプラットフォーマー | ビルド全体が Nexium：`build.nx` が Odin のシミュレーションの DLL を動かし、`tools/bindgen.nx` が Odin のエクスポートを読んで Unity が呼ぶ C# バインディングを書き、`tools/levels.nx` がレベルのマップをゲームが読む JSON にコンパイルし（どのレベルも同じシミュレーションで育つので解ける）、`tools/chapters.nx` がそこからドキュメントを書く |
| [QNI](https://github.com/Londopy/qni)、Cal Poly アマチュア無線クラブ（W6BHZ）の Discord のためのネットのリマインダー、チェックインの手助け、ネットコントロールのチュートリアル | プログラム全体が Nexium：スラッシュコマンドとボタンを Webhook で応答し、ボットユーザーも権限もなし。すべてのリクエストは何かを読む前に Discord の Ed25519 署名を検査（nxtls 経由）。ネットのカード、ネットコントロールの練習モード、役員のシート形式のネットログ。偽の Discord を相手にエンドツーエンドでテスト |
| [nxtls](https://github.com/Londopy/nxtls)、純粋な Nexium の暗号と TLS 1.3 | SHA-2、HMAC、HKDF、X25519、ChaCha20-Poly1305、署名検証（Ed25519、ECDSA、RSA）、X.509 チェーンと、その上の TLS 1.3 クライアント。C も `unsafe` もなく、規格のベクタ、Python の `cryptography`、OpenSSL でテスト。QNI はこれを通して Discord と通信。パッケージ：`nx add nxtls --git https://github.com/Londopy/nxtls --tag v0.4.0` |

どこかで Nexium を使っていますか？ issue か pull request を開けばここに載ります。

## コマンド

| コマンド | 何をするか |
| --- | --- |
| `nx build file.nx` | 実行可能ファイルにコンパイル（`main` がなければオブジェクト） |
| `nx run file.nx` | ビルドして実行。`--watch` はプログラムのファイルが変わるたびに再実行 |
| `nx test file.nx [filter]` | `test "..."` ブロックを実行。`--watch` も可 |
| `nx check file.nx` | 型検査とエフェクト違反の報告 |
| `nx effects file.nx` | すべての関数の推論されたエフェクトを表示 |
| `nx explain file.nx f effect` | `f` がその効果を持つ理由：効果を持ち込む呼び出しを、プリミティブまで木として表示 |
| `nx audit file.nx` | `unsafe` ブロックと可変グローバルを列挙。`--lock` は効果のロックファイルを書き、`--check` は効果が増えると失敗 |
| `nx ship file.nx` | 宣言されたすべての `artifact` を生成 |
| `nx init`、`nx add`、`nx fetch`、`nx update` | パッケージのマニフェスト、git またはパスからの依存、ロックファイル（[docs/packages.md](../../packages.md)） |
| `nx emit-c file.nx` | 生成された C を表示 |
| `nx tir file.nx [--sigs]` | 検査済みプログラムを S 式で（コンパイラ自身のテストが読む） |
| `nx fmt file.nx [--check]` | 正規の整形 |
| `nx debug file.nx` | デバッグ用にビルドし gdb か lldb で実行する（`.nx` の行と、String・List・Map・スライス・オプショナルのフォーマッタ） |
| `nx bench file.nx` | `bench "name" { }` ブロックを計測する（最適化ビルド、1 反復あたりの時間の中央値） |
| `nx fix file.nx` | チェッカーの機械的な修正（`.clone()`、`@escape(...)`、`_ = `）を適用し、非推奨の形を移行する（[docs/stability.md](../../stability.md) 参照） |
| `nx doc file.nx` | 推論されたエフェクトつきの HTML ドキュメント |
| `nx size file.nx` | バイナリのバイト数を宣言ごとに帰属 |
| `nx layout file.nx [Type...]` | struct や enum のオフセット、サイズ、パディングと、小さくできる整列順 |
| `nx upgrade` | この実行ファイルを最新リリースに置き換え（検証済み）。`--check` は報告のみ |
| `nx install [DIR]` | このコピーを同梱物ごとユーザーの場所にインストールし PATH に追加（ポータブル zip が自らをインストール） |
| `nx refcounts file.nx` | すべての retain と release の箇所 |
| `nx leaks file.nx` | 確保を追跡しながら実行し、リークを報告 |
| `nx lsp` | stdio 上の言語サーバ |
| `nx doctor` | どの C コンパイラが使われるか、インストールが動くか |
| `nx version` | バージョンとリリース名 |
| `nx completions <shell>`、`nx man` | bash、zsh、fish、PowerShell の補完とマニュアルページ |
| `nx repl`、または単に `nx` | 対話セッション：コードを打ち、値を見て、束縛を保つ |
| `nx -e CODE`、`nx -p EXPR` | プロンプトと同じように一行を実行。`-p` は値を表示 |

オプション：`--mode debug|safe|fast|small`、`--target x86_64-linux-gnu`
（`zig cc` が知るあらゆるターゲット）、`--cpu baseline|native|<名前>`（既定は
baseline。同じアーキテクチャのどのマシンでも動くバイナリになります）、`--out-dir`、
`--keep-c`、`--cc`、`--strict`（警告をエラーに）、`--sanitize
address,undefined`（C コンパイラのサニタイザ。`address` には gcc か clang が必要）、C 連携用に `-I`、
`--link`、`--link-path`、`--c-source`。

## 現状

**1.3：言語は安定、ツールチェーンは成熟。** 言語は
[安定性ポリシー](../../stability.md)のもと追加によってのみ変わります。コンパイラは
Nexium で書かれ、自分自身をビルドします。すべてのサンプル、仕様ケース、チュートリアルの
プログラムが CI で 3 プラットフォーム上、サニタイザとファザーのもとで実行され、gdb と
lldb も `nx debug` を通してそこで動かされます。メモリ安全性はビュー規則で、1.3 から
エラーです。1.4、補う必要のない標準ライブラリは進行中で、コレクション（`std.sort`、
`std.heap`、`std.set`、`std.deque`）、`std.hash`、`random.secure`、パス、環境変数と設定
フォルダ、UUID、ログ、CSV、TOML、base64 が入り、全部で 28 モジュール。`Map` はハッシュ攻撃に強く、
キーを入れた順を保ちます。`std.time` はプラットフォームのデータベースからタイムゾーンを読みます。次は TLS 付きの HTTP クライアントと WebSocket です。Nexium がまだ
何でないか、そしてそれぞれがどこで答えられるかは[ロードマップの一節](../../../ROADMAP.md#what-10-is-not-yet)に
あります。ベンチマークは 4 つのプログラム（[速度](#速度)）だけで、エコシステムは
メンテナ一人とツリー外のプロジェクト四つです（[上](#実際の利用)）。
[`KNOWN_ISSUES.md`](../../../KNOWN_ISSUES.md) は未修正のバグをその修正案とともに、
[`DECISIONS.md`](../../../DECISIONS.md) は仕様が開いていた箇所でのすべての判断を
列挙します。

## リリース名

メジャーバージョンはそれぞれ山の頂上で、14 座の 8000 メートル峰が初登頂された順に
並びます。`X.0.0` は `<山>: Summit` です。登攀は前の系の途中、その `.5` から始まり、
初登頂ルートをキャンプごとに登ります。頂上のあとのマイナーバージョンは `.4` まで、その山の
別のルートと下山です。パッチには初登頂遠征隊のメンバーを当てます。0.x 系は最初に登られた
8000 メートル峰アンナプルナ（1950 年）へのアプローチとキャンプだったので、1.0.0 は
`Annapurna: Summit`、コンパイラが自分自身をビルドし始めた 0.7.0 は頂上アタック前の最終
キャンプ `Annapurna: Camp V` で、1.5 からのリリースは 2.0.0 に向けてエベレストを登ります。名前は changelog、リリースのタイトル、`nx version` に現れます。
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
| 検査器 | [`self/check.nx`](../../../self/check.nx)、`self/check_*.nx`、[`self/cimport.nx`](../../../self/cimport.nx) | 型、エフェクト、所有権、ジェネリクス、コンパイル時インタプリタ、C ヘッダのインポート、すべての診断 |
| C 生成器 | [`self/cgen.nx`](../../../self/cgen.nx) | プログラムごとに 1 つの C ファイル。大きなプログラムのデバッグビルドではモジュールごとに 1 つで、変わっていないものは再利用 |
| ドライバ | [`self/nx.nx`](../../../self/nx.nx) | build、run、test、bench、debug、check、emit-c、tir。標準ライブラリを内蔵 |
| ツール | [`self/fmt.nx`](../../../self/fmt.nx)、[`self/doc.nx`](../../../self/doc.nx)、[`self/tools.nx`](../../../self/tools.nx)、[`self/size.nx`](../../../self/size.nx)、[`self/manifest.nx`](../../../self/manifest.nx)、[`self/ship.nx`](../../../self/ship.nx)、[`self/lsp.nx`](../../../self/lsp.nx)、[`self/lsp_index.nx`](../../../self/lsp_index.nx)、[`self/fix.nx`](../../../self/fix.nx)、[`self/repl.nx`](../../../self/repl.nx) | 整形器、ドキュメント生成器、各種レポート、パッケージ、`ship`、言語サーバとその検査済みプログラムの索引、`nx fix`、REPL |

すべてのサンプル、仕様ケース、コンパイル失敗ケースが、それ自体 Nexium プログラムである
テストハーネス（`nx run tests/run.nx`）に駆動されて、ブートストラップされたコンパイラを
通り、CI で 3 プラットフォーム上、Rust ツールチェーンをいっさい使わずに実行されます。
Rust で書かれた最初のコンパイラは移植を牽引し、1.0 で削除されました（決定 90）。

## リポジトリ内の言語

ビルド出力、依存関係、生成ファイル（`bootstrap/nx.c`、tree-sitter のパーサ、
`gui/font.bin`、ロックファイル）を除いた、空行以外のコード行数：

| 言語 | 行数 | 割合 | 何か |
| --- | --- | --- | --- |
| Nexium | 48,804 | 87.2% | コンパイラとそのツール（`self/` 配下に 33,000 行）、標準ライブラリ（21 モジュール）、テストハーネスとファザー、サンプル、チュートリアルのプログラム、nexium-gui、サイト生成器、ベンチマーク 4 つ |
| C | 2,542 | 4.5% | ランタイム `nx_rt.h`、GUI のウィンドウ層、同梱のテスト用 C、ベンチマーク一つ |
| Python | 1,492 | 2.7% | リリース用スクリプト（ノート、パッケージのマニフェスト、wheel と npm パッケージ、std ドキュメント）、gdb と lldb のフォーマッタ、ベンチマークランナーとベンチマーク 4 つ |
| エディタ用ファイル | 1,103 | 2.0% | tree-sitter クエリ、Emacs Lisp、Vim script、Neovim 用 Lua、Pygments のレキサ、そして Zed が拡張に要求する 25 行の Rust |
| JavaScript、TypeScript | 939 | 1.7% | VS Code 拡張、tree-sitter 文法、プレイグラウンドの WASI 層 |
| Inno Setup、シェル、PowerShell | 855 | 1.5% | Windows インストーラのスクリプト、`install.sh`、`install.ps1`、Chocolatey のスクリプト、ブートストラップのスクリプト |
| Rust、Go、Ruby | 213 | 0.4% | Rust と Go にベンチマークが 4 つずつ、そして Homebrew の formula |

コンパイラに Rust はありません。最初のコンパイラは移植を牽引して 1.0 で削除され
（決定 90）、残る Rust は Zed が WebAssembly にコンパイルする Zed 拡張の接着部分と、
比較対象として書かれたベンチマークプログラム四つ（Go の双子と並んで）です。
Zig が表にないのは、ツリーに Zig のソースがないからです。`zig cc` は `nx` が実行する
C コンパイラであり（Windows インストーラが同梱し、インストールスクリプトがダウンロード
します）、C コンパイラが書くものではなく使うものであるのと同じです。

## 構成

```
bootstrap/      コンパイラの元になる C のシードと、ビルドスクリプト
runtime/        nx_rt.h。生成されるすべての C ファイルに埋め込まれる。nx debug の gdb と lldb のフォーマッタ
std/            Nexium で書かれた標準ライブラリ。コンパイラに内蔵
self/           Nexium で書かれたコンパイラ、段階ごとに
gui/            nexium-gui：Nexium の即時モード GUI、デモ、C のプラットフォーム層
editors/        VS Code 拡張、tree-sitter 文法、さらに 10 のエディタ向けファイル
examples/       出力を記録したプログラム。テストが実行する
topo/           チュートリアル：各章と、そこで示すプログラム（テストが実行する）
site/           ドキュメントサイトの生成器。Nexium プログラム
tests/          ハーネス（run.nx）、仕様の適合スイート（tests/spec）、コンパイル失敗ケース、デバッガの検査
docs/           仕組み、言語リファレンス、組み込みガイド、i18n/ の翻訳
bench/          数値のページを支える 5 言語 4 プログラム
installers/     Windows インストーラのスクリプト、install.sh と install.ps1、winget と Chocolatey のマニフェスト
docker/         ghcr.io 向けのコンパイラのイメージ（Debian と Alpine）
Formula/, bucket/  Homebrew の tap と Scoop の bucket としてのこのリポジトリ（リリースごとに書き出し）
scripts/        リリースノート、パッケージのマニフェスト、wheel と npm パッケージ、std のドキュメント
assets/         ロゴ、バナー、ソーシャルプレビュー
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
