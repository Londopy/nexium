# Cómo funciona Nexium

Este es el recorrido guiado por el compilador para quien quiera entenderlo o
cambiarlo. La referencia del lenguaje está en [`language.md`](language.md);
llamar a Nexium desde otros lenguajes está en
[`embedding.md`](../../embedding.md) (inglés); cada decisión tomada donde la
especificación quedaba abierta está en [`DECISIONS.md`](../../../DECISIONS.md)
(inglés).

## La versión de un párrafo

`nx` es un programa en Rust sin dependencias. Lee fuentes `.nx`, las
comprueba y escribe un único archivo C. Ese archivo C incluye
`runtime/nx_rt.h` (900 líneas de C llano incrustadas en el binario del
compilador) y se entrega a `zig cc`, que es Clang con una libc de compilación
cruzada empaquetada. No hay recolector de basura, ni máquina virtual, ni
biblioteca de runtime que instalar: la salida es un ejecutable nativo, una
biblioteca compartida o un archivo estático que solo depende de la biblioteca
C del destino.

```
 hello.nx ──lex──▶ tokens ──parse──▶ AST ──check──▶ IR tipada ──cgen──▶ hello.c ──zig cc──▶ hello.exe
                                            │
                                            └──comptime──▶ valores constantes, pruebas comptime
```

## La tubería, etapa por etapa

### 1. Lexer (`src/lexer.rs`)

Entran bytes, salen tokens. Dos decisiones aquí moldean todo lo que sigue:

- **Los saltos de línea son tokens.** Una sentencia termina en un salto de
  línea, así que el lexer emite `Newline` y colapsa las series. Dentro de `(`
  y `[` los saltos se suprimen, que es lo que permite que una lista de
  argumentos ocupe varias líneas. Dentro de `{ }` se conservan, porque las
  sentencias de un bloque se separan con ellos.
- **`<<` y `>>` son `LtLt` y `GtGt`, nunca operadores de desplazamiento.** El
  parser decide por el contexto si abren un patrón binario o desplazan bits.

Identificadores, palabras clave y nombres de integrados son todos `Ident`.
Las palabras clave las reconoce el parser. Un carácter no ASCII fuera de
cadenas y comentarios es un error.

`nx tokens file.nx` vuelca la secuencia en un formato fijo; es el oráculo
contra el que se comprueba el lexer autoalojado de `self/lexer.nx`.

### 2. Parser (`src/parser.rs`, `src/ast.rs`)

Descenso recursivo, una función por regla gramatical, que produce un
`ast::Module` de elementos: funciones, structs, enums, traits, impls,
constantes, globales, imports, pruebas, artefactos. Las condiciones van entre
sin paréntesis (`if c {`), los literales de struct son `Point{ .x = 1 }`, los
anónimos `.{ .x = 1 }`, y las capturas de las clausuras son explícitas
`|[x, &mut y] a: i32|`.

En el parser viven las reglas de continuación: una línea que empieza con
`|>`, `.method(`, `catch`, `orelse`, `and` u `or` se une a la anterior, igual
que una línea que termina en operador binario o paréntesis abierto.

### 3. Verificador (`src/check/`)

La parte más grande del compilador, unas 7 500 líneas. Convierte el AST en la
IR tipada de `src/tir.rs` y produce todos los diagnósticos. Las piezas:

**Los tipos se internan** (`src/types.rs`). Un `TyId` es un índice en una
tabla de `TyKind`; dos tipos son iguales exactamente cuando sus ids lo son.
Las variables de inferencia también son `TyKind`, así que `let x = 0` da a
`x` una variable entera que se resuelve con el primer uso que la fija, o por
defecto a `i64`.

**Los genéricos se monomorfizan.** `fn max(comptime T: type, a: T, b: T)` se
comprueba una vez por cada `T` distinto con el que se llama, produciendo una
instancia `TFunc` por combinación (`instantiate` en `check/mod.rs`). No hay
representación en tiempo de ejecución de un parámetro de tipo. Las cotas de
trait (`where T: Ord`) se comprueban al instanciar, y los bloques `impl` se
emparejan estructuralmente.

