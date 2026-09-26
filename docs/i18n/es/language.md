# Referencia del lenguaje Nexium

Describe lo que el compilador `nx` implementa hoy. Los números de sección se
refieren a `nexium-spec.txt` (el diseño) y, donde se marca "archivado", a las
secciones 4 a 9 de `nexium-systems-spec.txt` (la referencia de sintaxis).

## Archivos y módulos

Un archivo es un módulo. `import foo.bar` carga `foo/bar.nx` junto al archivo
raíz; sus elementos `pub` se alcanzan como `bar.item`. `import std.strings`
carga un módulo de la biblioteca estándar (`std.json`, `std.fs`, `std.http` y
los demás), que está escrita en Nexium e incrustada en el compilador; véase
[`std.md`](../../std.md) (inglés). Los espacios de nombres integrados `math`,
`io`, `os`, `time`, `random`, `mem`, `process`, `net`, `thread` y `sync`
siempre están en ámbito y no necesitan `import`.

## Estructura léxica (archivado 4)

- Comentarios `//`; los comentarios de documentación `///` se adjuntan a la
  siguiente declaración.
- Los identificadores son ASCII. Un carácter no ASCII fuera de cadenas y
  comentarios es un error.
- Enteros: `42`, `0xFF`, `0o755`, `0b1010_1100`. Flotantes: `3.14`, `1e-9`,
  `0x1.8p3`. Cadenas `"text"` (deben ser UTF-8), cadenas crudas `r"..."`,
  cadenas de bytes `b"\x00\xff"`, caracteres `'a'` (un escalar Unicode, de
  tipo `char`).
- Las sentencias terminan en un salto de línea. Una línea que empieza con
  `|>`, `.method(`, `catch`, `orelse`, `and` u `or` continúa la línea
  anterior, igual que una línea que termina con un operador binario o con un
  paréntesis, corchete o llave sin cerrar.
- Convenciones: tipos `PascalCase`, funciones y variables `snake_case`,
  constantes `SCREAMING_SNAKE_CASE`.

## Declaraciones

```
fn name(a: T, b: U) -> R effects { ... }         // efectos: p. ej. !allocates !panics
pub fn f(x: i32) -> i32 export(c) { ... }        // exportada con la ABI de C
extern fn puts(s: *u8) -> i32                    // foránea; llamarla requiere `unsafe`
struct Point derive(Eq, Ord) { x: f64, y: f64 }
struct Frame layout(c) { id: u32, len: u16 }     // disposición de C, usable a través de la frontera
struct Pair(T) { a: T, b: T }                    // genérica
record Dose { mg: f64 where value > 0.0 }        // datos validados
ref class Node { value: i32, next: ?Node }       // con conteo de referencias
enum Shape { Circle(f64), Rect { w: f64, h: f64 }, Empty }
type Meters = distinct f64                       // sin conversión implícita
type Bytes = []u8                                // alias
error ParseError { Empty, NotANumber }
trait Shape { fn area(self: *Self) -> f64 }
impl Shape for Circle { fn area(self: *Self) -> f64 { ... } }
impl Point { fn origin() -> Point { ... } }
impl(T) Pair(T) { fn swap(self: *mut Self) { ... } }
const TABLE: [256]u8 = comptime build_table()
var counter: u32 = 0                             // global mutable; acceder requiere `unsafe`
test "name" { ... }
bench "name" { ... }
artifact cabi { name = "lib", exports = [f] }
```

Los campos dentro de `{ }` se separan con comas o saltos de línea. Un campo
puede tener un valor por defecto (`verbose: u8 = 0`).

## Tipos (archivado 5)

| sintaxis | significado |
| --- | --- |
| `i8 … i128`, `u8 … u128`, `isize`, `usize` | enteros; sin conversión implícita entre anchos |
| `f32`, `f64`, `bool`, `char`, `void`, `never` | primitivos |
| `[N]T` | array, `N` una constante en tiempo de compilación |
| `[]T`, `[]mut T` | slice: puntero más longitud; `[]u8` es texto |
| `*T`, `*mut T` | puntero a un valor (`&x`, `&mut x`, `p.*`) |
| `?T` | opcional; `null` es el valor vacío |
| `!T`, `Set!T` | unión de error |
| `error` | cualquier valor de error (el conjunto de errores anónimo) |
| `List(T)`, `String`, `Map(K, V)` | colecciones propietarias (valores, sección 5.3) |
| `fn(A, B) -> R !effects` | valor función (las clausuras y las funciones se convierten a él) |
| `(A, B)` | tupla; campos `.0`, `.1`; también vale como argumento de tipo, `List((A, B))` |
| `weak T` | referencia débil a una `ref class` |

