<p align="center">
  <img src="https://raw.githubusercontent.com/Londopy/nexium/main/assets/banner.svg" alt="Nexium" width="880">
</p>

<p align="center">
  <a href="../../../README.md">English</a> ·
  <a href="../es/README.md">Español</a> ·
  <b>简体中文</b> ·
  <a href="../ja/README.md">日本語</a> ·
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
  <b>Nexium 是一门完整到足以构建一切的语言，同时也是为别的系统添上一块拼图时的最佳选择。</b>
</p>

<p align="center">
  <a href="https://londopy.github.io/nexium/"><b>文档与教程「the Topo」 &rarr; londopy.github.io/nexium</b></a><br>
  <sub><a href="https://londopy.github.io/nexium/topo/01-base-camp.html">从 Topo 开始</a> &middot; <a href="https://londopy.github.io/nexium/docs/language.html">语言参考</a> &middot; <a href="https://londopy.github.io/nexium/docs/install.html">安装</a> &middot; <a href="https://londopy.github.io/nexium/docs/std.html">标准库</a> &middot; <a href="https://londopy.github.io/nexium/docs/embedding.html">嵌入</a>（英文）</sub>
</p>

Nexium 经由 C 编译为原生代码，拥有无追踪式垃圾回收器的自动引用计数，
一套由机器检查的效应系统——它能说明一个函数是否分配内存、是否阻塞、是否可能
panic——以及一个能把同一份源码树变成 C 库、Python wheel、Rust crate 或命令行
工具的编译器。

<table>
<tr>
<td width="50%" valign="top">

**一个文件**

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

**每个目标**

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

效应签名决定 C ABI：`checksum` 被证明为 `!panics`，因此得到一个朴素的
`uint32_t checksum(const uint8_t*, size_t)`。可能失败的函数返回状态码，其中发生的
Nexium panic 会在边界处被转换，而不是让宿主进程中止。

## 亮点

| | |
| --- | --- |
| 🧾 **效应：推断并检查** | `allocates` `refcounts` `blocks` `shared_mutable` `nondeterministic` `panics` `ffi`。声明 `!allocates`，编译器会穿过调用链，指出会破坏它的那一行。 |
| 🧠 **没有借用检查器的所有权** | 集合按移动传递，`.clone()` 复制，`ref class` 值按引用计数，`weak` 打破循环。移动后再使用是编译错误。 |
| 🔬 **二进制模式** | `<<version:4, ihl:4, len:16/big, rest:bytes>>` 在校验长度的前提下匹配和构造数据包。 |
| 🧵 **并行循环、arena、trait 对象** | `for parallel`、`using arena { }`、`dyn Trait !allocates`。 |
| 🔌 **无需绑定的 C** | `@cImport("header.h")` 直接读取头文件；`artifact link` 把随附的 C 编译进程序。 |
| 📦 **一份源码，多处交付** | `nx ship` 生成 C 头文件和库、Python wheel，以及带安全封装的 Rust crate。 |
| 🖼 **用 Nexium 写的 GUI** | [`gui/`](../../../gui)：即时模式 GUI（按钮、滑块、文本框），带软件光栅化器和位图字体，200 行 C 窗口层之上全是 Nexium。 |
| 🛠 **自带工具** | `fmt`、`doc`、`lsp`、`size`、`leaks`、`refcounts`、`effects`、`audit`。零依赖。 |

## 安装

