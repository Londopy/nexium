<p align="center">
  <img src="https://raw.githubusercontent.com/Londopy/nexium/main/assets/banner.svg" alt="Nexium" width="880">
</p>

<p align="center">
  <a href="../../../README.md">English</a> ·
  <a href="../es/README.md">Español</a> ·
  <a href="../zh-CN/README.md">简体中文</a> ·
  <a href="../ja/README.md">日本語</a> ·
  <a href="../ko/README.md">한국어</a> ·
  <a href="../fr/README.md">Français</a> ·
  <b>Deutsch</b>
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
  <b>Eine Sprache, vollständig genug, um alles darin zu bauen, und zugleich die beste Wahl, um nur ein Teil von etwas anderem zu sein.</b>
</p>

Nexium kompiliert über C zu nativem Code, hat automatische Referenzzählung
ohne Garbage Collector, ein maschinell geprüftes Effektsystem, das sagt, ob
eine Funktion Speicher anfordert, blockiert oder in Panik geraten kann, und
einen Compiler, der aus einem einzigen Quellbaum eine C-Bibliothek, ein
Python-Wheel, ein Rust-Crate oder ein Kommandozeilenwerkzeug macht.

<table>
<tr>
<td width="50%" valign="top">

**Eine Datei**

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

**Jedes Ziel**

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

Die Effektsignatur bestimmt die C-ABI: `checksum` ist als `!panics`
bewiesen und bekommt deshalb ein schlichtes
`uint32_t checksum(const uint8_t*, size_t)`. Eine Funktion, die scheitern
kann, liefert einen Statuscode, und eine Nexium-Panik in ihrem Inneren wird
an der Grenze umgewandelt, statt den Wirtsprozess abzubrechen.

## Auf einen Blick

| | |
| --- | --- |
| 🧾 **Effekte, inferiert und geprüft** | `allocates` `refcounts` `blocks` `shared_mutable` `nondeterministic` `panics` `ffi`. Deklariere `!allocates`, und der Compiler zeigt über alle Aufrufe hinweg auf die Zeile, die es verletzen würde. |
| 🧠 **Besitz ohne Borrow-Checker** | Sammlungen werden verschoben, `.clone()` kopiert, `ref class`-Werte werden referenzgezählt, `weak` bricht Zyklen. Verwendung nach dem Verschieben ist ein Kompilierfehler. |
| 🔬 **Binärmuster** | `<<version:4, ihl:4, len:16/big, rest:bytes>>` erkennt und baut Pakete mit geprüften Größen. |
| 🧵 **Parallele Schleifen, Arenen, Trait-Objekte** | `for parallel`, `using arena { }`, `dyn Trait !allocates`. |
| 🔌 **C ohne Bindings** | `@cImport("header.h")` liest den Header direkt; `artifact link` kompiliert mitgeliefertes C ins Programm. |
| 📦 **Ausliefern aus einer Quelle** | `nx ship` erzeugt C-Header und -Bibliotheken, Python-Wheels und Rust-Crates mit sicheren Wrappern. |
| 🖼 **Eine GUI, in Nexium** | [`gui/`](../../../gui): eine Immediate-Mode-GUI (Buttons, Schieberegler, Textfelder) mit Software-Rasterizer und Bitmap-Schrift, komplett in Nexium über einer 200-zeiligen C-Fensterschicht. |
| 🛠 **Werkzeuge inklusive** | `fmt`, `doc`, `lsp`, `size`, `leaks`, `refcounts`, `effects`, `audit`. Keine Abhängigkeiten. |

## Installation