Los literales enteros toman el tipo que pide el contexto y por defecto son
`i64`; los flotantes, `f64`. Un literal de carácter cabe en cualquier tipo
entero que pueda contenerlo (`c == 'a'` con `c: u8`).

Conversiones: `x as T` entre números (el estrechamiento se comprueba en tiempo
de ejecución salvo que el rango esté demostrado), entre un tipo `distinct` y
su representación, entre `char` y enteros, de `bool` a entero y de un enum
unitario a entero. `@truncate(T, x)` da la vuelta en lugar de comprobar.

## Valores, ligaduras, propiedad

`let x = e` liga de forma inmutable, `var x = e` de forma mutable. La
mutabilidad es superficial (archivado 5.10): un `let` que contiene `*mut T`
sigue pudiendo mutar a través de él.

Las colecciones poseen un búfer en el montón (5.3). `let b = a` mueve `a`;
usar `a` después es un error de compilación; `a.clone()` copia. Los
movimientos se siguen por rama: un valor movido en una rama de un `if` o en un
brazo de un `match` sigue disponible en las demás, y cuenta como movido
después de la construcción. Los parámetros se prestan: una función que recibe
una `List` la lee; para mutarla, toma `*mut List(T)`. Para tomar la propiedad,
marca el parámetro con `own`:

```
fn token(kind: u8, own text: String) -> Token {
    return Token{ .kind = kind, .text = text }     // entra movido y sigue movido: sin clone
}
let t = token(1, name)                             // `name` se mueve; volver a usarlo es un error
```

Un parámetro `own` es mutable, se libera cuando la función retorna salvo que
se haya movido a otro sitio, y solo tiene sentido para tipos propietarios. Los
receptores no pueden ser `own`, las funciones exportadas no pueden tomar
parámetros `own`, y una función que tenga uno no puede usarse como valor
función (su tipo no diría quién es el dueño del argumento). `List(T)` y
`String` se convierten a `[]T` / `[]u8` cuando se pasan donde se espera un
slice. Mover fuera de un campo o de un elemento es un error. Los valores
propios se liberan cuando termina su ámbito; esa es la única acción automática
al salir de un ámbito (H7).

Los valores `ref class` son referencias; copiar uno lo retiene y el objeto se
libera cuando desaparece la última referencia (5.1). Los ciclos fugan; usa
`weak` para las aristas de vuelta (`@weak(x)` o `x.weak()`, luego
`w.upgrade()`).

## Expresiones

- La aritmética `+ - * / %` atrapa el desbordamiento (un pánico definido). Las
  formas envolventes `+% -% *%` y saturantes `+| -| *|` nunca fallan. Bit a
  bit: `& | ^ ~ << >>`. Comparación `== != < <= > >=` sobre números,
  caracteres, booleanos, `[]u8`, `String`, enums unitarios y tipos con
  `derive(Eq)` / `derive(Ord)`. Lógicos: `and`, `or`, `!`.
- `derive(Clone)` da a un struct o enum `.clone()`, una copia profunda campo
  a campo, siempre que todos sus campos se puedan clonar (un campo puntero no
  puede); las tuplas, los opcionales y los arrays de cosas clonables se clonan
  sin él, y `where T: Clone` acota un parámetro de tipo.
- `x |> f(a)` es `f(x, a)`.
- `if c { a } else { b }` es una expresión; `if let v = opt { } else { }`
  desenvuelve (sobre un lugar, como una variable o un campo, la ligadura es una
  vista, igual que una variable de bucle: clónala o toma el valor con
  `opt.?`). Las condiciones no llevan paréntesis y los cuerpos siempre llevan
  llaves, así que en una sola línea se escribe `if c { return v }`; `else`
  puede empezar la línea siguiente.