**La propiedad es un análisis de flujo por función.** Los valores `List`,
`String` y `Map` son propios: asignar o pasar uno por valor lo mueve, y el
verificador marca el local de origen como movido (`TFunc::moved`). Usarlo de
nuevo es el error `use after move`; mover fuera de un campo o un elemento se
rechaza porque el contenedor quedaría a medio poseer. Los parámetros se
prestan, así que un llamado no puede moverlos. Los valores `ref class` son
punteros con conteo de referencias y se copian libremente; el verificador
solo registra que necesitan `refcounts`.

**Los efectos se infieren por punto fijo** (`check/effects.rs`). Mientras
comprueba un cuerpo, el verificador registra `own_effects`, los efectos que
la función realiza directamente, cada uno con un *testigo*: la posición y una
frase que lo explica ("appending to a List may grow it"). También registra
cada llamado. Después

```
effects(f) = own(f) ∪ ⋃ effects(callee)      iterado hasta un punto fijo
```

sobre todas las instancias. Las funciones extern se asume que hacen de todo.
Una cota negativa como `!allocates` en una firma, o en un tipo función, se
comprueba tras la propagación; el diagnóstico recorre el grafo de llamadas
hasta la función que introdujo el efecto e imprime su testigo, así que el
error nombra la línea exacta al fondo de la cadena de llamadas. `panics` se
descarga donde el verificador puede demostrar que la operación no puede
fallar: indexar con el índice de un `for` sobre el mismo slice, índices
conocidos en tiempo de compilación, aritmética cuyos rangos de operandos
caben.

El retículo son ocho bits en `src/effects.rs`: `allocates`, `refcounts`,
`blocks`, `shared_mutable`, `nondeterministic`, `panics`, `ffi` y
`unbounded_stack` (reservado).

**Los patrones** (`check/pattern.rs`) compilan `match` a árboles de decisión
con comprobación de exhaustividad para enums y booleanos. **Los patrones
binarios** (`check/binpat.rs`) convierten `<<len:16/little, payload:len*8,
rest:bytes>>` en una secuencia de lecturas de bits comprobadas cuyos tamaños
pueden depender de ligaduras anteriores.

**Las vistas** se rastrean por sus orígenes (`self/check_views.nx`): cada local
sabe a qué almacenamiento apuntan las vistas que contiene, y las reglas V1 a
V5 de `SPEC.md` 5.6 y 5.7 se comprueban en el uso que leería almacenamiento
liberado, como errores desde 1.3 (avisos en 1.2). Devolver un slice
o puntero a un local de la función (regla R1) es un error desde 1.0.

**Los objetos de trait** reciben una vtable por par (trait, tipo), generada
como thunks que adaptan el receptor; `dyn Shape !allocates` es un tipo
distinto y cada implementación convertida a él debe satisfacer la cota.

**`@cImport`** (`src/cimport.rs`) ejecuta `zig cc -E` sobre la cabecera,
analiza las declaraciones resultantes con un pequeño parser de declaraciones
C e inyecta un módulo sintético. El C generado usa los nombres de tipo de la
propia cabecera, así que la cabecera sigue siendo la única fuente de verdad
para la disposición.

### 4. Evaluación en tiempo de compilación (`src/comptime.rs`)

Un intérprete sobre la IR tipada. `comptime expr`, los inicializadores
`const`, `@embedFile` y los bloques `comptime test` se ejecutan aquí durante
la comprobación. Permite cómputo puro, colecciones y llamadas a funciones
Nexium; prohíbe E/S, relojes, aleatoriedad, llamadas foráneas y globales; y
tiene un presupuesto de pasos para que una evaluación desbocada sea un error
de compilación y no un cuelgue. Un `comptime test` que falla se informa en su
línea `expect` como cualquier otro error.

