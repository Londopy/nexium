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
  <a href="https://crates.io/crates/nexium"><img alt="crates.io" src="https://img.shields.io/crates/v/nexium?logo=rust&color=4fd1c5"></a>
  <a href="../../../LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-blue"></a>
  <a href="https://ziglang.org/download/"><img alt="Zig" src="https://img.shields.io/badge/backend-zig%20cc-f7a41d?logo=zig&logoColor=white"></a>
  <img alt="Platforms" src="https://img.shields.io/badge/platforms-windows%20%7C%20linux%20%7C%20macos-2b3a55">
</p>

<p align="center">
  <b>모든 것을 만들 수 있을 만큼 완전하면서도, 다른 무언가의 한 조각으로 도입하기에 가장 좋은 언어.</b>
</p>

Nexium은 C를 거쳐 네이티브 코드로 컴파일되고, 추적 가비지 컬렉터 없이 자동
참조 계수를 사용하며, 함수가 메모리를 할당하는지, 블로킹하는지, 패닉할 수
있는지를 기계가 검사하는 효과 시스템을 갖추고 있습니다. 컴파일러는 하나의
소스 트리를 C 라이브러리, Python wheel, Rust crate, 또는 명령줄 도구로
바꿉니다.

<table>
<tr>
<td width="50%" valign="top">

**하나의 파일**

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

**모든 대상**

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

효과 시그니처가 C ABI를 결정합니다. `checksum`은 `!panics`로 증명되었으므로
평범한 `uint32_t checksum(const uint8_t*, size_t)`를 얻습니다. 실패할 수 있는
함수는 상태 코드를 반환하고, 그 안에서 발생한 Nexium 패닉은 호스트 프로세스를
중단시키는 대신 경계에서 변환됩니다.

## 주요 특징

| | |
| --- | --- |
| 🧾 **추론되고 검사되는 효과** | `allocates` `refcounts` `blocks` `shared_mutable` `nondeterministic` `panics` `ffi`. `!allocates`를 선언하면 컴파일러가 호출을 따라가 그것을 깨뜨리는 정확한 줄을 가리킵니다. |
| 🧠 **빌림 검사기 없는 소유권** | 컬렉션은 이동되고, `.clone()`은 복사하며, `ref class` 값은 참조 계수되고, `weak`가 순환을 끊습니다. 이동 후 사용은 컴파일 오류입니다. |
| 🔬 **이진 패턴** | `<<version:4, ihl:4, len:16/big, rest:bytes>>`가 검사된 크기로 패킷을 매칭하고 구성합니다. |
| 🧵 **병렬 루프, 아레나, 트레이트 객체** | `for parallel`, `using arena { }`, `dyn Trait !allocates`. |
| 🔌 **바인딩 없는 C 호출** | `@cImport("header.h")`가 헤더를 직접 읽고, `artifact link`가 동봉된 C를 프로그램에 컴파일해 넣습니다. |
| 📦 **하나의 소스에서 배포** | `nx ship`이 C 헤더와 라이브러리, Python wheel, 안전한 래퍼가 있는 Rust crate를 만듭니다. |
| 🖼 **Nexium으로 쓴 GUI** | [`gui/`](../../../gui): 소프트웨어 래스터라이저와 비트맵 폰트를 갖춘 즉시 모드 GUI(버튼, 슬라이더, 텍스트 필드). 200줄짜리 C 창 계층 위는 전부 Nexium입니다. |
| 🛠 **기본 제공 도구** | `fmt`, `doc`, `lsp`, `size`, `leaks`, `refcounts`, `effects`, `audit`. 의존성 없음. |

## 설치

