# Nexium for Windows, from a console:
#
#   irm https://raw.githubusercontent.com/Londopy/nexium/main/installers/install.ps1 | iex
#
# Installs the portable build (nx.exe, the standard library, the examples,
# the docs) into $env:LOCALAPPDATA\Programs\Nexium, verifies it against the
# release's SHA256SUMS.txt, adds the directory to the user's PATH, and
# downloads Zig beside it when no C compiler is found. The installer from the
# Releases page does the same with a wizard, plus the file type, the folder
# menu and the Windows Terminal profile.
#
#   $env:NEXIUM_VERSION = 'v1.0.3'       a specific release (default: the latest)
#   $env:NEXIUM_HOME = 'C:\tools\nexium'  the directory
#   $env:NEXIUM_NO_MODIFY_PATH = '1'      leave the PATH alone
#   $env:NEXIUM_NO_ZIG = '1'              never download Zig

$ErrorActionPreference = 'Stop'
[Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12

$repo = 'Londopy/nexium'
$zigVersion = '0.14.1'
$version = if ($env:NEXIUM_VERSION) { $env:NEXIUM_VERSION } else { 'latest' }
$dest = if ($env:NEXIUM_HOME) { $env:NEXIUM_HOME } else { Join-Path $env:LOCALAPPDATA 'Programs\Nexium' }

function Say($text) { Write-Host $text }

$arch = switch ($env:PROCESSOR_ARCHITECTURE) {
    'AMD64' { 'x86_64' }
    'ARM64' { 'aarch64' }
    default { throw "install.ps1: no release is built for $($env:PROCESSOR_ARCHITECTURE); see docs/install.md for the one C file" }
}

# the release
$headers = @{ 'User-Agent' = 'nexium-install.ps1' }
if ($version -eq 'latest') {
    $latest = Invoke-RestMethod "https://api.github.com/repos/$repo/releases/latest" -Headers $headers
    $version = $latest.tag_name
}
$base = "https://github.com/$repo/releases/download/$version"
$asset = "nx-$version-$arch-pc-windows-msvc.zip"

$tmp = Join-Path ([IO.Path]::GetTempPath()) ('nexium-install-' + [IO.Path]::GetRandomFileName())
New-Item -ItemType Directory -Force $tmp | Out-Null
try {
    Say "downloading $asset"
    Invoke-WebRequest "$base/$asset" -OutFile (Join-Path $tmp $asset) -UseBasicParsing -Headers $headers
    Invoke-WebRequest "$base/SHA256SUMS.txt" -OutFile (Join-Path $tmp 'SHA256SUMS.txt') -UseBasicParsing -Headers $headers

    # verify
    $line = Get-Content (Join-Path $tmp 'SHA256SUMS.txt') | Where-Object { $_ -match ('\s' + [regex]::Escape($asset) + '$') }
    if (-not $line) { throw "no checksum for $asset in SHA256SUMS.txt" }
    $expected = (($line | Select-Object -First 1) -split '\s+')[0].ToLower()
    $actual = (Get-FileHash (Join-Path $tmp $asset) -Algorithm SHA256).Hash.ToLower()
    if ($actual -ne $expected) { throw "checksum mismatch for $asset (expected $expected, got $actual)" }
    Say 'checksum ok'

    # install: the archive's contents, the directories replaced whole
    $stage = Join-Path $tmp 'stage'
    Expand-Archive (Join-Path $tmp $asset) -DestinationPath $stage -Force
    New-Item -ItemType Directory -Force $dest | Out-Null
    foreach ($d in 'examples', 'std', 'docs') {
        $old = Join-Path $dest $d
        if (Test-Path $old) { Remove-Item -Recurse -Force $old }
    }
    Copy-Item -Recurse -Force (Join-Path $stage '*') $dest
    Say "installed nx $version to $dest\nx.exe"

    # a C compiler: nx looks for zig next to itself first, then on the PATH
    $compiler = $null
    if (Test-Path (Join-Path $dest 'zig\zig.exe')) { $compiler = 'bundled zig' }
    elseif (Get-Command zig -ErrorAction SilentlyContinue) { $compiler = 'zig on the PATH' }
    if (-not $compiler -and $env:NEXIUM_NO_ZIG -ne '1') {
        $name = "zig-$arch-windows-$zigVersion"
        Say "no C compiler found; downloading Zig $zigVersion into $dest\zig"
        $zigZip = Join-Path $tmp 'zig.zip'
        Invoke-WebRequest "https://ziglang.org/download/$zigVersion/$name.zip" -OutFile $zigZip -UseBasicParsing
        $index = Invoke-RestMethod 'https://ziglang.org/download/index.json'
        $want = $index.$zigVersion."$arch-windows".shasum
        if ($want) {
            $got = (Get-FileHash $zigZip -Algorithm SHA256).Hash.ToLower()
            if ($got -ne $want.ToLower()) { throw 'checksum mismatch for the Zig download' }
            Say 'zig checksum ok'
        } else {
            Say 'warning: could not verify the Zig download (index.json unavailable)'
        }
        $zigStage = Join-Path $tmp 'zig'
        Expand-Archive $zigZip -DestinationPath $zigStage -Force
        $inner = Get-ChildItem $zigStage -Directory | Select-Object -First 1
        $zigDir = Join-Path $dest 'zig'
        if (Test-Path $zigDir) { Remove-Item -Recurse -Force $zigDir }
        Move-Item $inner.FullName $zigDir
        $compiler = 'bundled zig'
    }
    if (-not $compiler) { Say 'warning: no C compiler found; install Zig (https://ziglang.org/download) or set NX_CC' }
    else { Say "C compiler: $compiler" }
} finally {
    Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
}

# PATH, for this user
if ($env:NEXIUM_NO_MODIFY_PATH -ne '1') {
    $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    $parts = @()
    if ($userPath) { $parts = @($userPath -split ';' | Where-Object { $_ }) }
    if ($parts -notcontains $dest) {
        [Environment]::SetEnvironmentVariable('Path', (($parts + $dest) -join ';'), 'User')
        Say "added $dest to the user PATH (new consoles see it)"
    }
}
$env:Path = "$dest;$env:Path"

Say ''
Say 'done. In a new console, try:'
Say '  nx doctor'
Say "  nx run $dest\examples\hello.nx"
& (Join-Path $dest 'nx.exe') doctor
