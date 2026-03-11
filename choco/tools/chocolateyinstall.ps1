$ErrorActionPreference = 'Stop'

$toolsDir = "$(Split-Path -Parent $MyInvocation.MyCommand.Definition)"
$url = "https://github.com/cat-forgor/AssignmentPacker/releases/download/v$($env:ChocolateyPackageVersion)/ap-windows-x64.exe"

$packageArgs = @{
  packageName   = $env:ChocolateyPackageName
  fileFullPath  = Join-Path $toolsDir 'ap.exe'
  url64bit      = $url
  checksum64    = 'a25788bc6c27a9b36c1e4a0631997c1a7a63c07f272a6bff31bfa6d4ae1dbddd'
  checksumType64 = 'sha256'
}

Get-ChocolateyWebFile @packageArgs
