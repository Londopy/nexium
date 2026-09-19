# Nexium 的工作原理

这是为想理解或修改编译器的人准备的导览。语言参考见
[`language.md`](language.md)；从其他语言调用 Nexium 见
[`embedding.md`](../../embedding.md)（英文）；规范未明确之处所做的每个决定见
[`DECISIONS.md`](../../../DECISIONS.md)（英文）。

## 一段话版本

`nx` 是一个没有依赖的 Rust 程序。它读入 `.nx` 源码，做检查，然后写出一个 C
文件。这个 C 文件包含 `runtime/nx_rt.h`（900 行平实的 C，嵌入在编译器二进制
里），再交给 `zig cc`，也就是自带交叉编译 libc 的 Clang。没有垃圾回收器，没有
虚拟机，也没有需要安装的运行时库：输出是原生可执行文件、共享库或静态库，只
依赖目标平台的 C 库。

```
 hello.nx ──lex──▶ tokens ──parse──▶ AST ──check──▶ 带类型 IR ──cgen──▶ hello.c ──zig cc──▶ hello.exe
                                            │
                                            └──comptime──▶ 常量值、comptime 测试
```

## 流水线，逐阶段

### 1. 词法分析器（`src/lexer.rs`）

字节进，词法单元出。这里的两个决定塑造了下游的一切：

- **换行是词法单元。** 语句在换行处结束，所以词法分析器发出 `Newline` 并合并
  连续的换行。在 `(` 和 `[` 内换行被抑制，这就是参数列表可以跨行的原因。在
  `{ }` 内换行被保留，因为块中的语句正是用它分隔的。
- **`<<` 和 `>>` 是 `LtLt` 和 `GtGt`，从不是移位运算符。** 由语法分析器根据
  上下文决定它们是打开二进制模式还是做位移。

标识符、关键字和内置函数名都是 `Ident`。关键字由语法分析器识别。字符串和注
释之外的非 ASCII 字符是错误。

`nx tokens file.nx` 以固定格式输出词法单元流；这是 `self/lexer.nx` 中自举词法
分析器的对照基准。

### 2. 语法分析器（`src/parser.rs`、`src/ast.rs`）

递归下降，每条文法规则一个函数，产出由各种项组成的 `ast::Module`：函数、
struct、enum、trait、impl、常量、全局量、import、测试、artifact。条件带括号
（`if (c)`），结构体字面量是 `Point{ .x = 1 }`，匿名的是 `.{ .x = 1 }`，闭包
的捕获是显式的 `|[x, &mut y] a: i32|`。

行延续规则住在语法分析器里：以 `|>`、`.method(`、`catch`、`orelse`、`and`、
`or` 开头的行接到上一行，以二元运算符或未闭合括号结尾的行同样如此。

### 3. 检查器（`src/check/`）

编译器最大的部分，约 7 500 行。它把 AST 变成 `src/tir.rs` 中的带类型 IR，并
产生所有诊断。组成部分：

**类型被驻留**（`src/types.rs`）。`TyId` 是 `TyKind` 表的索引；两个类型相等当
且仅当 id 相等。推断变量也是 `TyKind`，所以 `let x = 0` 给 `x` 一个整数变
量，由第一个确定它的使用解决，否则默认为 `i64`。

**泛型被单态化。** `fn max(comptime T: type, a: T, b: T)` 对每个被调用到的不
同 `T` 检查一次，每种组合产生一个 `TFunc` 实例（`check/mod.rs` 中的
`instantiate`）。类型参数没有运行时表示。trait 约束（`where T: Ord`）在实例
化时检查，`impl` 块按结构匹配。

**所有权是按函数的流分析。** `List`、`String`、`Map` 值是被拥有的：按值赋值
或传递会移动它，检查器把来源局部变量标记为已移动（`TFunc::moved`）。再次使
用就是 `use after move` 错误；从字段或元素中移出会被拒绝，因为容器会处于半
拥有状态。参数是借用的，所以被调用者不能移动它们。`ref class` 值是带引用计
数的指针，可以自由复制；检查器只记录它们需要 `refcounts`。

**效应通过不动点推断**（`check/effects.rs`）。检查函数体时，检查器记录
`own_effects`，即函数直接执行的效应，每个都带一个*见证*：位置和一句解释
（"appending to a List may grow it"）。它也记录每个被调用者。然后

```
effects(f) = own(f) ∪ ⋃ effects(callee)      迭代到不动点
```

