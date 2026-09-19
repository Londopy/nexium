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
  <a href="https://crates.io/crates/nexium"><img alt="crates.io" src="https://img.shields.io/crates/v/nexium?logo=rust&color=4fd1c5"></a>
  <a href="../../../LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-blue"></a>
  <a href="https://ziglang.org/download/"><img alt="Zig" src="https://img.shields.io/badge/backend-zig%20cc-f7a41d?logo=zig&logoColor=white"></a>
  <img alt="Platforms" src="https://img.shields.io/badge/platforms-windows%20%7C%20linux%20%7C%20macos-2b3a55">
</p>

<p align="center">
  <b>Un langage assez complet pour tout construire, et en même temps le meilleur choix pour n'adopter qu'une pièce d'autre chose.</b>
</p>

Nexium compile en code natif en passant par le C, possède un comptage de
références automatique sans ramasse-miettes, un système d'effets vérifié par
la machine qui dit si une fonction alloue, bloque ou peut paniquer, et un
compilateur qui transforme un seul arbre de sources en bibliothèque C, en
wheel Python, en crate Rust ou en outil en ligne de commande.

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
sein est convertie à la frontière au lieu d'abattre le processus hôte.

## Points forts

| | |
| --- | --- |
| 🧾 **Effets inférés et vérifiés** | `allocates` `refcounts` `blocks` `shared_mutable` `nondeterministic` `panics` `ffi`. Déclarez `!allocates` et le compilateur désigne la ligne exacte qui le violerait, à travers les appels. |
| 🧠 **Propriété sans vérificateur d'emprunts** | Les collections se déplacent, `.clone()` copie, les valeurs `ref class` sont comptées par références, `weak` brise les cycles. Utiliser après déplacement est une erreur de compilation. |
| 🔬 **Motifs binaires** | `<<version:4, ihl:4, len:16/big, rest:bytes>>` reconnaît et construit des paquets avec des tailles vérifiées. |
| 🧵 **Boucles parallèles, arènes, objets de trait** | `for parallel`, `using arena { }`, `dyn Trait !allocates`. |
| 🔌 **Le C sans bindings** | `@cImport("header.h")` lit l'en-tête directement ; `artifact link` compile du C embarqué dans le programme. |
| 📦 **Livrer depuis une seule source** | `nx ship` produit en-têtes et bibliothèques C, wheels Python et crates Rust avec des enveloppes sûres. |
| 🖼 **Une GUI, en Nexium** | [`gui/`](../../../gui) : une GUI en mode immédiat (boutons, curseurs, champs de texte) avec rastérisation logicielle et police bitmap, entièrement en Nexium au-dessus d'une couche fenêtre de 200 lignes de C. |
| 🛠 **Outils fournis** | `fmt`, `doc`, `lsp`, `size`, `leaks`, `refcounts`, `effects`, `audit`. Zéro dépendance. |

## Installation

La seule exigence à l'exécution est [Zig](https://ziglang.org/download/)
dans votre `PATH`, utilisé comme compilateur C (`zig cc` compile aussi en
croisé ; `--cc clang` fonctionne également).

