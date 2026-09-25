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
  <a href="https://londopy.github.io/nexium/"><img alt="Docs" src="https://img.shields.io/badge/docs-londopy.github.io%2Fnexium-5b4bd6"></a>
  <a href="../../../LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-blue"></a>
  <a href="https://ziglang.org/download/"><img alt="Zig" src="https://img.shields.io/badge/backend-zig%20cc-f7a41d?logo=zig&logoColor=white"></a>
  <img alt="Platforms" src="https://img.shields.io/badge/platforms-windows%20%7C%20linux%20%7C%20macos-2b3a55">
</p>

<p align="center">
  <b>Nexium ist eine Sprache, die vollständig genug ist, um alles darin zu bauen, und zugleich die beste Wahl für ein einzelnes Teil von etwas anderem.</b>
</p>

<p align="center">
  <a href="https://londopy.github.io/nexium/"><b>Dokumentation und das Tutorial „der Topo“ &rarr; londopy.github.io/nexium</b></a><br>
  <sub><a href="https://londopy.github.io/nexium/topo/01-base-camp.html">Mit dem Topo anfangen</a> &middot; <a href="https://londopy.github.io/nexium/docs/language.html">Sprachreferenz</a> &middot; <a href="https://londopy.github.io/nexium/docs/install.html">Installation</a> &middot; <a href="https://londopy.github.io/nexium/docs/std.html">Standardbibliothek</a> &middot; <a href="https://londopy.github.io/nexium/docs/embedding.html">Einbettung</a> (auf Englisch)</sub>
</p>

Nexium kompiliert über C zu nativem Code, hat automatische Referenzzählung
ohne Tracing-Garbage-Collector, ein maschinell geprüftes Effektsystem, das
sagt, ob eine Funktion allokiert, blockiert oder in Panik geraten kann, und
einen Compiler, der aus einem Quellbaum eine C-Bibliothek, ein Python-Wheel,
ein Rust-Crate oder ein Kommandozeilenwerkzeug macht.

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

Die Effektsignatur bestimmt die C-ABI: `checksum` ist als `!panics` bewiesen,
also bekommt es ein schlichtes `uint32_t checksum(const uint8_t*, size_t)`.
Eine Funktion, die fehlschlagen kann, gibt einen Statuscode zurück, und eine
Nexium-Panik darin wird an der Grenze umgewandelt, statt den Wirtsprozess
abzubrechen.

## Das Wichtigste

