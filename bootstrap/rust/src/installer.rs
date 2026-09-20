//! `artifact installer`: an installer for a Nexium program.
//!
//! ```nexium
//! artifact cli { name = "taskdesk" }
//! artifact installer { name = "TaskDesk", publisher = "Londopy", version = "1.2.0",
//!                      url = "https://example.com", license = "LICENSE", readme = "README.md",
//!                      files = ["assets", "config.toml"], add_to_path = true }
//! ```
//!
//! `nx ship` writes, next to the built executable:
//! - on Windows, `<name>.iss` for Inno Setup and runs `ISCC.exe` when it is
//!   installed, producing `<name>-<version>-setup-x64.exe`: wizard, install
//!   for one user or all, Start menu entry, optional PATH entry, uninstaller;
//! - on Linux and macOS, `install.sh`, which copies the program to
//!   `~/.local/bin` (or a `--prefix`) with its files under
//!   `~/.local/share/<name>` and prints the PATH line to add, plus a
//!   `<name>-<version>-<os>.tar.gz` holding both.
//!
//! Only the values of the artifact fields feed the templates; strings are
//! escaped for the format they land in.

use std::path::{Path, PathBuf};
use std::process::Command;

pub struct Spec {
    pub name: String,
    pub exe: String,
    pub version: String,
    pub publisher: String,
    pub url: String,
    pub license: Option<String>,
    pub readme: Option<String>,
    pub files: Vec<String>,
    pub add_to_path: bool,
}

fn iss_str(s: &str) -> String {
    s.replace('"', "\"\"")
}

/// A stable GUID-shaped id from the name, so upgrades find the previous install.
fn app_id(name: &str) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in name.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    let mut g: u64 = 0x9e3779b97f4a7c15;
    for b in name.bytes().rev() {
        g ^= b as u64;
        g = g.wrapping_mul(0x100000001b3);
    }
    // Inno reads a leading `{` as a constant; `{{` is a literal brace
    format!("{{{{{:08X}-{:04X}-4{:03X}-9{:03X}-{:012X}}}", (h >> 32) as u32, (h >> 16) as u16, (h & 0xfff) as u16, (g >> 52) as u16 & 0xfff, g & 0xffffffffffff)
}

