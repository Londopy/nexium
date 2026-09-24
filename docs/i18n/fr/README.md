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
| 🧾 **Effets inférés et vérifiés** | `allocates` `refcounts` `blocks` `shared_mutable` `nondeterministic` `panics` `ffi`. Déclarez `!allocates` et le compilateur pointe la ligne exacte qui le violerait, à travers les appels. |
| 🧠 **Propriété sans borrow checker** | Les collections se déplacent, `.clone()` copie, les valeurs `ref class` sont comptées par références, `weak` casse les cycles. Utiliser après un déplacement est une erreur de compilation. |
| 🔬 **Motifs binaires** | `<<version:4, ihl:4, len:16/big, rest:bytes>>` reconnaît et construit des paquets aux tailles vérifiées. |
| 🧵 **Boucles parallèles, arènes, objets de trait** | `for parallel`, `using arena { }`, `dyn Trait !allocates`. |
| 🔌 **Du C sans bindings** | `@cImport("header.h")` lit l'en-tête directement ; `artifact link` compile du C embarqué dans le programme. |
| 📦 **Livrer depuis une seule source** | `nx ship` produit des en-têtes et bibliothèques C, des wheels Python et des crates Rust avec des enveloppes sûres. |
| 🖼 **Une GUI, en Nexium** | [`gui/`](../../../gui) : une GUI en mode immédiat (boutons, curseurs, champs de texte) avec un rastériseur logiciel et une police bitmap, tout en Nexium au-dessus d'une couche fenêtre C de 200 lignes. |
| 🛠 **Outils inclus** | `fmt`, `doc`, `lsp`, `size`, `leaks`, `refcounts`, `effects`, `audit`. Zéro dépendance. |

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

