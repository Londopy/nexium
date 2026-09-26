$ErrorActionPreference = 'Stop'
$packageArgs = @{
    packageName    = 'nexium'
    fileType       = 'exe'
    url64bit       = 'https://github.com/Londopy/nexium/releases/download/v1.4.0/nexium-1.4.0-setup-x64.exe'
    checksum64     = '1195110a3724a891abd1cf063a77e0fd8db5c51fa2343fdaea84b3acd02ac5a2'
    checksumType64 = 'sha256'
    silentArgs     = '/VERYSILENT /SUPPRESSMSGBOXES /NORESTART /TASKS=addtopath'
    validExitCodes = @(0)
    softwareName   = 'Nexium*'
}
Install-ChocolateyPackage @packageArgs
