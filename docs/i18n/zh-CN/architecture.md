# Nexium 的工作原理

这是为想理解或修改编译器的人准备的导览。语言参考见 [`language.md`](language.md)；从其他语言调用 Nexium 见 [`embedding.md`](../../embedding.md)（英文）；在规范没有规定之处所做的每个取舍都记录在 [`DECISIONS.md`](../../../DECISIONS.md)（英文）中。

## 一个用自身写成的编译器

编译器用 Nexium 写成，位于 `self/` 下：`lexer.nx`、`parser.nx`、检查器（`check.nx` 及其旁边的 `check_*.nx` 模块，外加 `cimport.nx`）、`cgen.nx` 和驱动程序 `nx.nx`，各对应下面的一个阶段；然后是各种工具（`fmt.nx`、`doc.nx`、`tools.nx`、`size.nx`、`manifest.nx`、`ship.nx`、`ship_node.nx`、`installer.nx`、`lsp.nx` 与 `lsp_index.nx`、`repl.nx`、`topo.nx`、`completions.nx`、`fix.nx`）。它从 `bootstrap/` 中的 C 种子构建自身（决定 90）：最初的编译器用 Rust 写成，主导了移植，并在 1.0 时退役。

## 一段话版本

`nx` 读取 `.nx` 源码，检查它，然后写出一个 C 文件。该 C 文件包含 `runtime/nx_rt.h`（嵌入在编译器二进制文件中的约 4,200 行朴素 C 代码），并交给 `zig cc`，即捆绑了可交叉编译 libc 的 Clang。没有垃圾回收器，没有虚拟机，也没有需要安装的运行时库：输出是原生可执行文件、共享库或静态库，只依赖目标平台的 C 库。

```
 hello.nx ──lex──▶ tokens ──parse──▶ AST ──check──▶ 类型化 IR ──cgen──▶ hello.c ──zig cc──▶ hello.exe
                                            │
                                            └──comptime──▶ 常量值、comptime 测试
```

## 流水线，逐阶段

### 1. 词法分析器（`self/lexer.nx`）

输入字节，输出 token。这里的两个决定塑造了下游的一切：

- **换行是 token。** 语句在换行处结束，所以词法分析器输出 `Newline`，并把连续的换行合并为一个。在 `(` 和 `[` 内部换行会被抑制，这使得参数列表可以跨行书写。在 `{ }` 内部换行会被保留，因为块中的语句靠它们分隔。
- **`<<` 和 `>>` 是 `LtLt` 和 `GtGt`，从不是移位运算符。** 由语法分析器根据上下文决定它们是开启一个二进制模式还是移位。

标识符、关键字和内置函数的名字都是 `Ident`。关键字由语法分析器识别（`KEYWORDS` 是其列表）。字符串和注释之外出现非 ASCII 字符是错误。在需要时，词法分析器会把 `//` 注释保留为 token；格式化器需要它们，语法分析器从不会看到它们。

### 2. 语法分析器（`self/parser.nx`）

递归下降，每条语法规则一个函数，产生一个 `Tree`：由 `Node`（种类、子节点、一个名字、一段文本、三个槽位 `x`、`y`、`z`）组成的 id arena，上面挂着模块的各个项：函数、结构体、枚举、trait、impl、常量、全局变量、import、测试和产物。文档注释以指向 `Tree.docs` 的索引保存。条件不加括号（`if c {`；在那里写结构体字面量需要加括号），结构体字面量写作 `Point{ .x = 1 }`，匿名的写作 `.{ .x = 1 }`，闭包的捕获是显式的：`|[x, &mut y] a: i32|`。

续行规则位于语法分析器中：以 `|>`、`.method(`、`catch`、`orelse`、`and` 或 `or` 开头的行接续上一行，以二元运算符或未闭合括号结尾的行也一样。

### 3. 检查器（`self/check.nx` 与 `self/check_*.nx`）

编译器最大的部分，约 16,000 行。它把语法树变成类型化 IR（`Tir`，另一个 id arena，每个节点一个 `TKind`），并产生所有诊断。它的状态是 `check.nx` 中的一个结构体 `Checker`，同一文件中还有核心部分：类型、定义、加载、实例和针对整个程序的各遍、泛型环境、所有权、合一以及范围。其余方法分别位于各自的模块中，每个模块是一个 `impl Checker` 块（决定 110），`check.nx` 导入它们全部，正是这一点让它们成为程序的一部分：

