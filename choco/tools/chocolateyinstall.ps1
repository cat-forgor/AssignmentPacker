$ErrorActionPreference = 'Stop'

$toolsDir = "$(Split-Path -Parent $MyInvocation.MyCommand.Definition)"
$url = "https://github.com/cat-forgor/AssignmentPacker/releases/download/v$($env:ChocolateyPackageVersion)/ap-windows-x64.exe"

$packageArgs = @{
  packageName   = $env:ChocolateyPackageName
  fileFullPath  = Join-Path $toolsDir 'ap.exe'
  url64bit      = $url
  checksum64    = '87af5a55dc18d976257015f544d8b3f80fc8bf6c27b84456b14439ee0465be0b'
  checksumType64 = 'sha256'
}

Get-ChocolateyWebFile @packageArgs