- `match v { pat => expr, ... }` sobre enteros (literales, rangos `1..=9`),
  cadenas, booleanos, caracteres, enums (`.Variant(p)`), opcionales (`null`,
  ligadura), uniones de error (`error.Name`, ligadura), tuplas, slices por su
  forma (`[]`, `[first, rest..]`, `[.., last]`; el resto es una vista `[]T`),
  `whole @ pat` y slices de bytes (patrones binarios). Los `match` sobre enums
  y booleanos deben ser exhaustivos, y los de slices por longitud (`[]` con
  `[x, rest..]`); los demás necesitan `_ =>`.
- Los bloques son expresiones cuyo valor es la expresión final. Un bloque
  etiquetado produce su valor con `break :label value`.
- `try e` propaga un error; `e catch |err| handler`; `opt orelse default`;
  `opt.?` desenvuelve (pánico si es null). El lado derecho de `orelse` y de
  `catch` puede ser un salto: `let v = opt orelse return null`,
  `let v = r catch |e| return -1`.
- `opt?.field` y `opt?.m(x)` leen a través de un opcional: `null` cuando lo
  es y, si no, el miembro como opcional; el resto de una cadena se aplica a la
  carga (`a?.name.len orelse 0`), y un resultado opcional no se envuelve dos
  veces (`a?.b?.c`).
- `let (a, b) = pair` y `for (k, v) in pairs` ligan un nombre por cada
  elemento de la tupla (`_` salta uno): sobre un valor propio, los nombres son
  dueños de los elementos; sobre un lugar, son vistas de él, como una ligadura
  de `if let`.
- Un literal entero o flotante se convierte a `?T`: `f(1)` con `f(x: ?i32)`.
- `defer stmt` se ejecuta al salir del ámbito, `errdefer stmt` solo cuando el
  ámbito sale por un error; ambos en orden inverso de registro.
- Clausuras: `|[captures] params| -> R { body }`. Las capturas son explícitas:
  `[x]` copia, `[&x]` y `[&mut x]` toman referencias.
- `unsafe { }` es obligatorio para globales mutables, llamadas foráneas,
  conversiones de punteros y `.ptr` de un slice.
- `comptime expr` se evalúa en tiempo de compilación; `@embedFile("path")`
  incrusta una entrada de compilación declarada como `[]u8`.

## Sentencias y bucles

```
while cond { }
for x in items { }               // arrays, slices, listas, cadenas, claves de un mapa
for (k, v) in m { }              // las entradas de un mapa
for x in it { }                  // cualquier valor con next(self: *mut Self) -> ?T
for x, i in items { }            // con índice
for x, y in a, b { }             // a la par; las longitudes deben coincidir
for i in 0..10 step 2 { }        // 0 2 4 6 8; `for i in 10..0 step -1` cuenta hacia atrás (con signo)
while cond { } else { }          // el else se ejecuta cuando cond pasa a ser falsa, no tras un break
for i in 0..n { }
outer: for a in ... { for b in ... { continue :outer } }
break, continue, return
_ = expr                          // descarte explícito; los valores sin usar son errores
```

## Patrones binarios (archivado 6)

```
match packet {
    <<version:4, ihl:4, total_len:16/big, rest:bytes>> => ...
    <<0x1b, '[', 'A', rest:bytes>> => Key.Up
    <<len:16/little, payload:len*8, rest:bytes>> => payload
    _ => ...
}
let written = try <<4:4, 5:4, 0:8, 1500:16/big, "ab">> into buf[..]
```

Los segmentos son `name:size/modifiers` o literales. Los tamaños son bits (por
defecto 8); `bytes` consume el resto. Modificadores: `big` (por defecto),
`little`, `native`, `signed`, `unsigned`, `float`, `utf8`. Una ligadura con un
tamaño constante de hasta 64 bits es un entero del menor ancho en que quepa;
los tamaños mayores o dinámicos ligan una vista `[]u8`. La aritmética de
tamaños se hace al ancho de puntero y siempre se comprueba. La construcción
escribe en un búfer `[]mut u8` y devuelve `![]u8` (el prefijo escrito), y
falla con `error.BufferTooSmall`.

## Efectos (sección 8)

`allocates refcounts blocks shared_mutable nondeterministic panics ffi`

