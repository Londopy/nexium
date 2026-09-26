<p align="center">
  <img src="https://raw.githubusercontent.com/Londopy/nexium/main/assets/banner.svg" alt="Nexium" width="880">
</p>

<p align="center">
  <a href="../../../README.md">English</a> ·
  <a href="../es/README.md">Español</a> ·
  <a href="../zh-CN/README.md">简体中文</a> ·
  <a href="../ja/README.md">日本語</a> ·
  <a href="../ko/README.md">한국어</a> ·
  <b>Français</b> ·
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
  <b>Nexium est un langage assez complet pour tout construire, et qui est aussi le meilleur choix pour une seule pièce d'autre chose.</b>
</p>

<p align="center">
  <a href="https://londopy.github.io/nexium/"><b>Documentation et le tutoriel « le Topo » &rarr; londopy.github.io/nexium</b></a><br>
  <sub><a href="https://londopy.github.io/nexium/topo/01-base-camp.html">Commencer par le Topo</a> &middot; <a href="https://londopy.github.io/nexium/docs/language.html">Référence du langage</a> &middot; <a href="https://londopy.github.io/nexium/docs/install.html">Installation</a> &middot; <a href="https://londopy.github.io/nexium/docs/std.html">Bibliothèque standard</a> &middot; <a href="https://londopy.github.io/nexium/docs/embedding.html">Intégration</a> (en anglais)</sub>
</p>

Nexium compile en code natif via C, possède un comptage de références
automatique sans ramasse-miettes, un système d'effets vérifié par la machine
qui dit si une fonction alloue, bloque ou peut paniquer, et un compilateur
qui transforme une seule arborescence de sources en bibliothèque C, en wheel
Python, en crate Rust ou en outil en ligne de commande.

<table>
<tr>
<td width="50%" valign="top">

**Un fichier**

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

**Toutes les cibles**

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

La signature d'effets décide de l'ABI C : `checksum` est prouvée `!panics`,
elle reçoit donc un simple `uint32_t checksum(const uint8_t*, size_t)`. Une
fonction qui peut échouer renvoie un code d'état, et une panique Nexium en son
sein est convertie à la frontière au lieu d'interrompre le processus hôte.

## L'essentiel

| | |
| --- | --- |
| 🧾 **Effets inférés et vérifiés** | `allocates` `refcounts` `blocks` `shared_mutable` `nondeterministic` `panics` `ffi`. Déclarez `!allocates` et le compilateur désigne la ligne exacte qui le romprait, à travers les appels. |
| 🛡 **La sûreté mémoire sans ramasse-miettes** | Les collections se déplacent, `.clone()` copie, les valeurs `ref class` comptent leurs références, `weak` brise les cycles, et une tranche ou un pointeur ne survit jamais au stockage qu'il désigne : les règles de vues sont vérifiées à travers les appels, les boucles et les branches, sans durées de vie à écrire. Là où la correction est mécanique, `nx fix` la fait. |
| 🔬 **Motifs binaires** | `<<version:4, ihl:4, len:16/big, rest:bytes>>` reconnaît et construit des paquets aux tailles vérifiées. |
| 🧵 **Boucles parallèles, arènes, objets de trait** | `for parallel`, `using arena { }`, `dyn Trait !allocates`. |
| 🔌 **Du C sans bindings** | `@cImport("header.h")` lit l'en-tête directement ; `artifact link` compile du C embarqué dans le programme ; `if comptime @target().0 == "windows"` ne construit que la branche que prend la plateforme. |
| 📦 **Livrer depuis une seule source** | `nx ship` produit des en-têtes et bibliothèques C, des wheels Python et des crates Rust avec des enveloppes sûres. |
| 🐞 **Le déboguer, le mesurer** | `nx debug` arrête gdb ou lldb sur des lignes `.nx` et montre chaînes, listes, maps et optionnels comme des valeurs ; les blocs `bench "name" { }` côtoient les tests ; `--sanitize address,undefined` place AddressSanitizer et UBSan sous n'importe quelle compilation. |
| 🧭 **L'apprendre dans le navigateur** | [Le Topo](https://londopy.github.io/nexium/topo/01-base-camp.html), le tutoriel, exécute ses programmes dans la page, avec le compilateur compilé en WebAssembly, et note ses exercices comme `nx topo` le fait dans le terminal. `nx repl` est une invite. |
| 🪞 **Écrit en lui-même** | Le compilateur est en Nexium, construit à partir d'un seul fichier C par n'importe quel compilateur C ; la compilation de débogage d'un gros programme ne recompile que les modules modifiés. |
| 🖼 **Une GUI, en Nexium** | [`gui/`](../../../gui) : une GUI en mode immédiat (boutons, curseurs, champs de texte) avec un rastériseur logiciel et une police bitmap, tout en Nexium au-dessus d'une couche fenêtre C de 200 lignes. |
| 🛠 **Outils inclus** | `fmt`, `fix`, `doc`, `lsp` (définition, survol et renommage depuis le vérificateur), `debug`, `bench`, `size`, `layout`, `leaks`, `refcounts`, `effects`, `explain`, `audit`, `repl`. Zéro dépendance. |