**pip ou npm** : `pip install nexium-lang` ou `npm install -g nexium-lang` (le binaire, par plateforme ; un compilateur C comme d'habitude).

**Docker** : `docker run --rm -v "$PWD":/work ghcr.io/londopy/nexium run hello.nx`
(Debian ; `:alpine` aussi ; amd64 et arm64).

**Homebrew et Scoop** : le dépôt est son propre tap et son propre bucket.

```bash
brew tap londopy/tap https://github.com/Londopy/nexium && brew install londopy/tap/nexium
```

```powershell
scoop install https://raw.githubusercontent.com/Londopy/nexium/main/bucket/nexium.json
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
- [Bibliothèque standard](../../std.md) (anglais) : les modules écrits en Nexium (`std.strings`, `std.lists`, `std.bytes`, `std.num`, `std.json`, `std.args`, `std.fs`, `std.time`, `std.regex`, `std.text`, `std.testing`, `std.stream`, `std.net`, `std.http`, `std.thread`, `std.process`).
- [nexium-gui](../../gui.md) (anglais) : la bibliothèque GUI en mode immédiat et comment écrire un widget.
- [Publier votre programme](../../releasing-your-program.md) (anglais) : des binaires pour trois plateformes à partir d'un tag, installateurs en option.
- [Prise en charge des éditeurs](../../../editors) (anglais) : VS Code, Vim, Neovim, Helix, Zed, Emacs, Kate, JetBrains, Sublime Text, Notepad++, nano, et `nx lsp` pour les autres.
- [Linguist](../../../linguist) (anglais) : la pull request prête à appliquer qui fera reconnaître `.nx` par GitHub une fois le seuil d'usage atteint.
- [Traductions](../README.md) : ce README en six langues ; la référence du langage et le tour de l'architecture en espagnol, chinois et japonais.
- [Noms des versions](../../release-names.md) (anglais) : chaque version est un lieu sur une montagne ; le schéma, le registre et les noms encore à utiliser.
- [Décisions](../../../DECISIONS.md) (anglais) : chaque choix fait là où la spécification était ouverte.
- [Problèmes connus](../../../KNOWN_ISSUES.md) (anglais) : bogues ouverts, manques et limites, avec reproductions.

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
| `nx emit-c file.nx` | affiche le C généré |
| `nx tir file.nx [--sigs]` | le programme vérifié en S-expressions (les tests du compilateur le lisent) |
| `nx fmt file.nx [--check]` | formatage canonique |
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
| `nx repl`, ou simplement `nx` | une session interactive : tapez du code, voyez les valeurs, gardez les liaisons |

Options : `--mode debug|safe|fast|small`, `--target x86_64-linux-gnu` (toute
cible connue de `zig cc`), `--cpu baseline|native|<nom>` (baseline par défaut,
pour qu'un binaire tourne sur toute machine de son architecture), `--out-dir`,
`--keep-c`, `--cc`, `--strict` (les avertissements sont des
erreurs), `--sanitize address,undefined` (les sanitizers du compilateur C ;
`address` demande gcc ou clang), et pour l'interopérabilité C `-I`, `--link`, `--link-path`,
`--c-source`.

## État

**1.0 : langage stable, écosystème naissant.** Le langage ne change que par
ajout, sous la [politique de stabilité](../../stability.md) ; le compilateur
est écrit en Nexium et se construit lui-même ; chaque exemple, chaque cas de
la spécification et chaque programme du tutoriel s'exécute en CI sur trois
plateformes, sous les sanitizers et le fuzzer. Ce que 1.0 n'est pas encore,
et où chaque point trouve sa réponse, est la première section de [la feuille
de route](../../../ROADMAP.md) : la sûreté mémoire, ce sont les règles de vues
de 1.2, des erreurs depuis 1.3 (`nx fix` fait les corrections mécaniques), il n'y
a pas de chiffres de performance au-delà de [la page des chiffres](../../numbers.md)
(quatre programmes en cinq langages sur un même runner, régénérée chaque
semaine), et l'écosystème se résume à un seul mainteneur, seize modules de
bibliothèque standard et un paquet venu d'ailleurs (le [SDK Discord Rich
Presence](../../discord.md) de statusmith, `nx add discord_rpc ...`).
[`KNOWN_ISSUES.md`](../../../KNOWN_ISSUES.md) liste chaque bogue ouvert avec
son correctif ; [`DECISIONS.md`](../../../DECISIONS.md), chaque choix fait là
où la spécification était ouverte.

## Noms des versions

Une version majeure est une montagne, dans l'ordre où les quatorze sommets de
8 000 mètres ont été gravis pour la première fois ; les versions en dessous
sont l'ascension : camps, voies et faces pour les versions mineures, les
membres de l'expédition de la première ascension pour les correctifs,
`Summit` pour `X.0.0`. La ligne 0.x est l'approche et les camps de
l'Annapurna, le premier 8 000 gravi (1950), donc 1.0.0 est
`Annapurna: Summit` ; 0.7.0, où le compilateur a commencé à se construire
lui-même, est `Annapurna: Camp V`, le dernier camp avant l'assaut du sommet.
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
| émetteur C | [`self/cgen.nx`](../../../self/cgen.nx) | un fichier C par programme |
| pilote | [`self/nx.nx`](../../../self/nx.nx) | build, run, test, check, emit-c, tir ; la bibliothèque standard embarquée |
| outils | [`self/fmt.nx`](../../../self/fmt.nx), [`self/doc.nx`](../../../self/doc.nx), [`self/tools.nx`](../../../self/tools.nx), [`self/size.nx`](../../../self/size.nx), [`self/manifest.nx`](../../../self/manifest.nx), [`self/ship.nx`](../../../self/ship.nx), [`self/lsp.nx`](../../../self/lsp.nx), [`self/repl.nx`](../../../self/repl.nx) | le formateur, le générateur de documentation, les rapports, les paquets, `ship`, le serveur de langage, le REPL |

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
| Nexium | 38 374 | 85,4 % | le compilateur et ses outils (27 100 lignes sous `self/`), la bibliothèque standard, le harnais de tests et le fuzzer, les exemples, les programmes du tutoriel, nexium-gui, le générateur du site, la suite de la spécification, quatre benchmarks |
| C | 2 925 | 6,5 % | le runtime `nx_rt.h`, la couche fenêtre de la GUI, du C de test embarqué, un benchmark |
| Python | 1 063 | 2,4 % | les scripts de release (notes, manifestes de paquets, wheels et paquets npm, la documentation de std), le lanceur de benchmarks, un benchmark |
| fichiers d'éditeurs | 1 028 | 2,3 % | requêtes tree-sitter, Emacs Lisp, Vim script, Lua pour Neovim, et les 25 lignes de Rust que Zed exige d'une extension |
| JavaScript, TypeScript | 550 | 1,2 % | l'extension VS Code et la grammaire tree-sitter |
| Inno Setup, shell, PowerShell | 777 | 1,7 % | le script de l'installateur Windows, `install.sh`, `install.ps1`, les scripts Chocolatey |
| Rust, Go, Ruby | 236 | 0,5 % | un benchmark en Rust et un en Go, et la formule Homebrew |

Il n'y a pas de Rust dans le compilateur : le premier compilateur a servi au
portage et a été supprimé en 1.0 (décision 90). Le Rust qui reste est la
colle de l'extension Zed, que Zed compile en WebAssembly, et un programme de
benchmark écrit pour servir de mesure, à côté de son jumeau en Go. Zig n'est pas dans
le tableau parce qu'il n'y a aucune source Zig dans l'arbre : `zig cc` est le
compilateur C que `nx` lance (embarqué par l'installateur Windows, téléchargé
par le script d'installation), de la même façon qu'un compilateur C s'utilise
et ne s'écrit pas.

## Organisation

```
bootstrap/      la graine en C dont le compilateur est construit, et les scripts de build
runtime/        nx_rt.h, embarqué dans chaque fichier C généré
std/            la bibliothèque standard en Nexium, embarquée dans le compilateur
self/           le compilateur en Nexium, étape par étape
gui/            nexium-gui : GUI en mode immédiat en Nexium, démo, et la couche plateforme en C
editors/        extension VS Code, grammaire tree-sitter, et les fichiers de dix autres éditeurs
examples/       programmes avec sortie enregistrée, exécutés par les tests
topo/           le tutoriel : les chapitres et les programmes qu'ils montrent (exécutés par les tests)
site/           le générateur du site de documentation, un programme Nexium
tests/          le harnais (run.nx), la suite de conformité à la spécification (tests/spec) et les cas de compile-fail
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
