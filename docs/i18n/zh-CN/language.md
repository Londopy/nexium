# Nexium 语言参考

本文描述 `nx` 编译器目前实现的内容。章节编号指向 `nexium-spec.txt`（设计
文档）；标注"归档"的地方指向 `nexium-systems-spec.txt` 的第 4 到 9 节（语法
参考）。

## 文件与模块

一个文件就是一个模块。`import foo.bar` 加载根文件旁边的 `foo/bar.nx`，其
`pub` 项通过 `bar.item` 访问。`import std.math`（或任何 std 模块）可以写但不
是必需的：内置命名空间 `math`、`io`、`os`、`time`、`random`、`mem` 始终在
作用域内。

## 词法结构（归档 4）

- `//` 注释；`///` 文档注释附着到下一个声明。
- 标识符仅限 ASCII。字符串和注释之外出现非 ASCII 字符是错误。
- 整数：`42`、`0xFF`、`0o755`、`0b1010_1100`。浮点：`3.14`、`1e-9`、
  `0x1.8p3`。字符串 `"text"`（必须是 UTF-8）、原始字符串 `r"..."`、字节串
  `b"\x00\xff"`、字符 `'a'`（一个 Unicode 标量，类型为 `char`）。
- 语句在换行处结束。以 `|>`、`.method(`、`catch`、`orelse`、`and`、`or` 开头
  的行延续上一行；以二元运算符或未闭合括号结尾的行同样延续。
- 约定：类型 `PascalCase`，函数和变量 `snake_case`，常量
  `SCREAMING_SNAKE_CASE`。

## 声明

```
fn name(a: T, b: U) -> R effects { ... }         // 效应：例如 !allocates !panics
pub fn f(x: i32) -> i32 export(c) { ... }        // 以 C ABI 导出
extern fn puts(s: *u8) -> i32                    // 外部函数；调用需要 `unsafe`
struct Point derive(Eq, Ord) { x: f64, y: f64 }
struct Frame layout(c) { id: u32, len: u16 }     // C 布局，可跨边界使用
struct Pair(T) { a: T, b: T }                    // 泛型
record Dose { mg: f64 where value > 0.0 }        // 带校验的数据
ref class Node { value: i32, next: ?Node }       // 引用计数
enum Shape { Circle(f64), Rect { w: f64, h: f64 }, Empty }
type Meters = distinct f64                       // 无隐式转换
type Bytes = []u8                                // 别名
error ParseError { Empty, NotANumber }
trait Shape { fn area(self: *Self) -> f64 }
impl Shape for Circle { fn area(self: *Self) -> f64 { ... } }
impl Point { fn origin() -> Point { ... } }
impl(T) Pair(T) { fn swap(self: *mut Self) { ... } }
const TABLE: [256]u8 = comptime build_table()
var counter: u32 = 0                             // 可变全局量；访问需要 `unsafe`
test "name" { ... }
artifact cabi { name = "lib", exports = [f] }
```

`{ }` 内的字段以逗号或换行分隔。字段可以有默认值（`verbose: u8 = 0`）。

## 类型（归档 5）

| 语法 | 含义 |
| --- | --- |
| `i8 … i128`、`u8 … u128`、`isize`、`usize` | 整数；不同宽度之间没有隐式转换 |
| `f32`、`f64`、`bool`、`char`、`void`、`never` | 基本类型 |
| `[N]T` | 数组，`N` 为编译期常量 |
| `[]T`、`[]mut T` | 切片：指针加长度；`[]u8` 即文本 |
| `*T`、`*mut T` | 指向单个值的指针（`&x`、`&mut x`、`p.*`） |
| `?T` | 可选值；`null` 为空值 |
| `!T`、`Set!T` | 错误联合 |
| `List(T)`、`String`、`Map(K, V)` | 拥有所有权的集合（值语义，5.3 节） |
| `fn(A, B) -> R !effects` | 函数值（闭包和函数都能转换成它） |
| `(A, B)` | 元组；字段 `.0`、`.1` |
| `weak T` | 指向 `ref class` 的弱引用 |