| | |
| --- | --- |
| 🧾 **Effekte, inferiert und geprüft** | `allocates` `refcounts` `blocks` `shared_mutable` `nondeterministic` `panics` `ffi`. Wer `!allocates` erklärt, dem zeigt der Compiler die genaue Zeile, die das bräche, auch über Aufrufe hinweg. |
| 🛡 **Speichersicherheit ohne Garbage Collector** | Collections werden bewegt, `.clone()` kopiert, `ref class`-Werte werden referenzgezählt, `weak` bricht Zyklen, und ein Slice oder Zeiger überlebt nie den Speicher, auf den er zeigt: die Sichtregeln werden über Aufrufe, Schleifen und Verzweigungen geprüft, ohne dass man Lebensdauern schreibt. Wo die Korrektur mechanisch ist, macht sie `nx fix`. |
| 🔬 **Binärmuster** | `<<version:4, ihl:4, len:16/big, rest:bytes>>` gleicht Pakete ab und baut sie, mit geprüften Größen. |
| 🧵 **Parallele Schleifen, Arenen, Trait-Objekte** | `for parallel`, `using arena { }`, `dyn Trait !allocates`. |
| 🔌 **C ohne Bindings** | `@cImport("header.h")` liest den Header direkt; `artifact link` kompiliert mitgeliefertes C ins Programm; `if comptime @target().0 == "windows"` baut nur den Zweig, den die Plattform nimmt. |
| 📦 **Aus einer Quelle ausliefern** | `nx ship` erzeugt C-Header und -Bibliotheken, Python-Wheels und Rust-Crates mit sicheren Wrappern. |
| 🐞 **Debuggen und messen** | `nx debug` hält gdb oder lldb an `.nx`-Zeilen an und zeigt Strings, Listen, Maps und Optionals als Werte; `bench "name" { }`-Blöcke stehen neben den Tests; `--sanitize address,undefined` legt AddressSanitizer und UBSan unter jeden Build. |
| 🧭 **Im Browser lernen** | [Der Topo](https://londopy.github.io/nexium/topo/01-base-camp.html), das Tutorial, führt seine Programme auf der Seite aus, mit dem zu WebAssembly kompilierten Compiler, und bewertet seine Übungen wie `nx topo` im Terminal. `nx repl` ist eine Eingabeaufforderung. |
| 🪞 **In sich selbst geschrieben** | Der Compiler ist Nexium und wird aus einer C-Datei von jedem C-Compiler gebaut; der Debug-Build eines großen Programms kompiliert nur die geänderten Module neu. |
| 🖼 **Eine GUI, in Nexium** | [`gui/`](../../../gui): eine Immediate-Mode-GUI (Buttons, Slider, Textfelder) mit Software-Rasterizer und Bitmap-Font, alles Nexium über einer 200-zeiligen C-Fensterschicht. |
| 🛠 **Werkzeuge inklusive** | `fmt`, `fix`, `doc`, `lsp` (Definition, Hover und Umbenennen aus dem Checker), `debug`, `bench`, `size`, `layout`, `leaks`, `refcounts`, `effects`, `explain`, `audit`, `repl`. Keine Abhängigkeiten. |

## Installation

**Windows**: den Installer von der
[Releases](https://github.com/Londopy/nexium/releases)-Seite laden und
ausführen. Er installiert `nx`, eine mitgelieferte Zig-Toolchain (den
C-Compiler, den `nx` verwendet), die Standardbibliothek, Beispiele,
Dokumentation und die VS-Code-Erweiterung und nimmt `nx` in den PATH auf.
Sonst ist nichts zu installieren.

**macOS und Linux**:

```bash
curl -fsSL https://raw.githubusercontent.com/Londopy/nexium/main/installers/install.sh | sh
```

Das Skript prüft den Download gegen die Prüfsummen der Release, installiert
nach `~/.nexium`, richtet einen C-Compiler ein (die Xcode-Werkzeuge auf macOS;
unter Linux wird Zig geladen, wenn nichts gefunden wird) und nimmt `nx` in den
PATH auf.

**Windows, aus PowerShell**: `irm https://raw.githubusercontent.com/Londopy/nexium/main/installers/install.ps1 | iex`
(der portable Build, geprüft, im PATH; kein Assistent).

**pip oder npm**: `pip install nexium-lang` ([PyPI](https://pypi.org/project/nexium-lang/)) oder `npm install -g nexium-lang` ([npm](https://www.npmjs.com/package/nexium-lang)); das Binary, je Plattform; ein C-Compiler wie üblich.

**Docker**: `docker run --rm -v "$PWD":/work ghcr.io/londopy/nexium run hello.nx`
(Debian; auch `:alpine`; amd64 und arm64).

**Chocolatey und winget**: `choco install nexium` ([das Paket](https://community.chocolatey.org/packages/nexium)), jede Version, sobald die Moderatoren von Chocolatey sie freigegeben haben (sie kann dem neuesten Release einige Tage hinterherhinken), und `winget install Londopy.Nexium`, sobald winget die erste Version aufgenommen hat ([der Stand](../../install.md#where-to-get-it), Englisch).

**Debian, RPM, Nix, mise**: jede Release bringt `.deb`- und `.rpm`-Pakete mit (`sudo dpkg -i nexium_*_amd64.deb`); `nix run github:Londopy/nexium` baut es aus der einen C-Datei; `mise use -g "ubi:Londopy/nexium[exe=nx]"` installiert das Release-Binary. Jedes Asset trägt eine signierte Herkunftsangabe: `gh attestation verify nx --owner Londopy`. [Alle Wege](../../install.md#where-to-get-it) (Englisch).

**Im Browser**: [das Repository in einem Codespace öffnen](https://codespaces.new/Londopy/nexium), und `nx run examples/hello.nx` läuft nach einer Minute, ohne dass etwas installiert wird.

**In einem Notebook**: in [Google Colab](https://colab.research.google.com) oder Jupyter `!pip install -q nexium-lang`, dann `!nx run hello.nx` auf einer Datei, die eine `%%writefile`-Zelle geschrieben hat ([die drei Zellen](../../install.md#in-a-notebook-colab-and-jupyter), Englisch).

**Homebrew und Scoop**: das Repository ist sein eigener Tap, und Scoops Bucket ist [Londopy/scoop-bucket](https://github.com/Londopy/scoop-bucket), aktuell gehalten von Scoops eigenem Updater.

```bash
brew tap londopy/tap https://github.com/Londopy/nexium && brew install londopy/tap/nexium
```

```powershell
scoop bucket add londopy https://github.com/Londopy/scoop-bucket && scoop install nexium
```

Danach zeigt `nx doctor` in einer neuen Konsole, was verwendet wird. Alle
Einzelheiten, einschließlich der Prüfsummen und jeder Umgebungsvariable,
stehen in [docs/install.md](../../install.md) (Englisch).

Oder aus dem Quelltext bauen, mit nichts als einem C-Compiler (Zig im PATH
oder `CC`); das baut den in Nexium geschriebenen Compiler aus seinem
C-Saatkorn:

```bash
git clone https://github.com/Londopy/nexium && cd nexium && sh bootstrap/build.sh
```

Das Ergebnis ist `nx-out/bootstrap/nx2` (`build.ps1` unter Windows). Dann:

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
<summary><b>Binärmuster-Abgleich</b></summary>

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
<summary><b>Eine Sicht überlebt nie ihren Speicher</b></summary>

```
fn main() {
    var names = List(String).new()
    names.append(String.from("ada"))
    let first = names[0][..]
    names.append(String.from("grace"))
    println("{}", .{first})
}
```

```
error: `first` is a view into `names`, which changed on line 5 after the view
was taken; its storage may have moved (rule V3); take the view after the
change, or keep an owned copy of the container (`.clone()`) taken before it
  --> views.nx:6:21
```

Ein Slice oder Zeiger wird gegen den Speicher geprüft, auf den er zeigt
(SPEC 5.6, Regeln V1 bis V5): kein Garbage Collector und keine
Lebensdauern zum Aufschreiben.

</details>

<details>
<summary><b>Tests und Benchmarks nebeneinander</b></summary>

```
fn sum_to(n: i64) -> i64 {
    var s: i64 = 0
    for i in 0..n { s += i }
    return s
}

test "sums" {
    expect(sum_to(4) == 6)
}

bench "sum to 1000" {
    sum_to(1000)
}
```

```
$ nx bench sums.nx
bench  sum to 1000  189 ns/iter  (min 188 ns, max 197 ns; 21 samples of 63856)

1 benchmark(s), safe mode
```

`nx test` führt die Tests aus; `nx bench` baut die Datei optimiert,
kalibriert die Iterationen und bewahrt den Wert des Blocks vor dem
Optimierer.

</details>

<details>
<summary><b>C aufrufen ist einen Header-Import entfernt</b></summary>

```
const libc = @cImport("string.h")
const cv = @cImport("cvendor.h")
artifact link { c_sources = ["cvendor.c"] }

unsafe { println("{}", .{libc.strlen(@cstr("hello"))}) }
```

Kein Binding-Generator, kein Build-Schritt: der Header ist die einzige Quelle
der Wahrheit für das Layout, und fremde Aufrufe tragen den Effekt `ffi`.

</details>

<details>
<summary><b>Parallele Schleifen und Arenen</b></summary>

```
for parallel p, i in positions {
    out[i] = integrate(p)          // hier ist kein shared_mutable erlaubt
}

using arena {
    var scratch = List(Frame).new()   // bump-allokiert, auf einmal freigegeben
    ...
}
```

</details>

**Dokumentation**

- [Spezifikation](../../../SPEC.md) (Englisch): die Sprache, wie sie implementiert ist, geplante Teile markiert.
- [Roadmap](../../../ROADMAP.md) (Englisch): Phasen, Abschlusskriterien und was nicht geplant ist.
- [Wie Nexium funktioniert](../../architecture.md) (Englisch): die Pipeline von der Quelle zum Binary, Effektinferenz, Besitz, die Laufzeit und das Ausliefern.
- [Sprachreferenz](../../language.md) (Englisch): jedes Konstrukt, das der Compiler implementiert.
- [Einbettung](../../embedding.md) (Englisch): ausgelieferte Bibliotheken aus Python, Rust und C aufrufen.
- [Die interaktive Sitzung](../../repl.md) (Englisch): `nx` an einer Eingabeaufforderung, wie `python`.
- [Stabilität](../../stability.md) und [Plattformen](../../platforms.md) (Englisch): was eine Version verspricht, der Deprecation-Zyklus, `nx fix`, die Stufen.
- [Der Topo](https://londopy.github.io/nexium/topo/01-base-camp.html) (Englisch): das Tutorial, von der Installation des Compilers bis zu einem neuronalen Netz, einer GUI und einer ausgelieferten Bibliothek; die Quelle liegt in [`topo/`](../../../topo/). Alles Obige, gerendert, steht auf [londopy.github.io/nexium](https://londopy.github.io/nexium/).
- [Installation](../../install.md) (Englisch): der Windows-Installer, das macOS/Linux-Skript, Bauen aus dem Quelltext, Prüfsummen, und wie `nx` einen C-Compiler findet.
- [Pakete](../../packages.md) (Englisch): `nexium.toml`, `nx add`, `nx fetch`, Git- oder Pfadabhängigkeiten, die Lock-Datei.
- [Standardbibliothek](../../std.md) (Englisch): die in Nexium geschriebenen Module (`std.strings`, `std.lists`, `std.bytes`, `std.num`, `std.json`, `std.args`, `std.fs`, `std.time`, `std.regex`, `std.text`, `std.testing`, `std.stream`, `std.net`, `std.http`, `std.thread`, `std.process`, `std.sort`, `std.heap`, `std.set`, `std.deque`, `std.hash`, `std.path`, `std.env`, `std.uuid`, `std.log`, `std.csv`, `std.toml`, `std.base64`).
- [Die Zahlen](https://londopy.github.io/nexium/docs/numbers.html) (Englisch): vier Programme in fünf Sprachen, wöchentlich auf einem Runner gemessen.
- [nexium-gui](../../gui.md) (Englisch): die Immediate-Mode-GUI-Bibliothek und wie man ein Widget schreibt.
- [Dein Programm veröffentlichen](../../releasing-your-program.md) (Englisch): Binaries für drei Plattformen aus einem Tag, Installer optional.
- [Editor-Unterstützung](../../../editors) (Englisch): VS Code, Vim, Neovim, Helix, Zed, Emacs, Kate, JetBrains, Sublime Text, Notepad++, nano, und `nx lsp` für alle anderen.
- [Linguist](../../../linguist) (Englisch): der fertige Pull Request, mit dem GitHub `.nx` erkennt, sobald die Nutzungsschwelle erreicht ist.
- [Übersetzungen](../README.md): dieses README in sechs Sprachen; die Sprachreferenz und der Architekturrundgang auf Spanisch, Chinesisch und Japanisch.
- [Release-Namen](../../release-names.md) (Englisch): jede Release ist ein Ort auf einem Berg; das Schema, das Verzeichnis und die noch unbenutzten Namen.
- [Entscheidungen](../../../DECISIONS.md) (Englisch): jede Entscheidung, die getroffen wurde, wo die Spezifikation offen war.
- [Bekannte Probleme](../../../KNOWN_ISSUES.md) (Englisch): offene Fehler, Lücken und Grenzen, mit Reproduktionen.

## Geschwindigkeit

Vier Programme, in fünf Sprachen gleich geschrieben, in den kompilierten
je etwa eine Sekunde lang, gemessen auf einem GitHub-Runner (2026-09-25; der
Median von sieben Läufen, bei über fünf Sekunden von dreien; in Sekunden,
kleiner ist besser):

| Programm | Nexium safe | Nexium fast | C | Rust | Go | Python |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `fib` (Aufrufe) | 1,32 | 0,78 | 0,39 | 0,78 | 1,34 | 28,87 |
| `nbody` (Gleitkomma) | 0,66 | 0,66 | 0,59 | 0,68 | 0,71 | 48,54 |
| `sieve` (Arrays) | 0,62 | 0,60 | 0,48 | 0,53 | 0,53 | 4,29 |
| `words` (Maps und Strings) | 1,16 | 1,11 | 0,57 | 0,97 | 1,12 | 3,60 |

Bei Gleitkomma und Arrays braucht Nexium höchstens ein Drittel länger als C:
bei Gleitkomma etwas schneller als Rust und Go, bei Arrays etwas langsamer. Bei
Aufrufen ist `fast` so schnell wie Rust, mit der doppelten Zeit von C; die
Überlaufprüfungen von `safe` kosten dort 70 % mehr. Maps und Strings laufen so
schnell wie in Go, mit etwa der doppelten Zeit von C. Python braucht 3- bis
74-mal so lange wie Nexium `fast`.

`safe` behält die Überlauf- und Grenzprüfungen, wie `nx ship` und `nx bench`
ohne Angabe bauen; `fast` (`--mode fast`) lässt sie weg. [Die
Zahlenseite](https://londopy.github.io/nexium/docs/numbers.html) (Englisch)
zeigt jede Zeit auch als Vielfaches der von C, die Versionen und die Regeln;
der Bench-Workflow misst jede Woche und bei jedem Release neu und schlägt
fehl, wenn Nexiums Zeit als Vielfaches der von C von einem Lauf zum nächsten
um ein Viertel wächst.

## In freier Wildbahn

Programme und Pakete außerhalb dieses Repositorys, die in Nexium geschrieben sind:

| Projekt | was Nexium dort tut |
| --- | --- |
| [statusmith](https://github.com/Londopy/statusmith), Discord Rich Presence aus der Taskleiste | sein SDK ist ein Nexium-Paket: `nx add discord_rpc --git https://github.com/Londopy/statusmith --tag sdk-v0.1.0 --dir nexium` setzt eine Presence aus jedem Nexium-Programm ([die Seite](../../discord.md)) |
| [Point of Origin](https://github.com/Londopy/point-of-origin), ein Plattformer, in dem der Boden das Rätsel ist | der ganze Build ist Nexium: `build.nx` treibt die DLL der Odin-Simulation, `tools/bindgen.nx` liest die Odin-Exporte und schreibt die C#-Bindings, die Unity aufruft, `tools/levels.nx` kompiliert die Levelkarten zum JSON, das das Spiel lädt (jedes Level von derselben Simulation gewachsen, also lösbar), `tools/chapters.nx` schreibt daraus die Dokumentation |
| [QNI](https://github.com/Londopy/qni), Netz-Erinnerungen, Check-in-Hilfe und ein Tutorial für die Netzleitung im Discord des Amateurfunkclubs der Cal Poly (W6BHZ) | das ganze Programm ist Nexium: Slash-Befehle und Buttons, über Webhooks beantwortet, ohne Bot-Benutzer und ohne Berechtigungen; jede Anfrage wird auf Discords Ed25519-Signatur geprüft, bevor irgendetwas gelesen wird (über nxtls); Netzkarten, ein Übungsmodus für die Netzleitung und Netzprotokolle im Tabellenformat der Vorstände; durchgehend gegen ein nachgebautes Discord getestet |
| [nxtls](https://github.com/Londopy/nxtls), Kryptografie und TLS 1.3 in reinem Nexium | SHA-2, HMAC, HKDF, X25519, ChaCha20-Poly1305, Signaturprüfung (Ed25519, ECDSA, RSA) und X.509-Ketten, darauf ein TLS-1.3-Client, ohne C und ohne `unsafe`, getestet gegen die Vektoren der Standards, Pythons `cryptography` und OpenSSL; QNI spricht darüber mit Discord; ein Paket: `nx add nxtls --git https://github.com/Londopy/nxtls --tag v0.4.0` |

Nexium irgendwo im Einsatz? Ein Issue oder ein Pull Request, und es steht hier.

## Befehle

| Befehl | was er tut |
| --- | --- |
| `nx build file.nx` | zu einer ausführbaren Datei kompilieren (oder zu einem Objekt, wenn es kein `main` gibt) |
| `nx run file.nx` | bauen und ausführen; `--watch` führt erneut aus, sobald sich eine Datei des Programms ändert |
| `nx test file.nx [filter]` | die `test "..."`-Blöcke ausführen; auch mit `--watch` |
| `nx check file.nx` | Typen prüfen und Effektverletzungen melden |
| `nx effects file.nx` | die inferierten Effekte jeder Funktion ausgeben |
| `nx explain file.nx f effect` | warum `f` den Effekt hat: die Aufrufe, die ihn hereintragen, bis zum Primitiv, als Baum |
| `nx audit file.nx` | `unsafe`-Blöcke und veränderliche Globale auflisten; `--lock` schreibt die Effekt-Lockdatei, `--check` schlägt bei einem hinzugekommenen Effekt fehl |
| `nx ship file.nx` | jedes deklarierte `artifact` erzeugen |
| `nx init`, `nx add`, `nx fetch`, `nx update` | das Manifest eines Pakets, Abhängigkeiten aus Git oder einem Pfad, die Lock-Datei ([docs/packages.md](../../packages.md)) |
| `nx emit-c file.nx` | das erzeugte C ausgeben |
| `nx tir file.nx [--sigs]` | das geprüfte Programm als S-Ausdrücke (die eigenen Tests des Compilers lesen es) |
| `nx fmt file.nx [--check]` | kanonische Formatierung |
| `nx debug file.nx` | für das Debuggen bauen und unter gdb oder lldb starten: `.nx`-Zeilen und Formatierer für String, List, Map, Slices und Optionals |
| `nx bench file.nx` | die `bench "name" { }`-Blöcke messen: ein optimierter Build, die mittlere Zeit (Median) pro Iteration |
| `nx fix file.nx` | die mechanischen Korrekturen des Checkers anwenden (`.clone()`, `@escape(...)`, `_ = `) und veraltete Formen migrieren; siehe [docs/stability.md](../../stability.md) |
| `nx doc file.nx` | HTML-Dokumentation mit inferierten Effekten |
| `nx size file.nx` | Bytes des Binarys den Deklarationen zuordnen |
| `nx layout file.nx [Type...]` | Offsets, Größen und Padding eines Structs oder Enums, und die Reihenfolge nach Ausrichtung, die es verkleinern würde |
| `nx upgrade` | die neueste Release anstelle dieser ausführbaren Datei, geprüft; `--check` meldet nur |
| `nx install [DIR]` | diese Kopie mit allem daneben an den Platz des Benutzers und in den PATH (das portable Zip installiert sich selbst) |
| `nx refcounts file.nx` | jede Retain- und Release-Stelle |
| `nx leaks file.nx` | mit Allokationsverfolgung ausführen und Lecks melden |
| `nx lsp` | Language Server über stdio |
| `nx doctor` | welcher C-Compiler verwendet wird und ob die Installation funktioniert |
| `nx version` | die Version und ihr Release-Name |
| `nx completions <shell>`, `nx man` | Vervollständigungen für bash, zsh, fish und PowerShell, und die Manpage |
| `nx repl`, oder einfach `nx` | eine interaktive Sitzung: Code tippen, Werte sehen, Bindungen behalten |
| `nx -e CODE`, `nx -p EXPR` | eine Zeile wie an der Eingabeaufforderung ausführen; `-p` gibt ihren Wert aus |

Optionen: `--mode debug|safe|fast|small`, `--target x86_64-linux-gnu` (jedes
Ziel, das `zig cc` kennt), `--cpu baseline|native|<name>` (standardmäßig
baseline, damit ein Binary auf jeder Maschine seiner Architektur läuft),
`--out-dir`, `--keep-c`, `--cc`, `--strict` (Warnungen sind
Fehler), `--sanitize address,undefined` (die Sanitizer des C-Compilers;
`address` braucht gcc oder clang), und für die C-Interoperabilität `-I`, `--link`,
`--link-path`, `--c-source`.

## Stand

**1.3: sprachstabil, die Werkzeugkette erwachsen.** Die Sprache ändert sich
nur durch Ergänzung, unter der [Stabilitätsrichtlinie](../../stability.md);
der Compiler ist in Nexium geschrieben und baut sich selbst; jedes Beispiel,
jeder Spezifikationsfall und jedes Tutorial-Programm läuft in CI auf drei
Plattformen, unter den Sanitizern und dem Fuzzer, und auch gdb und lldb
werden dort durch `nx debug` gesteuert. Speichersicherheit sind die
Sichtregeln, seit 1.3 Fehler. 1.4, eine Standardbibliothek, die niemand mehr
ergänzen muss, ist unterwegs: Collections (`std.sort`, `std.heap`, `std.set`,
`std.deque`), `std.hash`, `random.secure`, Pfade, Umgebung und
Konfigurationsordner, UUIDs, Logging, CSV, TOML und Base64 sind da, achtundzwanzig
Module insgesamt; `Map` ist gegen Hash-Flooding geschützt und behält die
Reihenfolge, in der die Schlüssel kamen; `std.time` liest Zeitzonen aus der
Datenbank der Plattform, und der Client von `std.http` spricht HTTPS über eine
TLS-Schicht, die nxtls stellt; als Nächstes kommen Websockets und ein
Discord-Paket. Was Nexium noch nicht ist, und wo jeder Punkt beantwortet wird,
steht in [einem Abschnitt der Roadmap](../../../ROADMAP.md#what-10-is-not-yet): die
Benchmarks sind vier Programme ([Geschwindigkeit](#geschwindigkeit)), und das
Ökosystem besteht aus einem Maintainer und vier Projekten außerhalb des
Baums ([oben](#in-freier-wildbahn)). [`KNOWN_ISSUES.md`](../../../KNOWN_ISSUES.md)
führt jeden offenen Fehler mit seiner Lösung;
[`DECISIONS.md`](../../../DECISIONS.md) jede Entscheidung, die getroffen
wurde, wo die Spezifikation offen war.

## Release-Namen

Jede Hauptversion ist der Gipfel eines Berges, in der Reihenfolge, in der
die vierzehn Achttausender erstbestiegen wurden: `X.0.0` ist
`<Berg>: Summit`. Der Aufstieg beginnt in der Mitte der Linie davor, bei
ihrer `.5`, und folgt der Route der Erstbesteigung Lager für Lager; die
Nebenversionen nach einem Gipfel, bis `.4`, sind die anderen Routen des
Berges und der Abstieg; Patches tragen die Namen der Mitglieder der
Erstbesteigungsexpedition. Die 0.x-Linie war der Anmarsch und die Lager der
Annapurna, des ersten bestiegenen Achttausenders (1950), also ist 1.0.0
`Annapurna: Summit`; 0.7.0, wo der Compiler begann, sich selbst zu bauen,
ist `Annapurna: Camp V`, das letzte Lager vor dem Gipfelgang, und ab 1.5
steigen die Releases auf den Everest, zu 2.0.0.
Der Name steht im Changelog, im Release-Titel und
in `nx version`; [docs/release-names.md](../../release-names.md) (Englisch)
enthält die Regel, das Verzeichnis und die Berge, die noch zu besteigen sind.

## Selbsthosting

Der Compiler ist in Nexium geschrieben, unter [`self/`](../../../self), und
baut sich selbst. Eine Maschine ohne `nx` baut eines aus
[`bootstrap/nx.c`](../../../bootstrap/nx.c), dem C, das der Compiler für sich
selbst ausgibt, mit einem beliebigen C-Compiler und ohne Rust:

```sh
sh bootstrap/build.sh     # nx.c -> nx0; nx0 baut self/nx.nx -> nx1; nx1 baut sich zum selben C neu -> nx2
```

| Stufe | Datei | Aufgabe |
| --- | --- | --- |
| Lexer | [`self/lexer.nx`](../../../self/lexer.nx) | Tokens |
| Parser | [`self/parser.nx`](../../../self/parser.nx) | ein Syntaxbaum in einer Id-Arena |
| Prüfer | [`self/check.nx`](../../../self/check.nx), `self/check_*.nx`, [`self/cimport.nx`](../../../self/cimport.nx) | Typen, Effekte, Besitz, Generics, der Compile-Zeit-Interpreter, C-Header-Import, jede Diagnose |
| C-Emitter | [`self/cgen.nx`](../../../self/cgen.nx) | eine C-Datei pro Programm, oder eine pro Modul beim Debug-Build eines großen Programms, die unveränderten wiederverwendet |
| Treiber | [`self/nx.nx`](../../../self/nx.nx) | build, run, test, bench, debug, check, emit-c, tir; die Standardbibliothek eingebettet |
| Werkzeuge | [`self/fmt.nx`](../../../self/fmt.nx), [`self/doc.nx`](../../../self/doc.nx), [`self/tools.nx`](../../../self/tools.nx), [`self/size.nx`](../../../self/size.nx), [`self/manifest.nx`](../../../self/manifest.nx), [`self/ship.nx`](../../../self/ship.nx), [`self/lsp.nx`](../../../self/lsp.nx), [`self/lsp_index.nx`](../../../self/lsp_index.nx), [`self/fix.nx`](../../../self/fix.nx), [`self/repl.nx`](../../../self/repl.nx) | der Formatierer, der Dokumentationsgenerator, die Berichte, Pakete, `ship`, der Language Server und sein Index des geprüften Programms, `nx fix`, die REPL |

Jedes Beispiel, jeder Spezifikationsfall und jeder Compile-Fail-Fall läuft
durch den gebootstrappten Compiler, gesteuert von einem Test-Harness, der
selbst ein Nexium-Programm ist (`nx run tests/run.nx`), in CI auf drei
Plattformen ganz ohne Rust-Toolchain. Der erste Compiler, in Rust, trieb die
Portierung voran und wurde in 1.0 gelöscht (Entscheidung 90).

## Sprachen im Repository

Nicht leere Codezeilen, ohne Build-Ausgabe, Abhängigkeiten und generierte
Dateien (`bootstrap/nx.c`, der tree-sitter-Parser, `gui/font.bin`,
Lock-Dateien):

| Sprache | Zeilen | Anteil | was es ist |
| --- | --- | --- | --- |
| Nexium | 48.804 | 87,2 % | der Compiler und seine Werkzeuge (33.000 Zeilen unter `self/`), die Standardbibliothek (21 Module), der Test-Harness und der Fuzzer, Beispiele, die Programme des Tutorials, nexium-gui, der Site-Generator, vier Benchmarks |
| C | 2.542 | 4,5 % | die Laufzeit `nx_rt.h`, die GUI-Fensterschicht, mitgeliefertes Test-C, ein Benchmark |
| Python | 1.492 | 2,7 % | die Release-Skripte (Notes, Paketmanifeste, Wheels und npm-Pakete, die std-Dokumentation), die gdb- und lldb-Formatierer, der Benchmark-Runner und vier Benchmarks |
| Editor-Dateien | 1.103 | 2,0 % | tree-sitter-Queries, Emacs Lisp, Vim-Script, Lua für Neovim, ein Pygments-Lexer und die 25 Zeilen Rust, die Zed von einer Erweiterung verlangt |
| JavaScript, TypeScript | 939 | 1,7 % | die VS-Code-Erweiterung, die tree-sitter-Grammatik und die WASI-Schicht des Playgrounds |
| Inno Setup, Shell, PowerShell | 855 | 1,5 % | das Skript des Windows-Installers, `install.sh`, `install.ps1`, die Chocolatey-Skripte, die Bootstrap-Skripte |
| Rust, Go, Ruby | 213 | 0,4 % | je vier Benchmarks in Rust und Go, und die Homebrew-Formel |

Im Compiler steckt kein Rust: der erste Compiler trieb die Portierung voran
und wurde in 1.0 gelöscht (Entscheidung 90). Das verbliebene Rust ist der
Kleber der Zed-Erweiterung, den Zed zu WebAssembly kompiliert, und vier
Benchmark-Programme, geschrieben, um sich daran zu messen, neben ihren
Go-Zwillingen. Zig steht nicht
in der Tabelle, weil es im Baum keine Zig-Quellen gibt: `zig cc` ist der
C-Compiler, den `nx` aufruft (vom Windows-Installer mitgeliefert, vom
Installationsskript geladen), so wie ein C-Compiler benutzt und nicht
geschrieben wird.

## Aufbau

```
bootstrap/      das C-Saatkorn, aus dem der Compiler gebaut wird, und die Build-Skripte
runtime/        nx_rt.h, eingebettet in jede erzeugte C-Datei; die gdb- und lldb-Formatierer von nx debug
std/            die Standardbibliothek in Nexium, eingebettet in den Compiler
self/           der Compiler in Nexium, Stufe für Stufe
gui/            nexium-gui: Immediate-Mode-GUI in Nexium, Demo und die C-Plattformschicht
editors/        VS-Code-Erweiterung, tree-sitter-Grammatik und die Dateien für zehn weitere Editoren
examples/       Programme mit aufgezeichneter Ausgabe, von den Tests ausgeführt
topo/           das Tutorial: die Kapitel und die Programme, die sie zeigen (von den Tests ausgeführt)
site/           der Generator der Dokumentationsseite, ein Nexium-Programm
tests/          der Harness (run.nx), die Konformitätssuite der Spezifikation (tests/spec), die Compile-Fail-Fälle, die Debugger-Prüfung
docs/           wie es funktioniert, Sprachreferenz, Einbettungsleitfaden, Übersetzungen in i18n/
bench/          vier Programme in fünf Sprachen hinter der Zahlenseite
installers/     das Skript des Windows-Installers, install.sh und install.ps1, die winget- und Chocolatey-Manifeste
docker/         die Compiler-Images für ghcr.io (Debian und Alpine)
Formula/, bucket/  dieses Repository als Homebrew-Tap und Scoop-Bucket (bei jeder Release geschrieben)
scripts/        Release-Notes, Paketmanifeste, Wheels und npm-Pakete, die std-Dokumentation
assets/         Logo, Banner und die Social-Vorschau
nexium-spec.txt          der Entwurf
nexium-systems-spec.txt  die archivierte Systemsprache; Abschnitte 4 bis 9 sind die Syntaxreferenz
DECISIONS.md    Entscheidungen, wo die Spezifikation offen war
KNOWN_ISSUES.md offene Fehler und Grenzen; Lösungen wandern in den Changelog
```

## Mitwirken

Siehe [`CONTRIBUTING.md`](../../../CONTRIBUTING.md). Fehler und Vorschläge
laufen über GitHub-Issues; eine Sprachänderung muss die harte Einschränkung
aus Abschnitt 3 der Spezifikation nennen, der sie dient. Pull Requests
bestehen vor dem Merge die Tests auf drei Plattformen, die Formatierer, eine
Changelog-Prüfung und das [Contributor License Agreement](../../../CLA.md);
das Urheberrecht bleibt bei dir.

## Lizenz

MIT. Copyright (c) 2026 Londopy.

<p align="center"><img src="https://raw.githubusercontent.com/Londopy/nexium/main/assets/logo.svg" alt="" width="48"></p>
