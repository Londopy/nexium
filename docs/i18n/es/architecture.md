# Cómo funciona Nexium

Este es el recorrido guiado por el compilador para quien quiera entenderlo o
cambiarlo. La referencia del lenguaje está en [`language.md`](language.md);
llamar a Nexium desde otros lenguajes, en [`embedding.md`](../../embedding.md)
(inglés); cada decisión tomada donde la especificación quedaba abierta, en
[`DECISIONS.md`](../../../DECISIONS.md) (inglés).

## Un compilador escrito en sí mismo

El compilador está escrito en Nexium, bajo `self/`: `lexer.nx`, `parser.nx`,
el comprobador (`check.nx` y los módulos `check_*.nx` que tiene al lado, con
`cimport.nx`), `cgen.nx` y el controlador `nx.nx`, cada uno una de las etapas
de abajo, y después las herramientas (`fmt.nx`, `doc.nx`, `tools.nx`,
`size.nx`, `manifest.nx`, `ship.nx`, `ship_node.nx`, `installer.nx`,
`lsp.nx` con `lsp_index.nx`, `repl.nx`, `topo.nx`, `completions.nx`,
`fix.nx`). Se compila a sí mismo a partir de la semilla en C de `bootstrap/`
(decisión 90): el primer compilador, en Rust, sirvió para la migración y se
retiró en la 1.0.

## La versión en un párrafo

`nx` lee código `.nx`, lo comprueba y escribe un único archivo C. Ese archivo
incluye `runtime/nx_rt.h` (unas 4200 líneas de C sencillo incrustadas en el
binario del compilador) y se entrega a `zig cc`, que es Clang con una libc de
compilación cruzada incluida. No hay recolector de basura, ni máquina
virtual, ni biblioteca de runtime que instalar: la salida es un ejecutable
nativo, una biblioteca compartida o un archivo estático que solo depende de la
biblioteca C del destino.

```
 hello.nx ──lex──▶ tokens ──parse──▶ AST ──check──▶ IR tipado ──cgen──▶ hello.c ──zig cc──▶ hello.exe
                                            │
                                            └──comptime──▶ valores constantes, pruebas comptime
```

## La tubería, etapa por etapa

### 1. Analizador léxico (`self/lexer.nx`)

Entran bytes, salen tokens. Dos decisiones tomadas aquí dan forma a todo lo
que viene después:

- **Los saltos de línea son tokens.** Una sentencia termina en un salto de
  línea, así que el analizador léxico emite `Newline` y colapsa las rachas
  seguidas. Dentro de `(` y `[` los saltos de línea se suprimen, y eso es lo
  que permite que una lista de argumentos ocupe varias líneas. Dentro de
  `{ }` se conservan, porque separan las sentencias de un bloque.
- **`<<` y `>>` son `LtLt` y `GtGt`, nunca operadores de desplazamiento.** El
  analizador sintáctico decide por el contexto si abren un patrón binario o
  desplazan bits.

Los identificadores, las palabras clave y los nombres de los integrados son
todos `Ident`. Las palabras clave las reconoce el analizador sintáctico
(`KEYWORDS` es la lista). Un carácter no ASCII fuera de cadenas y comentarios
es un error. Si se le pide, el analizador léxico conserva los comentarios `//`
como tokens; el formateador los necesita y el analizador sintáctico nunca los
ve.

### 2. Analizador sintáctico (`self/parser.nx`)

Descenso recursivo, una función por regla de la gramática, que produce un
`Tree`: una arena de ids de `Node` (clase, hijos, un nombre, un texto, tres
ranuras `x`, `y`, `z`) con los elementos del módulo encima: funciones,
structs, enums, traits, impls, constantes, globales, imports, pruebas y
artefactos. Los comentarios de documentación se guardan por índice en
`Tree.docs`. Las condiciones van sin paréntesis (`if c {`; un literal de
struct ahí necesita paréntesis), los literales de struct son
`Point{ .x = 1 }`, los anónimos `.{ .x = 1 }`, y las capturas de las
clausuras son explícitas: `|[x, &mut y] a: i32|`.

El analizador sintáctico es donde viven las reglas de continuación: una línea
que empieza con `|>`, `.method(`, `catch`, `orelse`, `and` u `or` se une a la
anterior, igual que una línea que termina en un operador binario o en un
paréntesis, corchete o llave sin cerrar.