Les binaires `nx` précompilés pour Windows, Linux et macOS sont sur la page
[Releases](https://github.com/Londopy/nexium/releases). Décompressez et
placez `nx` dans votre `PATH`.

Ou compilez depuis les sources avec Rust 1.75 ou plus récent :

```bash
cargo install nexium
```

```bash
cargo install --git https://github.com/Londopy/nexium
```

Puis :

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
<summary><b>Filtrage par motifs binaires</b></summary>

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
<summary><b>Appeler le C tient en une importation d'en-tête</b></summary>

```
const libc = @cImport("string.h")
const cv = @cImport("cvendor.h")
artifact link { c_sources = ["cvendor.c"] }

unsafe { println("{}", .{libc.strlen(@cstr("hello"))}) }
```

Pas de générateur de bindings, pas d'étape de construction : l'en-tête est la
seule source de vérité pour la disposition mémoire, et les appels étrangers
portent l'effet `ffi`.

</details>

<details>
<summary><b>Boucles parallèles et arènes</b></summary>

```
for parallel p, i in positions {
    out[i] = integrate(p)          // shared_mutable interdit ici
}

using arena {
    var scratch = List(Frame).new()   // allocation bump, libérée d'un coup
    ...
}
```

</details>

**Documentation**

- [Comment fonctionne Nexium](../../architecture.md) : le pipeline de la source au binaire, l'inférence d'effets, la propriété, le runtime et la livraison.
- [Référence du langage](../../language.md) : chaque construction implémentée par le compilateur.
- [Intégration](../../embedding.md) (anglais) : appeler les bibliothèques livrées depuis Python, Rust et C.
- [nexium-gui](../../gui.md) (anglais) : la bibliothèque GUI en mode immédiat et l'écriture d'un widget.
- [Publier votre programme](../../releasing-your-program.md) (anglais) : des binaires pour trois plateformes à partir d'un tag, installateurs en option.
- [Support des éditeurs](../../../editors) (anglais) : extension VS Code, syntaxe Sublime, LSP.
- [Décisions](../../../DECISIONS.md) (anglais) : chaque choix fait là où la spécification restait ouverte.

## Commandes

| commande | effet |
| --- | --- |
| `nx build file.nx` | compile en exécutable (ou en objet s'il n'y a pas de `main`) |
| `nx run file.nx` | compile et exécute |
| `nx test file.nx [filter]` | exécute les blocs `test "..."` |
| `nx check file.nx` | vérifie les types et signale les violations d'effets |
| `nx effects file.nx` | affiche les effets inférés de chaque fonction |
| `nx audit file.nx` | liste les blocs `unsafe` et les globales mutables |
| `nx ship file.nx` | produit chaque `artifact` déclaré |
| `nx emit-c file.nx` | affiche le C généré |
| `nx tokens file.nx` | affiche le flux de jetons (l'oracle de l'auto-hébergement) |
| `nx fmt file.nx [--check]` | formatage canonique |
| `nx doc file.nx` | documentation HTML avec effets inférés |
| `nx size file.nx` | attribue les octets du binaire aux déclarations |
| `nx refcounts file.nx` | chaque site de rétention et de libération |
| `nx leaks file.nx` | exécute avec suivi des allocations et signale les fuites |
| `nx lsp` | serveur de langage sur stdio |

Options : `--mode debug|safe|fast|small`, `--target x86_64-linux-gnu`
(toute cible connue de `zig cc`), `--out-dir`, `--keep-c`, `--cc`, et pour
l'interopérabilité C `-I`, `--link`, `--link-path`, `--c-source`.

## État

C'est la première implémentation du design de `nexium-spec.txt`. Elle est
assez complète pour écrire de vrais programmes (voir
[`examples/`](../../../examples)) et livrer un composant Python, Rust ou C à
partir d'un seul fichier. Les objets de trait, les boucles parallèles, les
portées d'arène, l'importation directe d'en-têtes C et les outils sont tous
là. C'est encore tôt : la bibliothèque standard n'est qu'une fraction de la
section 16, et la vérification des régions ne couvre que les vues renvoyées.
[`DECISIONS.md`](../../../DECISIONS.md) recense chaque choix fait là où la
spécification restait ouverte, et le point 27 liste ce qui reste.

## Auto-hébergement

Le compilateur est en Rust aujourd'hui. La version Nexium grandit dans
[`self/`](../../../self), une étape à la fois, chacune vérifiée contre le
compilateur Rust sur les mêmes entrées :

| étape | fichier | oracle | état |
| --- | --- | --- | --- |
| lexeur | [`self/lexer.nx`](../../../self/lexer.nx) | `nx tokens` | ✅ identique sur chaque exemple et sur lui-même |
| analyseur | | `nx parse` | prochaine |
| vérificateur | | `nx check`, la suite compile-fail | |
| émetteur C | | `nx emit-c` | |

`cargo test` compile `self/lexer.nx` avec le compilateur Rust et compare sa
sortie à l'oracle.

## Langages du dépôt

Lignes de code non vides, hors sorties de compilation, dépendances et
fichiers générés :

| langage | lignes | part | rôle |
| --- | --- | --- | --- |
| Rust | 22 393 | 86,9 % | le compilateur `nx` |
| Nexium | 2 111 | 8,2 % | exemples, lexeur auto-hébergé, nexium-gui, tests |
| C | 1 108 | 4,3 % | le runtime `nx_rt.h` et la couche fenêtre de la GUI |
| JavaScript, TypeScript | 159 | 0,6 % | l'extension VS Code |

## Organisation

```
src/            le compilateur (lexeur, analyseur, vérificateur, comptime, backend C, pilote)
runtime/        nx_rt.h, intégré dans chaque fichier C généré
self/           le compilateur en Nexium, étape par étape
gui/            nexium-gui : GUI en mode immédiat en Nexium, démo et couche C de plateforme
editors/        extension VS Code et syntaxe Sublime Text
examples/       programmes avec sortie enregistrée, exécutés par `cargo test`
tests/          tests d'intégration et cas compile-fail
docs/           fonctionnement, référence du langage, guide d'intégration, traductions
assets/         logo et bannière
nexium-spec.txt          le design
nexium-systems-spec.txt  le langage système archivé ; les sections 4 à 9 sont la référence de syntaxe
DECISIONS.md    choix faits là où la spécification restait ouverte
```

## Contribuer

Voir [`CONTRIBUTING.md`](../../../CONTRIBUTING.md). Bugs et propositions
passent par les issues GitHub ; un changement du langage doit nommer la
contrainte dure de la section 3 de la spécification qu'il sert.

## Licence

MIT. Copyright (c) 2026 Londopy.

<p align="center"><img src="https://raw.githubusercontent.com/Londopy/nexium/main/assets/logo.svg" alt="" width="48"></p>