| 模块 | 检查的内容 |
| --- | --- |
| `check_stmts.nx` | 函数体、块、语句、循环 |
| `check_views.nx` | 视图及其来源，规则 V1 到 V5 |
| `check_exprs.nx` | 表达式、函数调用、`if`、类型转换 |
| `check_fields.nx` | 字段、字面量、索引、`try`、`catch`、`orelse` |
| `check_calls.nx` | 方法、静态成员、内置命名空间 |
| `check_match.nx` | `match`、模式、穷尽性 |
| `check_builtins.nx` | `@` 内置函数、布局 |
| `check_closures.nx` | 闭包 |
| `check_print.nx` | 以文本形式输出类型化 IR，供 `nx tir` 使用 |
| `check_interp.nx` | 编译期解释器（第 4 节） |

各个组成部分：

**类型是驻留的**（`Types`）。一个类型是 `Ty` 表中的一个索引；两个类型相等当且仅当它们的索引相等。推断变量也是表中的条目，所以 `let x = 0` 给 `x` 一个整数变量，由第一个确定它的使用来解析，否则默认为 `i64`。

**泛型是单态化的。** `fn max(comptime T: type, a: T, b: T)` 对调用时用到的每个不同的 `T` 各检查一次，每种组合产生一个 `Inst`（`instantiate`）。类型参数没有运行时表示。trait 约束（`where T: Ord`）在实例化时检查，`impl` 块按结构匹配。

**所有权是逐函数的流分析。** `List`、`String` 和 `Map` 的值是被拥有的：按值赋值或传递会移动它，检查器把源局部变量标记为已移动（`FnCtx.moved`，按分支做快照再合并）。再次使用它就是 `use after move` 错误；从字段、元素、循环变量或作用于位置的 `if let` 绑定中移出会被拒绝，因为那会让容器处于半拥有状态。参数是借用的，所以被调用方不能移动它们。`ref class` 的值是带引用计数的指针，可以自由复制；检查器只记录它们需要 `refcounts`。

**效应通过不动点推断**（`propagate_effects`）。检查函数体时，检查器记录 `own_effects`，即函数直接产生的效应，每个都带一个*见证*：位置和一句解释（"appending to a List may grow it"）。它还记录每个被调用者。然后

```
effects(f) = own(f) ∪ ⋃ effects(callee)      迭代到不动点
```

对所有实例进行计算。外部函数被假定会做任何事。签名或函数类型上的负约束（如 `!allocates`）在传播之后检查；诊断沿调用图走到引入该效应的函数并打印其见证，因此错误会指出调用链最底部的确切行。在检查器能证明操作不会失败的地方，`panics` 被消除：用遍历同一切片的 `for` 的循环索引进行索引、编译期已知的索引、操作数范围可容纳的算术。

效应格有八位（`EFF_*`）：`allocates`、`refcounts`、`blocks`、`shared_mutable`、`nondeterministic`、`panics`、`ffi`，以及来自调用图的 `unbounded_stack`：处于环上的函数（`mark_recursion` 中的 Tarjan 强连通分量），或通过函数值或 `dyn` 进行调用的函数，都带有它。

**模式**（`check_match`）把 `match` 编译为决策树，并对枚举和布尔进行穷尽性检查。**二进制模式**把 `<<len:16/little, payload:len*8, rest:bytes>>` 变成一串带检查的位读取，其大小可以依赖于前面的绑定。

**视图**按其来源跟踪（`self/check_views.nx`）：每个局部变量都知道它持有的视图指向哪块存储、是何时取得的，`SPEC.md` 5.6 与 5.7 的规则 V1 到 V5 在会读取已释放存储的使用处检查，自 1.3 起为错误（1.2 中为警告）。当修法是机械性的，检查器会给 `nx fix` 一个编辑（`check.Fix`：V4 在移动处加的 `.clone()`、V5 的 `@escape`、在未使用的值前加的 `_ = `），只有当检查编辑后的程序得到更少的错误且没有新错误时，`self/fix.nx` 才会保留它。返回指向函数局部变量的切片或指针（规则 R1）自 1.0 起就是错误。

**Trait 对象**为每个（trait, 类型）对得到一个 vtable，以适配接收者的 thunk 形式生成；`dyn Shape !allocates` 是一个独立的类型，转换进去的每个实现都必须满足该约束。

**`@cImport`**（`self/cimport.nx`）对头文件运行 `zig cc -E`，用一个小型 C 声明解析器解析输出的声明，并注入一个合成模块。生成的 C 使用头文件自己的类型名，因此在布局上，头文件始终是唯一的事实来源。

### 4. 编译期求值（`self/check_interp.nx`）