### 3. Comprobador (`self/check.nx` y `self/check_*.nx`)

La parte más grande del compilador, unas 16 000 líneas. Convierte el árbol
sintáctico en el IR tipado (`Tir`, otra arena de ids, con un `TKind` por nodo)
y produce todos los diagnósticos. Su estado es un único struct, `Checker`, en
`check.nx`, junto con el núcleo: los tipos, las definiciones, la carga, las
instancias y las pasadas sobre el programa entero, el entorno genérico, la
propiedad, la unificación y los rangos. El resto de sus métodos está en
módulos propios, cada uno un bloque `impl Checker` (decisión 110), y
`check.nx` los importa todos, que es lo que los hace parte del programa:

| módulo | qué comprueba |
| --- | --- |
| `check_stmts.nx` | cuerpos, bloques, sentencias, bucles |
| `check_views.nx` | las vistas y sus orígenes, las reglas V1 a V5 |
| `check_exprs.nx` | expresiones, llamadas a funciones, `if`, conversiones |
| `check_fields.nx` | campos, literales, indexación, `try`, `catch`, `orelse` |
| `check_calls.nx` | métodos, miembros estáticos, los espacios de nombres integrados |
| `check_match.nx` | `match`, patrones, exhaustividad |
| `check_builtins.nx` | los integrados `@`, la disposición en memoria |
| `check_closures.nx` | clausuras |
| `check_print.nx` | el IR tipado como texto, para `nx tir` |
| `check_interp.nx` | el intérprete en tiempo de compilación (sección 4) |

Las piezas:

**Los tipos están internados** (`Types`). Un tipo es un índice en una tabla de
`Ty`; dos tipos son iguales exactamente cuando sus índices lo son. Las
variables de inferencia también son entradas de la tabla, así que `let x = 0`
da a `x` una variable entera que se resuelve con el primer uso que la fija, o
que por defecto es `i64`.

**Los genéricos se monomorfizan.** `fn max(comptime T: type, a: T, b: T)` se
comprueba una vez por cada `T` distinto con el que se llama, lo que produce un
`Inst` por combinación (`instantiate`). Un parámetro de tipo no tiene
representación en tiempo de ejecución. Las cotas de trait (`where T: Ord`) se
comprueban al instanciar, y los bloques `impl` se emparejan estructuralmente.

**La propiedad es un análisis de flujo por función.** Los valores `List`,
`String` y `Map` son propios: asignar o pasar uno por valor lo mueve, y el
comprobador marca como movido el local de origen (`FnCtx.moved`, del que se
toma una instantánea por rama y que luego se fusiona). Volver a usarlo es el
error `use after move`; mover fuera de un campo, un elemento, una variable de
bucle o una ligadura `if let` sobre un lugar se rechaza porque el contenedor
quedaría a medio poseer. Los parámetros se prestan, así que la función llamada
no puede moverlos. Los valores `ref class` son punteros con un contador de
referencias y se copian libremente; el comprobador solo registra que
necesitan `refcounts`.

**Los efectos se infieren por punto fijo** (`propagate_effects`). Mientras
comprueba un cuerpo, el comprobador registra `own_effects`, los efectos que la
función realiza directamente, cada uno con un *testigo*: la posición y una
frase que lo explica ("appending to a List may grow it"). También registra
cada función llamada. Después

```
effects(f) = own(f) ∪ ⋃ effects(callee)      iterado hasta un punto fijo
```

sobre todas las instancias. Se supone que las funciones extern lo hacen todo.
Una cota negativa como `!allocates` en una firma, o en un tipo función, se
comprueba después de la propagación; el diagnóstico recorre el grafo de
llamadas hasta la función que introdujo el efecto e imprime su testigo, de
modo que el error nombra la línea exacta al fondo de la cadena de llamadas.
`panics` se descarga donde el comprobador puede demostrar que la operación no
puede fallar: indexar con el índice de un `for` sobre el mismo slice, índices
conocidos en tiempo de compilación, aritmética cuyos rangos de operandos
caben.

