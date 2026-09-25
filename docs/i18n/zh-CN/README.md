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
| 🧾 **推断并检查的效应** | `allocates` `refcounts` `blocks` `shared_mutable` `nondeterministic` `panics` `ffi`。声明 `!allocates`，编译器会顺着调用指出会打破它的那一行。 |
| 🛡 **没有垃圾回收器的内存安全** | 集合会移动，`.clone()` 复制，`ref class` 的值做引用计数，`weak` 打破循环；切片或指针绝不会比它指向的存储活得更久：视图规则跨调用、循环和分支检查，不必写生命周期。修复是机械性的时候，`nx fix` 替你改。 |
| 🔬 **二进制模式** | `<<version:4, ihl:4, len:16/big, rest:bytes>>` 匹配并构造数据包，大小都经过检查。 |
| 🧵 **并行循环、arena、trait 对象** | `for parallel`、`using arena { }`、`dyn Trait !allocates`。 |
| 🔌 **无需绑定的 C** | `@cImport("header.h")` 直接读取头文件；`artifact link` 把随附的 C 编译进程序；`if comptime @target().0 == "windows"` 只构建平台所走的分支。 |
| 📦 **一份源码，处处交付** | `nx ship` 生成 C 头文件与库、Python wheel，以及带安全封装的 Rust crate。 |
| 🐞 **调试与测量** | `nx debug` 让 gdb 或 lldb 停在 `.nx` 行上，把字符串、列表、映射和可选值显示为值；`bench "name" { }` 块与测试并列；`--sanitize address,undefined` 为任意构建加上 AddressSanitizer 与 UBSan。 |
| 🧭 **在浏览器里学** | 教程 [Topo](https://londopy.github.io/nexium/topo/01-base-camp.html) 用编译成 WebAssembly 的编译器在页面中运行它的程序，并像终端里的 `nx topo` 一样给练习评分。`nx repl` 是一个提示符。 |
| 🪞 **用自己写成** | 编译器用 Nexium 写成，任何 C 编译器都能从一个 C 文件构建它；大型程序的调试构建只重新编译改动过的模块。 |
| 🖼 **Nexium 写的 GUI** | [`gui/`](../../../gui)：带软件光栅化器和位图字体的即时模式 GUI（按钮、滑块、文本框），在 200 行 C 窗口层之上全部是 Nexium。 |
| 🛠 **自带工具** | `fmt`、`fix`、`doc`、`lsp`（定义、悬停和重命名来自检查器）、`debug`、`bench`、`size`、`layout`、`leaks`、`refcounts`、`effects`、`explain`、`audit`、`repl`。零依赖。 |

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

**pip 或 npm**：`pip install nexium-lang`（[PyPI](https://pypi.org/project/nexium-lang/)）或 `npm install -g nexium-lang`（[npm](https://www.npmjs.com/package/nexium-lang)）；按平台提供的二进制；C 编译器照常需要。

**Docker**：`docker run --rm -v "$PWD":/work ghcr.io/londopy/nexium run hello.nx`
（Debian；也有 `:alpine`；amd64 与 arm64）。

**Chocolatey 与 winget**：`choco install nexium`（[软件包](https://community.chocolatey.org/packages/nexium)）在 Chocolatey 审核员批准各版本后可用（可能比最新发布晚几天）；`winget install Londopy.Nexium` 在 winget 收录第一个版本之后可用（[状态](../../install.md#where-to-get-it)，英文）。

**Debian、RPM、Nix、mise**：每个发布都附带 `.deb` 和 `.rpm` 包（`sudo dpkg -i nexium_*_amd64.deb`）；`nix run github:Londopy/nexium` 从那一个 C 文件构建；`mise use -g "ubi:Londopy/nexium[exe=nx]"` 安装发布的二进制。每个产物都带有签名的来源证明：`gh attestation verify nx --owner Londopy`。[所有途径](../../install.md#where-to-get-it)（英文）。

**在浏览器中**：[在 Codespace 中打开仓库](https://codespaces.new/Londopy/nexium)，一分钟内即可运行 `nx run examples/hello.nx`，无需安装任何东西。

**在笔记本中**：在 [Google Colab](https://colab.research.google.com) 或 Jupyter 里先 `!pip install -q nexium-lang`，再对 `%%writefile` 单元写出的文件运行 `!nx run hello.nx`（[三个单元](../../install.md#in-a-notebook-colab-and-jupyter)，英文）。

**Homebrew 与 Scoop**：这个仓库本身就是 tap，Scoop 的 bucket 是 [Londopy/scoop-bucket](https://github.com/Londopy/scoop-bucket)，由 Scoop 自己的更新器保持最新。

```bash
brew tap londopy/tap https://github.com/Londopy/nexium && brew install londopy/tap/nexium
```

```powershell
scoop bucket add londopy https://github.com/Londopy/scoop-bucket && scoop install nexium
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
<summary><b>视图绝不比它的存储活得久</b></summary>

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

切片或指针会对照它指向的存储检查（SPEC 5.6，规则 V1 到 V5）：没有垃圾回收器，
也不必写生命周期。

</details>

<details>
<summary><b>测试和基准并排</b></summary>

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

`nx test` 运行测试；`nx bench` 以优化模式构建文件、校准迭代次数，并让块的值
不被优化器消掉。

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
- [标准库](../../std.md)（英文）：用 Nexium 写的模块（`std.strings`、`std.lists`、`std.bytes`、`std.num`、`std.json`、`std.args`、`std.fs`、`std.time`、`std.regex`、`std.text`、`std.testing`、`std.stream`、`std.net`、`std.http`、`std.thread`、`std.process`、`std.sort`、`std.heap`、`std.set`、`std.deque`、`std.hash`、`std.path`、`std.env`、`std.uuid`、`std.log`、`std.csv`、`std.toml`、`std.base64`、`std.websocket`）。
- [数字](https://londopy.github.io/nexium/docs/numbers.html)（英文）：五种语言写的四个程序，每周在同一台 runner 上测量。
- [nexium-gui](../../gui.md)（英文）：即时模式 GUI 库以及如何编写一个控件。
- [发布你的程序](../../releasing-your-program.md)（英文）：从一个标签得到三个平台的二进制，安装程序可选。
- [编辑器支持](../../../editors)（英文）：VS Code、Vim、Neovim、Helix、Zed、Emacs、Kate、JetBrains、Sublime Text、Notepad++、nano，其余的用 `nx lsp`。
- [Linguist](../../../linguist)（英文）：达到使用量门槛后，让 GitHub 识别 `.nx` 的现成 pull request。
- [翻译](../README.md)：本 README 的六种语言版本；语言参考和架构导览有西班牙语、中文和日语版本。
- [发布版名称](../../release-names.md)（英文）：每个发布版都是山上的一个地方；命名规则、台账，以及尚未用过的名字。
- [决策记录](../../../DECISIONS.md)（英文）：规范未定之处做出的每一个决定。
- [已知问题](../../../KNOWN_ISSUES.md)（英文）：未修复的缺陷、缺口与限制，附复现步骤。

## 速度

用五种语言以同样方式编写的四个程序，在编译型语言中各运行约一秒，在 GitHub 的
runner 上测量（2026-09-25；七次运行的中位数，超过五秒的取三次；单位为秒，越小越快）：

| 程序 | Nexium safe | Nexium fast | C | Rust | Go | Python |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `fib` (函数调用) | 1.32 | 0.78 | 0.39 | 0.78 | 1.34 | 28.87 |
| `nbody` (浮点) | 0.66 | 0.66 | 0.59 | 0.68 | 0.71 | 48.54 |
| `sieve` (数组) | 0.62 | 0.60 | 0.48 | 0.53 | 0.53 | 4.29 |
| `words` (映射与字符串) | 1.16 | 1.11 | 0.57 | 0.97 | 1.12 | 3.60 |

在浮点和数组上，Nexium 的用时最多比 C 多三分之一：浮点上比 Rust 和 Go 略快，数组上略慢。
在函数调用上，`fast` 与 Rust 一样用时是 C 的两倍，`safe` 的溢出检查在此之上再多 70%。映射与
字符串的速度与 Go 相同，用时约为 C 的两倍。Python 的用时是 Nexium `fast` 的 3 到 74 倍。

`safe` 保留溢出和边界检查，是 `nx ship` 和 `nx bench` 的默认模式；`fast`
（`--mode fast`）去掉这些检查。[数字页面](https://londopy.github.io/nexium/docs/numbers.html)（英文）还给出每个时间相对 C 的倍数、各工具版本和规则；Bench 工作流
每周和每次发布都会重新测量，Nexium 用时相对 C 的倍数比上次增加四分之一时就会失败。

## 实际使用

本仓库之外用 Nexium 写的程序和包：

| 项目 | Nexium 在其中做什么 |
| --- | --- |
| [statusmith](https://github.com/Londopy/statusmith)，从托盘设置 Discord Rich Presence | 它的 SDK 是一个 Nexium 包：`nx add discord_rpc --git https://github.com/Londopy/statusmith --tag sdk-v0.1.0 --dir nexium` 让任何 Nexium 程序都能设置状态（[说明页](../../discord.md)） |
| [Point of Origin](https://github.com/Londopy/point-of-origin)，地面本身就是谜题的平台游戏 | 整个构建都是 Nexium：`build.nx` 驱动 Odin 模拟的 DLL，`tools/bindgen.nx` 读取 Odin 的导出并写出 Unity 调用的 C# 绑定，`tools/levels.nx` 把关卡地图编译成游戏加载的 JSON（每一关都由同一个模拟生成，因此必有解），`tools/chapters.nx` 据此写出文档 |
| [QNI](https://github.com/Londopy/qni)，为 Cal Poly 业余无线电俱乐部（W6BHZ）的 Discord 提供网络提醒、签到帮助和网络控制教程 | 整个程序都是 Nexium：斜杠命令和按钮通过 webhook 应答，没有机器人用户也不要权限；每个请求在读取任何内容之前先检查 Discord 的 Ed25519 签名（借助 nxtls）；网络卡片、网络控制练习模式，以及干事自己表格格式的网络日志；针对一个假的 Discord 做端到端测试 |
| [nxtls](https://github.com/Londopy/nxtls)，纯 Nexium 的密码学与 TLS 1.3 | SHA-2、HMAC、HKDF、X25519、ChaCha20-Poly1305、签名验证（Ed25519、ECDSA、RSA）和 X.509 证书链，以及其上的 TLS 1.3 客户端，没有 C 也没有 `unsafe`，用标准的向量、Python 的 `cryptography` 和 OpenSSL 测试；QNI 通过它与 Discord 通信；一个包：`nx add nxtls --git https://github.com/Londopy/nxtls --tag v0.4.0` |

在哪里用了 Nexium？开一个 issue 或 pull request，它就会出现在这里。

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
| `nx init`、`nx add`、`nx fetch`、`nx update` | 包的清单、来自 git 或路径的依赖、锁文件（[docs/packages.md](../../packages.md)） |
| `nx emit-c file.nx` | 打印生成的 C |
| `nx tir file.nx [--sigs]` | 以 S 表达式输出检查后的程序（编译器自己的测试会读取它） |
| `nx fmt file.nx [--check]` | 规范化格式 |
| `nx debug file.nx` | 为调试而构建并在 gdb 或 lldb 下运行：`.nx` 行号，以及 String、List、Map、切片和可选值的格式化器 |
| `nx bench file.nx` | 测量 `bench "name" { }` 块：优化构建，每次迭代时间的中位数 |
| `nx fix file.nx` | 应用检查器的机械修复（`.clone()`、`@escape(...)`、`_ = `）并迁移已弃用写法；见 [docs/stability.md](../../stability.md) |
| `nx doc file.nx` | 带推断效应的 HTML 文档 |
| `nx size file.nx` | 把二进制的字节归因到各声明 |
| `nx layout file.nx [Type...]` | struct 或 enum 的偏移、大小和填充，以及能缩小它的按对齐排序 |
| `nx upgrade` | 用最新发布版替换这个可执行文件，经过校验；`--check` 只报告 |
| `nx install [DIR]` | 把这份副本连同旁边的文件装到用户目录并加入 PATH（便携 zip 自行安装） |
| `nx refcounts file.nx` | 每一处 retain 和 release |
| `nx leaks file.nx` | 带分配跟踪运行并报告泄漏 |
| `nx lsp` | 基于 stdio 的语言服务器 |
| `nx doctor` | 将使用哪个 C 编译器，安装是否正常 |
| `nx version` | 版本及其发布名 |
| `nx completions <shell>`、`nx man` | bash、zsh、fish 和 PowerShell 的补全，以及手册页 |
| `nx repl`，或直接 `nx` | 交互式会话：输入代码，查看值，保留绑定 |
| `nx -e CODE`、`nx -p EXPR` | 像在提示符下那样运行一行；`-p` 打印它的值 |

选项：`--mode debug|safe|fast|small`、`--target x86_64-linux-gnu`（`zig cc` 认识的
任何目标）、`--cpu baseline|native|<名称>`（默认 baseline，这样二进制能在同架构的
任何机器上运行）、`--out-dir`、`--keep-c`、`--cc`、`--strict`（警告即
错误）、`--sanitize address,undefined`（C 编译器的 sanitizer；`address`
需要 gcc 或 clang），以及用于 C 互操作的 `-I`、`--link`、
`--link-path`、`--c-source`。

## 现状

**1.3：语言已稳定，工具链已成熟。** 语言只按[稳定性策略](../../stability.md)以增添的
方式变化；编译器用 Nexium 写成并能构建自身；每个示例、规范用例和教程程序都在 CI 中于
三个平台上、在 sanitizer 和 fuzzer 之下运行，gdb 和 lldb 也在那里经由 `nx debug` 驱动。
内存安全是视图规则，自 1.3 起是错误。1.4，一个不必再补充的标准库，正在进行：集合
（`std.sort`、`std.heap`、`std.set`、`std.deque`）、`std.hash`、`random.secure`、路径、环境
变量与配置目录、UUID、日志、CSV、TOML 和 base64 已经加入，一共二十九个模块；`Map` 能抵御哈希洪水
攻击并保持键的插入顺序；`std.time` 从平台的数据库读取时区，`std.http` 的客户端通过 nxtls 提供的 TLS 层发起 HTTPS，`std.websocket` 也走同一层，[nexium-discord](https://github.com/Londopy/nexium-discord) 在其上构建 Discord 机器人；接下来是平台的 TLS。Nexium 还不是什么、每一点在哪里得到回答，见
[路线图的一节](../../../ROADMAP.md#what-10-is-not-yet)：基准只有四个程序（[速度](#速度)），
生态只有一位维护者和四个树外的项目（[上文](#实际使用)）。
[`KNOWN_ISSUES.md`](../../../KNOWN_ISSUES.md) 列出每个未修复的缺陷及其修法；
[`DECISIONS.md`](../../../DECISIONS.md) 列出规范未定之处做出的每一个决定。

## 发布版名称

每个主版本是一座山的山顶，按十四座八千米高峰首登的顺序排列：`X.0.0` 是
`<山>: Summit`。攀登从前一系列的中途、它的 `.5` 开始，沿首登路线一个营地一个营地往上；
登顶之后的次版本，到 `.4` 为止，是这座山的其他路线和下山路；补丁版用首登远征队的成员。
0.x 系列是最早被登顶的八千米峰安纳普尔纳（1950 年）的进山路与营地，所以 1.0.0 是
`Annapurna: Summit`；编译器开始构建自身的 0.7.0 是冲顶前的最后一个营地
`Annapurna: Camp V`；从 1.5 起，各版本向 2.0.0 攀登珠穆朗玛峰。名字出现在
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
| C 生成器 | [`self/cgen.nx`](../../../self/cgen.nx) | 每个程序一个 C 文件；大型程序的调试构建每个模块一个，未改动的复用 |
| 驱动 | [`self/nx.nx`](../../../self/nx.nx) | build、run、test、bench、debug、check、emit-c、tir；内嵌标准库 |
| 工具 | [`self/fmt.nx`](../../../self/fmt.nx)、[`self/doc.nx`](../../../self/doc.nx)、[`self/tools.nx`](../../../self/tools.nx)、[`self/size.nx`](../../../self/size.nx)、[`self/manifest.nx`](../../../self/manifest.nx)、[`self/ship.nx`](../../../self/ship.nx)、[`self/lsp.nx`](../../../self/lsp.nx)、[`self/lsp_index.nx`](../../../self/lsp_index.nx)、[`self/fix.nx`](../../../self/fix.nx)、[`self/repl.nx`](../../../self/repl.nx) | 格式化器、文档生成器、各类报告、包管理、`ship`、语言服务器及其对已检查程序的索引、`nx fix`、REPL |

每个示例、每个规范用例和每个编译失败用例都经由自举出的编译器运行，由本身就是
Nexium 程序的测试框架（`nx run tests/run.nx`）驱动，在 CI 中于三个平台上运行，完全
不用 Rust 工具链。用 Rust 写的第一个编译器推动了移植，并在 1.0 时被删除（决策 90）。

## 仓库中的语言

非空代码行数，不含构建输出、依赖和生成文件（`bootstrap/nx.c`、tree-sitter 解析器、
`gui/font.bin`、锁文件）：

| 语言 | 行数 | 占比 | 是什么 |
| --- | --- | --- | --- |
| Nexium | 48,804 | 87.2% | 编译器及其工具（`self/` 下 33,000 行）、标准库（21 个模块）、测试框架与 fuzzer、示例、教程程序、nexium-gui、站点生成器、四个基准程序 |
| C | 2,542 | 4.5% | 运行时 `nx_rt.h`、GUI 窗口层、随附的测试用 C、一个基准程序 |
| Python | 1,492 | 2.7% | 发布脚本（说明、包清单、wheel 与 npm 包、std 文档）、gdb 与 lldb 格式化器、基准运行器和四个基准程序 |
| 编辑器文件 | 1,103 | 2.0% | tree-sitter 查询、Emacs Lisp、Vim script、Neovim 用的 Lua、一个 Pygments 词法分析器，以及 Zed 对扩展要求的 25 行 Rust |
| JavaScript、TypeScript | 939 | 1.7% | VS Code 扩展、tree-sitter 语法，以及 playground 的 WASI 层 |
| Inno Setup、shell、PowerShell | 855 | 1.5% | Windows 安装程序脚本、`install.sh`、`install.ps1`、Chocolatey 脚本、自举脚本 |
| Rust、Go、Ruby | 213 | 0.4% | Rust 与 Go 各四个基准程序，以及 Homebrew 公式 |

编译器里没有 Rust：第一个编译器推动了移植并在 1.0 时被删除（决策 90）。剩下的 Rust
是 Zed 扩展的胶水代码，由 Zed 编译为 WebAssembly，以及四个用来对照测量的基准程序，
旁边是它们的 Go 孪生版本。Zig 不在表中，因为树里没有 Zig
源码：`zig cc` 是 `nx` 运行的 C 编译器（Windows 安装程序随附，安装脚本下载），正如
C 编译器是拿来用的，不是拿来写的。

## 目录结构

```
bootstrap/      构建编译器所用的 C 种子，以及构建脚本
runtime/        nx_rt.h，嵌入每一个生成的 C 文件；nx debug 的 gdb 与 lldb 格式化器
std/            用 Nexium 写的标准库，内嵌于编译器
self/           用 Nexium 写的编译器，逐阶段
gui/            nexium-gui：Nexium 即时模式 GUI、演示，以及 C 平台层
editors/        VS Code 扩展、tree-sitter 语法，以及另外十种编辑器的文件
examples/       带记录输出的程序，由测试运行
topo/           教程：各章及其展示的程序（由测试运行）
site/           文档站点生成器，一个 Nexium 程序
tests/          测试框架（run.nx）、规范一致性套件（tests/spec）、编译失败用例、调试器检查
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
