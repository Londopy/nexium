<p align="center">
  <img src="https://raw.githubusercontent.com/Londopy/nexium/main/assets/banner.svg" alt="Nexium" width="880">
</p>

<p align="center">
  <a href="../../../README.md">English</a> ·
  <a href="../es/README.md">Español</a> ·
  <a href="../zh-CN/README.md">简体中文</a> ·
  <a href="../ja/README.md">日本語</a> ·
  <b>한국어</b> ·
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
  <b>Nexium은 모든 것을 만들 수 있을 만큼 완전하면서도, 다른 무언가의 한 조각으로 도입하기에도 가장 좋은 언어.</b>
</p>

<p align="center">
  <a href="https://londopy.github.io/nexium/"><b>문서와 튜토리얼 「the Topo」 &rarr; londopy.github.io/nexium</b></a><br>
  <sub><a href="https://londopy.github.io/nexium/topo/01-base-camp.html">Topo로 시작하기</a> &middot; <a href="https://londopy.github.io/nexium/docs/language.html">언어 레퍼런스</a> &middot; <a href="https://londopy.github.io/nexium/docs/install.html">설치</a> &middot; <a href="https://londopy.github.io/nexium/docs/std.html">표준 라이브러리</a> &middot; <a href="https://londopy.github.io/nexium/docs/embedding.html">임베딩</a> (영어)</sub>
</p>

Nexium은 C를 거쳐 네이티브 코드로 컴파일되고, 추적 가비지 컬렉터 없는 자동 참조
카운팅을 갖추며, 함수가 메모리를 할당하는지, 블로킹하는지, 패닉할 수 있는지를
기계적으로 검사하는 이펙트 시스템을 가지고, 하나의 소스 트리를 C 라이브러리, Python
wheel, Rust 크레이트 또는 명령줄 도구로 만들어 내는 컴파일러를 갖습니다.

<table>
<tr>
<td width="50%" valign="top">

**파일 하나**

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

**모든 타깃**

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

이펙트 시그니처가 C ABI를 결정합니다. `checksum`은 `!panics`가 증명되었으므로 평범한
`uint32_t checksum(const uint8_t*, size_t)`를 얻습니다. 실패할 수 있는 함수는 상태
코드를 반환하고, 그 안에서 일어난 Nexium 패닉은 호스트 프로세스를 중단시키는 대신
경계에서 변환됩니다.

## 핵심

| | |
| --- | --- |
| 🧾 **추론되고 검사되는 이펙트** | `allocates` `refcounts` `blocks` `shared_mutable` `nondeterministic` `panics` `ffi`. `!allocates`를 선언하면 컴파일러가 호출을 따라가 이를 깨뜨릴 정확한 줄을 가리킵니다. |
| 🧠 **빌림 검사기 없는 소유권** | 컬렉션은 이동하고, `.clone()`은 복사하고, `ref class` 값은 참조 카운팅되며, `weak`가 순환을 끊습니다. 이동 후 사용은 컴파일 오류입니다. |
| 🔬 **바이너리 패턴** | `<<version:4, ihl:4, len:16/big, rest:bytes>>`가 크기를 검사하며 패킷을 매칭하고 조립합니다. |
| 🧵 **병렬 루프, 아레나, 트레이트 객체** | `for parallel`, `using arena { }`, `dyn Trait !allocates`. |
| 🔌 **바인딩 없는 C** | `@cImport("header.h")`가 헤더를 직접 읽고, `artifact link`가 동봉한 C를 프로그램에 컴파일해 넣습니다. |
| 📦 **하나의 소스에서 배포** | `nx ship`이 C 헤더와 라이브러리, Python wheel, 안전한 래퍼를 갖춘 Rust 크레이트를 만듭니다. |
| 🖼 **Nexium으로 만든 GUI** | [`gui/`](../../../gui): 소프트웨어 래스터라이저와 비트맵 폰트를 갖춘 즉시 모드 GUI(버튼, 슬라이더, 텍스트 필드). 200줄짜리 C 창 계층 위는 전부 Nexium입니다. |
| 🛠 **기본 제공 도구** | `fmt`, `doc`, `lsp`, `size`, `leaks`, `refcounts`, `effects`, `audit`. 의존성 제로. |

