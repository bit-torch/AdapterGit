$ErrorActionPreference = 'Stop'

$packageName = 'agit-full'
$version = '0.14.0'
$url = "https://github.com/bit-torch/AdapterGit/releases/download/v$version/agit-full-windows-x86_64.zip"
$checksum = 'REPLACE_WITH_SHA256'

$toolsDir = "$(Split-Path -Parent $MyInvocation.MyCommand.Definition)"
$installDir = "$env:ChocolateyInstall\lib\$packageName\tools"

Install-ChocolateyZipPackage `
    -PackageName $packageName `
    -Url $url `
    -UnzipLocation $installDir `
    -Checksum $checksum `
    -ChecksumType 'sha256'

# Add to PATH (machine-wide)
Install-ChocolateyPath `
    -PathToInstall $installDir `
    -PathType 'Machine'

Write-Host "agit (Full Edition) v$version installed to $installDir"
Write-Host "Run 'agit --help' to get started."
