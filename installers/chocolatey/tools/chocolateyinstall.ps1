$ErrorActionPreference = 'Stop'
$packageArgs = @{
    packageName    = 'nexium'
    fileType       = 'exe'
    url64bit       = 'https://github.com/Londopy/nexium/releases/download/v1.3.2/nexium-1.3.2-setup-x64.exe'
    checksum64     = 'e25542715fce2812e7f99f02ba0d82e2bda4475639c888f86c764997a2f6bdfa'
    checksumType64 = 'sha256'
    silentArgs     = '/VERYSILENT /SUPPRESSMSGBOXES /NORESTART /TASKS=addtopath'
    validExitCodes = @(0)
    softwareName   = 'Nexium*'
}
Install-ChocolateyPackage @packageArgs