对所有实例进行。extern 函数被假定什么都会做。签名或函数类型上的负约束（如
`!allocates`）在传播后检查；诊断沿调用图走到引入该效应的函数并打印其见证，
所以错误会点出调用链最底部的那一行。在检查器能证明操作不会失败的地方，
`panics` 被免除：用同一切片上 `for` 的循环索引做索引、编译期已知的索引、操
作数范围可容纳的算术。

效应格是 `src/effects.rs` 中的八个位：`allocates`、`refcounts`、`blocks`、
`shared_mutable`、`nondeterministic`、`panics`、`ffi` 和 `unbounded_stack`
（保留）。

**模式**（`check/pattern.rs`）把 `match` 编译为决策树，对 enum 和 bool 做穷尽
性检查。**二进制模式**（`check/binpat.rs`）把 `<<len:16/little,
payload:len*8, rest:bytes>>` 变成一串经过检查的位读取，其大小可以依赖前面的
绑定。

**区域**做保守检查：返回指向函数自身局部变量的切片或指针是错误（规则 R1）。

**Trait 对象**每个 (trait, 类型) 对得到一张 vtable，由适配接收者的 thunk 生
成；`dyn Shape !allocates` 是独立的类型，转换进去的每个实现都必须满足约束。

**`@cImport`**（`src/cimport.rs`）对头文件运行 `zig cc -E`，用一个小型 C 声明
解析器读取输出的声明，并注入一个合成模块。生成的 C 使用头文件自己的类型
名，所以头文件仍是布局的唯一真相来源。

### 4. 编译期求值（`src/comptime.rs`）

一个基于带类型 IR 的解释器。`comptime expr`、`const` 初始化、`@embedFile` 和
`comptime test` 块在检查期间在这里运行。它允许纯计算、集合和对 Nexium 函数的
调用，禁止 I/O、时钟、随机数、外部调用和全局量，并且有步数预算，失控的求值
是编译错误而不是挂起。失败的 `comptime test` 像其他错误一样在其 `expect` 行
报告。

### 5. C 后端（`src/cgen/`）

带类型 IR 被降低为一个 C 翻译单元。几条约定解释了 `nx emit-c` 输出中的大部
分内容：

- **每个函数都带一个隐藏的 `nx_ctx* c`。** 上下文携带分配器、当前 arena、
  stdout/stderr、随机数状态、argv 和泄漏追踪计数器。运行时没有全局量，这正是
  交付的库可以安全加载进宿主进程的原因。
- **拥有的值在作用域退出时释放。** 后端维护一个作用域栈；每个拥有类型的
  `Let` 登记一次释放，离开作用域时（正常、`return`、`break` 或出错）按逆序
  发出释放和 `defer`。传给调用的临时值同样登记。
- **检查算术是一次调用。** `x * 7` 变成 `nx_mul_i64(x, 7, "hello.nx:3")`；位
  置字符串就是 panic 打印的内容。回绕（`*%`）和饱和（`*|`）形式映射到平实
  的 C 或钳位辅助函数。
- **panic 是 `longjmp`。** 线程局部的 `nx_boundary` 持有一个 `jmp_buf`；
  `nx_panic` 填入消息和位置后跳到最近的边界，边界由入口点或导出封装安装。
- **`for parallel`**（`cgen/parallel.rs`）把循环体提取为一个工作函数，通过指
  针结构体访问外层局部变量，运行时把索引范围分配到硬件线程上。工作线程的
  panic 被捕获，并在所有工作线程结束后在调用方重新抛出。
- **`using arena`** 把 bump 分配器换进该块的上下文副本。容器记得自己在哪个
  arena 中创建，所以在块内增长的外层 `List` 仍然住在堆上。

这是降低之后的 `examples/hello.nx`：

```c
static void nx_main(nx_ctx* c) {
  nx_sink _t1 = nx_sink_file(c, c->out);
  nx_w(&_t1, (const uint8_t*)nx_str_0, 13);
  nx_w(&_t1, (const uint8_t*)"\n", 1);
  nx_sink_flush(&_t1);
  int64_t x_0 = nx_mul_i64(((int64_t)6LL), ((int64_t)7LL), "examples/hello.nx:3");
  ...
}
```

格式化是编译而不是解释的：`println("x = {}", .{x})` 变成对一个 sink 的直接写
入，每个占位符一次，参数类型已知。

### 6. 运行时（`runtime/nx_rt.h`）

一个头文件，用 `include_str!` 嵌入编译器，并粘贴到每个生成文件的顶部。其分
节：切片、分配器接口、panic、带可选泄漏追踪的默认（malloc）分配器、arena、
parallel-for 线程池、列表、字符串、格式化、哈希映射、引用计数、二进制模式辅
助、检查算术，以及 Windows 和 POSIX 的平台部分（文件 I/O、时间、进程启动）。