一个作用于类型化 IR 的解释器，值用 `CV` 表示。它的 `it_*` 函数是在单独模块中声明的检查器方法（决定 110），因此检查器像调用自己的方法一样调用它们，它们也直接读取检查器的表。`comptime expr`、`const` 初始化器、`@embedFile` 和 `comptime test` 块都在检查期间在这里运行。它允许纯计算、集合和对 Nexium 函数的调用，禁止 I/O、时钟、随机数、外部调用和全局变量，并有步数预算，因此失控的求值会成为编译错误而不是挂起。失败的 `comptime test` 像其他错误一样在其 `expect` 行报告。同一个解释器运行 `nx repl`（`self/repl.nx`），其中 `repl_mode` 允许它与外界交互；它也运行 `nx play`，即演练场（playground）的运行器，其中 `play_mode` 会拒绝网页做不到的事。

### 5. C 后端（`self/cgen.nx`）

类型化 IR 被降级为一个 C 翻译单元。几条约定就能解释 `nx emit-c` 输出中的大部分内容：

- **每个函数都接收一个隐藏的 `nx_ctx* c`。** 上下文携带分配器、当前 arena、stdout/stderr、随机数生成器状态、argv 和泄漏跟踪计数器。运行时中没有全局变量，这使得交付的库可以安全地加载到宿主进程中。
- **拥有的值在作用域退出时释放。** 后端维护一个作用域栈；每个拥有类型的 `Let` 都会注册一次释放，离开作用域时（正常离开，或通过 `return`、`break`、错误），按逆序生成这些释放，连同所有 `defer`。传给调用的临时值也会被注册。
- **带检查的算术是一次调用。** `x * 7` 变成 `nx_mul_i64(x, 7, "hello.nx:3")`；位置字符串就是 panic 时打印的内容。回绕（`*%`）和饱和（`*|`）形式对应普通的 C 或做钳位的辅助函数。
- **panic 就是 `longjmp`。** 一个线程局部的 `nx_boundary` 持有 `jmp_buf`；`nx_panic` 填入消息和位置，然后跳到最近的边界，边界由入口点或导出包装器安装。
- **`for parallel`**（`parallel_for`）把循环体提取成一个工作函数，工作函数通过一个指针结构体访问外层的局部变量，由运行时把索引范围分给各个硬件线程。工作线程的 panic 会被捕获，并在所有工作线程结束后在调用方重新抛出。
- **`using arena`** 在块执行期间，把一个 bump 分配器换入上下文的副本中。容器会记住创建它们的 arena，因此在块内增长的外层 `List` 仍然位于堆上。

这是降级后的 `examples/hello.nx`：

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

格式化是编译的，而不是解释的：`println("x = {}", .{x})` 变成对一个输出槽（sink）的直接写入，每个占位符一次，且参数的类型是已知的。

### 6. 运行时（`runtime/nx_rt.h`）

一个头文件，用 `@embedFile` 嵌入编译器，并粘贴到每个生成文件的顶部。它的各部分：切片、分配器接口、panic、带可选泄漏跟踪的默认（malloc）分配器、arena、parallel-for 的线程池、列表、字符串、格式化、哈希表、引用计数、二进制模式辅助函数、带检查的算术，以及面向 Windows 和 POSIX 的平台部分：文件系统与文件句柄、时间、启动程序与子进程、套接字、借助系统自带库的 TLS、线程，以及终端的原始输入。

引用计数是每个 `ref class` 对象前面的两个字的头部（`rc`、`weak`）。`nx_retain` 是运行时中一个内联的自增；释放由后端为每个类生成（`cgen.nx` 中的 `drop_fn`），因为计数归零时它必须释放对象自身的字段。`weak` 引用让内存分配保持存活，但不让对象存活，一旦 `rc` 归零，`upgrade()` 就会失败。

一切都是 `static inline`，因此 C 编译器一次看到整个程序，未使用的运行时函数没有任何开销。

### 7. 驱动程序与 C 编译器（`self/nx.nx`）

`nx build` 把 C 文件写入 `nx-out/`，以对应构建模式（`debug`、`safe`、`fast`、`small`）、目标三元组和所有 `artifact link` 输入的参数调用 `zig cc`，除非给出 `--keep-c`，否则删除 C 文件。同一条路径服务于 `run`、`test`（生成的测试运行器作为入口点）、`leaks`（加上 `-DNX_LEAK_CHECK`）和 `size`（加上 `-ffunction-sections` 并读回目标文件）。

大型程序（约 200 KB 源码；对任何程序都可以用 `NX_UNITS` 决定）的调试构建则改为拆分（决定 116）：`cgen.generate_units` 为每个模块把一个 C 文件写入 `nx-out/<name>.units/`，每个目标文件以其 C 代码和命令行的哈希命名，驱动程序只在多个线程上编译目标文件缺失的那些文件，然后把它们链接起来。运行时的状态（`NX_STATE`）由根模块的文件定义，由其他文件声明。