Los efectos se infieren para cada función. Una cota negativa en una firma
(`!allocates`) se comprueba; el diagnóstico señala, a través de las llamadas,
el sitio que introdujo el efecto. Los tipos función pueden llevar cotas
negativas (`fn(i32) -> i32 !allocates`); una clausura o función convertida a
tal tipo debe cumplirlas. `panics` se descarga por demostración (E6): indexar
con el índice de un bucle sobre el mismo slice, los índices conocidos en
tiempo de compilación, la aritmética cuyos rangos de operandos caben y los
locales protegidos no contribuyen. `nx audit file.nx --lock` escribe
`file.effects.lock`, con los efectos de cada función, y
`nx audit file.nx --check` falla cuando una función ganó un efecto que el lock
no nombra (semver para el comportamiento: una dependencia que empieza a
asignar memoria o a bloquear rompe la compilación hasta que el lock se
regenera a propósito). `nx effects file.nx` imprime el conjunto inferido de
cada función. `nx explain file.nx f effect` imprime como un árbol por qué `f`
tiene un efecto: la razón registrada en cada función (una asignación, una
comprobación de desbordamiento, un pánico explícito) y las llamadas que lo
traen, hasta la primitiva que lo origina.

## Biblioteca estándar (integrados)

- `println(fmt, .{args})`, `print`, `eprintln`, `format(...) -> String`;
  argumentos con nombre `.{ .name = v }` con marcadores `{name}` /
  `{name:spec}`, y un ancho tomado de un argumento, `{v:>w}`. Marcadores:
  `{}`, `{x}`, `{X}`, `{b}`, `{o}`, `{e}`, `{c}`, `{:.N}`, `{>N}`, `{<N}`;
  `{{` y `}}` son llaves literales.
- `expect(cond)`, `expect_eq(a, b)`, `panic(msg)`.
- `List(T)`: `new`, `with_capacity`, `from`, `append`, `pop`, `clear`,
  `clone`, `last`, `first`, `insert`, `remove`, `swap_remove`, `extend`,
  `reserve`, `items`, `is_empty`, `len`, más los métodos de slice.
- `String`: `new`, `from`, `with_capacity`, `append`, `append_char` (un punto
  de código, codificado en UTF-8), `push_byte` (un byte crudo), `clone`,
  `clear`, `pop`, `bytes`, `len`, más los métodos de `[]u8`.
- `Map(K, V)` (claves: enteros, bool, char, `[]u8`, `String`; se recorre en el
  orden en que cada clave se insertó por primera vez, sea cual sea su hash):
  `new`, `put`, `get`, `contains`, `remove`, `clear`, `clone`, `keys`,
  `values`, `len`, `m[key]`; `for k in m` recorre las claves.
- Slices: `len`, `fill`, `reverse`, `sort`, `swap(i, j)`, `contains`,
  `index_of`, `copy_from`, `to_owned`, `is_empty`; `[]u8` además
  `starts_with`, `ends_with`, `find`, `trim`, `split`, `lines`, `to_string`,
  `parse_int(T)`, `parse_float`, `eq_ignore_case`.
- Enteros: `abs`, `min`, `max`, `checked_add/sub/mul` (devuelven `?T`),
  `to_string`. Flotantes: `abs`, `sqrt`, `floor`, `ceil`, `round`, `min`,
  `max`, `pow`, `to_string`. Caracteres: `is_digit`, `is_alpha`, `is_space`,
  `to_lower`, `to_upper`, `to_digit`.
- `math`: `PI E TAU INF NAN`, `sqrt abs floor ceil round sin cos tan exp log
  log2 min max pow atan2 clamp`.
- `io.read_file(path) -> !String`, `io.write_file(path, bytes) -> !void`,
  `io.append_file(path, bytes) -> !void`, `io.read_line() -> ?String`.
- Primitivas del sistema de archivos (`std.fs` las envuelve con rutas y
  recorridos de directorios): `io.file_kind(path) -> i32` (0 no existe,
  1 archivo, 2 directorio), `io.file_size(path) -> !u64`,
  `io.file_modified(path) -> !i64` (ms), `io.make_dir(path) -> !void`,
  `io.remove_file(path) -> !void`, `io.remove_dir(path) -> !void` (vacío),
  `io.rename(from, to) -> !void`, `io.list_dir(path) -> !List(String)`,
  `io.cwd() -> !String`, `io.temp_dir() -> String`. Los fallos son
  `error.NotFound` o `error.IoError`.