## 설치

**Windows**: [Releases](https://github.com/Londopy/nexium/releases) 페이지에서 설치
프로그램을 내려받아 실행합니다. `nx`, 동봉된 Zig 툴체인(`nx`가 쓰는 C 컴파일러), 표준
라이브러리, 예제, 문서, VS Code 확장을 설치하고 `nx`를 PATH에 추가합니다. 그 밖에
설치할 것은 없습니다.

**macOS와 Linux**:

```bash
curl -fsSL https://raw.githubusercontent.com/Londopy/nexium/main/installers/install.sh | sh
```

내려받은 파일을 릴리스 체크섬과 대조하고, `~/.nexium`에 설치하고, C 컴파일러를
준비하며(macOS에서는 Xcode 도구, Linux에서는 없으면 Zig를 내려받음), `nx`를 PATH에
추가합니다.

**Windows, PowerShell에서**: `irm https://raw.githubusercontent.com/Londopy/nexium/main/installers/install.ps1 | iex`
(포터블 빌드, 검증됨, PATH에 추가; 마법사 없음).

**pip 또는 npm**: `pip install nexium-lang` 또는 `npm install -g nexium-lang`(플랫폼별 바이너리; C 컴파일러는 평소처럼 필요).

**Docker**: `docker run --rm -v "$PWD":/work ghcr.io/londopy/nexium run hello.nx`
(Debian; `:alpine`도 있음; amd64와 arm64).

**Homebrew와 Scoop**: 이 저장소가 곧 tap이자 bucket입니다.

```bash
brew tap londopy/tap https://github.com/Londopy/nexium && brew install londopy/tap/nexium
```

```powershell
scoop install https://raw.githubusercontent.com/Londopy/nexium/main/bucket/nexium.json
```

그다음 새 콘솔에서 `nx doctor`를 실행하면 무엇이 사용될지 보여 줍니다. 체크섬 검증과
모든 환경 변수를 포함한 자세한 내용은 [docs/install.md](../../install.md)(영어)에
있습니다.

또는 C 컴파일러만으로(PATH의 Zig 또는 `CC`) 소스에서 빌드합니다. Nexium으로 쓰인
컴파일러를 그 C 시드에서 빌드합니다:

```bash
git clone https://github.com/Londopy/nexium && cd nexium && sh bootstrap/build.sh
```

결과물은 `nx-out/bootstrap/nx2`입니다(Windows에서는 `build.ps1`). 그다음:

```bash
nx run examples/hello.nx
```

## 둘러보기

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
<summary><b>바이너리 패턴 매칭</b></summary>

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
<summary><b>이펙트는 추론되고 검사된다</b></summary>

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
<summary><b>C 호출은 헤더 임포트 하나면 된다</b></summary>

```
const libc = @cImport("string.h")
const cv = @cImport("cvendor.h")
artifact link { c_sources = ["cvendor.c"] }

unsafe { println("{}", .{libc.strlen(@cstr("hello"))}) }
```

바인딩 생성기도, 빌드 단계도 없습니다. 헤더가 레이아웃의 유일한 진실이고, 외부
호출은 `ffi` 이펙트를 지닙니다.

</details>

<details>
<summary><b>병렬 루프와 아레나</b></summary>

```
for parallel p, i in positions {
    out[i] = integrate(p)          // 여기서는 shared_mutable이 허용되지 않음
}

using arena {
    var scratch = List(Frame).new()   // 범프 할당, 한꺼번에 해제
    ...
}
```

</details>

**문서**

- [명세](../../../SPEC.md) (영어): 구현된 그대로의 언어. 계획된 부분은 표시됨.
- [로드맵](../../../ROADMAP.md) (영어): 단계, 완료 기준, 그리고 계획하지 않는 것.
- [Nexium의 동작 원리](../../architecture.md) (영어): 소스에서 바이너리까지의 파이프라인, 이펙트 추론, 소유권, 런타임, 배포.
- [언어 레퍼런스](../../language.md) (영어): 컴파일러가 구현하는 모든 구문.
- [임베딩](../../embedding.md) (영어): 배포한 라이브러리를 Python, Rust, C에서 호출하기.
- [대화형 세션](../../repl.md) (영어): 프롬프트에서의 `nx`, `python`처럼.
- [안정성](../../stability.md)과 [플랫폼](../../platforms.md) (영어): 버전이 약속하는 것, 폐기 주기, `nx fix`, 티어.
- [the Topo](https://londopy.github.io/nexium/topo/01-base-camp.html) (영어): 튜토리얼. 컴파일러 설치부터 신경망, GUI, 배포 라이브러리까지. 소스는 [`topo/`](../../../topo/)에 있습니다. 위의 모든 것을 렌더링한 결과는 [londopy.github.io/nexium](https://londopy.github.io/nexium/)에 있습니다.
- [설치](../../install.md) (영어): Windows 설치 프로그램, macOS/Linux 스크립트, 소스 빌드, 체크섬, `nx`가 C 컴파일러를 찾는 방법.
- [패키지](../../packages.md) (영어): `nexium.toml`, `nx add`, `nx fetch`, git 또는 경로 의존성, 잠금 파일.
- [표준 라이브러리](../../std.md) (영어): Nexium으로 쓰인 모듈(`std.strings`, `std.lists`, `std.bytes`, `std.num`, `std.json`, `std.args`, `std.fs`, `std.time`, `std.regex`, `std.text`, `std.testing`, `std.stream`, `std.net`, `std.http`, `std.thread`, `std.process`).
- [nexium-gui](../../gui.md) (영어): 즉시 모드 GUI 라이브러리와 위젯 작성법.
- [프로그램 릴리스하기](../../releasing-your-program.md) (영어): 태그 하나로 세 플랫폼의 바이너리를, 설치 프로그램은 선택.
- [에디터 지원](../../../editors) (영어): VS Code, Vim, Neovim, Helix, Zed, Emacs, Kate, JetBrains, Sublime Text, Notepad++, nano, 나머지는 `nx lsp`로.
- [Linguist](../../../linguist) (영어): 사용량 기준을 넘기면 GitHub가 `.nx`를 인식하게 할, 바로 적용할 수 있는 풀 리퀘스트.
- [번역](../README.md): 이 README를 여섯 언어로. 언어 레퍼런스와 아키텍처 안내는 스페인어, 중국어, 일본어로.
- [릴리스 이름](../../release-names.md) (영어): 모든 릴리스는 산 위의 한 장소. 규칙, 장부, 아직 쓰지 않은 이름.
- [결정 기록](../../../DECISIONS.md) (영어): 명세가 열려 있던 곳에서 내린 모든 결정.
- [알려진 문제](../../../KNOWN_ISSUES.md) (영어): 미해결 버그, 빈틈, 제한. 재현 방법 포함.

## 명령

| 명령 | 하는 일 |
| --- | --- |
| `nx build file.nx` | 실행 파일로 컴파일(`main`이 없으면 오브젝트로) |
| `nx run file.nx` | 빌드하고 실행; `--watch`는 프로그램의 파일이 바뀔 때마다 다시 실행 |
| `nx test file.nx [filter]` | `test "..."` 블록 실행; `--watch`도 가능 |
| `nx check file.nx` | 타입 검사와 이펙트 위반 보고 |
| `nx effects file.nx` | 모든 함수의 추론된 이펙트 출력 |
| `nx explain file.nx f effect` | `f`가 그 효과를 갖는 이유: 효과를 들여오는 호출들을 원시 연산까지 트리로 |
| `nx audit file.nx` | `unsafe` 블록과 가변 전역 나열; `--lock`은 효과 잠금 파일을 쓰고, `--check`는 효과가 늘면 실패 |
| `nx ship file.nx` | 선언된 모든 `artifact` 생성 |
| `nx emit-c file.nx` | 생성된 C 출력 |
| `nx tir file.nx [--sigs]` | 검사된 프로그램을 S-식으로(컴파일러 자체 테스트가 읽음) |
| `nx fmt file.nx [--check]` | 표준 서식 |
| `nx fix file.nx` | 검사기의 기계적인 수정(`.clone()`, `@escape(...)`, `_ = `)을 적용하고 폐기된 형태를 이전함([docs/stability.md](../../stability.md) 참고) |
| `nx doc file.nx` | 추론된 이펙트가 담긴 HTML 문서 |
| `nx size file.nx` | 바이너리의 바이트를 선언별로 귀속 |
| `nx layout file.nx [Type...]` | struct나 enum의 오프셋, 크기, 패딩과 그것을 줄일 정렬 순서 |
| `nx upgrade` | 이 실행 파일을 최신 릴리스로 교체, 검증됨; `--check`는 보고만 |
| `nx install [DIR]` | 이 복사본을 곁의 파일들과 함께 사용자 위치에 설치하고 PATH에 추가(포터블 zip이 스스로 설치) |
| `nx refcounts file.nx` | 모든 retain과 release 지점 |
| `nx leaks file.nx` | 할당을 추적하며 실행하고 누수 보고 |
| `nx lsp` | stdio 위의 언어 서버 |
| `nx doctor` | 어떤 C 컴파일러가 쓰일지, 설치가 동작하는지 |
| `nx repl`, 또는 그냥 `nx` | 대화형 세션: 코드를 입력하고, 값을 보고, 바인딩을 유지 |

옵션: `--mode debug|safe|fast|small`, `--target x86_64-linux-gnu`(`zig cc`가 아는 모든
타깃), `--cpu baseline|native|<이름>`(기본은 baseline, 같은 아키텍처의 어느 기계에서도
바이너리가 돌도록), `--out-dir`, `--keep-c`, `--cc`, `--strict`(경고를
오류로), `--sanitize address,undefined`(C 컴파일러의 새니타이저, `address`는
gcc나 clang이 필요), C 연동용 `-I`, `--link`,
`--link-path`, `--c-source`.

## 현황

**1.0: 언어는 안정, 생태계는 초기.** 언어는 [안정성 정책](../../stability.md)에 따라
추가로만 바뀝니다. 컴파일러는 Nexium으로 쓰였고 스스로를 빌드합니다. 모든 예제, 명세
케이스, 튜토리얼 프로그램이 CI에서 세 플랫폼 위에서, 새니타이저와 퍼저 아래에서
실행됩니다. 1.0이 아직 아닌 것과 각각이 어디서 답을 얻는지는
[로드맵](../../../ROADMAP.md)의 첫 절에 있습니다. 메모리 안전성은 1.2의 뷰 규칙이며
1.3부터 오류이고(기계적인 수정은 `nx fix`가 합니다), 벤치마크 수치는
[수치 페이지](../../numbers.md)(다섯 언어로 쓴 네 프로그램을 한 러너에서, 매주 재생성)
말고는 없으며, 생태계는 메인테이너 한 명, 표준 라이브러리 열여섯 모듈, 그리고 트리
밖에서 온 패키지 하나(statusmith의 [Discord Rich Presence SDK](../../discord.md),
`nx add discord_rpc ...`)입니다.
[`KNOWN_ISSUES.md`](../../../KNOWN_ISSUES.md)는 미해결 버그를 수정 방안과 함께,
[`DECISIONS.md`](../../../DECISIONS.md)는 명세가 열려 있던 곳에서 내린 모든 결정을
나열합니다.

## 릴리스 이름

메이저 버전은 산이며, 열네 개의 8000미터 봉우리가 초등된 순서를 따릅니다. 그 아래
버전들은 등반입니다. 마이너 버전에는 캠프, 루트, 벽을, 패치에는 초등 원정대의 대원을,
`X.0.0`에는 `Summit`을 씁니다. 0.x 계열은 최초로 등정된 8000미터 봉 안나푸르나(1950년)의
접근로와 캠프이므로 1.0.0은 `Annapurna: Summit`이고, 컴파일러가 스스로를 빌드하기
시작한 0.7.0은 정상 공격 전 마지막 캠프인 `Annapurna: Camp V`입니다. 이름은 changelog,
릴리스 제목, `nx version`에 나타납니다.
[docs/release-names.md](../../release-names.md)(영어)에 규칙, 장부, 아직 오를 산이
있습니다.

## 셀프 호스팅

컴파일러는 Nexium으로 쓰였고, [`self/`](../../../self)에 있으며, 스스로를 빌드합니다.
`nx`가 없는 기계는 컴파일러가 자기 자신을 위해 내보낸 C인
[`bootstrap/nx.c`](../../../bootstrap/nx.c)로부터, 아무 C 컴파일러로, Rust 없이 하나를
만듭니다:

```sh
sh bootstrap/build.sh     # nx.c -> nx0; nx0가 self/nx.nx를 빌드 -> nx1; nx1이 자신을 같은 C로 다시 빌드 -> nx2
```

| 단계 | 파일 | 역할 |
| --- | --- | --- |
| 렉서 | [`self/lexer.nx`](../../../self/lexer.nx) | 토큰 |
| 파서 | [`self/parser.nx`](../../../self/parser.nx) | id 아레나 위의 구문 트리 |
| 검사기 | [`self/check.nx`](../../../self/check.nx), `self/check_*.nx`, [`self/cimport.nx`](../../../self/cimport.nx) | 타입, 이펙트, 소유권, 제네릭, 컴파일 타임 인터프리터, C 헤더 임포트, 모든 진단 |
| C 생성기 | [`self/cgen.nx`](../../../self/cgen.nx) | 프로그램당 C 파일 하나 |
| 드라이버 | [`self/nx.nx`](../../../self/nx.nx) | build, run, test, check, emit-c, tir; 표준 라이브러리 내장 |
| 도구 | [`self/fmt.nx`](../../../self/fmt.nx), [`self/doc.nx`](../../../self/doc.nx), [`self/tools.nx`](../../../self/tools.nx), [`self/size.nx`](../../../self/size.nx), [`self/manifest.nx`](../../../self/manifest.nx), [`self/ship.nx`](../../../self/ship.nx), [`self/lsp.nx`](../../../self/lsp.nx), [`self/repl.nx`](../../../self/repl.nx) | 포매터, 문서 생성기, 각종 보고서, 패키지, `ship`, 언어 서버, REPL |

모든 예제, 명세 케이스, 컴파일 실패 케이스가 그 자체로 Nexium 프로그램인 테스트
하네스(`nx run tests/run.nx`)의 구동으로 부트스트랩된 컴파일러를 거쳐, CI에서 세
플랫폼 위에서 Rust 툴체인 없이 실행됩니다. Rust로 쓰인 첫 컴파일러는 이식을 이끌었고
1.0에서 삭제되었습니다(결정 90).

## 저장소의 언어

빌드 출력, 의존성, 생성 파일(`bootstrap/nx.c`, tree-sitter 파서, `gui/font.bin`, 잠금
파일)을 제외한, 빈 줄이 아닌 코드 줄 수:

| 언어 | 줄 수 | 비율 | 무엇인가 |
| --- | --- | --- | --- |
| Nexium | 38,374 | 85.4% | 컴파일러와 그 도구(`self/` 아래 27,100줄), 표준 라이브러리, 테스트 하네스와 퍼저, 예제, 튜토리얼의 프로그램, nexium-gui, 사이트 생성기, 명세 스위트, 벤치마크 넷 |
| C | 2,925 | 6.5% | 런타임 `nx_rt.h`, GUI 창 계층, 동봉된 테스트용 C, 벤치마크 하나 |
| Python | 1,063 | 2.4% | 릴리스 스크립트(노트, 패키지 매니페스트, wheel과 npm 패키지, std 문서), 벤치마크 러너, 벤치마크 하나 |
| 에디터 파일 | 1,028 | 2.3% | tree-sitter 쿼리, Emacs Lisp, Vim script, Neovim용 Lua, 그리고 Zed가 확장에 요구하는 Rust 25줄 |
| JavaScript, TypeScript | 550 | 1.2% | VS Code 확장과 tree-sitter 문법 |
| Inno Setup, 셸, PowerShell | 777 | 1.7% | Windows 설치 프로그램 스크립트, `install.sh`, `install.ps1`, Chocolatey 스크립트 |
| Rust, Go, Ruby | 236 | 0.5% | Rust와 Go에 벤치마크 하나씩, 그리고 Homebrew 포뮬러 |

컴파일러에는 Rust가 없습니다. 첫 컴파일러는 이식을 이끌고 1.0에서 삭제되었으며(결정
90), 남은 Rust는 Zed가 WebAssembly로 컴파일하는 Zed 확장의 접착 코드와, 비교 측정용으로
쓴 벤치마크 프로그램 하나(Go 쌍둥이 곁에)뿐입니다. Zig가
표에 없는 이유는 트리에 Zig 소스가 없기 때문입니다. `zig cc`는 `nx`가 실행하는 C
컴파일러이며(Windows 설치 프로그램이 동봉하고, 설치 스크립트가 내려받음), C
컴파일러를 쓰는 것이지 작성하는 것이 아닌 것과 같습니다.

## 구성

```
bootstrap/      컴파일러를 빌드하는 C 시드와 빌드 스크립트
runtime/        nx_rt.h. 생성되는 모든 C 파일에 포함됨
std/            Nexium으로 쓰인 표준 라이브러리. 컴파일러에 내장
self/           Nexium으로 쓰인 컴파일러, 단계별로
gui/            nexium-gui: Nexium 즉시 모드 GUI, 데모, C 플랫폼 계층
editors/        VS Code 확장, tree-sitter 문법, 그리고 열 개 에디터를 위한 파일
examples/       출력이 기록된 프로그램. 테스트가 실행
topo/           튜토리얼: 각 장과 거기서 보여 주는 프로그램(테스트가 실행)
site/           문서 사이트 생성기. Nexium 프로그램
tests/          하네스(run.nx), 명세 적합성 스위트(tests/spec), 컴파일 실패 케이스
docs/           동작 원리, 언어 레퍼런스, 임베딩 가이드, i18n/의 번역
bench/          수치 페이지 뒤의 다섯 언어 네 프로그램
installers/     Windows 설치 프로그램 스크립트, install.sh와 install.ps1, winget과 Chocolatey 매니페스트
docker/         ghcr.io용 컴파일러 이미지(Debian과 Alpine)
Formula/, bucket/  Homebrew tap이자 Scoop bucket으로서의 이 저장소(릴리스마다 작성)
scripts/        릴리스 노트, 패키지 매니페스트, wheel과 npm 패키지, std 문서
assets/         로고, 배너와 소셜 미리보기
nexium-spec.txt          설계
nexium-systems-spec.txt  보관된 시스템 언어. 4~9절이 구문 레퍼런스
DECISIONS.md    명세가 열려 있던 곳에서 내린 결정
KNOWN_ISSUES.md 미해결 버그와 제한. 수정은 changelog로 옮겨감
```

## 기여

[`CONTRIBUTING.md`](../../../CONTRIBUTING.md)를 보세요. 버그와 제안은 GitHub 이슈로
다룹니다. 언어 변경은 명세 3절의 어떤 엄격한 제약을 위한 것인지 밝혀야 합니다. 풀
리퀘스트는 병합 전에 세 플랫폼의 테스트, 포매터, changelog 검사,
[기여자 라이선스 동의서](../../../CLA.md)를 통과합니다. 저작권은 당신에게 남습니다.

## 라이선스

MIT. Copyright (c) 2026 Londopy.

<p align="center"><img src="https://raw.githubusercontent.com/Londopy/nexium/main/assets/logo.svg" alt="" width="48"></p>
