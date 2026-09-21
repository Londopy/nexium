$ErrorActionPreference = 'Stop'
[array]$keys = Get-UninstallRegistryKey -SoftwareName 'Nexium*'
if ($keys.Count -eq 1) {
    $keys | ForEach-Object {
        Uninstall-ChocolateyPackage -PackageName 'nexium' -FileType 'exe' -SilentArgs '/VERYSILENT /SUPPRESSMSGBOXES /NORESTART' -File ($_.UninstallString.Trim('"'))
    }
} elseif ($keys.Count -eq 0) {
    Write-Warning 'nexium is not installed'
} else {
    Write-Warning "$($keys.Count) programs match Nexium*; uninstall from Settings"
}
