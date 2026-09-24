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
  <a href="https://londopy.github.io/nexium/"><img alt="Docs" src="https://img.shields.io/badge/docs-londopy.github.io%2Fnexium-5b4bd6"></a>
  <a href="../../../LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-blue"></a>
  <a href="https://ziglang.org/download/"><img alt="Zig" src="https://img.shields.io/badge/backend-zig%20cc-f7a41d?logo=zig&logoColor=white"></a>
  <img alt="Platforms" src="https://img.shields.io/badge/platforms-windows%20%7C%20linux%20%7C%20macos-2b3a55">
</p>

<p align="center">
  <b>Nexium es un lenguaje lo bastante completo para construirlo todo, y a la vez la mejor opción para adoptar en una sola pieza de otra cosa.</b>
</p>

<p align="center">
  <a href="https://londopy.github.io/nexium/"><b>Documentación y el tutorial «el Topo» &rarr; londopy.github.io/nexium</b></a><br>
  <sub><a href="https://londopy.github.io/nexium/topo/01-base-camp.html">Empieza por el Topo</a> &middot; <a href="https://londopy.github.io/nexium/docs/language.html">Referencia del lenguaje</a> &middot; <a href="https://londopy.github.io/nexium/docs/install.html">Instalación</a> &middot; <a href="https://londopy.github.io/nexium/docs/std.html">Biblioteca estándar</a> &middot; <a href="https://londopy.github.io/nexium/docs/embedding.html">Integración</a> (en inglés)</sub>
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
    for b in data {
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
así que recibe un `uint32_t checksum(const uint8_t*, size_t)` sin más. Una
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

**Windows**: descarga y ejecuta el instalador desde la página de
[Releases](https://github.com/Londopy/nexium/releases). Instala `nx`, una
cadena de herramientas Zig incluida (el compilador de C que `nx` usa), la
biblioteca estándar, los ejemplos, la documentación y la extensión de VS
Code, y añade `nx` a tu PATH. No hace falta instalar nada más.

**macOS y Linux**:

```bash
curl -fsSL https://raw.githubusercontent.com/Londopy/nexium/main/installers/install.sh | sh
```

Verifica la descarga contra las sumas de comprobación de la release, instala
en `~/.nexium`, prepara un compilador de C (las herramientas de Xcode en
macOS; en Linux descarga Zig cuando no encuentra ninguno) y añade `nx` a tu
PATH.

**Windows, desde PowerShell**: `irm https://raw.githubusercontent.com/Londopy/nexium/main/installers/install.ps1 | iex`
(la build portable, verificada, en el PATH; sin asistente).

**pip o npm**: `pip install nexium-lang` o `npm install -g nexium-lang` (el binario, por plataforma; un compilador de C como siempre).

**Docker**: `docker run --rm -v "$PWD":/work ghcr.io/londopy/nexium run hello.nx`
(Debian; también `:alpine`; amd64 y arm64).

**Homebrew y Scoop**: el repositorio es su propio tap y su propio bucket.

```bash
brew tap londopy/tap https://github.com/Londopy/nexium && brew install londopy/tap/nexium
```

```powershell
scoop install https://raw.githubusercontent.com/Londopy/nexium/main/bucket/nexium.json
```

Después, en una consola nueva, `nx doctor` muestra qué se usará. Todos los
detalles, incluida la verificación de las sumas y cada variable de entorno,
están en [docs/install.md](../../install.md) (inglés).

O compila desde el código fuente sin nada más que un compilador de C (Zig en
el PATH, o `CC`), que construye el compilador escrito en Nexium a partir de su
semilla en C:

```bash
git clone https://github.com/Londopy/nexium && cd nexium && sh bootstrap/build.sh
```

El resultado es `nx-out/bootstrap/nx2` (`build.ps1` en Windows). Después:

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
    if text.len == 0 { return error.Empty }
    var total: i64 = 0
    for c in text {
        if c < '0' or c > '9' { return error.NotANumber }
        total = total * 10 + (c - '0') as i64
    }
    return total
}

fn max(comptime T: type where T: Ord, a: T, b: T) -> T {
    return if a > b { a } else { b }
}

fn main() -> !void {
    let n = try parse_num("1234")
    let bad = parse_num("12x") catch |e| {
        println("caught {}", .{e})
        -1
    }
    var xs = List(i32).new()
    for i in 0..10 { xs.append((i * i) as i32) }
    let found = outer: {
        for x, i in xs { if x > 30 { break :outer i as i64 } }
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
<summary><b>Llamar a C está a un import de cabecera</b></summary>

```
const libc = @cImport("string.h")
const cv = @cImport("cvendor.h")
artifact link { c_sources = ["cvendor.c"] }

unsafe { println("{}", .{libc.strlen(@cstr("hello"))}) }
```

Sin generador de bindings, sin paso de construcción: la cabecera es la única
fuente de verdad para la disposición en memoria, y las llamadas foráneas
llevan el efecto `ffi`.

</details>

<details>
<summary><b>Bucles paralelos y arenas</b></summary>

```
for parallel p, i in positions {
    out[i] = integrate(p)          // aquí no se permite shared_mutable
}

using arena {
    var scratch = List(Frame).new()   // asignación bump, liberada toda a la vez
    ...
}
```

</details>

**Documentación**

- [Especificación](../../../SPEC.md) (inglés): el lenguaje tal como está implementado, con las partes planificadas marcadas.
- [Hoja de ruta](../../../ROADMAP.md) (inglés): fases, criterios de salida y lo que no está planeado.
- [Cómo funciona Nexium](architecture.md): la tubería de fuente a binario, la inferencia de efectos, la propiedad, el runtime y la distribución.
- [Referencia del lenguaje](language.md): cada construcción que el compilador implementa.
- [Integración](../../embedding.md) (inglés): llamar a bibliotecas distribuidas desde Python, Rust y C.
- [La sesión interactiva](../../repl.md) (inglés): `nx` en un prompt, como `python`.
- [Estabilidad](../../stability.md) y [plataformas](../../platforms.md) (inglés): qué promete una versión, el ciclo de obsolescencia, `nx fix`, los niveles.
- [El Topo](https://londopy.github.io/nexium/topo/01-base-camp.html) (inglés): el tutorial, desde instalar el compilador hasta una red neuronal, una GUI y una biblioteca distribuida; la fuente está en [`topo/`](../../../topo/). Todo lo anterior, renderizado, está en [londopy.github.io/nexium](https://londopy.github.io/nexium/).
- [Instalación](../../install.md) (inglés): el instalador de Windows, el script de macOS/Linux, la compilación desde fuente, las sumas de comprobación y cómo `nx` encuentra un compilador de C.
- [Paquetes](../../packages.md) (inglés): `nexium.toml`, `nx add`, `nx fetch`, dependencias por git o por ruta, el archivo de bloqueo.
- [Biblioteca estándar](../../std.md) (inglés): los módulos escritos en Nexium (`std.strings`, `std.lists`, `std.bytes`, `std.num`, `std.json`, `std.args`, `std.fs`, `std.time`, `std.regex`, `std.text`, `std.testing`, `std.stream`, `std.net`, `std.http`, `std.thread`, `std.process`).
- [nexium-gui](../../gui.md) (inglés): la biblioteca de GUI de modo inmediato y cómo escribir un widget.
- [Publicar tu programa](../../releasing-your-program.md) (inglés): binarios para tres plataformas a partir de una etiqueta, instaladores opcionales.
- [Soporte de editores](../../../editors) (inglés): VS Code, Vim, Neovim, Helix, Zed, Emacs, Kate, JetBrains, Sublime Text, Notepad++, nano, y `nx lsp` para el resto.
- [Linguist](../../../linguist) (inglés): el pull request listo para aplicar que hará que GitHub reconozca `.nx` cuando se alcance el umbral de uso.
- [Traducciones](../README.md): este README en seis idiomas; la referencia del lenguaje y el recorrido por la arquitectura en español, chino y japonés.
- [Nombres de las versiones](../../release-names.md) (inglés): cada versión es un lugar de una montaña; el esquema, el registro y los nombres por usar.
- [Decisiones](../../../DECISIONS.md) (inglés): cada decisión tomada donde la especificación estaba abierta.
- [Problemas conocidos](../../../KNOWN_ISSUES.md) (inglés): errores abiertos, carencias y limitaciones, con reproducciones.

## Comandos

| comando | qué hace |
| --- | --- |
| `nx build file.nx` | compila a un ejecutable (o a un objeto cuando no hay `main`) |
| `nx run file.nx` | compila y ejecuta; `--watch` vuelve a ejecutar cuando cambia un archivo del programa |
| `nx test file.nx [filter]` | ejecuta los bloques `test "..."`; también con `--watch` |
| `nx check file.nx` | verifica tipos e informa de violaciones de efectos |
| `nx effects file.nx` | imprime los efectos inferidos de cada función |
| `nx explain file.nx f effect` | por qué `f` tiene el efecto: las llamadas que lo introducen, hasta la primitiva, como un árbol |
| `nx audit file.nx` | lista los bloques `unsafe` y las globales mutables; `--lock` escribe el archivo de bloqueo de efectos, `--check` falla ante un efecto nuevo |
| `nx ship file.nx` | produce cada `artifact` declarado |
| `nx emit-c file.nx` | imprime el C generado |
| `nx tir file.nx [--sigs]` | el programa verificado como S-expresiones (las pruebas del propio compilador lo leen) |
| `nx fmt file.nx [--check]` | formato canónico |
| `nx debug file.nx` | compila para depurar y ejecuta bajo gdb o lldb: líneas de `.nx` y formateadores para String, List, Map, slices y opcionales |
| `nx bench file.nx` | mide los bloques `bench "name" { }`: una compilación optimizada, la mediana del tiempo por iteración |
| `nx fix file.nx` | aplica las correcciones mecánicas del verificador (`.clone()`, `@escape(...)`, `_ = `) y migra las formas obsoletas; ver [docs/stability.md](../../stability.md) |
| `nx doc file.nx` | documentación HTML con los efectos inferidos |
| `nx size file.nx` | atribuye los bytes del binario a las declaraciones |
| `nx layout file.nx [Type...]` | desplazamientos, tamaños y relleno de un struct o enum, y el orden por alineación que lo reduciría |
| `nx upgrade` | la última release en lugar de este ejecutable, verificada; `--check` solo informa |
| `nx install [DIR]` | esta copia, con lo que la acompaña, al lugar del usuario y al PATH (el zip portable instalándose a sí mismo) |
| `nx refcounts file.nx` | cada punto de retain y release |
| `nx leaks file.nx` | ejecuta con seguimiento de asignaciones e informa de fugas |
| `nx lsp` | servidor de lenguaje sobre stdio |
| `nx doctor` | qué compilador de C se usará y si la instalación funciona |
| `nx repl`, o simplemente `nx` | una sesión interactiva: escribe código, ve los valores, conserva las ligaduras |

Opciones: `--mode debug|safe|fast|small`, `--target x86_64-linux-gnu`
(cualquier destino que `zig cc` conozca), `--cpu baseline|native|<nombre>`
(baseline por defecto, para que un binario corra en cualquier máquina de su
arquitectura), `--out-dir`, `--keep-c`, `--cc`, `--strict` (los avisos son
errores), `--sanitize address,undefined` (los sanitizers del compilador de C;
`address` necesita gcc o clang), y para la interoperabilidad
con C `-I`, `--link`, `--link-path`, `--c-source`.

## Estado

**1.0: lenguaje estable, ecosistema incipiente.** El lenguaje solo cambia por
adición, bajo la [política de estabilidad](../../stability.md); el compilador
está escrito en Nexium y se construye a sí mismo; cada ejemplo, cada caso de
la especificación y cada programa del tutorial se ejecuta en CI en tres
plataformas, bajo los sanitizadores y el fuzzer. Lo que 1.0 todavía no es, y
dónde se responde cada punto, es la primera sección de [la hoja de
ruta](../../../ROADMAP.md): la seguridad de memoria son las reglas de vistas
de 1.2, errores desde 1.3 (`nx fix` hace las correcciones mecánicas),
no hay cifras de rendimiento más allá de [la página de cifras](../../numbers.md)
(cuatro programas en cinco lenguajes en un mismo runner, regenerada cada semana),
y el ecosistema es un solo mantenedor, dieciséis módulos de la biblioteca
estándar y un paquete de fuera del árbol (el [SDK de Discord Rich Presence](../../discord.md)
de statusmith, `nx add discord_rpc ...`).
[`KNOWN_ISSUES.md`](../../../KNOWN_ISSUES.md) lista cada error abierto con su
arreglo; [`DECISIONS.md`](../../../DECISIONS.md), cada decisión tomada donde
la especificación estaba abierta.

## Nombres de las versiones

Una versión mayor es una montaña, en el orden en que se escalaron por primera
vez los catorce ochomiles; las versiones que cuelgan de ella son la ascensión:
campamentos, rutas y caras para las versiones menores, los miembros de la
expedición del primer ascenso para los parches, `Summit` para `X.0.0`. La
línea 0.x es la aproximación y los campamentos del Annapurna, el primer
ochomil escalado (1950), así que 1.0.0 es `Annapurna: Summit`; 0.7.0, donde el
compilador empezó a construirse a sí mismo, es `Annapurna: Camp V`, el último
campamento antes del ataque a la cumbre. El nombre está en el changelog, en el
título de la release y en `nx version`;
[docs/release-names.md](../../release-names.md) (inglés) tiene la regla, el
registro y las montañas que quedan por subir.

## Autoalojamiento

El compilador está escrito en Nexium, en [`self/`](../../../self), y se
construye a sí mismo. Una máquina sin `nx` construye uno a partir de
[`bootstrap/nx.c`](../../../bootstrap/nx.c), el C que el compilador emite para
sí mismo, con cualquier compilador de C y sin Rust:

```sh
sh bootstrap/build.sh     # nx.c -> nx0; nx0 construye self/nx.nx -> nx1; nx1 se reconstruye al mismo C -> nx2
```

| etapa | archivo | qué hace |
| --- | --- | --- |
| lexer | [`self/lexer.nx`](../../../self/lexer.nx) | tokens |
| parser | [`self/parser.nx`](../../../self/parser.nx) | un árbol sintáctico en una arena de ids |
| verificador | [`self/check.nx`](../../../self/check.nx), `self/check_*.nx`, [`self/cimport.nx`](../../../self/cimport.nx) | tipos, efectos, propiedad, genéricos, el intérprete de tiempo de compilación, la importación de cabeceras C, cada diagnóstico |
| emisor de C | [`self/cgen.nx`](../../../self/cgen.nx) | un archivo C por programa |
| driver | [`self/nx.nx`](../../../self/nx.nx) | build, run, test, check, emit-c, tir; la biblioteca estándar incrustada |
| herramientas | [`self/fmt.nx`](../../../self/fmt.nx), [`self/doc.nx`](../../../self/doc.nx), [`self/tools.nx`](../../../self/tools.nx), [`self/size.nx`](../../../self/size.nx), [`self/manifest.nx`](../../../self/manifest.nx), [`self/ship.nx`](../../../self/ship.nx), [`self/lsp.nx`](../../../self/lsp.nx), [`self/repl.nx`](../../../self/repl.nx) | el formateador, el generador de documentación, los informes, los paquetes, `ship`, el servidor de lenguaje, el REPL |

Cada ejemplo, cada caso de la especificación y cada caso de compile-fail pasa
por el compilador arrancado, dirigido por un arnés de pruebas que es a su vez
un programa Nexium (`nx run tests/run.nx`), en CI en tres plataformas sin
ninguna cadena de herramientas de Rust. El primer compilador, en Rust,
impulsó el port y se eliminó en 1.0 (decisión 90).

## Lenguajes del repositorio

Líneas de código no vacías, excluyendo la salida de compilación, las
dependencias y los archivos generados (`bootstrap/nx.c`, el parser de
tree-sitter, `gui/font.bin`, los archivos de bloqueo):

| lenguaje | líneas | proporción | qué es |
| --- | --- | --- | --- |
| Nexium | 38 374 | 85,4 % | el compilador y sus herramientas (27 100 líneas bajo `self/`), la biblioteca estándar, el arnés de pruebas y el fuzzer, los ejemplos, los programas del tutorial, nexium-gui, el generador del sitio, la suite de la especificación, cuatro benchmarks |
| C | 2 925 | 6,5 % | el runtime `nx_rt.h`, la capa de ventana de la GUI, C de prueba incluido, un benchmark |
| Python | 1 063 | 2,4 % | los scripts de release (notas, manifiestos de paquetes, wheels y paquetes npm, la documentación de std), el ejecutor de benchmarks, un benchmark |
| archivos de editores | 1 028 | 2,3 % | consultas de tree-sitter, Emacs Lisp, Vim script, Lua para Neovim, y las 25 líneas de Rust que Zed exige a una extensión |
| JavaScript, TypeScript | 550 | 1,2 % | la extensión de VS Code y la gramática de tree-sitter |
| Inno Setup, shell, PowerShell | 777 | 1,7 % | el script del instalador de Windows, `install.sh`, `install.ps1`, los scripts de Chocolatey |
| Rust, Go, Ruby | 236 | 0,5 % | un benchmark en Rust y otro en Go, y la fórmula de Homebrew |

No hay Rust en el compilador: el primer compilador impulsó el port y se
eliminó en 1.0 (decisión 90). El Rust que queda es el pegamento de la
extensión de Zed, que Zed compila a WebAssembly, y un programa de benchmark
escrito para medirse contra él, junto a su gemelo en Go. Zig no está en la tabla
porque no hay fuentes Zig en el árbol: `zig cc` es el compilador de C que `nx`
ejecuta (incluido por el instalador de Windows, descargado por el script de
instalación), del mismo modo que un compilador de C se usa y no se escribe.

## Estructura

```
bootstrap/      la semilla en C desde la que se construye el compilador, y los scripts de construcción
runtime/        nx_rt.h, incrustado en cada archivo C generado
std/            la biblioteca estándar en Nexium, incrustada en el compilador
self/           el compilador en Nexium, etapa por etapa
gui/            nexium-gui: GUI de modo inmediato en Nexium, demo y la capa de plataforma en C
editors/        extensión de VS Code, gramática de tree-sitter y los archivos de diez editores más
examples/       programas con salida registrada, ejecutados por las pruebas
topo/           el tutorial: los capítulos y los programas que muestran (ejecutados por las pruebas)
site/           el generador del sitio de documentación, un programa Nexium
tests/          el arnés (run.nx), la suite de conformidad de la especificación (tests/spec) y los casos de compile-fail
docs/           cómo funciona, referencia del lenguaje, guía de integración, traducciones en i18n/
bench/          cuatro programas en cinco lenguajes detrás de la página de cifras
installers/     el script del instalador de Windows, install.sh e install.ps1, los manifiestos de winget y Chocolatey
docker/         las imágenes del compilador para ghcr.io (Debian y Alpine)
Formula/, bucket/  este repositorio como tap de Homebrew y bucket de Scoop (escritos en cada release)
scripts/        notas de release, manifiestos de paquetes, wheels y paquetes npm, la documentación de std
assets/         logotipo, banner y la vista previa social
nexium-spec.txt          el diseño
nexium-systems-spec.txt  el lenguaje de sistemas archivado; las secciones 4 a 9 son la referencia de sintaxis
DECISIONS.md    decisiones tomadas donde la especificación estaba abierta
KNOWN_ISSUES.md errores abiertos y limitaciones; los arreglos pasan al changelog
```

## Contribuir

Consulta [`CONTRIBUTING.md`](../../../CONTRIBUTING.md). Los errores y las
propuestas pasan por los issues de GitHub; un cambio en el lenguaje debe
nombrar la restricción dura de la sección 3 de la especificación a la que
sirve. Los pull requests pasan las pruebas en tres plataformas, los
formateadores, una comprobación del changelog y el [Acuerdo de Licencia de
Contribuidor](../../../CLA.md) antes de fusionarse; conservas tus derechos de
autor.

## Licencia

MIT. Copyright (c) 2026 Londopy.

<p align="center"><img src="https://raw.githubusercontent.com/Londopy/nexium/main/assets/logo.svg" alt="" width="48"></p>