/// The script sits next to the program's files, so every path in it is
/// relative to `{#SourcePath}`, the script's own directory.
pub fn inno_script(spec: &Spec, source_dir: &Path) -> String {
    let mut s = String::new();
    s.push_str(&format!("; {} installer, written by `nx ship`\n\n", spec.name));
    s.push_str("[Setup]\n");
    s.push_str(&format!("AppId={}\n", app_id(&spec.name)));
    s.push_str(&format!("AppName={}\n", iss_str(&spec.name)));
    s.push_str(&format!("AppVersion={}\n", iss_str(&spec.version)));
    s.push_str(&format!("AppVerName={} {}\n", iss_str(&spec.name), iss_str(&spec.version)));
    s.push_str(&format!("AppPublisher={}\n", iss_str(&spec.publisher)));
    if !spec.url.is_empty() {
        s.push_str(&format!("AppPublisherURL={}\nAppSupportURL={}\n", iss_str(&spec.url), iss_str(&spec.url)));
    }
    s.push_str(&format!("DefaultDirName={{autopf}}\\{}\n", iss_str(&spec.name)));
    s.push_str(&format!("DefaultGroupName={}\n", iss_str(&spec.name)));
    s.push_str("DisableProgramGroupPage=yes\n");
    if let Some(l) = &spec.license {
        s.push_str(&format!("LicenseFile={{#SourcePath}}\\{}\n", iss_str(l)));
    }
    s.push_str("OutputDir={#SourcePath}\n");
    s.push_str(&format!("OutputBaseFilename={}-{}-setup-x64\n", iss_str(&spec.name.replace(' ', "")), iss_str(&spec.version)));
    s.push_str(&format!("UninstallDisplayIcon={{app}}\\{}\n", iss_str(&spec.exe)));
    s.push_str("Compression=lzma2\nSolidCompression=yes\nWizardStyle=modern\nPrivilegesRequired=lowest\nPrivilegesRequiredOverridesAllowed=dialog\nArchitecturesAllowed=x64compatible\nArchitecturesInstallIn64BitMode=x64compatible\n");
    if spec.add_to_path {
        s.push_str("ChangesEnvironment=yes\n");
    }
    s.push_str("\n[Languages]\nName: \"english\"; MessagesFile: \"compiler:Default.isl\"\n\n");
    if spec.add_to_path {
        s.push_str("[Tasks]\nName: \"addtopath\"; Description: \"Add the program to the PATH\"; GroupDescription: \"Environment:\"\n\n");
    }
    s.push_str("[Files]\n");
    s.push_str(&format!("Source: \"{{#SourcePath}}\\{}\"; DestDir: \"{{app}}\"; Flags: ignoreversion\n", iss_str(&spec.exe)));
    if let Some(r) = &spec.readme {
        s.push_str(&format!("Source: \"{{#SourcePath}}\\{}\"; DestDir: \"{{app}}\"; Flags: ignoreversion skipifsourcedoesntexist\n", iss_str(r)));
    }
    if let Some(l) = &spec.license {
        s.push_str(&format!("Source: \"{{#SourcePath}}\\{}\"; DestDir: \"{{app}}\"; Flags: ignoreversion skipifsourcedoesntexist\n", iss_str(l)));
    }
    for f in &spec.files {
        if source_dir.join(f).is_dir() {
            s.push_str(&format!(
                "Source: \"{{#SourcePath}}\\{}\\*\"; DestDir: \"{{app}}\\{}\"; Flags: recursesubdirs createallsubdirs ignoreversion skipifsourcedoesntexist\n",
                iss_str(f),
                iss_str(f)
            ));
        } else {
            s.push_str(&format!("Source: \"{{#SourcePath}}\\{}\"; DestDir: \"{{app}}\"; Flags: ignoreversion skipifsourcedoesntexist\n", iss_str(f)));
        }
    }
    s.push_str("\n[Icons]\n");
    s.push_str(&format!("Name: \"{{group}}\\{}\"; Filename: \"{{app}}\\{}\"; WorkingDir: \"{{app}}\"\n", iss_str(&spec.name), iss_str(&spec.exe)));
    s.push_str(&format!("Name: \"{{group}}\\Uninstall {}\"; Filename: \"{{uninstallexe}}\"\n", iss_str(&spec.name)));
    s.push_str("\n[Run]\n");
    if let Some(r) = &spec.readme {
        s.push_str(&format!(
            "Filename: \"{{app}}\\{}\"; Description: \"Open the README\"; Flags: postinstall shellexec skipifsilent unchecked\n",
            iss_str(Path::new(r).file_name().map(|x| x.to_string_lossy().to_string()).unwrap_or_default().as_str())
        ));
    }
    if spec.add_to_path {
        s.push_str(
            r#"
[Code]
function EnvKey(): string;
begin
  if IsAdminInstallMode then
    Result := 'SYSTEM\CurrentControlSet\Control\Session Manager\Environment'
  else
    Result := 'Environment';
end;

function EnvRoot(): Integer;
begin
  if IsAdminInstallMode then
    Result := HKEY_LOCAL_MACHINE
  else
    Result := HKEY_CURRENT_USER;
end;

procedure EnvAddPath(Dir: string);
var
  Paths: string;
begin
  if not RegQueryStringValue(EnvRoot(), EnvKey(), 'Path', Paths) then
    Paths := '';
  if Pos(';' + Uppercase(Dir) + ';', ';' + Uppercase(Paths) + ';') > 0 then
    exit;
  if (Paths <> '') and (Paths[Length(Paths)] <> ';') then
    Paths := Paths + ';';
  Paths := Paths + Dir;
  RegWriteExpandStringValue(EnvRoot(), EnvKey(), 'Path', Paths);
end;

procedure EnvRemovePath(Dir: string);
var
  Paths: string;
  P: Integer;
begin
  if not RegQueryStringValue(EnvRoot(), EnvKey(), 'Path', Paths) then
    exit;
  P := Pos(';' + Uppercase(Dir) + ';', ';' + Uppercase(Paths) + ';');
  if P = 0 then
    exit;
  Delete(Paths, P - 1, Length(Dir) + 1);
  RegWriteExpandStringValue(EnvRoot(), EnvKey(), 'Path', Paths);
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if (CurStep = ssPostInstall) and WizardIsTaskSelected('addtopath') then
    EnvAddPath(ExpandConstant('{app}'));
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then
    EnvRemovePath(ExpandConstant('{app}'));
end;
"#,
        );
    }
    s
}

fn sh_str(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

pub fn install_script(spec: &Spec) -> String {
    let mut extra = String::new();
    for f in &spec.files {
        extra.push_str(&format!("  copy_tree {} \"$share\"\n", sh_str(f)));
    }
    if let Some(r) = &spec.readme {
        extra.push_str(&format!("  copy_tree {} \"$share\"\n", sh_str(r)));
    }
    if let Some(l) = &spec.license {
        extra.push_str(&format!("  copy_tree {} \"$share\"\n", sh_str(l)));
    }
    format!(
        r#"#!/bin/sh
# {name} {version} installer, written by `nx ship`.
# Usage: ./install.sh [--prefix DIR] [--uninstall]
#   installs {exe} into DIR/bin (default ~/.local) and the program's files
#   into DIR/share/{slug}.
set -e
name={name_q}
exe={exe_q}
slug={slug_q}
prefix="$HOME/.local"
mode=install
while [ $# -gt 0 ]; do
  case "$1" in
    --prefix) prefix="$2"; shift 2 ;;
    --uninstall) mode=uninstall; shift ;;
    -h|--help) sed -n '2,5p' "$0"; exit 0 ;;
    *) echo "unknown option: $1" >&2; exit 2 ;;
  esac
