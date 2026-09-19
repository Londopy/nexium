<p align="center">
  <img src="https://raw.githubusercontent.com/Londopy/nexium/main/assets/banner.svg" alt="Nexium" width="880">
</p>

<p align="center">
  <a href="../../../README.md">English</a> ·
  <b>Español</b> ·
  <a href="../zh-CN/README.md">简体中文</a> ·
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
  <b>Un lenguaje lo bastante completo para construirlo todo, y a la vez la mejor opción para adoptar en una sola pieza de otra cosa.</b>
</p>

Nexium compila a código nativo a través de C, tiene conteo automático de
referencias sin recolector de basura, un sistema de efectos verificado por la
máquina que dice si una función reserva memoria, bloquea o puede fallar con
pánico, y un compilador que convierte un solo árbol de fuentes en una
biblioteca C, una rueda de Python, un crate de Rust o una herramienta de línea
de comandos.

<table>
<tr>
<td width="50%" valign="top">

**Un archivo**

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

**Todos los destinos**

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

La firma de efectos decide la ABI de C: `checksum` está demostrada `!panics`,
así que recibe un simple `uint32_t checksum(const uint8_t*, size_t)`. Una
función que puede fallar devuelve un código de estado, y un pánico de Nexium
dentro de ella se convierte en la frontera en lugar de abortar el proceso
anfitrión.

## Lo esencial

| | |
| --- | --- |
| 🧾 **Efectos inferidos y verificados** | `allocates` `refcounts` `blocks` `shared_mutable` `nondeterministic` `panics` `ffi`. Declara `!allocates` y el compilador señala la línea exacta que lo rompería, a través de las llamadas. |
| 🧠 **Propiedad sin verificador de préstamos** | Las colecciones se mueven, `.clone()` copia, los valores `ref class` se cuentan por referencias, `weak` rompe los ciclos. Usar después de mover es un error de compilación. |
| 🔬 **Patrones binarios** | `<<version:4, ihl:4, len:16/big, rest:bytes>>` reconoce y construye paquetes con tamaños verificados. |
| 🧵 **Bucles paralelos, arenas, objetos de trait** | `for parallel`, `using arena { }`, `dyn Trait !allocates`. |
| 🔌 **C sin bindings** | `@cImport("header.h")` lee la cabecera directamente; `artifact link` compila C propio dentro del programa. |
| 📦 **Distribuir desde una fuente** | `nx ship` produce cabeceras y bibliotecas C, ruedas de Python y crates de Rust con envoltorios seguros. |
| 🖼 **Una GUI, en Nexium** | [`gui/`](../../../gui): una GUI de modo inmediato (botones, deslizadores, campos de texto) con rasterizador por software y fuente de mapa de bits, todo Nexium sobre una capa de ventana de 200 líneas en C. |
| 🛠 **Herramientas incluidas** | `fmt`, `doc`, `lsp`, `size`, `leaks`, `refcounts`, `effects`, `audit`. Cero dependencias. |

## Instalación