- Manejadores de archivo (`std.stream` los envuelve con un búfer):
  `io.open(path, mode) -> !i64` (modo `r`, `w`, `a`),
  `io.read(h, n) -> !String` (hasta `n` bytes; vacío al final de la entrada),
  `io.write(h, bytes) -> !void`, `io.flush(h) -> !void`,
  `io.close(h) -> !void`. Los manejadores 1, 2 y 3 son stdin, stdout y
  stderr; `io.is_terminal(h) -> bool` dice si uno de ellos es un terminal.
  `io.raw_mode(on) -> bool` entrega los bytes de la consola tal como se
  teclean, sin eco, con secuencias VT a la entrada y a la salida (el editor de
  línea del REPL; al salir se restaura la consola), `io.read_key() -> ?i64` es
  un byte de entrada e `io.pending_input() -> i64` cuántos hay en el búfer
  (una secuencia de escape llega entera). No disponibles en el REPL.
- Sockets (`std.net` y `std.http` se construyen sobre ellos; toda llamada
  lleva `blocks`): `net.connect(host, port, timeout_ms) -> !i64`,
  `net.listen(host, port) -> !i64`,
  `net.accept(listener, timeout_ms) -> !i64`,
  `net.send(sock, bytes) -> !void`,
  `net.recv(sock, n, timeout_ms) -> !String` (vacío cuando el otro extremo
  cerró), `net.close(sock)`, `net.peer(sock)` / `net.local(sock) -> !String`
  (`ip:port`), `net.resolve(host) -> !List(String)`,
  `net.udp_bind(host, port) -> !i64`, `net.send_to(sock, host, port, bytes)`,
  `net.recv_from(sock, n, timeout_ms) -> !String`, con `net.last_peer()`
  nombrando al remitente. Un tiempo de espera de 0 espera para siempre.
  Errores: `NotFound` (búsqueda del nombre), `ConnectionRefused`, `Timeout`,
  `IoError`. No disponibles en el REPL.
- TLS sobre TCP mediante la biblioteca propia de la plataforma, cargada en el
  primer uso (SChannel en Windows, Security.framework en macOS, libssl 3 o 1.1
  de OpenSSL en los demás; `SystemTls` de `std.http` los envuelve, y toda
  llamada lleva `blocks`): `net.tls_available() -> bool`,
  `net.tls_connect(host, port, timeout_ms) -> !i64` (la conexión y la
  negociación dentro del tiempo de espera; el certificado del servidor se
  comprueba contra las raíces del sistema y el nombre del host),
  `net.tls_send(h, bytes) -> !void`,
  `net.tls_recv(h, n, timeout_ms) -> !String` (vacío al final),
  `net.tls_truncated(h) -> bool` (el final llegó sin close_notify),
  `net.tls_close(h)` y `net.tls_problem() -> String`, por qué falló la última
  llamada TLS en este hilo. Los errores son los de los sockets; un certificado
  que no supera la comprobación es `IoError`. No disponibles en el REPL.
- Hilos (`std.thread` construye `Thread`, `Channel`, `Mutex`, `Atomic`,
  `select`, `each` y `both` sobre estos):
  `thread.start(f: fn(*mut T) -> void, arg: *mut T) -> i64` ejecuta `f(arg)`
  en un hilo nuevo con su propio contexto, `thread.join(h)` lo espera y vuelve
  a lanzar su pánico, `thread.join_all(hs: []i64)` espera a todos antes de
  volver a lanzar el primer pánico entre ellos, y `thread.count() -> usize` es
  el número de hilos del hardware. `sync.mutex_new() -> i64`, `sync.lock(m)`,
  `sync.unlock(m)`, `sync.mutex_free(m)`, `sync.cond_new() -> i64`,
  `sync.wait(cv, m)`, `sync.wait_for(cv, m, ms) -> bool` (false cuando
  pasaron `ms`; por debajo de 0 espera para siempre), `sync.signal(cv)`,
  `sync.broadcast(cv)`, `sync.cond_free(cv)`. Una campana la toca cualquier
  hilo y la espera uno, y un toque anterior a la espera se guarda para ella:
  `sync.bell_new() -> i64`, `sync.bell_ring(b)`,
  `sync.bell_wait(b, ms) -> bool`, `sync.bell_free(b)`. Atómicos sobre un
  `i64`, secuencialmente consistentes: `sync.atomic_load(p: *i64) -> i64`,
  `sync.atomic_store(p: *mut i64, v)`, `sync.atomic_add(p, n) -> i64` y
  `sync.atomic_swap(p, v) -> i64` (ambos dan el valor anterior),
  `sync.atomic_cas(p, expected, new) -> bool`. Iniciar un hilo lleva
  `nondeterministic` y `shared_mutable`; esperar a un hilo, bloquear un mutex
  y esperar llevan `blocks`, y las llamadas `sync` son `shared_mutable`. No
  disponibles en el REPL.