El retículo tiene ocho bits (`EFF_*`): `allocates`, `refcounts`, `blocks`,
`shared_mutable`, `nondeterministic`, `panics`, `ffi` y `unbounded_stack`,
este último a partir del grafo de llamadas: lo lleva una función que está en
un ciclo (las componentes de Tarjan en `mark_recursion`) o que llama a través
de un valor función o de un `dyn`.

**Los patrones** (`check_match`) compilan `match` a árboles de decisión, con
comprobación de exhaustividad para enums y booleanos. **Los patrones
binarios** convierten `<<len:16/little, payload:len*8, rest:bytes>>` en una
secuencia de lecturas de bits comprobadas cuyos tamaños pueden depender de
ligaduras anteriores.

**Las vistas** se siguen por sus orígenes (`self/check_views.nx`): cada local
sabe a qué almacenamiento apuntan las vistas que contiene y cuándo se
tomaron, y las reglas V1 a V5 de `SPEC.md` 5.6 y 5.7 se comprueban en el uso
que leería almacenamiento liberado, como errores desde 1.3 (avisos en 1.2).
Donde el arreglo es mecánico, el comprobador ofrece a `nx fix` una edición
(`check.Fix`: el `.clone()` de V4 en el movimiento, el `@escape` de V5,
`_ = ` delante de un valor sin usar), que `self/fix.nx` conserva solo cuando
comprobar el programa editado deja menos errores y ninguno nuevo. Devolver un
slice o un puntero a un local de la función (regla R1) es un error desde 1.0.

**Los objetos de trait** obtienen una vtable por cada par (trait, tipo),
generada como thunks que adaptan el receptor; `dyn Shape !allocates` es un
tipo distinto y toda implementación convertida a él debe cumplir la cota.

**`@cImport`** (`self/cimport.nx`) ejecuta `zig cc -E` sobre la cabecera,
analiza las declaraciones resultantes con un pequeño analizador de
declaraciones de C e inyecta un módulo sintético. El C generado usa los
nombres de tipo de la propia cabecera, así que la cabecera sigue siendo la
única fuente de verdad sobre la disposición en memoria.

### 4. Evaluación en tiempo de compilación (`self/check_interp.nx`)

Un intérprete sobre el IR tipado, con los valores en `CV`. Sus funciones
`it_*` son métodos del comprobador declarados en un módulo propio (decisión
110), así que el comprobador las llama como suyas y ellas leen sus tablas
directamente. `comptime expr`, los inicializadores de `const`, `@embedFile` y
los bloques `comptime test` se ejecutan aquí durante la comprobación. Permite
el cálculo puro, las colecciones y las llamadas a funciones Nexium; prohíbe
la E/S, los relojes, la aleatoriedad, las llamadas foráneas y los globales; y
tiene un presupuesto de pasos para que una evaluación desbocada sea un error
de compilación en lugar de un cuelgue. Un `comptime test` que falla se
informa en su línea `expect` como cualquier otro error. El mismo intérprete
ejecuta `nx repl` (`self/repl.nx`), donde `repl_mode` le deja hablar con el
mundo, y `nx play`, el ejecutor del playground, donde `play_mode` rechaza lo
que una página no puede hacer.

### 5. Backend de C (`self/cgen.nx`)

El IR tipado se baja a una única unidad de traducción de C. Unas pocas
convenciones explican casi todo lo que se ve en la salida de `nx emit-c`:

- **Cada función recibe un `nx_ctx* c` oculto.** El contexto lleva el
  asignador, la arena actual, stdout/stderr, el estado del generador de
  números aleatorios, argv y los contadores de seguimiento de fugas. No hay
  globales en el runtime, y eso es lo que hace seguro cargar una biblioteca
  distribuida en un proceso anfitrión.
- **Los valores propios se liberan al salir del ámbito.** El backend mantiene
  una pila de ámbitos; cada `Let` de un tipo propio registra una liberación,
  y al dejar el ámbito (normalmente, por `return`, por `break` o por un error)
  emite las liberaciones en orden inverso junto con los `defer`. También se
  registran los temporales pasados a llamadas.
- **La aritmética comprobada es una llamada.** `x * 7` se convierte en
  `nx_mul_i64(x, 7, "hello.nx:3")`; la cadena de ubicación es lo que imprime
  un pánico. Las formas envolventes (`*%`) y saturantes (`*|`) corresponden a
  C simple o a funciones auxiliares que recortan el valor.