Die einzige Laufzeitvoraussetzung ist [Zig](https://ziglang.org/download/)
im `PATH`, das als C-Compiler dient (`zig cc` kann auch cross-kompilieren;
`--cc clang` geht ebenfalls).

Vorgefertigte `nx`-Binärdateien für Windows, Linux und macOS liegen auf der
[Releases](https://github.com/Londopy/nexium/releases)-Seite. Entpacken und
`nx` in den `PATH` legen.

Oder aus den Quellen mit Rust 1.75 oder neuer bauen:

```bash
cargo install nexium
```

```bash
cargo install --git https://github.com/Londopy/nexium
```

Danach:

```bash
nx run examples/hello.nx
```

## Ein Rundgang

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
<summary><b>Binäres Pattern Matching</b></summary>

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
<summary><b>Effekte werden inferiert und geprüft</b></summary>

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
<summary><b>C aufrufen ist ein Header-Import</b></summary>

```
const libc = @cImport("string.h")
const cv = @cImport("cvendor.h")
artifact link { c_sources = ["cvendor.c"] }

unsafe { println("{}", .{libc.strlen(@cstr("hello"))}) }
```

Kein Binding-Generator, kein Build-Schritt: der Header ist die einzige
Wahrheit über das Speicherlayout, und Fremdaufrufe tragen den Effekt `ffi`.

</details>

<details>
<summary><b>Parallele Schleifen und Arenen</b></summary>

```
for parallel p, i in positions {
    out[i] = integrate(p)          // hier ist shared_mutable verboten
}

using arena {
    var scratch = List(Frame).new()   // Bump-Allokation, alles auf einmal freigegeben
    ...
}
```

</details>

**Dokumentation**

- [Wie Nexium funktioniert](../../architecture.md): die Pipeline von der Quelle zur Binärdatei, Effektinferenz, Besitz, Laufzeit und Auslieferung.
- [Sprachreferenz](../../language.md): jedes Konstrukt, das der Compiler implementiert.
- [Einbetten](../../embedding.md) (Englisch): ausgelieferte Bibliotheken aus Python, Rust und C aufrufen.
- [nexium-gui](../../gui.md) (Englisch): die Immediate-Mode-GUI-Bibliothek und wie man ein Widget schreibt.
- [Dein Programm veröffentlichen](../../releasing-your-program.md) (Englisch): Binärdateien für drei Plattformen aus einem Tag, Installer optional.
- [Editor-Unterstützung](../../../editors) (Englisch): VS-Code-Erweiterung, Sublime-Syntax, LSP.
- [Entscheidungen](../../../DECISIONS.md) (Englisch): jede Entscheidung dort, wo die Spezifikation offen war.

## Befehle

| Befehl | Wirkung |
| --- | --- |
| `nx build file.nx` | zu einer ausführbaren Datei kompilieren (oder Objekt, wenn es kein `main` gibt) |
| `nx run file.nx` | bauen und ausführen |
| `nx test file.nx [filter]` | die `test "..."`-Blöcke ausführen |
| `nx check file.nx` | Typen prüfen und Effektverletzungen melden |
| `nx effects file.nx` | die inferierten Effekte jeder Funktion ausgeben |
| `nx audit file.nx` | `unsafe`-Blöcke und veränderliche Globale auflisten |
| `nx ship file.nx` | jedes deklarierte `artifact` erzeugen |
| `nx emit-c file.nx` | das erzeugte C ausgeben |
| `nx tokens file.nx` | den Tokenstrom ausgeben (das Orakel für das Self-Hosting) |
| `nx fmt file.nx [--check]` | kanonische Formatierung |
| `nx doc file.nx` | HTML-Dokumentation mit inferierten Effekten |
| `nx size file.nx` | Bytes der Binärdatei den Deklarationen zuordnen |
| `nx refcounts file.nx` | jede Retain- und Release-Stelle |
| `nx leaks file.nx` | mit Allokationsverfolgung ausführen und Lecks melden |
| `nx lsp` | Sprachserver über stdio |

Optionen: `--mode debug|safe|fast|small`, `--target x86_64-linux-gnu`
(jedes Ziel, das `zig cc` kennt), `--out-dir`, `--keep-c`, `--cc`, und für die
C-Anbindung `-I`, `--link`, `--link-path`, `--c-source`.

## Stand

Dies ist die erste Implementierung des Entwurfs in `nexium-spec.txt`. Sie ist
vollständig genug, um echte Programme zu schreiben (siehe
[`examples/`](../../../examples)) und aus einer Datei eine Python-, Rust- oder
C-Komponente auszuliefern. Trait-Objekte, parallele Schleifen, Arena-Bereiche,
direkter Import von C-Headern und die Werkzeuge sind alle da. Noch früh: die
Standardbibliothek ist ein Bruchteil von Abschnitt 16, und die Regionenprüfung
deckt nur zurückgegebene Sichten ab. [`DECISIONS.md`](../../../DECISIONS.md)
führt jede Entscheidung auf, die dort getroffen wurde, wo die Spezifikation
offen war; Punkt 27 listet, was fehlt.

## Self-Hosting

Der Compiler ist heute Rust. Die Nexium-Version wächst in
[`self/`](../../../self), Stufe für Stufe, jede gegen den Rust-Compiler mit
denselben Eingaben geprüft:

| Stufe | Datei | Orakel | Stand |
| --- | --- | --- | --- |
| Lexer | [`self/lexer.nx`](../../../self/lexer.nx) | `nx tokens` | ✅ identisch bei jedem Beispiel und bei sich selbst |
| Parser | | `nx parse` | als Nächstes |
| Prüfer | | `nx check`, die Compile-Fail-Suite | |
| C-Emitter | | `nx emit-c` | |

`cargo test` baut `self/lexer.nx` mit dem Rust-Compiler und vergleicht die
Ausgabe mit dem Orakel.

## Sprachen im Repository

Nichtleere Codezeilen ohne Build-Ausgaben, Abhängigkeiten und generierte
Dateien:

| Sprache | Zeilen | Anteil | Zweck |
| --- | --- | --- | --- |
| Rust | 22 393 | 86,9 % | der Compiler `nx` |
| Nexium | 2 111 | 8,2 % | Beispiele, der selbstgehostete Lexer, nexium-gui, Tests |
| C | 1 108 | 4,3 % | die Laufzeit `nx_rt.h` und die Fensterschicht der GUI |
| JavaScript, TypeScript | 159 | 0,6 % | die VS-Code-Erweiterung |

## Aufbau

```
src/            der Compiler (Lexer, Parser, Prüfer, comptime, C-Backend, Treiber)
runtime/        nx_rt.h, in jede erzeugte C-Datei eingebettet
self/           der Compiler in Nexium, Stufe für Stufe
gui/            nexium-gui: Immediate-Mode-GUI in Nexium, Demo und die C-Plattformschicht
editors/        VS-Code-Erweiterung und Sublime-Text-Syntax
examples/       Programme mit aufgezeichneter Ausgabe, von `cargo test` ausgeführt
tests/          Integrationstests und Compile-Fail-Fälle
docs/           Funktionsweise, Sprachreferenz, Einbettungsleitfaden, Übersetzungen
assets/         Logo und Banner
nexium-spec.txt          der Entwurf
nexium-systems-spec.txt  die archivierte Systemsprache; Abschnitte 4 bis 9 sind die Syntaxreferenz
DECISIONS.md    Entscheidungen dort, wo die Spezifikation offen war
```

## Mitwirken

Siehe [`CONTRIBUTING.md`](../../../CONTRIBUTING.md). Fehler und Vorschläge
laufen über GitHub-Issues; eine Sprachänderung muss die harte Randbedingung in
Abschnitt 3 der Spezifikation nennen, der sie dient.

## Lizenz

MIT. Copyright (c) 2026 Londopy.

<p align="center"><img src="https://raw.githubusercontent.com/Londopy/nexium/main/assets/logo.svg" alt="" width="48"></p>