El único requisito en tiempo de ejecución es [Zig](https://ziglang.org/download/)
en tu `PATH`, usado como compilador de C (`zig cc` también compila de forma
cruzada; `--cc clang` funciona igualmente).

Los binarios `nx` precompilados para Windows, Linux y macOS están en la página
de [Releases](https://github.com/Londopy/nexium/releases). Descomprime y pon
`nx` en tu `PATH`.

O compila desde el código fuente con Rust 1.75 o más reciente:

```bash
cargo install nexium
```

```bash
cargo install --git https://github.com/Londopy/nexium
```

Después:

```bash
nx run examples/hello.nx
```

## Un recorrido

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
<summary><b>Reconocimiento de patrones binarios</b></summary>

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
<summary><b>Los efectos se infieren y se verifican</b></summary>

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
<summary><b>Llamar a C está a una importación de cabecera</b></summary>

```
const libc = @cImport("string.h")
const cv = @cImport("cvendor.h")
artifact link { c_sources = ["cvendor.c"] }

unsafe { println("{}", .{libc.strlen(@cstr("hello"))}) }
```

Sin generador de bindings ni paso de construcción: la cabecera es la única
fuente de verdad para el diseño en memoria, y las llamadas foráneas llevan el
efecto `ffi`.

</details>

<details>
<summary><b>Bucles paralelos y arenas</b></summary>

```
for parallel (positions) |p, i| {
    out[i] = integrate(p)          // aquí no se permite shared_mutable
}

using arena {
    var scratch = List(Frame).new()   // reservado por bump, liberado todo a la vez
    ...
}
```

</details>

**Documentación**

- [Cómo funciona Nexium](architecture.md): la tubería de fuente a binario, la inferencia de efectos, la propiedad, el runtime y la distribución.
- [Referencia del lenguaje](language.md): cada construcción que implementa el compilador.
- [Integración](../../embedding.md) (inglés): llamar a bibliotecas distribuidas desde Python, Rust y C.
- [nexium-gui](../../gui.md) (inglés): la biblioteca de GUI de modo inmediato y cómo escribir un widget.
- [Publicar tu programa](../../releasing-your-program.md) (inglés): binarios para tres plataformas a partir de una etiqueta, instaladores opcionales.
- [Soporte de editores](../../../editors) (inglés): extensión de VS Code, sintaxis de Sublime, LSP.
- [Decisiones](../../../DECISIONS.md) (inglés): cada decisión tomada donde la especificación quedaba abierta.

## Comandos

| comando | qué hace |
| --- | --- |
| `nx build file.nx` | compila a un ejecutable (u objeto cuando no hay `main`) |
| `nx run file.nx` | compila y ejecuta |
| `nx test file.nx [filter]` | ejecuta los bloques `test "..."` |
| `nx check file.nx` | verifica tipos e informa violaciones de efectos |
| `nx effects file.nx` | imprime los efectos inferidos de cada función |
| `nx audit file.nx` | lista los bloques `unsafe` y las globales mutables |
| `nx ship file.nx` | produce cada `artifact` declarado |
| `nx emit-c file.nx` | imprime el C generado |
| `nx tokens file.nx` | vuelca la secuencia de tokens (el oráculo del autoalojamiento) |
| `nx fmt file.nx [--check]` | formato canónico |
| `nx doc file.nx` | documentación HTML con efectos inferidos |
| `nx size file.nx` | atribuye los bytes del binario a las declaraciones |
| `nx refcounts file.nx` | cada sitio de retención y liberación |
| `nx leaks file.nx` | ejecuta con seguimiento de asignaciones e informa fugas |
| `nx lsp` | servidor de lenguaje sobre stdio |

Opciones: `--mode debug|safe|fast|small`, `--target x86_64-linux-gnu`
(cualquier destino que `zig cc` conozca), `--out-dir`, `--keep-c`, `--cc`, y
para interoperar con C `-I`, `--link`, `--link-path`, `--c-source`.

## Estado

Esta es la primera implementación del diseño de `nexium-spec.txt`. Es lo
bastante completa para escribir programas reales (ver
[`examples/`](../../../examples)) y para distribuir un componente de Python,
Rust o C desde un solo archivo. Los objetos de trait, los bucles paralelos,
los ámbitos de arena, la importación directa de cabeceras C y las herramientas
están todos. Aún es temprano: la biblioteca estándar es una fracción de la
sección 16, y la verificación de regiones cubre solo las vistas devueltas.
[`DECISIONS.md`](../../../DECISIONS.md) recoge cada decisión tomada donde la
especificación quedaba abierta, y el punto 27 lista lo que falta.

## Autoalojamiento

Hoy el compilador es Rust. La versión en Nexium crece en
[`self/`](../../../self), una etapa a la vez, cada una verificada contra el
compilador de Rust con las mismas entradas:

| etapa | archivo | oráculo | estado |
| --- | --- | --- | --- |
| lexer | [`self/lexer.nx`](../../../self/lexer.nx) | `nx tokens` | ✅ coincide en cada ejemplo y en sí mismo |
| parser | | `nx parse` | siguiente |
| verificador | | `nx check`, la suite de compile-fail | |
| emisor de C | | `nx emit-c` | |

`cargo test` compila `self/lexer.nx` con el compilador de Rust y compara su
salida con el oráculo.

## Lenguajes del repositorio

Líneas no vacías de código, sin salidas de compilación, dependencias ni
archivos generados:

| lenguaje | líneas | parte | qué es |
| --- | --- | --- | --- |
| Rust | 22 393 | 86.9 % | el compilador `nx` |
| Nexium | 2 111 | 8.2 % | ejemplos, el lexer autoalojado, nexium-gui, pruebas |
| C | 1 108 | 4.3 % | el runtime `nx_rt.h` y la capa de ventana de la GUI |
| JavaScript, TypeScript | 159 | 0.6 % | la extensión de VS Code |

## Estructura

```
src/            el compilador (lexer, parser, verificador, comptime, backend C, driver)
runtime/        nx_rt.h, incrustado en cada archivo C generado
self/           el compilador en Nexium, etapa por etapa
gui/            nexium-gui: GUI de modo inmediato en Nexium, demo y la capa C de plataforma
editors/        extensión de VS Code y sintaxis de Sublime Text
examples/       programas con salida registrada, ejecutados por `cargo test`
tests/          pruebas de integración y casos de compile-fail
docs/           cómo funciona, referencia del lenguaje, guía de integración, traducciones
assets/         logotipo y banner
nexium-spec.txt          el diseño
nexium-systems-spec.txt  el lenguaje de sistemas archivado; las secciones 4 a 9 son la referencia de sintaxis
DECISIONS.md    decisiones tomadas donde la especificación quedaba abierta
```

## Contribuir

Ver [`CONTRIBUTING.md`](../../../CONTRIBUTING.md). Los errores y propuestas van
por GitHub issues; un cambio del lenguaje debe nombrar la restricción dura de
la sección 3 de la especificación a la que sirve.

## Licencia

MIT. Copyright (c) 2026 Londopy.

<p align="center"><img src="https://raw.githubusercontent.com/Londopy/nexium/main/assets/logo.svg" alt="" width="48"></p>