- **Los pánicos son `longjmp`.** Un `nx_boundary` local al hilo guarda un
  `jmp_buf`; `nx_panic` rellena el mensaje y la ubicación y salta a la
  frontera más cercana, que instaló el punto de entrada o un envoltorio de
  exportación.
- **`for parallel`** (`parallel_for`) extrae el cuerpo a una función
  trabajadora que llega a los locales circundantes a través de un struct de
  punteros, y el runtime reparte el rango de índices entre los hilos del
  hardware. El pánico de un trabajador se captura y se vuelve a lanzar en el
  llamador cuando todos los trabajadores terminan.
- **`using arena`** coloca un asignador bump en una copia del contexto
  durante el bloque. Los contenedores recuerdan la arena en la que se
  crearon, así que una `List` exterior que crece dentro del bloque sigue
  viviendo en el montón.

Esto es `examples/hello.nx` después de la bajada:

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

Una sola cabecera, incrustada en el compilador con `@embedFile` y pegada al
principio de cada archivo generado. Sus secciones: slices, la interfaz del
asignador, pánicos, el asignador por defecto (malloc) con seguimiento de
fugas opcional, arenas, el grupo de hilos de parallel-for, listas, cadenas,
formato, tablas hash, conteo de referencias, auxiliares de patrones binarios,
aritmética comprobada y las partes de la plataforma para Windows y POSIX: el
sistema de archivos y los manejadores de archivo, el tiempo, el lanzamiento de
programas y los procesos hijos, los sockets, TLS mediante la biblioteca propia
del sistema, los hilos y la entrada cruda del terminal.

El conteo de referencias es una cabecera de dos palabras (`rc`, `weak`)
delante de cada objeto `ref class`. `nx_retain` es un incremento en línea en
el runtime; la liberación la genera el backend para cada clase (`drop_fn` en
`cgen.nx`), porque tiene que liberar los campos del propio objeto cuando la
cuenta llega a cero. Una referencia `weak` mantiene viva la asignación pero no
el objeto, y `upgrade()` falla en cuanto `rc` llega a cero.

Todo es `static inline`, así que el compilador de C ve el programa entero de
una vez y las funciones del runtime que no se usan no cuestan nada.

### 7. Controlador y compilador de C (`self/nx.nx`)

`nx build` escribe el archivo C en `nx-out/`, invoca `zig cc` con las opciones
del modo de compilación (`debug`, `safe`, `fast`, `small`), el triplete de
destino y las entradas de `artifact link`, y borra el C salvo que se pase
`--keep-c`. El mismo camino sirve a `run`, `test` (un ejecutor de pruebas
generado es el punto de entrada), `leaks` (añade `-DNX_LEAK_CHECK`) y `size`
(añade `-ffunction-sections` y vuelve a leer el objeto).

Una compilación de depuración de un programa grande (unos 200 KB de código
fuente; `NX_UNITS` lo decide para cualquiera) se divide en su lugar (decisión
116): `cgen.generate_units` escribe un archivo C por módulo en
`nx-out/<name>.units/`, cada objeto nombrado por un hash de su C y de su línea
de órdenes, y el controlador compila solo los archivos cuyo objeto falta, en
varios hilos, y luego los enlaza. El estado del runtime (`NX_STATE`) lo define
el archivo del módulo raíz y lo declaran los demás.

`zig cc` es el predeterminado porque hace compilación cruzada sin más:
`--target aarch64-linux-gnu` desde una máquina Windows simplemente funciona.
`--cc clang` o `--cc gcc` se aceptan cuando no hace falta compilación
cruzada.

### 8. Distribución (`self/ship.nx`, `ship_node.nx`, `installer.nx`; `export_wrapper` en `cgen.nx`)

`nx ship` lee las declaraciones `artifact` y produce:

- **`cabi`**: una biblioteca compartida, un archivo estático y una cabecera
  de C.
- **`python`**: un paquete basado en ctypes y un wheel.
- **`rustlib`**: un crate de Cargo con un script de compilación que enlaza el
  archivo estático, declaraciones `extern "C"`, structs `#[repr(C)]` y
  envoltorios seguros.
