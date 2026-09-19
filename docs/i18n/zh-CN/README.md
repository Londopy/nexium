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
  <a href="https://crates.io/crates/nexium"><img alt="crates.io" src="https://img.shields.io/crates/v/nexium?logo=rust&color=4fd1c5"></a>
  <a href="../../../LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-blue"></a>
  <a href="https://ziglang.org/download/"><img alt="Zig" src="https://img.shields.io/badge/backend-zig%20cc-f7a41d?logo=zig&logoColor=white"></a>
  <img alt="Platforms" src="https://img.shields.io/badge/platforms-windows%20%7C%20linux%20%7C%20macos-2b3a55">
</p>

<p align="center">
  <b>一门足以构建一切的语言，同时也是为别的项目引入一个组件时的最佳选择。</b>
</p>

Nexium 经由 C 编译为原生代码，采用自动引用计数而没有追踪式垃圾回收器，
拥有由机器检查的效应系统，能说明一个函数是否分配内存、是否阻塞、是否可能
panic；它的编译器能把同一份源码树变成 C 库、Python wheel、Rust crate 或命令行
工具。

<table>
<tr>
<td width="50%" valign="top">

**一个文件**

```
fn checksum(data: []u8) -> u32 export(c) {
    var h: u32 = 2166136261
    for (data) |b| {
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

**所有目标**

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

效应签名决定 C ABI：`checksum` 被证明为 `!panics`，因此得到一个普通的
`uint32_t checksum(const uint8_t*, size_t)`。可能失败的函数返回状态码，函数
内部的 Nexium panic 会在边界处被转换，而不是让宿主进程崩溃。

## 亮点

| | |
| --- | --- |
| 🧾 **效应：推断并检查** | `allocates` `refcounts` `blocks` `shared_mutable` `nondeterministic` `panics` `ffi`。声明 `!allocates`，编译器会沿着调用链指出确切会违反它的那一行。 |
| 🧠 **没有借用检查器的所有权** | 集合按移动传递，`.clone()` 复制，`ref class` 值引用计数，`weak` 打破循环。移动后再使用是编译错误。 |
| 🔬 **二进制模式** | `<<version:4, ihl:4, len:16/big, rest:bytes>>` 以经过检查的长度匹配和构造数据包。 |
| 🧵 **并行循环、arena、trait 对象** | `for parallel`、`using arena { }`、`dyn Trait !allocates`。 |
| 🔌 **无需绑定的 C 调用** | `@cImport("header.h")` 直接读取头文件；`artifact link` 把自带的 C 源码编进程序。 |
| 📦 **一份源码，多种交付** | `nx ship` 产出 C 头文件和库、Python wheel，以及带安全封装的 Rust crate。 |
| 🖼 **用 Nexium 写的 GUI** | [`gui/`](../../../gui)：即时模式 GUI（按钮、滑块、文本框），软件光栅化和位图字体，全部是 Nexium，只依赖 200 行 C 的窗口层。 |
| 🛠 **自带工具** | `fmt`、`doc`、`lsp`、`size`、`leaks`、`refcounts`、`effects`、`audit`。零依赖。 |

## 安装

唯一的运行时要求是 `PATH` 中有 [Zig](https://ziglang.org/download/)，它被用作
C 编译器（`zig cc` 也能交叉编译；`--cc clang` 同样可用）。

Windows、Linux 和 macOS 的预编译 `nx` 二进制文件在
[Releases](https://github.com/Londopy/nexium/releases) 页面。解压后把 `nx`
放进 `PATH`。

或者用 Rust 1.75 及以上从源码构建：

```bash
cargo install nexium
```

```bash
cargo install --git https://github.com/Londopy/nexium
```

然后：

```bash
nx run examples/hello.nx
```

## 快速一览

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
    if (text.len == 0) return error.Empty
    var total: i64 = 0
    for (text) |c| {
        if (c < '0' or c > '9') return error.NotANumber
        total = total * 10 + (c - '0') as i64
    }
    return total
}

fn max(comptime T: type where T: Ord, a: T, b: T) -> T {
    return if (a > b) a else b
}

fn main() -> !void {
    let n = try parse_num("1234")
    let bad = parse_num("12x") catch |e| {
        println("caught {}", .{e})
        -1
    }
    var xs = List(i32).new()
    for (0..10) |i| { xs.append((i * i) as i32) }
    let found = outer: {
        for (xs) |x, i| { if (x > 30) break :outer i as i64 }
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

没有绑定生成器，也没有构建步骤：头文件是内存布局的唯一真相来源，外部调用带有
`ffi` 效应。

</details>

<details>
<summary><b>并行循环与 arena</b></summary>

```
for parallel (positions) |p, i| {
    out[i] = integrate(p)          // 这里不允许 shared_mutable
}

