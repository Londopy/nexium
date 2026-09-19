# Releasing a Nexium program

How to turn a `.nx` program into downloadable binaries for Windows, Linux,
and macOS from a GitHub repository, with installers as an optional extra.

## What `nx` produces

| command | output |
| --- | --- |
| `nx build app.nx` | a native executable: `app.exe` on Windows, an ELF binary on Linux, a Mach-O binary on macOS |
| `nx build app.nx --target x86_64-linux-gnu` | the same, cross-compiled; any target `zig cc` knows works from any host |
| `nx build app.nx --mode small` | size-optimized |
| `nx ship app.nx` | every declared `artifact`: shared and static libraries with a C header, a Python wheel, a Rust crate |

Executables depend only on the target's C library. There is no runtime to
install next to them.

Installers (`.dmg`, `.msi`, `.deb`) are not produced by `nx`; the platform
tools make them from the binary, and the workflow below shows where.

## The workflow

Copy [`templates/release-nexium-program.yml`](templates/release-nexium-program.yml)
to `.github/workflows/release.yml` in your repository and change the two
values at the top: the source file and the program name. Then:

```bash
git tag v1.0.0 && git push origin v1.0.0
```

The tag triggers a build on three runners. Each installs Zig and `nx`
through the `Londopy/nexium/.github/actions/setup-nexium` action, builds the
program, packages it as a `.zip` (Windows) or `.tar.gz` (Linux, macOS), and
uploads the three files to a GitHub Release whose notes are the tag's
annotation. The whole thing is:

```yaml
- uses: actions/checkout@v4
- uses: Londopy/nexium/.github/actions/setup-nexium@main
  with:
    version: v0.1.0          # pin the compiler
- run: nx build ${{ env.SOURCE }} --mode fast -o dist/${{ env.NAME }}
```

`setup-nexium` downloads the matching `nx` binary from Nexium's Releases and
puts it and Zig on `PATH`. On the macOS runner `nx` uses the system compiler
for native builds, since zig 0.14 cannot link against the newest Xcode SDKs;
zig is still what cross-compiles. Pinning `version` keeps your builds reproducible;
`latest` is the default.

## Installers, when you want them

The template has three optional jobs, off by default, each turning the built
binary into an installer:

- **macOS `.dmg`**: `hdiutil create` around a folder holding the binary (or a
  `.app` bundle, if you make one). No signing is required for a dmg to open,
  but Gatekeeper warns about unsigned apps; signing and notarization need an
  Apple developer account and are out of scope here.
- **Windows `.msi`**: [WiX](https://wixtoolset.org/) from a small `.wxs`
  file. Inno Setup is the simpler alternative if you prefer an `.exe`
  installer.
- **Linux `.deb`**: a `DEBIAN/control` file plus the binary under
  `usr/bin`, packed with `dpkg-deb`.

Each job is a few lines and is annotated in the template. Turn one on by
setting its `if:` to `true`.

## An installer for your program

Declare it next to the `cli` artifact and `nx ship` produces one:

```nexium
artifact cli { name = "taskdesk" }
artifact installer {
    name = "TaskDesk", publisher = "Londopy", version = "1.2.0",
    url = "https://example.com/taskdesk",
    license = "LICENSE", readme = "README.md",
    files = ["assets", "config.toml"],   // copied next to the program
    add_to_path = true,
}
```

- On Windows, `nx ship` writes `nx-out/taskdesk/TaskDesk.iss` and, when
  [Inno Setup 6](https://jrsoftware.org/isinfo.php) is installed (or the
  `ISCC` environment variable points at `ISCC.exe`),
  `TaskDesk-1.2.0-setup-x64.exe`: a wizard with license page, install for
  one user or all, Start menu entry, an optional PATH entry, and an
  uninstaller. The app id derives from the name, so a newer setup upgrades
  in place. Silent install: `setup.exe /VERYSILENT /CURRENTUSER`.
- On Linux and macOS, it writes `install.sh` (`--prefix DIR`, default
  `~/.local`; `--uninstall`) and `taskdesk-1.2.0-<os>.tar.gz` holding the
  program, the script and the listed files.

In the release template, the Windows job installs Inno Setup with
`choco install innosetup` before `nx ship`, and every job uploads
`nx-out/<name>/`.

## Shipping a library instead

If the program declares `artifact cabi`, `artifact python`, `artifact node`
or `artifact rustlib`, replace the build step with `nx ship` and upload the
`nx-out/<name>/` directory. The Python wheel and the npm package are
platform specific, so build them on each runner; the C header and the Rust
crate source are the same everywhere.

The npm package (`nx-out/<name>/node/`) is plain JavaScript over the shared
library through [koffi](https://koffi.dev), with `index.d.ts` typings: no
build step for the consumer, and `npm publish` from that directory ships
it. Its `os` and `cpu` fields name the platform it was built on; publish
one package per platform under a scoped name, or merge the shared libraries
of every platform into one package and pick by `process.platform`.

## Checklist before the first tag

- `nx test app.nx` and `nx check app.nx` pass locally.
- `nx build app.nx --target aarch64-macos` and `--target x86_64-linux-gnu`
  work from your machine, which catches platform assumptions early.
- The program handles `--version` or prints something on start, so a user can
  verify the download.
- A `LICENSE` file is in the repository; the template copies it into every
  archive.
