# Referencia del lenguaje Nexium

Describe lo que el compilador `nx` implementa hoy. Los números de sección se
refieren a `nexium-spec.txt` (el diseño) y, donde se marca "archivado", a las
secciones 4 a 9 de `nexium-systems-spec.txt` (la referencia de sintaxis).

## Archivos y módulos

Un archivo es un módulo. `import foo.bar` carga `foo/bar.nx` junto al archivo
raíz; sus elementos `pub` se alcanzan como `bar.item`. `import std.math` (o
cualquier módulo std) se acepta pero no es necesario: los espacios de nombres
integrados `math`, `io`, `os`, `time`, `random` y `mem` siempre están en
ámbito.

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
  anterior, igual que una línea que termina con un operador binario o un
  paréntesis abierto.
- Convenciones: tipos `PascalCase`, funciones y variables `snake_case`,
  constantes `SCREAMING_SNAKE_CASE`.

## Declaraciones

```
fn name(a: T, b: U) -> R effects { ... }         // efectos: p. ej. !allocates !panics
pub fn f(x: i32) -> i32 export(c) { ... }        // exportada con la ABI de C
extern fn puts(s: *u8) -> i32                    // foránea; llamarla requiere `unsafe`
struct Point derive(Eq, Ord) { x: f64, y: f64 }
struct Frame layout(c) { id: u32, len: u16 }     // disposición C, usable a través de la frontera
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
artifact cabi { name = "lib", exports = [f] }
```

Los campos dentro de `{ }` se separan con comas o saltos de línea. Un campo
puede tener un valor por defecto (`verbose: u8 = 0`).

## Tipos (archivado 5)

| sintaxis | significado |
| --- | --- |
| `i8 … i128`, `u8 … u128`, `isize`, `usize` | enteros; sin conversión implícita entre anchos |
| `f32`, `f64`, `bool`, `char`, `void`, `never` | primitivos |
| `[N]T` | array, `N` constante en tiempo de compilación |
| `[]T`, `[]mut T` | slice: puntero más longitud; `[]u8` es texto |
| `*T`, `*mut T` | puntero a un valor (`&x`, `&mut x`, `p.*`) |
| `?T` | opcional; `null` es el valor vacío |
| `!T`, `Set!T` | unión de error |
| `List(T)`, `String`, `Map(K, V)` | colecciones propietarias (valores, sección 5.3) |
| `fn(A, B) -> R !effects` | valor función (las clausuras y funciones se convierten a él) |
| `(A, B)` | tupla; campos `.0`, `.1` |
| `weak T` | referencia débil a una `ref class` |

Los literales enteros toman el tipo que pide el contexto y por defecto son
`i64`; los flotantes por defecto `f64`. Un literal de carácter cabe en
cualquier tipo entero que pueda contenerlo (`c == 'a'` con `c: u8`).

Conversiones: `x as T` entre números (el estrechamiento se comprueba en
tiempo de ejecución salvo que el rango esté demostrado), entre un tipo
`distinct` y su representación, entre `char` y enteros, de `bool` a entero,
de un enum unitario a entero. `@truncate(T, x)` envuelve.

## Valores, ligaduras, propiedad

`let x = e` liga de forma inmutable, `var x = e` de forma mutable. La
mutabilidad es superficial (archivado 5.10): un `let` que contiene `*mut T`
sigue mutando a través de él.

Las colecciones poseen un búfer en el montón (5.3). `let b = a` mueve `a`;
usar `a` después es un error de compilación; `a.clone()` copia. Los
parámetros se prestan: una función que recibe una `List` la lee; para mutar,
toma `*mut List(T)`. `List(T)` y `String` se convierten a `[]T` / `[]u8`
cuando se pasan donde se espera un slice. Mover fuera de un campo o de un
elemento es un error. Los valores propios se liberan cuando su ámbito
termina; esa es la única acción automática al salir del ámbito (H7).

Los valores `ref class` son referencias; copiar uno lo retiene y el objeto se
libera cuando la última referencia desaparece (5.1). Los ciclos fugan; usa
`weak` para las aristas de vuelta (`@weak(x)` o `x.weak()`, luego
`w.upgrade()`).

## Expresiones

- La aritmética `+ - * / %` atrapa el desbordamiento (un pánico definido).
  Las formas envolventes `+% -% *%` y saturantes `+| -| *|` nunca fallan.
  Bit a bit `& | ^ ~ << >>`. Comparación `== != < <= > >=` sobre números,
  caracteres, booleanos, `[]u8`, `String`, enums unitarios y tipos que
  `derive(Eq)` / `derive(Ord)`. Lógicos `and`, `or`, `!`.
