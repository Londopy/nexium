$ErrorActionPreference = 'Stop'
$packageArgs = @{
    packageName    = 'nexium'
    fileType       = 'exe'
    url64bit       = 'https://github.com/Londopy/nexium/releases/download/v1.5.0/nexium-1.5.0-setup-x64.exe'
    checksum64     = 'af0eb9303c8a5c23ef2d9a8cc63d993d0d7ba44f51c20a62399ad33f398fc74e'
    checksumType64 = 'sha256'
    silentArgs     = '/VERYSILENT /SUPPRESSMSGBOXES /NORESTART /TASKS=addtopath'
    validExitCodes = @(0)
    softwareName   = 'Nexium*'
}
Install-ChocolateyPackage @packageArgs
