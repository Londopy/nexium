$ErrorActionPreference = 'Stop'
$packageArgs = @{
    packageName    = 'nexium'
    fileType       = 'exe'
    url64bit       = 'https://github.com/Londopy/nexium/releases/download/v1.0.2/nexium-1.0.2-setup-x64.exe'
    checksum64     = '2b1e55270b25c4db77cd05f67dbbd4602c7e81d3f3e1f8a32944a60ce3cbef59'
    checksumType64 = 'sha256'
    silentArgs     = '/VERYSILENT /SUPPRESSMSGBOXES /NORESTART /TASKS=addtopath'
    validExitCodes = @(0)
    softwareName   = 'Nexium*'
}
Install-ChocolateyPackage @packageArgs