整数字面量采用上下文要求的类型，默认为 `i64`；浮点默认为 `f64`。字符字面
量可放入任何容得下它的整数类型（`c: u8` 时 `c == 'a'` 成立）。

转换：`x as T` 用于数值之间（除非范围已被证明，否则窄化在运行时检查）、
`distinct` 类型与其表示之间、`char` 与整数之间、`bool` 到整数、单元枚举到
整数。`@truncate(T, x)` 做回绕截断。

## 值、绑定、所有权

`let x = e` 不可变绑定，`var x = e` 可变绑定。可变性是浅层的（归档 5.10）：
持有 `*mut T` 的 `let` 仍可通过它修改。

集合拥有堆上的缓冲区（5.3）。`let b = a` 移动 `a`；之后再用 `a` 是编译错
误；`a.clone()` 复制。参数是借用的：接收 `List` 的函数只能读它；要修改，
取 `*mut List(T)`。传到期望切片的位置时，`List(T)` 和 `String` 会转换为
`[]T` / `[]u8`。从字段或元素中移出是错误。拥有的值在作用域结束时释放；这是
作用域退出时唯一的自动动作（H7）。

`ref class` 的值是引用；复制一个会增加引用，最后一个引用消失时对象被释放
（5.1）。循环会泄漏；对反向边使用 `weak`（`@weak(x)` 或 `x.weak()`，然后
`w.upgrade()`）。

## 表达式

- 算术 `+ - * / %` 在溢出时触发（定义好的 panic）。回绕形式 `+% -% *%` 和
  饱和形式 `+| -| *|` 永不失败。位运算 `& | ^ ~ << >>`。比较 `== != < <= > >=`
  适用于数值、字符、布尔、`[]u8`、`String`、单元枚举，以及 `derive(Eq)` /
  `derive(Ord)` 的类型。逻辑 `and`、`or`、`!`。
- `x |> f(a)` 即 `f(x, a)`。
- `if c { a } else { b }` 是表达式；`if let v = opt { } else { }` 解包。
  条件不加括号，主体总是带花括号。
- `match v { pat => expr, ... }` 可匹配整数（字面量、范围 `1..=9`）、字符
  串、布尔、字符、枚举（`.Variant(p)`）、可选值（`null`、绑定）、错误联合
  （`error.Name`、绑定）、元组，以及字节切片（二进制模式）。枚举和布尔的
  match 必须穷尽；其他需要 `_ =>`。
- 块是表达式，其值为最后一个表达式。带标签的块通过 `break :label value`
  产出值。
- `try e` 传播错误；`e catch |err| handler`；`opt orelse default`；`opt.?`
  解包（为 null 时 panic）。
- `defer stmt` 在作用域退出时执行，`errdefer stmt` 仅在因错误退出时执行；
  两者都按注册的逆序执行。
- 闭包：`|[captures] params| -> R { body }`。捕获是显式的：`[x]` 复制，
  `[&x]` 和 `[&mut x]` 取引用。
- 可变全局量、外部调用、指针转换和切片的 `.ptr` 都需要 `unsafe { }`。
- `comptime expr` 在编译期求值；`@embedFile("path")` 把声明过的构建输入嵌入
  为 `[]u8`。

## 语句与循环

```
while cond { }
for x in items { }               // 数组、切片、列表、字符串、映射的键
for x, i in items { }            // 带索引
for x, y in a, b { }             // 同步遍历；长度必须一致
for i in 0..n { }
outer: for a in ... { for b in ... { continue :outer } }
break, continue, return
_ = expr                          // 显式丢弃；未使用的值是错误
```

## 二进制模式（归档 6）

```
match packet {
    <<version:4, ihl:4, total_len:16/big, rest:bytes>> => ...
    <<0x1b, '[', 'A', rest:bytes>> => Key.Up
    <<len:16/little, payload:len*8, rest:bytes>> => payload
    _ => ...
}
let written = try <<4:4, 5:4, 0:8, 1500:16/big, "ab">> into buf[..]
```