- `x |> f(a)` es `f(x, a)`.
- `if c { a } else { b }` es una expresión; `if let v = opt { } else { }` desenvuelve.
  Las condiciones no llevan paréntesis y los cuerpos siempre llevan llaves.
- `match v { pat => expr, ... }` sobre enteros (literales, rangos `1..=9`),
  cadenas, booleanos, caracteres, enums (`.Variant(p)`), opcionales (`null`,
  ligadura), uniones de error (`error.Name`, ligadura), tuplas y slices de
  bytes (patrones binarios). Los `match` sobre enums y booleanos deben ser
  exhaustivos; los demás necesitan `_ =>`.
- Los bloques son expresiones cuyo valor es la expresión final. Un bloque
  etiquetado devuelve mediante `break :label value`.
- `try e` propaga un error; `e catch |err| handler`; `opt orelse default`;
  `opt.?` desenvuelve (pánico si es null).
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
for x in items { }               // arrays, slices, listas, cadenas, claves de mapa
for x, i in items { }            // con índice
for x, y in a, b { }             // en paralelo; las longitudes deben coincidir
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

Los segmentos son `name:size/modifiers` o literales. Los tamaños son bits
(por defecto 8); `bytes` consume el resto. Modificadores: `big` (por
defecto), `little`, `native`, `signed`, `unsigned`, `float`, `utf8`. Una
ligadura con tamaño constante de hasta 64 bits es un entero del ancho mínimo
que quepa; los tamaños mayores o dinámicos ligan una vista `[]u8`. La
aritmética de tamaños se hace al ancho de puntero y siempre se comprueba. La
construcción apunta a un búfer `[]mut u8` y devuelve `![]u8` (el prefijo
escrito), fallando con `error.BufferTooSmall`.

## Efectos (sección 8)

`allocates refcounts blocks shared_mutable nondeterministic panics ffi`

Los efectos se infieren para cada función. Una cota negativa en una firma
(`!allocates`) se comprueba; el diagnóstico señala el sitio que introdujo el
efecto, a través de las llamadas. Los tipos función pueden llevar cotas
negativas (`fn(i32) -> i32 !allocates`); una clausura o función convertida a
tal tipo debe satisfacerlas. `panics` se descarga por demostración (E6):
indexar con un índice de bucle sobre el mismo slice, índices conocidos en
tiempo de compilación, aritmética cuyos rangos de operandos caben y locales
protegidos no contribuyen. `nx effects file.nx` imprime el conjunto inferido
por función.

## Biblioteca estándar (integrados)

- `println(fmt, .{args})`, `print`, `eprintln`, `format(...) -> String`.
  Marcadores: `{}`, `{x}`, `{X}`, `{b}`, `{o}`, `{e}`, `{c}`, `{:.N}`,
  `{>N}`, `{<N}`; `{{` y `}}` son llaves literales.
- `expect(cond)`, `expect_eq(a, b)`, `panic(msg)`.
- `List(T)`: `new`, `with_capacity`, `from`, `append`, `pop`, `clear`, `clone`,
  `last`, `first`, `insert`, `remove`, `swap_remove`, `extend`, `reserve`,
  `items`, `is_empty`, `len`, más los métodos de slice.
- `String`: `new`, `from`, `with_capacity`, `append`, `append_char`, `clone`,
  `clear`, `pop`, `bytes`, `len`, más los métodos de `[]u8`.
- `Map(K, V)` (claves: enteros, bool, char, `[]u8`, `String`): `new`, `put`,
  `get`, `contains`, `remove`, `clear`, `clone`, `keys`, `values`, `len`,
  `m[key]`; `for k in m` itera las claves.
- Slices: `len`, `fill`, `reverse`, `sort`, `contains`, `index_of`,
  `copy_from`, `to_owned`, `is_empty`; `[]u8` además `starts_with`,
  `ends_with`, `find`, `trim`, `split`, `lines`, `to_string`, `parse_int(T)`,
  `parse_float`, `eq_ignore_case`.
- Enteros: `abs`, `min`, `max`, `checked_add/sub/mul` (devuelven `?T`),
  `to_string`. Flotantes: `abs`, `sqrt`, `floor`, `ceil`, `round`, `min`,
  `max`, `pow`, `to_string`. Caracteres: `is_digit`, `is_alpha`, `is_space`,
  `to_lower`, `to_upper`, `to_digit`.