引用计数是每个 `ref class` 对象前面的两字头部（`rc`、`weak`）。`nx_retain` 是
运行时里的内联自增；释放由后端按类生成（`cgen/mod.rs` 中的 `drop_fn`），因为
计数归零时它必须释放对象自己的字段。`weak` 引用让分配存活但不让对象存活，
`rc` 归零后 `upgrade()` 失败。

一切都是 `static inline`，所以 C 编译器一次看到整个程序，未使用的运行时函数
零成本。

### 7. 驱动与 C 编译器（`src/main.rs`）

`nx build` 把 C 文件写入 `nx-out/`，用构建模式（`debug`、`safe`、`fast`、
`small`）、目标三元组和任何 `artifact link` 输入对应的标志调用 `zig cc`，除非
给了 `--keep-c`，否则删除 C 文件。同一路径服务于 `run`、`test`（生成的测试
运行器是入口点）、`leaks`（加 `-DNX_LEAK_CHECK`）和 `size`（加
`-ffunction-sections` 并读回目标文件）。

`zig cc` 是默认值，因为它开箱即可交叉编译：在 Windows 机器上
`--target aarch64-linux-gnu` 直接可用。不需要交叉编译时也接受 `--cc clang`
或 `--cc gcc`。

### 8. 交付（`src/ship.rs`、`src/cgen/exports.rs`）

`nx ship` 读取 `artifact` 声明并产出：

- **`cabi`**：共享库、静态库和 C 头文件。
- **`python`**：基于 ctypes 的包和 wheel。
- **`rustlib`**：带链接静态库的构建脚本、`extern "C"` 声明、`#[repr(C)]` 结
  构体和安全封装的 Cargo crate。
- **`cli`**：可执行文件。

导出边界是效应带来回报的地方。被证明为 `!panics` 且不返回错误联合的导出函
数得到它自然的 C 签名。其他任何导出返回 `int32_t` 状态并通过输出参数交付
值；封装会安装 panic 边界，所以库内的 Nexium panic 在宿主中变成错误码而不是
中止。声明了可嵌入 artifact 的程序会拒绝可变全局量，这样加载同一个库的两个
宿主不会互相干扰。

## 工具

它们都是同一份带类型 IR 上的视图，所以总与编译器一致：

| 工具 | 读取的内容 |
| --- | --- |
| `nx effects` | 传播后的 `TFunc::effects` |
| `nx refcounts` | 每个 `Retain`/`Release`/`Weak`/`Upgrade` 节点及其所在函数 |
| `nx audit` | `unsafe` 块和全局量 |
| `nx doc` | 文档注释、签名和效应，渲染为 HTML |
| `nx lsp` | 每次编辑做完整检查得到的诊断，来自 `TFunc` 的悬停信息 |
| `nx size` | 目标文件的节大小映射回声明 |
| `nx fmt` | 只看词法单元流；从不合并或拆分行 |

## 自举

编译器正在 `self/` 下用 Nexium 重写，一次一个阶段，每个阶段通过把输出与
Rust 编译器在同一输入上的输出做比对来验证。词法分析器已完成并纳入
`cargo test`。下一个是语法分析器；它的 AST 将是一个 id 竞技场，节点存在
`List` 里并通过索引相互引用（见 `examples/tree.nx`），不需要递归类型，一次
释放。当四个阶段都存在时，Rust 版 `nx` 把 Nexium 版 `nx` 编译一次，那个二进
制再编译自己的源码，如果两份输出逐字节一致，语言就能构建自己了。

## 出问题时从哪里看起

| 症状 | 从这里开始 |
| --- | --- |
| 程序能解析但不该能，或反之 | `src/parser.rs`，然后看 `tests/compile_fail/` 中期望的消息 |
| 看起来不对的类型错误 | `src/check/expr.rs`（表达式）或 `src/check/method.rs`（方法和内置调用） |
| 不该有或该有的效应 | `own_effects` 中的见证；在 `src/check/` 中搜 `add_effect` |
| `nx leaks` 报告泄漏 | `src/cgen/expr.rs` 中的作用域栈；每个拥有的临时值都必须登记 |
| 生成的 C 编译不过 | `nx emit-c file.nx --keep-c` 然后读 `nx-out/file.c`；它调用的运行时辅助在 `runtime/nx_rt.h` |
| `for parallel` 或 `using arena` 内崩溃 | `src/cgen/parallel.rs` 和运行时的 arena 一节 |