段的形式是 `name:size/modifiers` 或字面量。大小以位计（默认 8）；`bytes`
吃掉剩余全部。修饰符：`big`（默认）、`little`、`native`、`signed`、
`unsigned`、`float`、`utf8`。常量大小不超过 64 位的绑定是能容纳它的最小宽度
整数；更大或动态大小的绑定是 `[]u8` 视图。大小运算按指针宽度进行且总是检
查。构造的目标是 `[]mut u8` 缓冲区，返回 `![]u8`（写入的前缀），失败时给出
`error.BufferTooSmall`。

## 效应（第 8 节）

`allocates refcounts blocks shared_mutable nondeterministic panics ffi`

每个函数的效应都会被推断。签名上的负约束（`!allocates`）会被检查；诊断沿
调用链指出引入该效应的位置。函数类型可以带负约束（`fn(i32) -> i32
!allocates`）；转换到该类型的闭包或函数必须满足它们。`panics` 可通过证明
消除（E6）：用同一切片的循环索引进行索引、编译期已知的索引、操作数范围可
容纳的算术、受保护的局部变量都不计入。`nx effects file.nx` 打印每个函数推
断出的效应集合。

## 标准库（内置）

- `println(fmt, .{args})`、`print`、`eprintln`、`format(...) -> String`。
  占位符：`{}`、`{x}`、`{X}`、`{b}`、`{o}`、`{e}`、`{c}`、`{:.N}`、`{>N}`、
  `{<N}`；`{{` 和 `}}` 是字面大括号。
- `expect(cond)`、`expect_eq(a, b)`、`panic(msg)`。
- `List(T)`：`new`、`with_capacity`、`from`、`append`、`pop`、`clear`、
  `clone`、`last`、`first`、`insert`、`remove`、`swap_remove`、`extend`、
  `reserve`、`items`、`is_empty`、`len`，以及切片方法。
- `String`：`new`、`from`、`with_capacity`、`append`、`append_char`、`clone`、
  `clear`、`pop`、`bytes`、`len`，以及 `[]u8` 的方法。
- `Map(K, V)`（键：整数、bool、char、`[]u8`、`String`）：`new`、`put`、
  `get`、`contains`、`remove`、`clear`、`clone`、`keys`、`values`、`len`、
  `m[key]`；`for k in m` 遍历键。
- 切片：`len`、`fill`、`reverse`、`sort`、`contains`、`index_of`、
  `copy_from`、`to_owned`、`is_empty`；`[]u8` 还有 `starts_with`、
  `ends_with`、`find`、`trim`、`split`、`lines`、`to_string`、`parse_int(T)`、
  `parse_float`、`eq_ignore_case`。
- 整数：`abs`、`min`、`max`、`checked_add/sub/mul`（返回 `?T`）、
  `to_string`。浮点：`abs`、`sqrt`、`floor`、`ceil`、`round`、`min`、`max`、
  `pow`、`to_string`。字符：`is_digit`、`is_alpha`、`is_space`、`to_lower`、
  `to_upper`、`to_digit`。
- `math`：`PI E TAU INF NAN`，`sqrt abs floor ceil round sin cos tan exp log
  log2 min max pow atan2 clamp`。
- `io.read_file(path) -> !String`、`io.write_file(path, bytes) -> !void`、
  `io.read_line() -> ?String`。
- `os.args() -> [][]u8`、`os.env(name) -> ?[]u8`、`os.exit(code)`、
  `process.run(argv: [][]u8) -> !i32`（启动进程、等待并返回退出码；程序无法
  启动时为 `error.IoError`）。
- `time.now() -> i64`（自纪元起的毫秒）、`time.monotonic() -> u64`（纳秒）、
  `time.sleep(ms)`。