실행 시 필요한 것은 `PATH`에 있는 [Zig](https://ziglang.org/download/)뿐이며,
C 컴파일러로 사용됩니다(`zig cc`는 크로스 컴파일도 가능하고, `--cc clang`도
됩니다).

Windows, Linux, macOS용으로 미리 빌드된 `nx` 바이너리는
[Releases](https://github.com/Londopy/nexium/releases) 페이지에 있습니다. 압축을
풀고 `nx`를 `PATH`에 두세요.

또는 Rust 1.75 이상으로 소스에서 빌드합니다:

```bash
cargo install nexium
```

```bash
cargo install --git https://github.com/Londopy/nexium
```

그다음:

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
<summary><b>이진 패턴 매칭</b></summary>

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
<summary><b>효과는 추론되고 검사됩니다</b></summary>

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
<summary><b>C 호출은 헤더 하나를 가져오는 것으로 끝납니다</b></summary>

```
const libc = @cImport("string.h")
const cv = @cImport("cvendor.h")
artifact link { c_sources = ["cvendor.c"] }

unsafe { println("{}", .{libc.strlen(@cstr("hello"))}) }
```

바인딩 생성기도 빌드 단계도 없습니다. 헤더가 메모리 배치의 유일한 진실이며,
외부 호출은 `ffi` 효과를 가집니다.

</details>

<details>
<summary><b>병렬 루프와 아레나</b></summary>

```
for parallel p, i in positions {
    out[i] = integrate(p)          // 여기서는 shared_mutable이 허용되지 않음
}

using arena {
    var scratch = List(Frame).new()   // 범프 할당, 한 번에 전부 해제
    ...
}
```

</details>

**문서**

- [Nexium의 동작 원리](../../architecture.md): 소스에서 바이너리까지의 파이프라인, 효과 추론, 소유권, 런타임, 배포.
- [언어 레퍼런스](../../language.md): 컴파일러가 구현하는 모든 구문.
- [임베딩](../../embedding.md) (영어): 배포된 라이브러리를 Python, Rust, C에서 호출하기.
- [nexium-gui](../../gui.md) (영어): 즉시 모드 GUI 라이브러리와 위젯 작성법.
- [프로그램 릴리스](../../releasing-your-program.md) (영어): 태그 하나로 세 플랫폼용 바이너리, 설치 프로그램은 선택.
- [편집기 지원](../../../editors) (영어): VS Code 확장, Sublime 문법, LSP.
- [설계 결정](../../../DECISIONS.md) (영어): 명세가 열어 둔 곳에서 내린 모든 결정.

## 명령

| 명령 | 하는 일 |
| --- | --- |
| `nx build file.nx` | 실행 파일로 컴파일(`main`이 없으면 오브젝트) |
| `nx run file.nx` | 빌드하고 실행 |
| `nx test file.nx [filter]` | `test "..."` 블록 실행 |
| `nx check file.nx` | 타입 검사와 효과 위반 보고 |
| `nx effects file.nx` | 각 함수의 추론된 효과 출력 |
| `nx audit file.nx` | `unsafe` 블록과 가변 전역 나열 |
| `nx ship file.nx` | 선언된 모든 `artifact` 생성 |
| `nx emit-c file.nx` | 생성된 C 출력 |
| `nx tokens file.nx` | 토큰 스트림 덤프(셀프 호스팅의 기준) |
| `nx fmt file.nx [--check]` | 정규 포매팅 |
| `nx doc file.nx` | 추론된 효과가 포함된 HTML 문서 |
| `nx size file.nx` | 바이너리 바이트를 선언별로 귀속 |
| `nx refcounts file.nx` | 모든 retain과 release 지점 |
| `nx leaks file.nx` | 할당 추적과 함께 실행하고 누수 보고 |
| `nx lsp` | stdio 기반 언어 서버 |

옵션: `--mode debug|safe|fast|small`, `--target x86_64-linux-gnu`(`zig cc`가
아는 모든 대상), `--out-dir`, `--keep-c`, `--cc`, 그리고 C 연동용 `-I`,
`--link`, `--link-path`, `--c-source`.

## 현재 상태

이것은 `nexium-spec.txt` 설계의 첫 구현입니다. 실제 프로그램을 쓰기에 충분하고
([`examples/`](../../../examples) 참조), 하나의 파일에서 Python, Rust, C
컴포넌트를 배포할 수 있습니다. 트레이트 객체, 병렬 루프, 아레나 스코프, C 헤더
직접 가져오기, 도구들이 모두 들어 있습니다. 아직 초기입니다. 표준 라이브러리는
16절의 일부이며, 영역 검사는 반환되는 뷰만 다룹니다.
[`DECISIONS.md`](../../../DECISIONS.md)에 명세가 열어 둔 곳에서 내린 모든 결정이
있고, 27번 항목에 남은 일이 정리되어 있습니다.

## 셀프 호스팅

컴파일러는 현재 Rust입니다. Nexium 버전은 [`self/`](../../../self)에서 한 단계씩
자라며, 각 단계는 같은 입력으로 Rust 컴파일러와 대조됩니다:

| 단계 | 파일 | 기준 | 상태 |
| --- | --- | --- | --- |
| 렉서 | [`self/lexer.nx`](../../../self/lexer.nx) | `nx tokens` | ✅ 모든 예제와 자기 자신에서 일치 |
| 파서 | [`self/parser.nx`](../../../self/parser.nx) | `nx sexp` | ✅ 46개 소스 전부에서 일치 |
| 검사기 | [`self/check.nx`](../../../self/check.nx) | `nx tir` | 🚧 선언과 시그니처 일치 (`--sigs`, 41개 소스); 본문 진행 중 |
| C 생성기 | | `nx emit-c` | |

`cargo test`는 Rust 컴파일러로 `self/lexer.nx`를 빌드하고 그 출력을 기준과
비교합니다.

## 저장소의 언어 구성

빌드 산출물, 의존성, 생성 파일을 제외한 비어 있지 않은 코드 줄 수:

| 언어 | 줄 | 비율 | 용도 |
| --- | --- | --- | --- |
| Rust | 22 393 | 86.9 % | `nx` 컴파일러 |
| Nexium | 2 111 | 8.2 % | 예제, 셀프 호스트 렉서, nexium-gui, 테스트 |
| C | 1 108 | 4.3 % | 런타임 `nx_rt.h`와 GUI 창 계층 |
| JavaScript, TypeScript | 159 | 0.6 % | VS Code 확장 |

## 구성

```
src/            컴파일러(렉서, 파서, 검사기, comptime, C 백엔드, 드라이버)
runtime/        nx_rt.h, 생성되는 모든 C 파일에 포함됨
self/           Nexium으로 쓴 컴파일러, 단계별
gui/            nexium-gui: Nexium 즉시 모드 GUI, 데모, C 플랫폼 계층
editors/        VS Code 확장과 Sublime Text 문법
examples/       출력이 기록된 프로그램, `cargo test`가 실행
tests/          통합 테스트와 compile-fail 케이스
docs/           동작 원리, 언어 레퍼런스, 임베딩 가이드, 번역
assets/         로고와 배너
nexium-spec.txt          설계
nexium-systems-spec.txt  보관된 시스템 언어. 4~9절이 문법 레퍼런스
DECISIONS.md    명세가 열어 둔 곳에서 내린 결정
```

## 기여

[`CONTRIBUTING.md`](../../../CONTRIBUTING.md)를 보세요. 버그와 제안은 GitHub
issues로 올립니다. 언어 변경은 그것이 명세 3절의 어떤 엄격한 제약에 기여하는지
밝혀야 합니다.

## 라이선스

MIT. Copyright (c) 2026 Londopy.

<p align="center"><img src="https://raw.githubusercontent.com/Londopy/nexium/main/assets/logo.svg" alt="" width="48"></p>