**Windows**：从 [Releases](https://github.com/Londopy/nexium/releases) 页面下载并运行
安装程序。它会安装 `nx`、随附的 Zig 工具链（`nx` 使用的 C 编译器）、标准库、示例、
文档和 VS Code 扩展，并把 `nx` 加入 PATH。无需再装别的东西。

**macOS 与 Linux**：

```bash
curl -fsSL https://raw.githubusercontent.com/Londopy/nexium/main/installers/install.sh | sh
```

脚本会用发布版的校验和核对下载内容，安装到 `~/.nexium`，准备好 C 编译器
（macOS 上是 Xcode 工具；Linux 上找不到时会下载 Zig），并把 `nx` 加入 PATH。

**Windows，在 PowerShell 中**：`irm https://raw.githubusercontent.com/Londopy/nexium/main/installers/install.ps1 | iex`
（便携版，已校验，加入 PATH；无向导）。

**pip 或 npm**：`pip install nexium-lang` 或 `npm install -g nexium-lang`（按平台提供的二进制；C 编译器照常需要）。

**Docker**：`docker run --rm -v "$PWD":/work ghcr.io/londopy/nexium run hello.nx`
（Debian；也有 `:alpine`；amd64 与 arm64）。

**Homebrew 与 Scoop**：这个仓库本身就是 tap 和 bucket。

```bash
brew tap londopy/tap https://github.com/Londopy/nexium && brew install londopy/tap/nexium
```

```powershell
scoop install https://raw.githubusercontent.com/Londopy/nexium/main/bucket/nexium.json
```

然后在新的终端里运行 `nx doctor`，它会显示将使用什么。包括校验和验证与每个
环境变量在内的全部细节，见 [docs/install.md](../../install.md)（英文）。

或者只用一个 C 编译器（PATH 上的 Zig，或 `CC`）从源码构建——这会从 C 种子构建出
用 Nexium 写的编译器：

```bash
git clone https://github.com/Londopy/nexium && cd nexium && sh bootstrap/build.sh
```

结果是 `nx-out/bootstrap/nx2`（Windows 上用 `build.ps1`）。然后：

```bash
nx run examples/hello.nx
```

## 概览

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
<summary><b>二进制模式匹配</b></summary>

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
<summary><b>效应被推断并检查</b></summary>

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
<summary><b>调用 C 只需导入一个头文件</b></summary>

```
const libc = @cImport("string.h")
const cv = @cImport("cvendor.h")
artifact link { c_sources = ["cvendor.c"] }

unsafe { println("{}", .{libc.strlen(@cstr("hello"))}) }
```

没有绑定生成器，没有构建步骤：头文件是内存布局唯一的真相来源，外部调用带有
`ffi` 效应。

</details>

<details>
<summary><b>并行循环与 arena</b></summary>

```
for parallel p, i in positions {
    out[i] = integrate(p)          // 这里不允许 shared_mutable
}

using arena {
    var scratch = List(Frame).new()   // 线性分配，一次性释放
    ...
}
```

</details>

**文档**

- [规范](../../../SPEC.md)（英文）：已实现的语言，计划中的部分有标注。
- [路线图](../../../ROADMAP.md)（英文）：阶段、完成标准，以及不打算做的事。
- [Nexium 如何工作](architecture.md)：从源码到二进制的流水线、效应推断、所有权、运行时与交付。
- [语言参考](language.md)：编译器实现的每一种构造。
- [嵌入](../../embedding.md)（英文）：从 Python、Rust 和 C 调用交付的库。
- [交互式会话](../../repl.md)（英文）：在提示符下的 `nx`，像 `python` 一样。
- [稳定性](../../stability.md)与[平台](../../platforms.md)（英文）：一个版本承诺什么、弃用周期、`nx fix`、支持层级。
- [the Topo](https://londopy.github.io/nexium/topo/01-base-camp.html)（英文）：教程，从安装编译器到神经网络、GUI 和一个交付出去的库；源码在 [`topo/`](../../../topo/)。以上全部的渲染版本在 [londopy.github.io/nexium](https://londopy.github.io/nexium/)。
- [安装](../../install.md)（英文）：Windows 安装程序、macOS/Linux 脚本、源码构建、校验和，以及 `nx` 如何找到 C 编译器。
- [包](../../packages.md)（英文）：`nexium.toml`、`nx add`、`nx fetch`、git 或路径依赖、锁文件。
- [标准库](../../std.md)（英文）：用 Nexium 写的模块（`std.strings`、`std.lists`、`std.bytes`、`std.num`、`std.json`、`std.args`、`std.fs`、`std.time`、`std.regex`、`std.text`、`std.testing`、`std.stream`、`std.net`、`std.http`、`std.thread`、`std.process`）。
- [nexium-gui](../../gui.md)（英文）：即时模式 GUI 库以及如何编写一个控件。
- [发布你的程序](../../releasing-your-program.md)（英文）：从一个标签得到三个平台的二进制，安装程序可选。
- [编辑器支持](../../../editors)（英文）：VS Code、Vim、Neovim、Helix、Zed、Emacs、Kate、JetBrains、Sublime Text、Notepad++、nano，其余的用 `nx lsp`。
- [Linguist](../../../linguist)（英文）：达到使用量门槛后，让 GitHub 识别 `.nx` 的现成 pull request。
- [翻译](../README.md)：本 README 的六种语言版本；语言参考和架构导览有西班牙语、中文和日语版本。
- [发布版名称](../../release-names.md)（英文）：每个发布版都是山上的一个地方；命名规则、台账，以及尚未用过的名字。
- [决策记录](../../../DECISIONS.md)（英文）：规范未定之处做出的每一个决定。
- [已知问题](../../../KNOWN_ISSUES.md)（英文）：未修复的缺陷、缺口与限制，附复现步骤。

## 命令

| 命令 | 作用 |
| --- | --- |
| `nx build file.nx` | 编译为可执行文件（没有 `main` 时为目标文件） |
| `nx run file.nx` | 构建并运行；`--watch` 在程序的任一文件变化时重新运行 |
| `nx test file.nx [filter]` | 运行 `test "..."` 块；也支持 `--watch` |
| `nx check file.nx` | 类型检查并报告效应违规 |
| `nx effects file.nx` | 打印每个函数推断出的效应 |
| `nx explain file.nx f effect` | `f` 为何带有该效应：把效应带进来的调用链，直到原语，以树的形式 |
| `nx audit file.nx` | 列出 `unsafe` 块和可变全局变量；`--lock` 写出效应锁文件，`--check` 在新增效应时失败 |
| `nx ship file.nx` | 生成声明的每个 `artifact` |
| `nx emit-c file.nx` | 打印生成的 C |
| `nx tir file.nx [--sigs]` | 以 S 表达式输出检查后的程序（编译器自己的测试会读取它） |
| `nx fmt file.nx [--check]` | 规范化格式 |
| `nx fix file.nx` | 改写编译器能迁移的已弃用写法（1.0 中没有；见 [docs/stability.md](../../stability.md)） |
| `nx doc file.nx` | 带推断效应的 HTML 文档 |
| `nx size file.nx` | 把二进制的字节归因到各声明 |
| `nx layout file.nx [Type...]` | struct 或 enum 的偏移、大小和填充，以及能缩小它的按对齐排序 |
| `nx upgrade` | 用最新发布版替换这个可执行文件，经过校验；`--check` 只报告 |
| `nx install [DIR]` | 把这份副本连同旁边的文件装到用户目录并加入 PATH（便携 zip 自行安装） |
| `nx refcounts file.nx` | 每一处 retain 和 release |
| `nx leaks file.nx` | 带分配跟踪运行并报告泄漏 |
| `nx lsp` | 基于 stdio 的语言服务器 |
| `nx doctor` | 将使用哪个 C 编译器，安装是否正常 |
| `nx repl`，或直接 `nx` | 交互式会话：输入代码，查看值，保留绑定 |

选项：`--mode debug|safe|fast|small`、`--target x86_64-linux-gnu`（`zig cc` 认识的
任何目标）、`--cpu baseline|native|<名称>`（默认 baseline，这样二进制能在同架构的
任何机器上运行）、`--out-dir`、`--keep-c`、`--cc`、`--strict`（警告即
错误）、`--sanitize address,undefined`（C 编译器的 sanitizer；`address`
需要 gcc 或 clang），以及用于 C 互操作的 `-I`、`--link`、
`--link-path`、`--c-source`。

## 现状

**1.0：语言已稳定，生态尚在早期。** 语言只按[稳定性策略](../../stability.md)以增添的
方式变化；编译器用 Nexium 写成并能构建自身；每个示例、规范用例和教程程序都在 CI 中于
三个平台上、在 sanitizer 和 fuzzer 之下运行。1.0 还不是什么、每一点在哪里得到回答，
是[路线图](../../../ROADMAP.md)的第一节：内存安全是 1.2 的视图规则，自 1.3 起是
错误（机械性的修正由 `nx fix` 完成），除[数字页](../../numbers.md)（五种语言的四个
程序在同一台 runner 上，每周重新生成）之外还没有基准数字，生态只有一位维护者、
十六个标准库模块和一个来自树外的包（statusmith 的 [Discord Rich Presence SDK](../../discord.md)，
`nx add discord_rpc ...`）。[`KNOWN_ISSUES.md`](../../../KNOWN_ISSUES.md) 列出每个未修复的缺陷
及其修法；[`DECISIONS.md`](../../../DECISIONS.md) 列出规范未定之处做出的每一个决定。

## 发布版名称

主版本是一座山，按十四座八千米高峰首登的顺序排列；它之下的版本是攀登本身：次版本用
营地、路线和山壁，补丁版用首登远征队的成员，`X.0.0` 用 `Summit`。0.x 系列是最早被登顶
的八千米峰安纳普尔纳（1950 年）的进山路与营地，所以 1.0.0 是 `Annapurna: Summit`；
编译器开始构建自身的 0.7.0 是冲顶前的最后一个营地 `Annapurna: Camp V`。名字出现在
changelog、发布标题和 `nx version` 里；[docs/release-names.md](../../release-names.md)
（英文）有规则、台账和尚待攀登的山。

## 自举

编译器用 Nexium 写成，位于 [`self/`](../../../self)，并能构建自身。一台没有 `nx` 的机器
可以从 [`bootstrap/nx.c`](../../../bootstrap/nx.c)——编译器为自己生成的 C——用任何 C
编译器、不用 Rust 构建出一个：

```sh
sh bootstrap/build.sh     # nx.c -> nx0；nx0 构建 self/nx.nx -> nx1；nx1 把自己重新构建成同样的 C -> nx2
```

| 阶段 | 文件 | 职责 |
| --- | --- | --- |
| 词法分析器 | [`self/lexer.nx`](../../../self/lexer.nx) | 记号 |
| 语法分析器 | [`self/parser.nx`](../../../self/parser.nx) | id arena 上的语法树 |
| 检查器 | [`self/check.nx`](../../../self/check.nx)、`self/check_*.nx`、[`self/cimport.nx`](../../../self/cimport.nx) | 类型、效应、所有权、泛型、编译期解释器、C 头文件导入、每一条诊断 |
| C 生成器 | [`self/cgen.nx`](../../../self/cgen.nx) | 每个程序一个 C 文件 |
| 驱动 | [`self/nx.nx`](../../../self/nx.nx) | build、run、test、check、emit-c、tir；内嵌标准库 |
| 工具 | [`self/fmt.nx`](../../../self/fmt.nx)、[`self/doc.nx`](../../../self/doc.nx)、[`self/tools.nx`](../../../self/tools.nx)、[`self/size.nx`](../../../self/size.nx)、[`self/manifest.nx`](../../../self/manifest.nx)、[`self/ship.nx`](../../../self/ship.nx)、[`self/lsp.nx`](../../../self/lsp.nx)、[`self/repl.nx`](../../../self/repl.nx) | 格式化器、文档生成器、各类报告、包管理、`ship`、语言服务器、REPL |

每个示例、每个规范用例和每个编译失败用例都经由自举出的编译器运行，由本身就是
Nexium 程序的测试框架（`nx run tests/run.nx`）驱动，在 CI 中于三个平台上运行，完全
不用 Rust 工具链。用 Rust 写的第一个编译器推动了移植，并在 1.0 时被删除（决策 90）。

## 仓库中的语言

非空代码行数，不含构建输出、依赖和生成文件（`bootstrap/nx.c`、tree-sitter 解析器、
`gui/font.bin`、锁文件）：

| 语言 | 行数 | 占比 | 是什么 |
| --- | --- | --- | --- |
| Nexium | 38,374 | 85.4% | 编译器及其工具（`self/` 下 27,100 行）、标准库、测试框架与 fuzzer、示例、教程程序、nexium-gui、站点生成器、规范测试套件、四个基准程序 |
| C | 2,925 | 6.5% | 运行时 `nx_rt.h`、GUI 窗口层、随附的测试用 C、一个基准程序 |
| Python | 1,063 | 2.4% | 发布脚本（说明、包清单、wheel 与 npm 包、std 文档）、基准运行器、一个基准程序 |
| 编辑器文件 | 1,028 | 2.3% | tree-sitter 查询、Emacs Lisp、Vim script、Neovim 用的 Lua，以及 Zed 对扩展要求的 25 行 Rust |
| JavaScript、TypeScript | 550 | 1.2% | VS Code 扩展和 tree-sitter 语法 |
| Inno Setup、shell、PowerShell | 777 | 1.7% | Windows 安装程序脚本、`install.sh`、`install.ps1`、Chocolatey 脚本 |
| Rust、Go、Ruby | 236 | 0.5% | Rust 与 Go 各一个基准程序，以及 Homebrew 公式 |

编译器里没有 Rust：第一个编译器推动了移植并在 1.0 时被删除（决策 90）。剩下的 Rust
是 Zed 扩展的胶水代码，由 Zed 编译为 WebAssembly，以及一个用来对照测量的基准程序，
旁边是它的 Go 孪生版本。Zig 不在表中，因为树里没有 Zig
源码：`zig cc` 是 `nx` 运行的 C 编译器（Windows 安装程序随附，安装脚本下载），正如
C 编译器是拿来用的，不是拿来写的。

## 目录结构

```
bootstrap/      构建编译器所用的 C 种子，以及构建脚本
runtime/        nx_rt.h，嵌入每一个生成的 C 文件
std/            用 Nexium 写的标准库，内嵌于编译器
self/           用 Nexium 写的编译器，逐阶段
gui/            nexium-gui：Nexium 即时模式 GUI、演示，以及 C 平台层
editors/        VS Code 扩展、tree-sitter 语法，以及另外十种编辑器的文件
examples/       带记录输出的程序，由测试运行
topo/           教程：各章及其展示的程序（由测试运行）
site/           文档站点生成器，一个 Nexium 程序
tests/          测试框架（run.nx）、规范一致性套件（tests/spec）和编译失败用例
docs/           工作原理、语言参考、嵌入指南、i18n/ 下的翻译
bench/          数字页背后的五种语言四个程序
installers/     Windows 安装程序脚本、install.sh 与 install.ps1、winget 与 Chocolatey 清单
docker/         ghcr.io 上的编译器镜像（Debian 与 Alpine）
Formula/, bucket/  这个仓库作为 Homebrew tap 与 Scoop bucket（每次发布时写入）
scripts/        发布说明、包清单、wheel 与 npm 包、std 文档
assets/         标志、横幅与社交预览图
nexium-spec.txt          设计
nexium-systems-spec.txt  已归档的系统语言；第 4 至 9 节是语法参考
DECISIONS.md    规范未定之处做出的决定
KNOWN_ISSUES.md 未修复的缺陷与限制；修复后移入 changelog
```

## 贡献

见 [`CONTRIBUTING.md`](../../../CONTRIBUTING.md)。缺陷与提案走 GitHub issue；对语言的
改动必须说明它服务于规范第 3 节中的哪一条硬性约束。Pull request 在合并前要通过三个
平台的测试、格式化器、changelog 检查以及[贡献者许可协议](../../../CLA.md)；版权仍归
你所有。

## 许可证

MIT. Copyright (c) 2026 Londopy.

<p align="center"><img src="https://raw.githubusercontent.com/Londopy/nexium/main/assets/logo.svg" alt="" width="48"></p>