done
here=$(cd "$(dirname "$0")" && pwd)
bin="$prefix/bin"
share="$prefix/share/$slug"
if [ "$mode" = uninstall ]; then
  rm -f "$bin/$exe"
  rm -rf "$share"
  echo "removed $name from $prefix"
  exit 0
fi
copy_tree() {{
  [ -e "$here/$1" ] || return 0
  mkdir -p "$2"
  cp -R "$here/$1" "$2/"
}}
mkdir -p "$bin" "$share"
cp "$here/$exe" "$bin/$exe"
chmod +x "$bin/$exe"
{extra}echo "installed $name $version to $bin/$exe"
case ":$PATH:" in
  *":$bin:"*) ;;
  *) echo "add it to your PATH:  export PATH=\"$bin:\$PATH\"" ;;
esac
"#,
        name = spec.name,
        version = spec.version,
        exe = spec.exe,
        slug = spec.name.to_lowercase().replace(' ', "-"),
        name_q = sh_str(&spec.name),
        exe_q = sh_str(&spec.exe),
        slug_q = sh_str(&spec.name.to_lowercase().replace(' ', "-")),
        extra = extra,
    )
}

/// Where Inno Setup's compiler lives, when it is installed.
pub fn find_iscc() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("ISCC") {
        let p = PathBuf::from(p);
        if p.exists() {
            return Some(p);
        }
    }
    for base in ["C:\\Program Files (x86)\\Inno Setup 6", "C:\\Program Files\\Inno Setup 6"] {
        let p = Path::new(base).join("ISCC.exe");
        if p.exists() {
            return Some(p);
        }
    }
    let local = std::env::var("LOCALAPPDATA").ok().map(|l| Path::new(&l).join("Programs").join("Inno Setup 6").join("ISCC.exe"));
    local.filter(|p| p.exists())
}

/// Build the installer for the program at `exe_path` into its directory.
/// Returns the files written (the script, and the setup exe or tarball when
/// the tools to make them were found).
pub fn produce(spec: &Spec, exe_path: &Path, windows: bool) -> Result<Vec<PathBuf>, String> {
    let dir = exe_path.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from("."));
    let mut out = Vec::new();
    if windows {
        let iss = dir.join(format!("{}.iss", spec.name.replace(' ', "")));
        std::fs::write(&iss, inno_script(spec, &dir)).map_err(|e| format!("cannot write {}: {}", iss.display(), e))?;
        out.push(iss.clone());
        match find_iscc() {
            Some(iscc) => {
                let abs = std::fs::canonicalize(&iss).unwrap_or_else(|_| iss.clone());
                let abs_text = abs.to_string_lossy().to_string();
                let abs_text = abs_text.strip_prefix("\\\\?\\").unwrap_or(&abs_text).to_string();
                let status = Command::new(&iscc).arg("/Q").arg(&abs_text).status().map_err(|e| format!("cannot run {}: {}", iscc.display(), e))?;
                if !status.success() {
                    return Err(format!("Inno Setup failed on {}", iss.display()));
                }
                out.push(dir.join(format!("{}-{}-setup-x64.exe", spec.name.replace(' ', ""), spec.version)));
            }
            None => eprintln!("note: Inno Setup (ISCC.exe) not found; wrote {} to compile later (install Inno Setup 6 or set ISCC)", iss.display()),
        }
    } else {
        let sh = dir.join("install.sh");
        std::fs::write(&sh, install_script(spec)).map_err(|e| format!("cannot write {}: {}", sh.display(), e))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&sh, std::fs::Permissions::from_mode(0o755));
        }
        out.push(sh.clone());
        let os = if cfg!(target_os = "macos") { "macos" } else { "linux" };
        let tar = dir.join(format!("{}-{}-{}.tar.gz", spec.name.to_lowercase().replace(' ', "-"), spec.version, os));
        let mut members: Vec<String> = vec![spec.exe.clone(), "install.sh".into()];
        for f in spec.files.iter().chain(spec.readme.iter()).chain(spec.license.iter()) {
            if dir.join(f).exists() {
                members.push(f.clone());
            }
        }
        let status = Command::new("tar").arg("-czf").arg(&tar).arg("-C").arg(&dir).args(&members).status();
        match status {
            Ok(s) if s.success() => out.push(tar),
            _ => eprintln!("note: tar not available; wrote install.sh without the archive"),
        }
    }
    Ok(out)
}
