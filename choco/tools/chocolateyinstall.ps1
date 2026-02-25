$ErrorActionPreference = 'Stop'

$toolsDir = "$(Split-Path -Parent $MyInvocation.MyCommand.Definition)"
$url = "https://github.com/cat-forgor/AssignmentPacker/releases/download/v$($env:ChocolateyPackageVersion)/ap-windows-x64.exe"

$packageArgs = @{
  packageName   = $env:ChocolateyPackageName
  fileFullPath  = Join-Path $toolsDir 'ap.exe'
  url64bit      = $url
  checksum64    = '066f2d6ed466236f8071d703bc498fa17d484cf118e5adce8ea4bf810cde5138'
  checksumType64 = 'sha256'
}

Get-ChocolateyWebFile @packageArgs