- `random.int(lo, hi)`、`random.float()`、`random.seed(n)`。
- `mem.copy(dst, src)`。
- `@typeName(T)`、`@sizeOf(T)`、`@truncate(T, x)`、`@errorName(e)`、
  `@embedFile(path)`、`@weak(x)`、`@refCount(x)`、`@cImport(header)`、
  `@cstr(literal)`。

预定义错误：`OutOfMemory Panic InvalidRecord Truncated Overflow InvalidUtf8
NotFound IoError InvalidInput BufferTooSmall`。任何 `error.Name` 都会创建一个
新错误。

## 入口点

`fn main()`、`fn main() -> !void` 或 `fn main() -> u8`。从 `main` 返回的错误
打印 `error: Name` 并以 1 退出；panic 打印其位置并以 101 退出。`test "name"
{ }` 块由 `nx test` 运行。

## Trait 对象

`dyn Trait` 是由 `*T` 或 `*mut T` 构成的胖指针，其中 `T` 实现该 trait：
`let s: dyn Shape = &circle`、`List(dyn Shape)`、`[]dyn Shape`。调用通过
vtable 分派，并获得对象类型允许的全部效应；`dyn Shape !allocates !blocks`
是一个独立的类型，只有满足这些约束的实现才能转换进去。用作对象的 trait 只能
在接收者位置提到 `Self`。

## 并行循环

`for parallel x, i in items { ... }` 在线程池上对索引范围运行循环体（规范
7.2）。循环体不能有 `shared_mutable` 效应，不能 `return` 或 `break`（用
`continue`），并通过以 `i` 索引的可变切片写出结果。工作线程中的 panic 会在
所有工作线程结束后在调用方重新抛出。该循环带有 `blocks` 效应（它会等待）。

## 分配作用域

`using arena { ... }` 为该块安装一个 bump 分配器：块内创建的值来自 arena，
它们的释放是空操作，整个 arena 在块结束时一次性释放（规范 5.1，"在任何作用
域可替换"）。块外创建的容器在块内增长时仍使用堆，因此把结果收集到外层的
`List`、`String` 或 `Map` 是安全的。块内创建的值不得逃逸出块。

## 调用 C

```
const libc = @cImport("string.h")
const cv = @cImport("cvendor.h")               // 在源文件旁边查找
artifact link { c_sources = ["cvendor.c"], libs = [], include = [], libs_windows = ["gdi32"] }

unsafe {
    let n = libc.strlen(@cstr("hello"))
    let p = cv.cv_point{ .x = 1.0, .y = 2.0 }
    _ = cstdio.printf(@cstr("%d\n"), 42)      // 变参接受标量和指针
}
```

`@cImport` 运行 C 预处理器并导入函数、typedef、struct、enum 和字面量宏。
`const T*` 变为 `*T`，其他指针变为 `*mut T`，`void*` 变为 `*mut u8`。外部
调用需要 `unsafe` 并带有 `ffi` 效应。无法翻译的声明（union、位域、函数指
针、函数式宏）在使用时会在错误信息中点名。

## 编译期测试

`comptime test "name" { ... }` 在检查阶段由解释器运行；失败即是指向该断言
的编译错误。

## 视图与其存储

函数不能返回指向自身局部变量的切片或指针（规则 R1，自 1.0 起是错误）；指向借用
参数的视图没问题，因为它们归调用方所有。1.2 加入视图规则 V1 到 V5（`SPEC.md` 5.6
与 5.7）：存到其存储之外的视图、在容器增长或值移动之后使用的视图、返回含视图的
值、在 `using arena` 块之外保留的值。它们在 1.2 是警告，自 1.3 起是错误；每个错误都指明修法，
机械性的由 `nx fix` 完成。`@escape(v)` 把值复制出
竞技场块。

## 尚未实现

`soa` 和 `packed` 布局、`node` 和 `installer` 产物、`nx publish` 与注册表、
`pool`/`stack` 分配策略，以及已归档的区域规则 R2 到 R4（决定 88）。
