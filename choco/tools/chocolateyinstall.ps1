$ErrorActionPreference = 'Stop'

$toolsDir = "$(Split-Path -Parent $MyInvocation.MyCommand.Definition)"
$url = "https://github.com/cat-forgor/AssignmentPacker/releases/download/v$($env:ChocolateyPackageVersion)/ap-windows-x64.exe"

$packageArgs = @{
  packageName   = $env:ChocolateyPackageName
  fileFullPath  = Join-Path $toolsDir 'ap.exe'
  url64bit      = $url
  checksum64    = '40ba4bcf0403c9bdb1aa3e5ba84836436ec77e3a4fc2f2be008d7dbea9ce4f1a'
  checksumType64 = 'sha256'
}

Get-ChocolateyWebFile @packageArgs
