$ErrorActionPreference = 'Stop'

$toolsDir = "$(Split-Path -Parent $MyInvocation.MyCommand.Definition)"
$url = "https://github.com/cat-forgor/AssignmentPacker/releases/download/v$($env:ChocolateyPackageVersion)/ap-windows-x64.exe"

$packageArgs = @{
  packageName   = $env:ChocolateyPackageName
  fileFullPath  = Join-Path $toolsDir 'ap.exe'
  url64bit      = $url
  checksum64    = 'dc64e7d647c00c751fffa71025df91e3cd5ca92cd1b34295ac8d4b21d9b45338'
  checksumType64 = 'sha256'
}

Get-ChocolateyWebFile @packageArgs
