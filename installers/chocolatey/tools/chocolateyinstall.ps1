$ErrorActionPreference = 'Stop'
$packageArgs = @{
    packageName    = 'nexium'
    fileType       = 'exe'
    url64bit       = 'https://github.com/Londopy/nexium/releases/download/v1.0.3/nexium-1.0.3-setup-x64.exe'
    checksum64     = '39505f16929cdf2af5978556fedf28242d8b6cc6c3a81af60eae1e630db0fea1'
    checksumType64 = 'sha256'
    silentArgs     = '/VERYSILENT /SUPPRESSMSGBOXES /NORESTART /TASKS=addtopath'
    validExitCodes = @(0)
    softwareName   = 'Nexium*'
}
Install-ChocolateyPackage @packageArgs
