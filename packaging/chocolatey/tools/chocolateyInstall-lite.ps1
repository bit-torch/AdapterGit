$ErrorActionPreference = 'Stop'

$packageName = 'agit-lite'
$version = '0.14.0'
$url = "https://github.com/bit-torch/AdapterGit/releases/download/v$version/agit-lite-windows-x86_64.exe"
$checksum = 'REPLACE_WITH_SHA256'

$toolsDir = "$(Split-Path -Parent $MyInvocation.MyCommand.Definition)"
$installDir = "$env:ChocolateyInstall\lib\$packageName\tools"

# Lite edition is a single binary — just download it
$exePath = Join-Path $installDir 'agit.exe'
Get-ChocolateyWebFile `
    -PackageName $packageName `
    -FileFullPath $exePath `
    -Url $url `
    -Checksum $checksum `
    -ChecksumType 'sha256'

# Add to PATH (machine-wide)
Install-ChocolateyPath `
    -PathToInstall $installDir `
    -PathType 'Machine'

Write-Host "agit (Lite Edition) v$version installed to $installDir"
Write-Host "Run 'agit --help' to get started."
