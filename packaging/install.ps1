# CartaColibri per-user installer (Windows). No administrator rights; installs to %LOCALAPPDATA%\CartaColibri.
#   powershell -ExecutionPolicy Bypass -File install.ps1             install (run from the unzipped package)
#   powershell -ExecutionPolicy Bypass -File install.ps1 -Uninstall  remove (your library in %USERPROFILE%\.cartacolibri is kept)
param([switch]$Uninstall)
$ErrorActionPreference = 'Stop'
$dest  = Join-Path $env:LOCALAPPDATA 'CartaColibri'
$menu  = Join-Path $env:APPDATA 'Microsoft\Windows\Start Menu\Programs'
$link  = Join-Path $menu 'CartaColibri.lnk'

function Set-UserPath([string]$dir, [bool]$add) {
  $p = [Environment]::GetEnvironmentVariable('Path', 'User'); if (-not $p) { $p = '' }
  $parts = @($p -split ';' | Where-Object { $_ -and ($_ -ne $dir) })
  if ($add) { $parts += $dir }
  [Environment]::SetEnvironmentVariable('Path', ($parts -join ';'), 'User')
}

if ($Uninstall) {
  if (Test-Path (Join-Path $dest '.cartacolibri-install')) { Get-Process cartacolibri -ErrorAction SilentlyContinue | Stop-Process -Force; Remove-Item -Recurse -Force $dest }
  elseif (Test-Path $dest) { throw "Refusing to remove $dest (not created by this installer)" }
  Remove-Item -Force $link -ErrorAction SilentlyContinue
  Set-UserPath (Join-Path $dest 'App\CartaColibri') $false
  Write-Host 'CartaColibri removed. Your library (%USERPROFILE%\.cartacolibri) was left untouched.'
  return
}

$src = $PSScriptRoot
if (-not (Test-Path (Join-Path $src 'App\CartaColibri\cartacolibri.exe'))) { $src = Split-Path $src -Parent }
if (-not (Test-Path (Join-Path $src 'App\CartaColibri\cartacolibri.exe'))) { throw 'Run this from the unzipped CartaColibri package (App\CartaColibri\cartacolibri.exe not found).' }

if (Test-Path $dest) { Get-Process cartacolibri -ErrorAction SilentlyContinue | Stop-Process -Force; Remove-Item -Recurse -Force $dest }
New-Item -ItemType Directory -Force $dest | Out-Null
Copy-Item -Recurse -Force (Join-Path $src 'App') $dest
if (Test-Path (Join-Path $src 'Other')) { Copy-Item -Recurse -Force (Join-Path $src 'Other') $dest }
Copy-Item -Force (Join-Path $src 'CartaColibri-hidden.vbs') $dest
Copy-Item -Force (Join-Path $src 'Stop-CartaColibri.bat') $dest -ErrorAction SilentlyContinue
New-Item -ItemType File -Force (Join-Path $dest '.cartacolibri-install') | Out-Null
# installed mode: no Data folder next to the vbs, so it uses %USERPROFILE%\.cartacolibri (same default as the CLI)
$vbs = Join-Path $dest 'CartaColibri-hidden.vbs'
$lib = Join-Path $env:USERPROFILE '.cartacolibri'
New-Item -ItemType Directory -Force $lib | Out-Null

New-Item -ItemType Directory -Force $menu | Out-Null
$ws = New-Object -ComObject WScript.Shell
$s = $ws.CreateShortcut($link)
$s.TargetPath = 'wscript.exe'
$s.Arguments = '"' + $vbs + '"'
$s.WorkingDirectory = $dest
$ico = Join-Path $dest 'Other\cartacolibri.ico'
if (Test-Path $ico) { $s.IconLocation = $ico } else { $s.IconLocation = (Join-Path $dest 'App\CartaColibri\cartacolibri.exe') + ',0' }
$s.Description = 'CartaColibri - local AI library'
$s.Save()
Set-UserPath (Join-Path $dest 'App\CartaColibri') $true

Write-Host "Installed in $dest"
Write-Host "Start it from the Start menu: CartaColibri. In a new terminal you can also run: cartacolibri"
Write-Host 'Uninstall: install.ps1 -Uninstall'