## Installation

**Windows** : téléchargez et lancez l'installateur depuis la page
[Releases](https://github.com/Londopy/nexium/releases). Il installe `nx`, une
chaîne d'outils Zig embarquée (le compilateur C que `nx` utilise), la
bibliothèque standard, les exemples, la documentation et l'extension VS Code,
et ajoute `nx` à votre PATH. Rien d'autre à installer.

**macOS et Linux** :

```bash
curl -fsSL https://raw.githubusercontent.com/Londopy/nexium/main/installers/install.sh | sh
```

Le script vérifie le téléchargement contre les sommes de contrôle de la
release, installe dans `~/.nexium`, met en place un compilateur C (les outils
Xcode sur macOS ; sur Linux, Zig est téléchargé quand rien n'est trouvé) et
ajoute `nx` à votre PATH.

**Windows, depuis PowerShell** : `irm https://raw.githubusercontent.com/Londopy/nexium/main/installers/install.ps1 | iex`
(la build portable, vérifiée, sur le PATH ; sans assistant).

**pip ou npm** : `pip install nexium-lang` ([PyPI](https://pypi.org/project/nexium-lang/)) ou `npm install -g nexium-lang` ([npm](https://www.npmjs.com/package/nexium-lang)) ; le binaire, par plateforme ; un compilateur C comme d'habitude.

**Docker** : `docker run --rm -v "$PWD":/work ghcr.io/londopy/nexium run hello.nx`
(Debian ; `:alpine` aussi ; amd64 et arm64).

**Chocolatey et winget** : `choco install nexium` ([le paquet](https://community.chocolatey.org/packages/nexium)), chaque version dès que les modérateurs de Chocolatey l'ont approuvée (elle peut avoir quelques jours de retard sur la dernière version), et `winget install Londopy.Nexium` dès que winget aura accepté sa première version ([l'état](../../install.md#where-to-get-it), en anglais).

**Debian, RPM, Nix, mise** : chaque version joint des paquets `.deb` et `.rpm` (`sudo dpkg -i nexium_*_amd64.deb`) ; `nix run github:Londopy/nexium` le construit à partir de l'unique fichier C ; `mise use -g "ubi:Londopy/nexium[exe=nx]"` installe le binaire publié. Chaque fichier publié porte une provenance signée : `gh attestation verify nx --owner Londopy`. [Toutes les voies](../../install.md#where-to-get-it) (anglais).

**Dans le navigateur** : [ouvrez le dépôt dans un Codespace](https://codespaces.new/Londopy/nexium) et `nx run examples/hello.nx` tourne en une minute, sans rien installer.

**Dans un notebook** : dans [Google Colab](https://colab.research.google.com) ou Jupyter, `!pip install -q nexium-lang`, puis `!nx run hello.nx` sur un fichier écrit par une cellule `%%writefile` ([les trois cellules](../../install.md#in-a-notebook-colab-and-jupyter), en anglais).

**Homebrew et Scoop** : le dépôt est son propre tap, et le bucket Scoop est [Londopy/scoop-bucket](https://github.com/Londopy/scoop-bucket), tenu à jour par l'outil de mise à jour de Scoop lui-même.

```bash
brew tap londopy/tap https://github.com/Londopy/nexium && brew install londopy/tap/nexium
```

```powershell
scoop bucket add londopy https://github.com/Londopy/scoop-bucket && scoop install nexium
```

Ensuite, dans une nouvelle console, `nx doctor` montre ce qui sera utilisé.
Tous les détails, y compris la vérification des sommes et chaque variable
d'environnement, sont dans [docs/install.md](../../install.md) (anglais).

Ou compilez depuis les sources avec rien d'autre qu'un compilateur C (Zig sur
le PATH, ou `CC`), ce qui construit le compilateur écrit en Nexium à partir
de sa graine en C :

```bash
git clone https://github.com/Londopy/nexium && cd nexium && sh bootstrap/build.sh
```

Le résultat est `nx-out/bootstrap/nx2` (`build.ps1` sous Windows). Ensuite :

```bash
nx run examples/hello.nx
```

## Un tour d'horizon

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
<summary><b>Reconnaissance de motifs binaires</b></summary>

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
<summary><b>Les effets sont inférés et vérifiés</b></summary>

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
<summary><b>Une vue ne survit jamais à son stockage</b></summary>

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

Une tranche ou un pointeur est vérifié par rapport au stockage qu'il désigne
(SPEC 5.6, règles V1 à V5) : pas de ramasse-miettes, et pas de durées de vie
à écrire.

</details>

<details>
<summary><b>Tests et benchmarks côte à côte</b></summary>

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

`nx test` lance les tests ; `nx bench` compile le fichier optimisé, calibre
les itérations et protège la valeur du bloc de l'optimiseur.

</details>

<details>
<summary><b>Appeler du C tient en un import d'en-tête</b></summary>

```
const libc = @cImport("string.h")
const cv = @cImport("cvendor.h")
artifact link { c_sources = ["cvendor.c"] }

unsafe { println("{}", .{libc.strlen(@cstr("hello"))}) }
```

Pas de générateur de bindings, pas d'étape de build : l'en-tête est l'unique
source de vérité pour la disposition mémoire, et les appels étrangers portent
l'effet `ffi`.

</details>

<details>
<summary><b>Boucles parallèles et arènes</b></summary>

```
for parallel p, i in positions {
    out[i] = integrate(p)          // pas de shared_mutable autorisé ici
}

using arena {
    var scratch = List(Frame).new()   // allocation bump, libérée d'un coup
    ...
}
```

</details>

**Documentation**

- [Spécification](../../../SPEC.md) (anglais) : le langage tel qu'implémenté, les parties prévues marquées.
- [Feuille de route](../../../ROADMAP.md) (anglais) : phases, critères de sortie, et ce qui n'est pas prévu.
- [Comment Nexium fonctionne](../../architecture.md) (anglais) : le pipeline de la source au binaire, l'inférence d'effets, la propriété, le runtime et la livraison.
- [Référence du langage](../../language.md) (anglais) : chaque construction que le compilateur implémente.
- [Intégration](../../embedding.md) (anglais) : appeler les bibliothèques livrées depuis Python, Rust et C.
- [La session interactive](../../repl.md) (anglais) : `nx` à une invite, comme `python`.
- [Stabilité](../../stability.md) et [plateformes](../../platforms.md) (anglais) : ce qu'une version promet, le cycle de dépréciation, `nx fix`, les niveaux.
- [Le Topo](https://londopy.github.io/nexium/topo/01-base-camp.html) (anglais) : le tutoriel, de l'installation du compilateur à un réseau de neurones, une GUI et une bibliothèque livrée ; la source est dans [`topo/`](../../../topo/). Tout ce qui précède, rendu, est sur [londopy.github.io/nexium](https://londopy.github.io/nexium/).
- [Installation](../../install.md) (anglais) : l'installateur Windows, le script macOS/Linux, la compilation depuis les sources, les sommes de contrôle, et comment `nx` trouve un compilateur C.
- [Paquets](../../packages.md) (anglais) : `nexium.toml`, `nx add`, `nx fetch`, dépendances git ou par chemin, le fichier de verrouillage.
- [Bibliothèque standard](../../std.md) (anglais) : les modules écrits en Nexium (`std.strings`, `std.lists`, `std.bytes`, `std.num`, `std.json`, `std.args`, `std.fs`, `std.time`, `std.regex`, `std.text`, `std.testing`, `std.stream`, `std.net`, `std.http`, `std.thread`, `std.process`, `std.sort`, `std.heap`, `std.set`, `std.deque`, `std.hash`, `std.path`, `std.env`, `std.uuid`, `std.log`, `std.csv`, `std.toml`, `std.base64`, `std.websocket`).
- [Les chiffres](https://londopy.github.io/nexium/docs/numbers.html) (anglais) : quatre programmes en cinq langages, mesurés chaque semaine sur un même runner.
- [nexium-gui](../../gui.md) (anglais) : la bibliothèque GUI en mode immédiat et comment écrire un widget.
- [Publier votre programme](../../releasing-your-program.md) (anglais) : des binaires pour trois plateformes à partir d'un tag, installateurs en option.
- [Prise en charge des éditeurs](../../../editors) (anglais) : VS Code, Vim, Neovim, Helix, Zed, Emacs, Kate, JetBrains, Sublime Text, Notepad++, nano, et `nx lsp` pour les autres.
- [Linguist](../../../linguist) (anglais) : la pull request prête à appliquer qui fera reconnaître `.nx` par GitHub une fois le seuil d'usage atteint.
- [Traductions](../README.md) : ce README en six langues ; la référence du langage et le tour de l'architecture en espagnol, chinois et japonais.
- [Noms des versions](../../release-names.md) (anglais) : chaque version est un lieu sur une montagne ; le schéma, le registre et les noms encore à utiliser.
- [Décisions](../../../DECISIONS.md) (anglais) : chaque choix fait là où la spécification était ouverte.
- [Problèmes connus](../../../KNOWN_ISSUES.md) (anglais) : bogues ouverts, manques et limites, avec reproductions.

## Vitesse

Quatre programmes écrits de la même façon dans cinq langages, d'environ une
seconde chacun dans les langages compilés, mesurés sur un runner GitHub
(2026-09-25 ; la médiane de sept exécutions, ou de trois au-delà de cinq secondes ;
en secondes, moins c'est mieux) :

| programme | Nexium safe | Nexium fast | C | Rust | Go | Python |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `fib` (appels) | 1,32 | 0,78 | 0,39 | 0,78 | 1,34 | 28,87 |
| `nbody` (flottants) | 0,66 | 0,66 | 0,59 | 0,68 | 0,71 | 48,54 |
| `sieve` (tableaux) | 0,62 | 0,60 | 0,48 | 0,53 | 0,53 | 4,29 |
| `words` (tables et chaînes) | 1,16 | 1,11 | 0,57 | 0,97 | 1,12 | 3,60 |

En flottants et en tableaux, Nexium prend au plus un tiers de temps de plus que
C : un peu plus rapide que Rust et Go en flottants, un peu plus lent en
tableaux. En appels, `fast` égale Rust avec deux fois le temps de C, et les
contrôles de débordement de `safe` y ajoutent 70 %. Les tables et les chaînes
vont au rythme de Go, environ deux fois le temps de C. Python met de 3 à 74 fois
plus de temps que Nexium `fast`.

`safe` garde les contrôles de débordement et de bornes, comme `nx ship` et
`nx bench` construisent sauf indication contraire ; `fast` (`--mode fast`)
les omet. [La page des chiffres](https://londopy.github.io/nexium/docs/numbers.html) (en anglais) donne aussi chaque temps en multiple de
celui de C, les versions et les règles ; le workflow Bench remesure chaque
semaine et à chaque version, et échoue quand le temps de Nexium, en multiple
de celui de C, augmente d'un quart d'une mesure à l'autre.

## Dans la nature

Programmes et paquets écrits en Nexium hors de ce dépôt :

| projet | ce que Nexium y fait |
| --- | --- |
| [statusmith](https://github.com/Londopy/statusmith), la Rich Presence de Discord depuis la barre des tâches | son SDK est un paquet Nexium : `nx add discord_rpc --git https://github.com/Londopy/statusmith --tag sdk-v0.1.0 --dir nexium` règle une présence depuis n'importe quel programme Nexium ([la page](../../discord.md)) |
| [Point of Origin](https://github.com/Londopy/point-of-origin), un jeu de plateforme où l'énigme est le sol | toute la construction est en Nexium : `build.nx` pilote la DLL de la simulation en Odin, `tools/bindgen.nx` lit les exports Odin et écrit les bindings C# qu'appelle Unity, `tools/levels.nx` compile les cartes des niveaux en JSON chargé par le jeu (chaque niveau poussé par la même simulation, donc soluble), `tools/chapters.nx` en tire la documentation |
| [QNI](https://github.com/Londopy/qni), rappels de réseaux, aide au check-in et un tutoriel de contrôle de réseau pour le Discord du club de radioamateurs de Cal Poly (W6BHZ) | tout le programme est en Nexium : commandes slash et boutons servis par webhooks, sans utilisateur bot ni permissions ; chaque requête vérifiée contre la signature Ed25519 de Discord avant toute lecture (grâce à nxtls) ; fiches de réseau, un mode d'entraînement au contrôle de réseau et des journaux au format de tableur des responsables ; testé de bout en bout contre un faux Discord |
| [nxtls](https://github.com/Londopy/nxtls), de la cryptographie et TLS 1.3 en Nexium pur | SHA-2, HMAC, HKDF, X25519, ChaCha20-Poly1305, vérification de signatures (Ed25519, ECDSA, RSA) et chaînes X.509, et par-dessus un client TLS 1.3, sans C et sans `unsafe`, testés contre les vecteurs des normes, le `cryptography` de Python et OpenSSL ; QNI parle à Discord à travers lui ; un paquet : `nx add nxtls --git https://github.com/Londopy/nxtls --tag v0.4.0` |

Vous utilisez Nexium quelque part ? Ouvrez une issue ou une pull request et il apparaîtra ici.

## Commandes

| commande | ce qu'elle fait |
| --- | --- |
| `nx build file.nx` | compile en exécutable (ou en objet quand il n'y a pas de `main`) |
| `nx run file.nx` | compile et exécute ; `--watch` relance dès qu'un fichier du programme change |
| `nx test file.nx [filter]` | exécute les blocs `test "..."` ; `--watch` aussi |
| `nx check file.nx` | vérifie les types et signale les violations d'effets |
| `nx effects file.nx` | affiche les effets inférés de chaque fonction |
| `nx explain file.nx f effect` | pourquoi `f` a l'effet : les appels qui l'apportent, jusqu'à la primitive, sous forme d'arbre |
| `nx audit file.nx` | liste les blocs `unsafe` et les globales mutables ; `--lock` écrit le fichier de verrouillage des effets, `--check` échoue sur un effet gagné |
| `nx ship file.nx` | produit chaque `artifact` déclaré |
| `nx init`, `nx add`, `nx fetch`, `nx update` | le manifeste d'un paquet, les dépendances depuis git ou un chemin, le fichier de verrouillage ([docs/packages.md](../../packages.md)) |
| `nx emit-c file.nx` | affiche le C généré |
| `nx tir file.nx [--sigs]` | le programme vérifié en S-expressions (les tests du compilateur le lisent) |
| `nx fmt file.nx [--check]` | formatage canonique |
| `nx debug file.nx` | compile pour le débogage et lance sous gdb ou lldb : lignes `.nx` et formateurs pour String, List, Map, tranches et optionnels |
| `nx bench file.nx` | mesure les blocs `bench "name" { }` : une compilation optimisée, le temps médian par itération |
| `nx fix file.nx` | applique les corrections mécaniques du vérificateur (`.clone()`, `@escape(...)`, `_ = `) et migre les formes dépréciées ; voir [docs/stability.md](../../stability.md) |
| `nx doc file.nx` | documentation HTML avec les effets inférés |
| `nx size file.nx` | attribue les octets du binaire aux déclarations |
| `nx layout file.nx [Type...]` | décalages, tailles et remplissage d'un struct ou d'un enum, et l'ordre par alignement qui le réduirait |
| `nx upgrade` | la dernière release à la place de cet exécutable, vérifiée ; `--check` se contente de signaler |
| `nx install [DIR]` | cette copie, avec ce qui l'accompagne, à la place de l'utilisateur et sur le PATH (le zip portable qui s'installe lui-même) |
| `nx refcounts file.nx` | chaque site de retain et de release |
| `nx leaks file.nx` | exécute avec suivi des allocations et signale les fuites |
| `nx lsp` | serveur de langage sur stdio |
| `nx doctor` | quel compilateur C sera utilisé, et si l'installation fonctionne |
| `nx version` | la version et son nom |
| `nx completions <shell>`, `nx man` | les complétions pour bash, zsh, fish et PowerShell, et la page de manuel |
| `nx repl`, ou simplement `nx` | une session interactive : tapez du code, voyez les valeurs, gardez les liaisons |
| `nx -e CODE`, `nx -p EXPR` | une ligne exécutée comme à l'invite ; `-p` affiche sa valeur |

Options : `--mode debug|safe|fast|small`, `--target x86_64-linux-gnu` (toute
cible connue de `zig cc`), `--cpu baseline|native|<nom>` (baseline par défaut,
pour qu'un binaire tourne sur toute machine de son architecture), `--out-dir`,
`--keep-c`, `--cc`, `--strict` (les avertissements sont des
erreurs), `--sanitize address,undefined` (les sanitizers du compilateur C ;
`address` demande gcc ou clang), et pour l'interopérabilité C `-I`, `--link`, `--link-path`,
`--c-source`.

## État

**1.4 : une bibliothèque standard qu'on n'a plus besoin de compléter.** Le
langage ne change que par ajout, sous la [politique de stabilité](../../stability.md) ;
le compilateur est écrit en Nexium et se construit lui-même ; chaque exemple,
chaque cas de la spécification et chaque programme du tutoriel s'exécute en
CI sur trois plateformes, sous les sanitizers et le fuzzer, et gdb et lldb y
sont aussi pilotés par `nx debug`. La sûreté mémoire, ce sont les règles de
vues, des erreurs depuis 1.3. La bibliothèque standard compte vingt-neuf
modules, dont les collections (`std.sort`, `std.heap`, `std.set`,
`std.deque`), `std.hash`, `random.secure`, les chemins, l'environnement et
les dossiers de configuration, les UUID, la journalisation, CSV, TOML et
base64 ; `Map` est protégé contre l'inondation de hachages et garde l'ordre
d'arrivée des clés ; `std.time` lit les fuseaux horaires dans la base de la
plateforme ; `std.http` parle HTTPS à travers le TLS de la plateforme, ou une
couche TLS comme nxtls, comme `std.websocket`, et
[nexium-discord](https://github.com/Londopy/nexium-discord) bâtit dessus un bot Discord ; `std.process`
parle aux programmes pendant qu'ils tournent, `std.thread` a select, les
opérations atomiques et des fils qui finissent avant l'appel, `std.text` les
grappes de graphèmes et la casse d'Unicode, et `std.testing` des tests de
propriétés qui réduisent ce qu'ils trouvent. Vient ensuite la 1.5, les
plateformes. Ce que Nexium n'est pas
encore, et où chaque point trouve sa réponse, est dans [une section de la
feuille de route](../../../ROADMAP.md#what-10-is-not-yet) : les mesures de performance
sont quatre programmes ([Vitesse](#vitesse)), et l'écosystème se résume à un
seul mainteneur et quatre projets hors de l'arbre ([plus haut](#dans-la-nature)).
[`KNOWN_ISSUES.md`](../../../KNOWN_ISSUES.md) liste chaque bogue ouvert avec
son correctif ; [`DECISIONS.md`](../../../DECISIONS.md), chaque choix fait
là où la spécification était ouverte.

## Noms des versions

Chaque version majeure est le sommet d'une montagne, dans l'ordre où les
quatorze sommets de 8 000 mètres ont été gravis pour la première fois :
`X.0.0` est `<Montagne>: Summit`. L'ascension commence au milieu de la ligne
précédente, à son `.5`, et suit la voie de la première ascension camp par
camp ; les versions mineures après un sommet, jusqu'à `.4`, sont les autres
voies de la montagne et la descente ; les correctifs portent les noms des
membres de l'expédition de la première ascension. La ligne 0.x a été
l'approche et les camps de l'Annapurna, le premier 8 000 gravi (1950), donc
1.0.0 est `Annapurna: Summit` ; 0.7.0, où le compilateur a commencé à se
construire lui-même, est `Annapurna: Camp V`, le dernier camp avant l'assaut
du sommet, et à partir de 1.5 les versions gravissent l'Everest vers 2.0.0.
Le nom figure dans le changelog, dans le titre de la release et dans
`nx version` ; [docs/release-names.md](../../release-names.md) (anglais)
donne la règle, le registre et les montagnes qu'il reste à gravir.

## Auto-hébergement

Le compilateur est écrit en Nexium, sous [`self/`](../../../self), et se
construit lui-même. Une machine sans `nx` en construit un à partir de
[`bootstrap/nx.c`](../../../bootstrap/nx.c), le C que le compilateur émet
pour lui-même, avec n'importe quel compilateur C et sans Rust :

```sh
sh bootstrap/build.sh     # nx.c -> nx0 ; nx0 construit self/nx.nx -> nx1 ; nx1 se reconstruit en le même C -> nx2
```

| étape | fichier | rôle |
| --- | --- | --- |
| lexer | [`self/lexer.nx`](../../../self/lexer.nx) | jetons |
| parser | [`self/parser.nx`](../../../self/parser.nx) | un arbre syntaxique dans une arène d'identifiants |
| vérificateur | [`self/check.nx`](../../../self/check.nx), `self/check_*.nx`, [`self/cimport.nx`](../../../self/cimport.nx) | types, effets, propriété, génériques, l'interprète à la compilation, l'import d'en-têtes C, chaque diagnostic |
| émetteur C | [`self/cgen.nx`](../../../self/cgen.nx) | un fichier C par programme, ou un par module pour la compilation de débogage d'un gros programme, les inchangés réutilisés |
| pilote | [`self/nx.nx`](../../../self/nx.nx) | build, run, test, bench, debug, check, emit-c, tir ; la bibliothèque standard embarquée |
| outils | [`self/fmt.nx`](../../../self/fmt.nx), [`self/doc.nx`](../../../self/doc.nx), [`self/tools.nx`](../../../self/tools.nx), [`self/size.nx`](../../../self/size.nx), [`self/manifest.nx`](../../../self/manifest.nx), [`self/ship.nx`](../../../self/ship.nx), [`self/lsp.nx`](../../../self/lsp.nx), [`self/lsp_index.nx`](../../../self/lsp_index.nx), [`self/fix.nx`](../../../self/fix.nx), [`self/repl.nx`](../../../self/repl.nx) | le formateur, le générateur de documentation, les rapports, les paquets, `ship`, le serveur de langage et son index du programme vérifié, `nx fix`, le REPL |

Chaque exemple, chaque cas de la spécification et chaque cas de compile-fail
passe par le compilateur amorcé, piloté par un harnais de tests qui est
lui-même un programme Nexium (`nx run tests/run.nx`), en CI sur trois
plateformes sans aucune chaîne d'outils Rust. Le premier compilateur, en
Rust, a servi au portage et a été supprimé en 1.0 (décision 90).

## Langages du dépôt

Lignes de code non vides, hors sortie de build, dépendances et fichiers
générés (`bootstrap/nx.c`, le parser tree-sitter, `gui/font.bin`, fichiers de
verrouillage) :

| langage | lignes | part | ce que c'est |
| --- | --- | --- | --- |
| Nexium | 48 804 | 87,2 % | le compilateur et ses outils (33 000 lignes sous `self/`), la bibliothèque standard (21 modules), le harnais de tests et le fuzzer, les exemples, les programmes du tutoriel, nexium-gui, le générateur du site, quatre benchmarks |
| C | 2 542 | 4,5 % | le runtime `nx_rt.h`, la couche fenêtre de la GUI, du C de test embarqué, un benchmark |
| Python | 1 492 | 2,7 % | les scripts de release (notes, manifestes de paquets, wheels et paquets npm, la documentation de std), les formateurs gdb et lldb, le lanceur de benchmarks et quatre benchmarks |
| fichiers d'éditeurs | 1 103 | 2,0 % | requêtes tree-sitter, Emacs Lisp, Vim script, Lua pour Neovim, un lexer Pygments, et les 25 lignes de Rust que Zed exige d'une extension |
| JavaScript, TypeScript | 939 | 1,7 % | l'extension VS Code, la grammaire tree-sitter et la couche WASI du playground |
| Inno Setup, shell, PowerShell | 855 | 1,5 % | le script de l'installateur Windows, `install.sh`, `install.ps1`, les scripts Chocolatey, les scripts de bootstrap |
| Rust, Go, Ruby | 213 | 0,4 % | quatre benchmarks en Rust et quatre en Go, et la formule Homebrew |

Il n'y a pas de Rust dans le compilateur : le premier compilateur a servi au
portage et a été supprimé en 1.0 (décision 90). Le Rust qui reste est la
colle de l'extension Zed, que Zed compile en WebAssembly, et quatre programmes de
benchmark écrits pour servir de mesure, à côté de leurs jumeaux en Go. Zig n'est pas dans
le tableau parce qu'il n'y a aucune source Zig dans l'arbre : `zig cc` est le
compilateur C que `nx` lance (embarqué par l'installateur Windows, téléchargé
par le script d'installation), de la même façon qu'un compilateur C s'utilise
et ne s'écrit pas.

## Organisation

```
bootstrap/      la graine en C dont le compilateur est construit, et les scripts de build
runtime/        nx_rt.h, embarqué dans chaque fichier C généré ; les formateurs gdb et lldb de nx debug
std/            la bibliothèque standard en Nexium, embarquée dans le compilateur
self/           le compilateur en Nexium, étape par étape
gui/            nexium-gui : GUI en mode immédiat en Nexium, démo, et la couche plateforme en C
editors/        extension VS Code, grammaire tree-sitter, et les fichiers de dix autres éditeurs
examples/       programmes avec sortie enregistrée, exécutés par les tests
topo/           le tutoriel : les chapitres et les programmes qu'ils montrent (exécutés par les tests)
site/           le générateur du site de documentation, un programme Nexium
tests/          le harnais (run.nx), la suite de conformité à la spécification (tests/spec), les cas de compile-fail, la vérification du débogueur
docs/           comment ça marche, référence du langage, guide d'intégration, traductions dans i18n/
bench/          quatre programmes en cinq langages derrière la page des chiffres
installers/     le script de l'installateur Windows, install.sh et install.ps1, les manifestes winget et Chocolatey
docker/         les images du compilateur pour ghcr.io (Debian et Alpine)
Formula/, bucket/  ce dépôt comme tap Homebrew et bucket Scoop (écrits à chaque release)
scripts/        notes de release, manifestes de paquets, wheels et paquets npm, la documentation de std
assets/         logo, bannière et l'aperçu social
nexium-spec.txt          la conception
nexium-systems-spec.txt  le langage système archivé ; les sections 4 à 9 sont la référence de syntaxe
DECISIONS.md    décisions prises là où la spécification était ouverte
KNOWN_ISSUES.md bogues ouverts et limites ; les correctifs passent dans le changelog
```

## Contribuer

Voir [`CONTRIBUTING.md`](../../../CONTRIBUTING.md). Les bogues et les
propositions passent par les issues GitHub ; un changement du langage doit
nommer la contrainte dure de la section 3 de la spécification qu'il sert. Les
pull requests passent les tests sur trois plateformes, les formateurs, une
vérification du changelog et l'[accord de licence de
contribution](../../../CLA.md) avant d'être fusionnées ; vous gardez votre
droit d'auteur.

## Licence

MIT. Copyright (c) 2026 Londopy.

<p align="center"><img src="https://raw.githubusercontent.com/Londopy/nexium/main/assets/logo.svg" alt="" width="48"></p>