`zig cc` 是默认选择，因为它开箱即可交叉编译：在 Windows 机器上使用 `--target aarch64-linux-gnu` 直接可行。不需要交叉编译时，也接受 `--cc clang` 或 `--cc gcc`。

### 8. 交付（`self/ship.nx`、`ship_node.nx`、`installer.nx`；`cgen.nx` 中的 `export_wrapper`）

`nx ship` 读取 `artifact` 声明，并产生：

- **`cabi`**：一个共享库、一个静态库和一个 C 头文件。
- **`python`**：一个基于 ctypes 的包和一个 wheel。
- **`rustlib`**：一个 Cargo crate，包含链接静态库的构建脚本、`extern "C"` 声明、`#[repr(C)]` 结构体和安全的包装。
- **`node`**：一个构建在共享库之上的 npm 包。
- **`cli`**：一个可执行文件；**`installer`**：一个 Inno Setup 脚本，或一个带有程序文件的安装脚本。

导出边界正是效应发挥价值的地方。被证明 `!panics` 且不返回错误联合的导出函数得到其自然的 C 签名。其他导出函数返回 `int32_t` 状态码，并通过输出参数交付其值；包装器安装一个 panic 边界，因此库内部的 Nexium panic 在宿主中变成错误码，而不是中止。任何声明了可嵌入产物的程序都会拒绝可变全局变量，因此加载同一个库的两个宿主不会相互干扰。

## 工具

它们都是同一份类型化 IR 之上的视图，因此与编译器的结论一致：

| 工具 | 读取的内容 |
| --- | --- |
| `nx effects` | 传播之后每个实例的 `effects` |
| `nx refcounts` | 每个 `Retain`/`Release`/`Weak`/`Upgrade` 节点及其所在函数 |
| `nx audit` | `unsafe` 块和全局变量 |
| `nx doc` | 文档注释、签名和效应，渲染为 HTML |
| `nx lsp` | 每次编辑时完整检查得到的诊断；来自已检查实例的代码透镜（由记录的见证得出的 panic 证明）；程序检查通过时，跳转到定义、重命名和悬停信息来自检查器对每个名字的解析结果（`self/lsp_index.nx`：局部变量、函数和方法、字段），重命名会编辑程序的每个文件；否则，以及补全、类型和常量，则来自 token 流和已解析的模块（`self/lsp.nx`），因此代码有错误时它们仍能响应 |
| `nx repl` | 解释器，逐行运行，作用于每次都整体重新检查的程序 |
| `nx size` | 目标文件的各段大小，映射回声明 |
| `nx layout` | 逐字段遍历检查器的 `size_of`/`align_of`：偏移、填充、总大小，以及能缩小结构体的按对齐重排 |
| `nx fmt` | 只看 token 流；从不合并或拆分行 |

## 自举

编译器用 Nexium 写在 `self/` 下，每个阶段一个文件，并能构建自身：`bootstrap/nx.c` 是它为自己生成的 C 代码，任何 C 编译器都能把它编译成 `nx0`，`nx0` 把 `self/nx.nx` 构建为 `nx1`，而 `nx1` 必须再次生成相同的 C（`nx2`），这就是被测试的编译器，也是发行版交付的那一个（`bootstrap/build.sh`、`bootstrap/README.md`）。语法树和类型化 IR 都是 id arena，即放在 `List` 中、通过索引相互引用的节点（见 `examples/tree.nx`），这不需要递归类型，并且一次释放即可。移植期间，每个阶段都通过把输出与最初的编译器在源码树中每个源文件上的输出做差异比较来验证；现在驱动各测试套件的测试工具本身也是一个 Nexium 程序，即 `tests/run.nx`。

## 出问题时从哪里查起

| 症状 | 从这里开始 |
| --- | --- |
| 程序能通过解析但不应该，或者反过来 | `self/parser.nx`，然后看 `tests/compile_fail/` 中的期望消息 |
| 看起来不对的类型错误 | `self/check_exprs.nx` 中的 `check_expr`，`self/check_calls.nx` 中的 `check_method_call` |
| 该有的效应没有，或不该有的效应有了 | `own_effects` 中的见证；在 `self/check*.nx` 中 grep `add_effect` |
| `nx leaks` 报告的泄漏 | `self/cgen.nx` 中的作用域栈（`register_drop`）；每个拥有的临时值都必须注册 |
| 生成的 C 无法编译 | `nx emit-c file.nx --keep-c`，然后读 `nx-out/file.c`（`emit-c` 不写 `#line`，所以 C 编译器报告的是 C 的行号）；它调用的运行时辅助函数在 `runtime/nx_rt.h` 中 |
| `for parallel` 或 `using arena` 内部的崩溃 | `self/cgen.nx` 中的 `parallel_for` 和运行时的 arena 部分 |