### 5. Backend C (`src/cgen/`)

La IR tipada se baja a una única unidad de traducción C. Unas pocas
convenciones explican casi todo lo que se ve en la salida de `nx emit-c`:

- **Toda función recibe un `nx_ctx* c` oculto.** El contexto lleva el
  asignador, la arena actual, stdout/stderr, el estado del RNG, argv y los
  contadores de seguimiento de fugas. No hay globales en el runtime, que es
  lo que hace seguro cargar una biblioteca distribuida en un proceso
  anfitrión.
- **Los valores propios se liberan al salir del ámbito.** El backend mantiene
  una pila de ámbitos; cada `Let` de un tipo propio registra una liberación,
  y salir del ámbito (normalmente, por `return`, `break` o por un error)
  emite las liberaciones en orden inverso junto con los `defer`. Los
  temporales pasados a llamadas también se registran.
- **La aritmética comprobada es una llamada.** `x * 7` se convierte en
  `nx_mul_i64(x, 7, "hello.nx:3")`; la cadena de posición es lo que imprime
  un pánico. Las formas envolventes (`*%`) y saturantes (`*|`) se traducen a
  C llano o a ayudantes que recortan.
- **Los pánicos son `longjmp`.** Un `nx_boundary` local al hilo guarda un
  `jmp_buf`; `nx_panic` rellena el mensaje y la posición y salta a la
  frontera más cercana, que instaló el punto de entrada o un envoltorio de
  exportación.
- **`for parallel`** (`cgen/parallel.rs`) extrae el cuerpo a una función
  trabajadora que alcanza los locales circundantes a través de un struct de
  punteros, y el runtime reparte el rango de índices entre los hilos de
  hardware. El pánico de un trabajador se captura y se vuelve a lanzar en el
  llamador cuando todos los trabajadores terminan.
- **`using arena`** cambia un asignador bump en una copia del contexto para
  el bloque. Los contenedores recuerdan la arena en la que se crearon, así
  que una `List` exterior que crece dentro del bloque sigue viviendo en el
  montón.

Esto es `examples/hello.nx` tras el descenso:

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

El formato se compila, no se interpreta: `println("x = {}", .{x})` se
convierte en escrituras directas a un sumidero, una por marcador, con el tipo
del argumento conocido.

### 6. El runtime (`runtime/nx_rt.h`)

Una sola cabecera, incrustada en el compilador con `include_str!` y pegada al
principio de cada archivo generado. Sus secciones: slices, la interfaz del
asignador, pánicos, el asignador por defecto (malloc) con seguimiento de fugas
opcional, arenas, el grupo de hilos de parallel-for, listas, cadenas,
formato, mapas hash, conteo de referencias, ayudantes de patrones binarios,
aritmética comprobada y las partes de plataforma (E/S de archivos, tiempo,
lanzamiento de procesos) para Windows y POSIX.

El conteo de referencias es una cabecera de dos palabras (`rc`, `weak`)
delante de cada objeto `ref class`. `nx_retain` es un incremento en línea en
el runtime; la liberación la genera el backend por clase (`drop_fn` en
`cgen/mod.rs`), porque tiene que liberar los propios campos del objeto cuando
el contador llega a cero. Una referencia `weak` mantiene viva la asignación
pero no el objeto, y `upgrade()` falla cuando `rc` llega a cero.

Todo es `static inline`, así que el compilador de C ve el programa entero de
una vez y las funciones del runtime que no se usan no cuestan nada.

### 7. Driver y compilador de C (`src/main.rs`)

`nx build` escribe el archivo C en `nx-out/`, invoca `zig cc` con las
banderas del modo de compilación (`debug`, `safe`, `fast`, `small`), el
triple de destino y cualquier entrada de `artifact link`, y borra el C salvo
que se pase `--keep-c`. La misma ruta sirve a `run`, `test` (un ejecutor de
pruebas generado es el punto de entrada), `leaks` (añade `-DNX_LEAK_CHECK`) y
`size` (añade `-ffunction-sections` y lee de vuelta el objeto).

