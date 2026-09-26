# Nexium 语言参考

本文描述 `nx` 编译器目前实现的内容。章节编号指向 `nexium-spec.txt`（设计文档）；标注“归档”的地方指向 `nexium-systems-spec.txt` 的第 4 到 9 节（语法参考）。

## 文件与模块

一个文件就是一个模块。`import foo.bar` 加载根文件旁边的 `foo/bar.nx`，其 `pub` 项通过 `bar.item` 访问。`import std.strings` 加载标准库的一个模块（`std.json`、`std.fs`、`std.http` 等），标准库用 Nexium 编写并嵌入在编译器中；见 [`std.md`](../../std.md)（英文）。内置命名空间 `math`、`io`、`os`、`time`、`random`、`mem`、`process`、`net`、`thread` 和 `sync` 始终在作用域内，无需 `import`。

## 词法结构（归档 4）

- `//` 注释；`///` 文档注释附着到下一个声明。
- 标识符仅限 ASCII。字符串和注释之外出现非 ASCII 字符是错误。
- 整数：`42`、`0xFF`、`0o755`、`0b1010_1100`。浮点数：`3.14`、`1e-9`、`0x1.8p3`。字符串 `"text"`（必须是 UTF-8）、原始字符串 `r"..."`、字节串 `b"\x00\xff"`、字符 `'a'`（一个 Unicode 标量值，类型为 `char`）。
- 语句在换行处结束。以 `|>`、`.method(`、`catch`、`orelse`、`and` 或 `or` 开头的行延续上一行；以二元运算符或未闭合的括号结尾的行同样延续。
- 命名约定：类型用 `PascalCase`，函数和变量用 `snake_case`，常量用 `SCREAMING_SNAKE_CASE`。

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
var counter: u32 = 0                             // 可变全局变量；访问需要 `unsafe`
test "name" { ... }
bench "name" { ... }
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
| `error` | 任意错误值（匿名错误集） |
| `List(T)`、`String`、`Map(K, V)` | 拥有所有权的集合（值语义，5.3 节） |
| `fn(A, B) -> R !effects` | 函数值（闭包和函数都能转换成它） |
| `(A, B)` | 元组；字段 `.0`、`.1`；也可用作类型参数，如 `List((A, B))` |
| `weak T` | 指向 `ref class` 的弱引用 |

整数字面量采用上下文要求的类型，默认为 `i64`；浮点字面量默认为 `f64`。字符字面量可放入任何容得下它的整数类型（`c: u8` 时可写 `c == 'a'`）。

转换：`x as T` 用于数值之间（除非范围已被证明，否则窄化在运行时检查）、`distinct` 类型与其表示之间、`char` 与整数之间、`bool` 到整数、单元枚举到整数。`@truncate(T, x)` 做回绕截断，不做检查。

## 值、绑定、所有权

`let x = e` 不可变绑定，`var x = e` 可变绑定。可变性是浅层的（归档 5.10）：持有 `*mut T` 的 `let` 仍可通过它修改。

集合拥有堆上的缓冲区（5.3）。`let b = a` 移动 `a`；之后再用 `a` 是编译错误；`a.clone()` 复制。移动按分支跟踪：在 `if` 的一个分支或 `match` 的一个分支臂中被移动的值，在其他分支中仍然可用，并在整个结构之后算作已移动。参数是借用的：接收 `List` 的函数读取它；要修改，则取 `*mut List(T)`。要取得所有权，给参数标上 `own`：

```
fn token(kind: u8, own text: String) -> Token {
    return Token{ .kind = kind, .text = text }     // 移入再移出：无需 clone
}
let t = token(1, name)                             // `name` 被移动；再次使用是错误
```

`own` 参数是可变的，除非被继续移走，否则在函数返回时释放，并且只对拥有所有权的类型有意义。接收者不能是 `own`，导出函数不能有 `own` 参数，带 `own` 参数的函数不能用作函数值（其类型无法说明谁拥有该参数）。传到期望切片的位置时，`List(T)` 和 `String` 会转换为 `[]T` / `[]u8`。从字段或元素中移出是错误。拥有的值在作用域结束时释放；这是作用域退出时唯一的自动动作（H7）。