using arena {
    var scratch = List(Frame).new()   // bump 分配，整体一次释放
    ...
}
```

</details>

**文档**

- [Nexium 的工作原理](architecture.md)：从源码到二进制的流水线、效应推断、所有权、运行时和交付。
- [语言参考](language.md)：编译器实现的每一个构造。
- [嵌入](../../embedding.md)（英文）：从 Python、Rust 和 C 调用交付的库。
- [nexium-gui](../../gui.md)（英文）：即时模式 GUI 库以及如何编写一个控件。
- [发布你的程序](../../releasing-your-program.md)（英文）：打一个标签即可得到三个平台的二进制，安装包可选。
- [编辑器支持](../../../editors)（英文）：VS Code 扩展、Sublime 语法、LSP。
- [设计决策](../../../DECISIONS.md)（英文）：规范未明确之处所做的每一个决定。

## 命令

| 命令 | 作用 |
| --- | --- |
| `nx build file.nx` | 编译为可执行文件（没有 `main` 时为目标文件） |
| `nx run file.nx` | 构建并运行 |
| `nx test file.nx [filter]` | 运行 `test "..."` 块 |
| `nx check file.nx` | 类型检查并报告效应违规 |
| `nx effects file.nx` | 打印每个函数推断出的效应 |
| `nx audit file.nx` | 列出 `unsafe` 块和可变全局量 |
| `nx ship file.nx` | 产出每一个声明的 `artifact` |
| `nx emit-c file.nx` | 打印生成的 C |
| `nx tokens file.nx` | 输出词法单元流（自举的对照基准） |
| `nx fmt file.nx [--check]` | 规范格式化 |
| `nx doc file.nx` | 带推断效应的 HTML 文档 |
| `nx size file.nx` | 把二进制字节归属到各声明 |
| `nx refcounts file.nx` | 每一处 retain 和 release |
| `nx leaks file.nx` | 带分配追踪运行并报告泄漏 |
| `nx lsp` | 基于 stdio 的语言服务器 |

选项：`--mode debug|safe|fast|small`、`--target x86_64-linux-gnu`（`zig cc`
认识的任何目标）、`--out-dir`、`--keep-c`、`--cc`，以及用于 C 互操作的
`-I`、`--link`、`--link-path`、`--c-source`。

## 现状

这是 `nexium-spec.txt` 设计的第一个实现。它已经足以编写真实的程序（见
[`examples/`](../../../examples)），并从一个文件交付 Python、Rust 或 C 组件。
trait 对象、并行循环、arena 作用域、直接导入 C 头文件和全部工具都已就位。
仍处于早期：标准库只覆盖第 16 节的一小部分，区域检查只覆盖返回的视图。
[`DECISIONS.md`](../../../DECISIONS.md) 记录了规范未明确之处的每一个决定，
其中第 27 条列出了尚未完成的内容。

## 自举

编译器目前是 Rust 写的。Nexium 版本在 [`self/`](../../../self) 中逐阶段生长，
每一阶段都用相同输入与 Rust 编译器比对：

| 阶段 | 文件 | 对照基准 | 状态 |
| --- | --- | --- | --- |
| 词法分析器 | [`self/lexer.nx`](../../../self/lexer.nx) | `nx tokens` | ✅ 在每个示例和它自身上完全一致 |
| 语法分析器 | | `nx parse` | 下一步 |
| 检查器 | | `nx check`、compile-fail 测试集 | |
| C 生成器 | | `nx emit-c` | |

`cargo test` 用 Rust 编译器构建 `self/lexer.nx`，并将其输出与对照基准比对。

## 仓库使用的语言

非空代码行数，不含构建产物、依赖和生成文件：

| 语言 | 行数 | 占比 | 用途 |
| --- | --- | --- | --- |
| Rust | 22 393 | 86.9 % | `nx` 编译器 |
| Nexium | 2 111 | 8.2 % | 示例、自举词法分析器、nexium-gui、测试 |
| C | 1 108 | 4.3 % | 运行时 `nx_rt.h` 和 GUI 窗口层 |
| JavaScript、TypeScript | 159 | 0.6 % | VS Code 扩展 |

## 目录结构

```
src/            编译器（词法、语法、检查器、comptime、C 后端、驱动）
runtime/        nx_rt.h，嵌入每一个生成的 C 文件
self/           用 Nexium 写的编译器，逐阶段推进
gui/            nexium-gui：Nexium 即时模式 GUI、演示程序和 C 平台层
editors/        VS Code 扩展和 Sublime Text 语法
examples/       带记录输出的程序，由 `cargo test` 运行
tests/          集成测试和 compile-fail 用例
docs/           工作原理、语言参考、嵌入指南、翻译
assets/         标志和横幅
nexium-spec.txt          设计文档
nexium-systems-spec.txt  已归档的系统语言；第 4 到 9 节是语法参考
DECISIONS.md    规范未明确之处所做的决定
```

## 参与贡献

见 [`CONTRIBUTING.md`](../../../CONTRIBUTING.md)。缺陷和提案通过 GitHub issues
提交；语言层面的改动必须说明它服务于规范第 3 节中的哪一条硬性约束。

## 许可证

MIT。Copyright (c) 2026 Londopy.

<p align="center"><img src="https://raw.githubusercontent.com/Londopy/nexium/main/assets/logo.svg" alt="" width="48"></p>