`zig cc` es el valor por defecto porque compila de forma cruzada sin más:
`--target aarch64-linux-gnu` desde una máquina Windows simplemente funciona.
`--cc clang` o `--cc gcc` se aceptan cuando no hace falta compilación
cruzada.

### 8. Distribución (`src/ship.rs`, `src/cgen/exports.rs`)

`nx ship` lee las declaraciones `artifact` y produce:

- **`cabi`**: una biblioteca compartida, un archivo estático y una cabecera
  C.
- **`python`**: un paquete basado en ctypes y una rueda.
- **`rustlib`**: un crate de Cargo con un script de compilación que enlaza el
  archivo, declaraciones `extern "C"`, structs `#[repr(C)]` y envoltorios
  seguros.
- **`cli`**: un ejecutable.

La frontera de exportación es donde los efectos rinden. Una función exportada
demostrada `!panics` y que no devuelve una unión de error recibe su firma C
natural. Cualquier otra exportación devuelve un estado `int32_t` y entrega su
valor por un parámetro de salida; el envoltorio instala una frontera de
pánico, así que un pánico de Nexium dentro de una biblioteca se convierte en
un código de error en el anfitrión en lugar de un aborto. Las globales
mutables se rechazan en cualquier programa que declare un artefacto
incrustable, para que dos anfitriones que carguen la misma biblioteca no
puedan interferir.

## Las herramientas

Todas son vistas sobre la misma IR tipada, y por eso coinciden con el
compilador:

| herramienta | qué lee |
| --- | --- |
| `nx effects` | `TFunc::effects` tras la propagación |
| `nx refcounts` | cada nodo `Retain`/`Release`/`Weak`/`Upgrade`, con su función |
| `nx audit` | bloques `unsafe` y globales |
| `nx doc` | comentarios de documentación, firmas y efectos, renderizados a HTML |
| `nx lsp` | diagnósticos de una comprobación completa en cada edición, hover desde `TFunc` |
| `nx size` | tamaños de sección del archivo objeto mapeados a declaraciones |
| `nx fmt` | solo la secuencia de tokens; nunca une ni parte líneas |

## Autoalojamiento

El compilador se está reescribiendo en Nexium bajo `self/`, una etapa a la
vez, cada etapa validada comparando su salida con la del compilador de Rust
sobre la misma entrada. El lexer está hecho y forma parte de `cargo test`. El
parser es el siguiente; su AST será una arena de ids, nodos en una `List` que
se refieren entre sí por índice (ver `examples/tree.nx`), lo que no necesita
tipos recursivos y se libera en una sola operación. Cuando existan las cuatro
etapas, el `nx` de Rust compila el `nx` de Nexium una vez, ese binario
compila de nuevo su propia fuente, y si las dos salidas coinciden byte a byte
el lenguaje se construye a sí mismo.

## Dónde mirar cuando algo falla

| síntoma | empieza por |
| --- | --- |
| un programa se analiza pero no debería, o al revés | `src/parser.rs`, luego `tests/compile_fail/` para el mensaje esperado |
| un error de tipo que parece incorrecto | `src/check/expr.rs` (expresiones) o `src/check/method.rs` (llamadas a métodos e integrados) |
| un efecto que debería o no debería estar | el testigo en `own_effects`; busca `add_effect` en `src/check/` |
| una fuga en `nx leaks` | la pila de ámbitos en `src/cgen/expr.rs`; todo temporal propio debe registrarse |
| C generado que no compila | `nx emit-c file.nx --keep-c` y lee `nx-out/file.c`; el ayudante del runtime que llama está en `runtime/nx_rt.h` |
| un fallo dentro de `for parallel` o `using arena` | `src/cgen/parallel.rs` y la sección de arena del runtime |