`ref class` 的值是引用；复制一个会增加它的引用计数，最后一个引用消失时对象被释放（5.1）。循环会泄漏；对反向边使用 `weak`（`@weak(x)` 或 `x.weak()`，然后 `w.upgrade()`）。

## 表达式

- 算术 `+ - * / %` 在溢出时陷入（一个有定义的 panic）。回绕形式 `+% -% *%` 和饱和形式 `+| -| *|` 永不失败。位运算 `& | ^ ~ << >>`。比较 `== != < <= > >=` 适用于数值、字符、布尔、`[]u8`、`String`、单元枚举，以及 `derive(Eq)` / `derive(Ord)` 的类型。逻辑运算 `and`、`or`、`!`。
- `derive(Clone)` 给结构体或枚举提供 `.clone()`，即逐字段的深拷贝，前提是每个字段都可克隆（指针字段不行）；由可克隆元素组成的元组、可选值和数组无需它即可克隆，`where T: Clone` 约束类型参数。
- `x |> f(a)` 即 `f(x, a)`。
- `if c { a } else { b }` 是表达式；`if let v = opt { } else { }` 解包（作用于变量、字段这样的位置时，绑定是一个视图，和循环变量一样：要么克隆它，要么用 `opt.?` 取出值）。条件不加括号，主体总是带花括号，所以单行写法是 `if c { return v }`；`else` 可以从下一行开始。
- `match v { pat => expr, ... }` 可匹配整数（字面量、范围 `1..=9`）、字符串、布尔、字符、枚举（`.Variant(p)`）、可选值（`null`、绑定）、错误联合（`error.Name`、绑定）、元组、按形状匹配的切片（`[]`、`[first, rest..]`、`[.., last]`；剩余部分是 `[]T` 视图）、`whole @ pat`，以及字节切片（二进制模式）。枚举和布尔的 match 必须穷尽，切片的 match 必须按长度穷尽（`[]` 加 `[x, rest..]`）；其他需要 `_ =>`。
- 块是表达式，其值为最后一个表达式。带标签的块通过 `break :label value` 产出值。
- `try e` 传播错误；`e catch |err| handler`；`opt orelse default`；`opt.?` 解包（为 null 时 panic）。`orelse` 和 `catch` 的右侧可以是跳转：`let v = opt orelse return null`、`let v = r catch |e| return -1`。
- `opt?.field` 和 `opt?.m(x)` 透过可选值读取：可选值为空时得到 `null`，否则得到作为可选值的成员；链的其余部分作用于载荷（`a?.name.len orelse 0`），可选结果不会被包装两次（`a?.b?.c`）。
- `let (a, b) = pair` 和 `for (k, v) in pairs` 为元组的每个元素绑定一个名字（`_` 跳过一个）：作用于拥有的值时，这些名字拥有各个元素；作用于位置时，它们是它的视图，和 `if let` 绑定一样。
- 整数或浮点字面量可转换为 `?T`：`f(x: ?i32)` 时可写 `f(1)`。
- `defer stmt` 在作用域退出时执行，`errdefer stmt` 仅在作用域因错误退出时执行；两者都按注册的逆序执行。
- 闭包：`|[captures] params| -> R { body }`。捕获是显式的：`[x]` 复制，`[&x]` 和 `[&mut x]` 取引用。
- 可变全局变量、外部调用、指针转换和切片的 `.ptr` 都需要 `unsafe { }`。
- `comptime expr` 在编译期求值；`@embedFile("path")` 把一个声明过的构建输入嵌入为 `[]u8`。

## 语句与循环