- **`node`**: un paquete npm sobre la biblioteca compartida.
- **`cli`**: un ejecutable, e **`installer`**: un script de Inno Setup o un
  script de instalación con los archivos del programa.

La frontera de exportación es donde los efectos rinden. Una función exportada
de la que se ha demostrado `!panics` y que no devuelve una unión de error
obtiene su firma natural de C. Cualquier otra exportación devuelve un estado
`int32_t` y entrega su valor a través de un parámetro de salida; el
envoltorio instala una frontera de pánico, así que un pánico de Nexium dentro
de una biblioteca se convierte en un código de error en el anfitrión en lugar
de un aborto. Los globales mutables se rechazan en cualquier programa que
declare un artefacto incrustable, de modo que dos anfitriones que carguen la
misma biblioteca no pueden interferir entre sí.

## Las herramientas

Todas son vistas sobre el mismo IR tipado, y por eso coinciden con el
compilador:

| herramienta | qué lee |
| --- | --- |
| `nx effects` | los `effects` de cada instancia después de la propagación |
| `nx refcounts` | cada nodo `Retain`/`Release`/`Weak`/`Upgrade`, con su función |
| `nx audit` | bloques `unsafe` y globales |
| `nx doc` | comentarios de documentación, firmas y efectos, en HTML |
| `nx lsp` | diagnósticos de una comprobación completa en cada edición; lentes de código (la demostración de pánico a partir del testigo registrado) de las instancias comprobadas; cuando el programa pasa la comprobación, ir a la definición, renombrar y la información al pasar el cursor salen de a qué resolvió el comprobador cada nombre (`self/lsp_index.nx`: locales, funciones y métodos, campos), y renombrar edita todos los archivos del programa; si no, y para el autocompletado, los tipos y las constantes, salen del flujo de tokens y de los módulos analizados (`self/lsp.nx`), así que responden aunque el código tenga errores |
| `nx repl` | el intérprete, línea a línea, sobre un programa que se vuelve a comprobar entero |
| `nx size` | los tamaños de las secciones del archivo objeto, asignados de vuelta a las declaraciones |
| `nx layout` | `size_of`/`align_of` del comprobador recorridos campo a campo: desplazamientos, relleno, el total y el reordenamiento por alineación que encogería un struct |
| `nx fmt` | solo el flujo de tokens; nunca une ni divide líneas |

## Autoalojamiento

El compilador está escrito en Nexium bajo `self/`, un archivo por etapa, y se
compila a sí mismo: `bootstrap/nx.c` es el C que emite para sí mismo,
cualquier compilador de C lo convierte en `nx0`, `nx0` compila `self/nx.nx`
en `nx1`, y `nx1` debe volver a emitir el mismo C (`nx2`), que es el
compilador que se prueba y el que distribuyen las versiones
(`bootstrap/build.sh`, `bootstrap/README.md`). El árbol sintáctico y el IR
tipado son arenas de ids, nodos en una `List` que se refieren unos a otros
por índice (véase `examples/tree.nx`), lo que no necesita tipos recursivos y
se libera de una sola vez. Cada etapa se validó durante la migración
comparando su salida con la del primer compilador sobre cada fuente del
árbol; el arnés de pruebas que ahora dirige las suites es a su vez un
programa Nexium, `tests/run.nx`.

## Dónde mirar cuando algo va mal

| síntoma | empieza aquí |
| --- | --- |
| un programa se analiza pero no debería, o al revés | `self/parser.nx`, luego `tests/compile_fail/` para el mensaje esperado |
| un error de tipos que parece incorrecto | `check_expr` en `self/check_exprs.nx`, `check_method_call` en `self/check_calls.nx` |
| un efecto que debería estar y no está, o al revés | el testigo en `own_effects`; busca `add_effect` en `self/check*.nx` |
| una fuga en `nx leaks` | la pila de ámbitos en `self/cgen.nx` (`register_drop`); todo temporal propio debe registrarse |
| C generado que no compila | `nx emit-c file.nx --keep-c` y lee `nx-out/file.c` (`emit-c` no escribe `#line`, así que el compilador de C nombra líneas del C); la función auxiliar del runtime a la que llama está en `runtime/nx_rt.h` |
| un fallo dentro de `for parallel` o `using arena` | `parallel_for` en `self/cgen.nx` y la sección de arenas del runtime |