- `os.arch() -> []u8`: la arquitectura en la que corre el programa
  (`x86_64`, `aarch64`, `x86`, `arm`, `riscv64` o `unknown`), decidida cuando
  se compiló el programa.
- `os.args() -> [][]u8`, `os.exe_path() -> String` (el ejecutable en marcha;
  vacío cuando la plataforma no lo dice), `os.env(name) -> ?[]u8`,
  `os.set_env(name, value)` (para este proceso y los que inicie; un valor
  vacío elimina la variable), `os.environ() -> List(String)` (cada
  `NAME=value`), `os.exit(code)`, `process.run(argv: [][]u8) -> !i32` (lanza,
  espera y devuelve el código de salida; `error.IoError` cuando el programa no
  puede iniciarse), `process.exec(argv, stdin, cwd) -> !i32` (lo mismo, con la
  entrada estándar tomada de `stdin`, ejecutado en `cwd` si no está vacío, y
  stdout/stderr capturados) seguido de `process.last_stdout()` /
  `process.last_stderr() -> String`; `std.process` los envuelve.
- Programas que corren a la par (`std.process` construye `Child` sobre estos;
  toda llamada salvo `child_pid` lleva `blocks`):
  `process.spawn(argv, cwd, flags) -> !i64` inicia uno (en `cwd`, o en el
  directorio de este programa si está vacío) con cada flujo estándar, dos bits
  de `flags` por flujo en el orden stdin, stdout, stderr, como el de este
  programa (0), una tubería (1), ninguna parte (2) o stderr hacia stdout (3).
  `process.child_write(h, bytes, timeout_ms) -> !void` escribe todo `bytes` a
  medida que el programa lo toma, `process.child_close_input(h)` cierra su
  entrada, `process.child_read(h, stream, n, timeout_ms) -> !String` es hasta
  `n` bytes del flujo 1 (stdout) o 2 (stderr), vacío al final,
  `process.child_wait(h, timeout_ms) -> !i64` es el código de salida en los
  32 bits bajos y la señal que lo terminó por encima de ellos,
  `process.child_signal(h, sig) -> !void` envía una (en Windows, que no tiene
  señales, toda señal salvo 0 termina el programa con el código de salida
  128 + `sig`), `process.child_pid(h) -> i64`, y `process.child_close(h)` lo
  suelta. Mientras una llamada espera, lo que el programa escribe se guarda
  para lecturas posteriores, así que nunca se atasca con una tubería llena. Un
  tiempo de espera por debajo de 0 espera para siempre, y 0 no espera.
  Errores: `NotFound` (no existe tal programa), `Timeout`, `IoError` (entre
  ellos un programa que ya no lee su entrada). `process.trap_signals()` evita
  que SIGINT, SIGTERM y SIGHUP terminen el programa (en Windows: Ctrl-C, 2;
  Ctrl-Break, 21; el cierre de la consola, 1; y el cierre de sesión o el
  apagado, 15), y `process.next_signal(timeout_ms) -> i32` toma la siguiente
  señal capturada, o 0 si no llegó ninguna a tiempo. No disponibles en el
  REPL.
- `time.now() -> i64` (ms desde la época), `time.monotonic() -> u64` (ns),
  `time.utc_offset(ms) -> i64` (minutos al este de UTC de la hora local en ese
  instante; `std.time` construye las fechas sobre estos), `time.sleep(ms)`.
- `time.zone_rules(name) -> !string`: una zona de la base de datos IANA como
  texto, para `std.time` en Windows, que guarda la base de datos en ICU en
  lugar de en archivos zoneinfo (Windows 10 1903 y posteriores; se carga en la
  primera llamada). La primera línea es el nombre de la zona, luego una línea
  por periodo, `start offset dst abbrev`; un nombre vacío es la zona del
  sistema. `NotFound` para una zona que no está en ICU, y en cualquier otra
  plataforma, donde `std.time` lee los archivos zoneinfo por su cuenta. Usa
  `time.zone(name)` de `std.time` en su lugar.