```
while cond { }
for x in items { }               // 数组、切片、列表、字符串、映射的键
for (k, v) in m { }              // 映射的条目
for x in it { }                  // 任何带有 next(self: *mut Self) -> ?T 的值
for x, i in items { }            // 带索引
for x, y in a, b { }             // 同步遍历；长度必须一致
for i in 0..10 step 2 { }        // 0 2 4 6 8；`for i in 10..0 step -1` 倒数（有符号）
while cond { } else { }          // cond 变为假时执行 else，break 之后不执行
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

段的形式是 `name:size/modifiers` 或字面量。大小以位计（默认 8）；`bytes` 消耗剩余的全部。修饰符：`big`（默认）、`little`、`native`、`signed`、`unsigned`、`float`、`utf8`。常量大小不超过 64 位的绑定是能容纳它的最小宽度的整数；更大或动态大小的绑定是 `[]u8` 视图。大小运算按指针宽度进行，并且总是检查。构造写入一个 `[]mut u8` 缓冲区，返回 `![]u8`（写入的前缀），空间不足时以 `error.BufferTooSmall` 失败。

## 效应（第 8 节）

`allocates refcounts blocks shared_mutable nondeterministic panics ffi`

每个函数的效应都会被推断。签名上的负约束（`!allocates`）会被检查；诊断沿调用链指出引入该效应的位置。函数类型可以带负约束（`fn(i32) -> i32 !allocates`）；转换到这种类型的闭包或函数必须满足它们。`panics` 可通过证明消除（E6）：用同一切片的循环索引进行索引、编译期已知的索引、操作数范围可容纳的算术，以及受保护的局部变量，都不计入。`nx audit file.nx --lock` 写出 `file.effects.lock`，即每个函数的效应；当某个函数获得了锁文件中没有的效应时，`nx audit file.nx --check` 会失败（行为的语义化版本：一个开始分配内存或阻塞的依赖会让构建失败，直到有意重新生成锁文件）。`nx effects file.nx` 打印每个函数推断出的效应集合。`nx explain file.nx f effect` 以树的形式打印 `f` 为何具有某个效应：每个函数中记录的原因（一次分配、一次溢出检查、一次显式 panic），以及把它带进来的调用，一直追到引入它的原语。

## 标准库（内置）

- `println(fmt, .{args})`、`print`、`eprintln`、`format(...) -> String`；具名参数 `.{ .name = v }` 配合 `{name}` / `{name:spec}` 占位符，还有取自参数的宽度 `{v:>w}`。占位符：`{}`、`{x}`、`{X}`、`{b}`、`{o}`、`{e}`、`{c}`、`{:.N}`、`{>N}`、`{<N}`；`{{` 和 `}}` 是字面大括号。
- `expect(cond)`、`expect_eq(a, b)`、`panic(msg)`。
- `List(T)`：`new`、`with_capacity`、`from`、`append`、`pop`、`clear`、`clone`、`last`、`first`、`insert`、`remove`、`swap_remove`、`extend`、`reserve`、`items`、`is_empty`、`len`，以及切片方法。
- `String`：`new`、`from`、`with_capacity`、`append`、`append_char`（一个码点，按 UTF-8 编码）、`push_byte`（一个原始字节）、`clone`、`clear`、`pop`、`bytes`、`len`，以及 `[]u8` 的方法。
- `Map(K, V)`（键：整数、bool、char、`[]u8`、`String`；无论哈希如何，都按键首次放入的顺序迭代）：`new`、`put`、`get`、`contains`、`remove`、`clear`、`clone`、`keys`、`values`、`len`、`m[key]`；`for k in m` 遍历键。
- 切片：`len`、`fill`、`reverse`、`sort`、`swap(i, j)`、`contains`、`index_of`、`copy_from`、`to_owned`、`is_empty`；`[]u8` 还有 `starts_with`、`ends_with`、`find`、`trim`、`split`、`lines`、`to_string`、`parse_int(T)`、`parse_float`、`eq_ignore_case`。
- 整数：`abs`、`min`、`max`、`checked_add/sub/mul`（返回 `?T`）、`to_string`。浮点数：`abs`、`sqrt`、`floor`、`ceil`、`round`、`min`、`max`、`pow`、`to_string`。字符：`is_digit`、`is_alpha`、`is_space`、`to_lower`、`to_upper`、`to_digit`。
- `math`：`PI E TAU INF NAN`，`sqrt abs floor ceil round sin cos tan exp log log2 min max pow atan2 clamp`。
- `io.read_file(path) -> !String`、`io.write_file(path, bytes) -> !void`、`io.append_file(path, bytes) -> !void`、`io.read_line() -> ?String`。
- 文件系统原语（`std.fs` 在其上加了路径处理和目录遍历）：`io.file_kind(path) -> i32`（0 不存在，1 文件，2 目录）、`io.file_size(path) -> !u64`、`io.file_modified(path) -> !i64`（毫秒）、`io.make_dir(path) -> !void`、`io.remove_file(path) -> !void`、`io.remove_dir(path) -> !void`（须为空）、`io.rename(from, to) -> !void`、`io.list_dir(path) -> !List(String)`、`io.cwd() -> !String`、`io.temp_dir() -> String`。失败时为 `error.NotFound` 或 `error.IoError`。
- 文件句柄（`std.stream` 在其上加了缓冲）：`io.open(path, mode) -> !i64`（模式 `r`、`w`、`a`）、`io.read(h, n) -> !String`（至多 `n` 字节；输入结束时为空）、`io.write(h, bytes) -> !void`、`io.flush(h) -> !void`、`io.close(h) -> !void`。句柄 1、2、3 分别是 stdin、stdout、stderr；`io.is_terminal(h) -> bool` 判断其中之一是否为终端。`io.raw_mode(on) -> bool` 让控制台按键入的原样交出字节，不回显，输入输出都带 VT 序列（REPL 的行编辑器用它；退出时恢复控制台），`io.read_key() -> ?i64` 是输入的一个字节，`io.pending_input() -> i64` 是缓冲中的字节数（转义序列整体到达）。在 REPL 中不可用。
- 套接字（`std.net` 和 `std.http` 构建于其上；每个调用都带 `blocks`）：`net.connect(host, port, timeout_ms) -> !i64`、`net.listen(host, port) -> !i64`、`net.accept(listener, timeout_ms) -> !i64`、`net.send(sock, bytes) -> !void`、`net.recv(sock, n, timeout_ms) -> !String`（对端关闭时为空）、`net.close(sock)`、`net.peer(sock)` / `net.local(sock) -> !String`（`ip:port`）、`net.resolve(host) -> !List(String)`、`net.udp_bind(host, port) -> !i64`、`net.send_to(sock, host, port, bytes)`、`net.recv_from(sock, n, timeout_ms) -> !String`，由 `net.last_peer()` 给出发送方。超时为 0 表示永远等待。错误：`NotFound`（名字解析）、`ConnectionRefused`、`Timeout`、`IoError`。在 REPL 中不可用。
- TCP 之上的 TLS，使用平台自带的库，首次使用时加载（Windows 上是 SChannel，macOS 上是 Security.framework，其他平台是 OpenSSL 的 libssl 3 或 1.1；`std.http` 的 `SystemTls` 封装了它们，每个调用都带 `blocks`）：`net.tls_available() -> bool`、`net.tls_connect(host, port, timeout_ms) -> !i64`（在超时内完成连接和握手；服务器证书依据系统的根证书和主机名进行校验）、`net.tls_send(h, bytes) -> !void`、`net.tls_recv(h, n, timeout_ms) -> !String`（结束时为空）、`net.tls_truncated(h) -> bool`（结束时没有 close_notify）、`net.tls_close(h)`，以及 `net.tls_problem() -> String`，即本线程上一次 TLS 调用失败的原因。错误与套接字的相同；校验不通过的证书是 `IoError`。在 REPL 中不可用。
- 线程（`std.thread` 在其上构建 `Thread`、`Channel`、`Mutex`、`Atomic`、`select`、`each` 和 `both`）：`thread.start(f: fn(*mut T) -> void, arg: *mut T) -> i64` 在一个拥有自己上下文的新线程上运行 `f(arg)`，`thread.join(h)` 等待它结束并重新抛出它的 panic，`thread.join_all(hs: []i64)` 等待所有线程结束后再重新抛出其中第一个 panic，`thread.count() -> usize` 是硬件线程数。`sync.mutex_new() -> i64`、`sync.lock(m)`、`sync.unlock(m)`、`sync.mutex_free(m)`、`sync.cond_new() -> i64`、`sync.wait(cv, m)`、`sync.wait_for(cv, m, ms) -> bool`（超过 `ms` 时为 false；小于 0 则永远等待）、`sync.signal(cv)`、`sync.broadcast(cv)`、`sync.cond_free(cv)`。铃可由任何线程敲响，由一个线程等待，等待之前的敲响会为这次等待保留：`sync.bell_new() -> i64`、`sync.bell_ring(b)`、`sync.bell_wait(b, ms) -> bool`、`sync.bell_free(b)`。`i64` 上的顺序一致原子操作：`sync.atomic_load(p: *i64) -> i64`、`sync.atomic_store(p: *mut i64, v)`、`sync.atomic_add(p, n) -> i64` 和 `sync.atomic_swap(p, v) -> i64`（两者都返回操作前的值）、`sync.atomic_cas(p, expected, new) -> bool`。启动线程带 `nondeterministic` 和 `shared_mutable`；join、加锁和等待带 `blocks`，`sync` 调用带 `shared_mutable`。在 REPL 中不可用。
- `os.arch() -> []u8`：程序运行所在的架构（`x86_64`、`aarch64`、`x86`、`arm`、`riscv64` 或 `unknown`），在程序编译时确定。
- `os.args() -> [][]u8`、`os.exe_path() -> String`（正在运行的可执行文件；平台不提供时为空）、`os.env(name) -> ?[]u8`、`os.set_env(name, value)`（作用于本进程及其启动的进程；空值会删除该变量）、`os.environ() -> List(String)`（每个 `NAME=value`）、`os.exit(code)`、`process.run(argv: [][]u8) -> !i32`（启动、等待并返回退出码；程序无法启动时为 `error.IoError`）、`process.exec(argv, stdin, cwd) -> !i32`（同上，但标准输入取自 `stdin`，`cwd` 非空时在其中运行，并捕获 stdout/stderr），之后用 `process.last_stdout()` / `process.last_stderr() -> String` 取得输出；`std.process` 封装了这些。
- 同时运行的程序（`std.process` 在其上构建 `Child`；除 `child_pid` 外每个调用都带 `blocks`）：`process.spawn(argv, cwd, flags) -> !i64` 启动一个程序（在 `cwd` 中，为空则在本程序的目录中），每个标准流占 `flags` 的两位，顺序为 stdin、stdout、stderr，取值为沿用本程序的（0）、管道（1）、丢弃（2），或 stderr 并入 stdout（3）。`process.child_write(h, bytes, timeout_ms) -> !void` 随程序读取的节奏写完整个 `bytes`，`process.child_close_input(h)` 结束其输入，`process.child_read(h, stream, n, timeout_ms) -> !String` 读取流 1（stdout）或 2（stderr）中至多 `n` 字节，流结束时为空，`process.child_wait(h, timeout_ms) -> !i64` 的低 32 位是退出码，其上是终止它的信号，`process.child_signal(h, sig) -> !void` 发送一个信号（Windows 没有信号，除 0 以外的每个信号都会以退出码 128 + `sig` 结束程序），还有 `process.child_pid(h) -> i64`，以及放手不管的 `process.child_close(h)`。在某个调用等待期间，程序写出的内容会保留给之后的读取，因此它绝不会因管道写满而卡住。小于 0 的超时表示永远等待，0 表示不等待。错误：`NotFound`（没有这个程序）、`Timeout`、`IoError`（其中包括不再读取输入的程序）。`process.trap_signals()` 阻止 SIGINT、SIGTERM 和 SIGHUP 结束程序（在 Windows 上：Ctrl-C 为 2，Ctrl-Break 为 21，关闭控制台为 1，注销或关机为 15），`process.next_signal(timeout_ms) -> i32` 取出下一个捕获到的信号，超时前没有信号则为 0。在 REPL 中不可用。
- `time.now() -> i64`（自纪元起的毫秒）、`time.monotonic() -> u64`（纳秒）、`time.utc_offset(ms) -> i64`（该时刻本地时间相对 UTC 向东偏移的分钟数；`std.time` 在这些之上构建日期）、`time.sleep(ms)`。
- `time.zone_rules(name) -> !string`：以文本形式给出 IANA 数据库中的一个时区，供 Windows 上的 `std.time` 使用，因为 Windows 把该数据库放在 ICU 中，而不是 zoneinfo 文件里（Windows 10 1903 及以后；首次调用时加载）。第一行是时区名，之后每个时段一行，`start offset dst abbrev`；空名字表示系统时区。ICU 中没有的时区给出 `NotFound`，在其他所有平台上也是如此，因为那里 `std.time` 自己读取 zoneinfo 文件。请改用 `std.time` 的 `time.zone(name)`。
- `random.int(lo, hi)`、`random.float()`、`random.seed(n)`：一个快速的生成器，`random.seed` 可使其结果可重复，绝不能用于机密。
- `random.secure(buf) -> !void`：用操作系统的安全生成器（BCryptGenRandom、getrandom、arc4random）填充一个 `[]mut u8`，用于密钥、令牌和 UUID；`random.seed` 不影响它，只有在系统没有生成器时才以 `IoError` 失败。REPL 和 `nx play` 不运行它：它需要编译后的程序。
- `mem.copy(dst, src)`。
- `@typeName(T)`、`@sizeOf(T)`、`@alignOf(T)`、`@truncate(T, x)`、`@bitCast(T, x)`（大小相同的标量）、`@min(a, b)`、`@max(a, b)`、`@errorName(e)`、`@embedFile(path)`、`@weak(x)`、`@refCount(x)`、`@cImport(header)`、`@cstr(literal)`、`@target()`（`(os, arch, bits)`，本次 C 构建的一个常量）、`@escape(v)`（在最内层 `using arena` 块之外制作的 `v` 的副本）。

预定义错误：`OutOfMemory Panic InvalidRecord Truncated Overflow InvalidUtf8 NotFound IoError InvalidInput BufferTooSmall`。任何 `error.Name` 都会创建一个新错误。

## 入口点

`fn main()`、`fn main() -> !void` 或 `fn main() -> u8`。从 `main` 返回的错误会打印 `error: Name` 并以 1 退出；panic 会打印其位置并以 101 退出。`test "name" { }` 块由 `nx test` 运行。

## 基准测试

`bench "name" { ... }` 与测试放在一起，由 `nx bench` 运行：它以优化方式构建文件（除非 `--mode` 另有要求，否则为 `safe` 模式），并测量每个块：先校准迭代次数，使每个样本约为 10 ms（校准即预热），再取 21 个样本，打印每次迭代时间的中位数，以及最快和最慢的样本：

```
bench  sum to 1000      187 ns/iter  (min 186 ns, max 193 ns; 21 samples of 64398)
bench  a list of 100    204 ns/iter  (min 202 ns, max 209 ns; 21 samples of 58241)
```

块中最后一个表达式的值会被保留，因此产生它的工作不会被优化掉：`bench "sum" { sum_to(1000) }` 测量的是求和，而 `_ = sum_to(1000)` 可能什么也测不到。错误联合类型的值必须经过 `try`，这样错误会使基准测试失败，而不是被当作结果测量。panic 或错误会使该基准测试失败，其他基准照常运行。文件之后的参数把运行范围缩小到名字包含它们的基准，`--quick` 使用 1 ms 的样本。`nx test` 不运行基准测试，`nx check` 会检查它们。

## 递归类型与透过指针的匹配

`List` 可以容纳正在定义的类型，因此树和 JSON 值就是普通的枚举：`enum Json { Null, Arr(List(Json)), Obj(List(Member)) }`。透过指针进行匹配时，拥有所有权的载荷按引用绑定：

```
fn push(v: *mut Json, own item: Json) {
    match v.* {
        .Arr(items) => items.append(item),   // items: *mut List(Json)，是载荷的别名
        _ => {},
    }
}
```

`v: *Json` 时绑定为 `*List(Json)`。标量（`.Num(n)`）会被复制。在期望切片的位置，`*String` 或 `*List(T)` 会转换为 `[]u8` 或 `[]T`。

## Trait 对象

`dyn Trait` 是由 `*T` 或 `*mut T` 构成的胖指针，其中 `T` 实现该 trait：`let s: dyn Shape = &circle`、`List(dyn Shape)`、`[]dyn Shape`。调用通过 vtable 分派，并获得对象类型允许的全部效应；`dyn Shape !allocates !blocks` 是一个独立的类型，只有满足这些约束的实现才能转换进去。用作对象的 trait 只能在接收者位置提到 `Self`。

## 并行循环

`for parallel x, i in items { ... }` 在线程池上对索引范围运行循环体（规范 7.2）。循环体不能有 `shared_mutable` 效应，不能 `return` 或 `break`（用 `continue`），并通过以 `i` 索引的可变切片写出结果。工作线程中的 panic 会在所有工作线程结束后在调用方重新抛出。该循环带有 `blocks` 效应（它要等待工作线程汇合）。

## 分配作用域

`using arena { ... }` 为该块安装一个 bump 分配器：块内创建的值来自 arena，它们的释放是空操作，整个 arena 在块结束时一次性释放（规范 5.1，“在任何作用域都可替换”）。块外创建的容器在块内增长时仍使用堆，因此把结果收集到外层的 `List`、`String` 或 `Map` 是安全的。块内创建的值不得逃逸出块。

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

`@cImport` 运行 C 预处理器，并导入函数、typedef、struct、enum 和字面量宏。`const T*` 变为 `*T`，其他指针变为 `*mut T`，`void*` 变为 `*mut u8`。外部调用需要 `unsafe`，并带有 `ffi` 效应。字段无法翻译的 struct（函数指针、位域、嵌套定义）作为不透明类型导入，可以像前向声明一样通过指针使用；每个 libc 上的 `FILE` 就是这样工作的。完全无法翻译的声明（union、函数指针 typedef、函数式宏）在使用时会在错误信息中点名。

## 条件编译

`if comptime C { ... } else { ... }` 在检查程序时求值 `C`，只构建它选中的分支。另一个分支从不检查，因此可以调用只在其他平台上存在的东西：

```nexium
fn line_ending() -> []u8 {
    if comptime @target().0 == "windows" {
        return "\r\n"
    } else {
        return "\n"
    }
}
```

`@target()` 是本次构建的 `(os, arch, pointer_bits)`，交叉编译时是 `--target` 的值。带值的 `if comptime` 具有所选分支的类型。

## 编译期测试

`comptime test "name" { ... }` 在检查阶段由解释器运行；失败即是指向该断言的编译错误。

## 视图与其存储

函数不能返回指向自身局部变量的切片或指针（规则 R1，自 1.0 起是错误）；指向借用参数的视图没问题，因为它们归调用方所有。1.2 加入了视图规则 V1 到 V5（`SPEC.md` 5.6 与 5.7）：保存得比其所指存储更久的视图、在容器增长或值被移动之后使用的视图、返回一个含视图的值、在 `using arena` 块结束后仍保留的值。它们在 1.2 是警告，自 1.3 起是错误；每个错误都指明修法，机械性的修改（值被移动处的 `.clone()`、`@escape`）由 `nx fix` 完成。`@escape(v)` 把值复制到 arena 块之外。

## 尚未实现

`nx publish` 与注册表。`soa` 和 `packed` 布局、`pool` 和 `stack` 分配策略，以及已归档的区域规则 R2 到 R4，都不属于这门语言（决定 88）；编译器会拒绝这些写法。