- `math`: `PI E TAU INF NAN`, `sqrt abs floor ceil round sin cos tan exp log
  log2 min max pow atan2 clamp`.
- `io.read_file(path) -> !String`, `io.write_file(path, bytes) -> !void`,
  `io.read_line() -> ?String`.
- `os.args() -> [][]u8`, `os.env(name) -> ?[]u8`, `os.exit(code)`,
  `process.run(argv: [][]u8) -> !i32` (lanza el proceso, espera y devuelve el
  código de salida; `error.IoError` cuando no se puede iniciar el programa).
- `time.now() -> i64` (ms desde la época), `time.monotonic() -> u64` (ns),
  `time.sleep(ms)`.
- `random.int(lo, hi)`, `random.float()`, `random.seed(n)`.
- `mem.copy(dst, src)`.
- `@typeName(T)`, `@sizeOf(T)`, `@truncate(T, x)`, `@errorName(e)`,
  `@embedFile(path)`, `@weak(x)`, `@refCount(x)`, `@cImport(header)`,
  `@cstr(literal)`.

Errores predefinidos: `OutOfMemory Panic InvalidRecord Truncated Overflow
InvalidUtf8 NotFound IoError InvalidInput BufferTooSmall`. Cualquier
`error.Name` crea uno nuevo.

## Puntos de entrada

`fn main()`, `fn main() -> !void` o `fn main() -> u8`. Un error desde `main`
imprime `error: Name` y sale con 1; un pánico imprime su ubicación y sale con
101. Los bloques `test "name" { }` se ejecutan con `nx test`.

## Objetos de trait

`dyn Trait` es un puntero gordo hecho a partir de `*T` o `*mut T` donde `T`
implementa el trait: `let s: dyn Shape = &circle`, `List(dyn Shape)`,
`[]dyn Shape`. Las llamadas se despachan a través de una vtable y adquieren
todos los efectos que el tipo del objeto permite; `dyn Shape !allocates
!blocks` es un tipo distinto al que solo se convierten las implementaciones
que satisfacen esas cotas. Un trait usado como objeto solo puede mencionar
`Self` en posición de receptor.

## Bucles paralelos

`for parallel x, i in items { ... }` ejecuta el cuerpo sobre el rango de
índices en un grupo de hilos (spec 7.2). El cuerpo no puede tener el efecto
`shared_mutable`, no puede hacer `return` ni `break` (usa `continue`), y
escribe resultados a través de un slice mutable indexado por `i`. Un pánico
en un trabajador se vuelve a lanzar en el llamador cuando todos los
trabajadores terminan. El bucle lleva el efecto `blocks` (espera).

## Ámbitos de asignación

`using arena { ... }` instala un asignador bump para el bloque: los valores
creados dentro vienen de la arena, sus liberaciones no hacen nada y toda la
arena se libera al terminar el bloque (spec 5.1, "reemplazable en cualquier
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
structs, enums y macros literales. `const T*` se vuelve `*T`, otros punteros
`*mut T`, `void*` se vuelve `*mut u8`. Las llamadas foráneas requieren
`unsafe` y llevan el efecto `ffi`. Las declaraciones que no se pueden
traducir (uniones, campos de bits, punteros a función, macros con forma de
función) se nombran en el error cuando se usan.

## Pruebas en tiempo de compilación

`comptime test "name" { ... }` se ejecuta en el intérprete durante la
comprobación; un fallo es un error de compilación que apunta a la
expectativa.

## Vistas y su almacenamiento

Una función no puede devolver un slice o puntero a uno de sus propios locales
(regla R1, un error desde 1.0); las vistas a parámetros prestados están bien
porque el llamador es su dueño. 1.2 añade las reglas de vistas V1 a V5
(`SPEC.md` 5.6 y 5.7): una vista guardada más allá de su almacenamiento, una
vista usada después de que su contenedor creció o su valor se movió, un valor
que contiene una vista devuelto, un valor conservado más allá de su bloque
`using arena`. Fueron avisos en 1.2 y son errores desde 1.3; cada error nombra
el arreglo, y `nx fix` hace los mecánicos. `@escape(v)` copia un valor fuera
de un bloque de arena.

## Aún no implementado

Disposiciones `soa` y `packed`, artefactos `node` e `installer`, `nx publish`
y el registro, estrategias de asignación `pool`/`stack`, y las reglas de
regiones archivadas R2 a R4 (decisión 88).