- `random.int(lo, hi)`, `random.float()`, `random.seed(n)`: un generador
  rápido que `random.seed` hace repetible, nunca para secretos.
- `random.secure(buf) -> !void`: llena un `[]mut u8` desde el generador
  seguro del sistema operativo (BCryptGenRandom, getrandom, arc4random), para
  claves, tokens y UUID; `random.seed` no lo toca, y solo falla con `IoError`
  donde el sistema no tiene generador. El REPL y `nx play` no lo ejecutan:
  necesita un programa compilado.
- `mem.copy(dst, src)`.
- `@typeName(T)`, `@sizeOf(T)`, `@alignOf(T)`, `@truncate(T, x)`,
  `@bitCast(T, x)` (mismo tamaño, escalares), `@min(a, b)`, `@max(a, b)`,
  `@errorName(e)`, `@embedFile(path)`, `@weak(x)`, `@refCount(x)`,
  `@cImport(header)`, `@cstr(literal)`, `@target()` (`(os, arch, bits)`, una
  constante de la compilación en C), `@escape(v)` (una copia de `v` hecha
  fuera del bloque `using arena` más interno).

Errores predefinidos: `OutOfMemory Panic InvalidRecord Truncated Overflow
InvalidUtf8 NotFound IoError InvalidInput BufferTooSmall`. Cualquier
`error.Name` crea uno nuevo.

## Puntos de entrada

`fn main()`, `fn main() -> !void` o `fn main() -> u8`. Un error que sale de
`main` imprime `error: Name` y sale con 1; un pánico imprime su ubicación y
sale con 101. Los bloques `test "name" { }` se ejecutan con `nx test`.

## Pruebas de rendimiento

`bench "name" { ... }` va junto a las pruebas y se ejecuta con `nx bench`,
que compila el archivo optimizado (modo `safe` salvo que `--mode` pida otro) y
mide cada bloque: calibra el número de iteraciones para una muestra de 10 ms
(la calibración sirve de calentamiento), toma 21 muestras e imprime la mediana
del tiempo por iteración, con la muestra más rápida y la más lenta:

```
bench  sum to 1000      187 ns/iter  (min 186 ns, max 193 ns; 21 samples of 64398)
bench  a list of 100    204 ns/iter  (min 202 ns, max 209 ns; 21 samples of 58241)
```

El valor de la última expresión del bloque se conserva, para que el trabajo
que lo produce no se elimine al optimizar: `bench "sum" { sum_to(1000) }` mide
la suma, mientras que `_ = sum_to(1000)` podría no medir nada. Un valor que es
una unión de error debe pasar por `try`, para que un error haga fallar la
prueba en lugar de medirse. Un pánico o un error la hacen fallar; las demás
siguen ejecutándose. Los argumentos tras el archivo limitan la ejecución a los
nombres que los contienen, y `--quick` toma muestras de 1 ms. `nx test` no
ejecuta las pruebas de rendimiento, y `nx check` las comprueba.

## Tipos recursivos y `match` a través de punteros

Una `List` puede contener el tipo que se está definiendo, así que los árboles
y los valores JSON son enums normales:
`enum Json { Null, Arr(List(Json)), Obj(List(Member)) }`. Un `match` a través
de un puntero liga por referencia las cargas propietarias:

```
fn push(v: *mut Json, own item: Json) {
    match v.* {
        .Arr(items) => items.append(item),   // items: *mut List(Json), un alias de la carga
        _ => {},
    }
}
```

Con `v: *Json` la ligadura es `*List(Json)`. Los escalares (`.Num(n)`) se
copian. Un `*String` o un `*List(T)` se convierte a `[]u8` o `[]T` donde se
espera un slice.

## Objetos de trait

`dyn Trait` es un puntero gordo hecho a partir de `*T` o `*mut T` donde `T`
implementa el trait: `let s: dyn Shape = &circle`, `List(dyn Shape)`,
`[]dyn Shape`. Las llamadas se despachan a través de una vtable y adquieren
todos los efectos que permite el tipo del objeto; `dyn Shape !allocates
!blocks` es un tipo distinto al que solo se convierten las implementaciones
que cumplen esas cotas. Un trait usado como objeto solo puede mencionar `Self`
en posición de receptor.

## Bucles paralelos

`for parallel x, i in items { ... }` ejecuta el cuerpo sobre el rango de
índices en un grupo de hilos (spec 7.2). El cuerpo no puede tener el efecto
`shared_mutable`, no puede hacer `return` ni `break` (usa `continue`), y
escribe los resultados a través de un slice mutable indexado por `i`. Un
pánico en un trabajador se vuelve a lanzar en el llamador cuando todos los
trabajadores terminan. El bucle lleva el efecto `blocks` (espera a los
trabajadores).

## Ámbitos de asignación

`using arena { ... }` instala un asignador bump para el bloque: los valores
creados dentro vienen de la arena, sus liberaciones no hacen nada y toda la
arena se libera cuando termina el bloque (spec 5.1, "reemplazable en cualquier
ámbito"). Los contenedores creados fuera del bloque siguen usando el montón
cuando crecen dentro, así que acumular resultados en una `List`, `String` o
`Map` exterior es seguro. Los valores creados dentro no deben escapar del
bloque.

## Llamar a C

```
const libc = @cImport("string.h")
const cv = @cImport("cvendor.h")               // se busca junto al archivo fuente
artifact link { c_sources = ["cvendor.c"], libs = [], include = [], libs_windows = ["gdi32"] }

unsafe {
    let n = libc.strlen(@cstr("hello"))
    let p = cv.cv_point{ .x = 1.0, .y = 2.0 }
    _ = cstdio.printf(@cstr("%d\n"), 42)      // los variádicos aceptan escalares y punteros
}
```

`@cImport` ejecuta el preprocesador de C e importa funciones, typedefs,
structs, enums y macros literales. `const T*` se vuelve `*T`, los demás
punteros `*mut T`, y `void*` se vuelve `*mut u8`. Las llamadas foráneas
requieren `unsafe` y llevan el efecto `ffi`. Un struct cuyos campos no se
pueden traducir (punteros a función, campos de bits, definiciones anidadas) se
importa como un tipo opaco, usable a través de punteros como una declaración
adelantada; así funciona `FILE` en cualquier libc. Las declaraciones que no se
pueden traducir en absoluto (uniones, typedefs de punteros a función, macros
con forma de función) se nombran en el error cuando se usan.

## Compilación condicional

`if comptime C { ... } else { ... }` evalúa `C` mientras se comprueba el
programa y compila solo la rama que elige. La otra nunca se comprueba, así que
puede llamar a lo que solo existe en otra plataforma:

```nexium
fn line_ending() -> []u8 {
    if comptime @target().0 == "windows" {
        return "\r\n"
    } else {
        return "\n"
    }
}
```

`@target()` es `(os, arch, pointer_bits)` de la compilación, el de `--target`
al compilar para otra plataforma. Un `if comptime` con valor tiene el tipo de
la rama tomada.

## Pruebas en tiempo de compilación

`comptime test "name" { ... }` se ejecuta en el intérprete durante la
comprobación; un fallo es un error de compilación que apunta a la expectativa.

## Vistas y su almacenamiento

Una función no puede devolver un slice o un puntero a uno de sus propios
locales (regla R1, un error desde 1.0); las vistas a parámetros prestados
están bien porque el llamador es su dueño. 1.2 añade las reglas de vistas V1
a V5 (`SPEC.md` 5.6 y 5.7): una vista guardada más allá del almacenamiento al
que apunta, una vista usada después de que su contenedor creció o su valor se
movió, un valor que contiene una vista devuelto, un valor conservado más allá
de su bloque `using arena`. Fueron avisos en 1.2 y son errores desde 1.3; cada
error nombra el arreglo, y `nx fix` hace los mecánicos (un `.clone()` donde el
valor se mueve, un `@escape`). `@escape(v)` copia un valor fuera de un bloque
de arena.

## Aún no implementado

`nx publish` y el registro. Las disposiciones `soa` y `packed`, las
estrategias de asignación `pool` y `stack`, y las reglas de regiones
archivadas R2 a R4 no forman parte del lenguaje (decisión 88); el compilador
rechaza esa sintaxis.
