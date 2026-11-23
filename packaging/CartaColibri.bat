@echo off
rem One-click start: no console window; data stays in Data\cartacolibri next to this file.
if exist "%~dp0CartaColibri-hidden.vbs" (
  wscript.exe "%~dp0CartaColibri-hidden.vbs" %*
) else (
  set "CARTACOLIBRI_HOME=%~dp0Data\cartacolibri"
  if not exist "%CARTACOLIBRI_HOME%" mkdir "%CARTACOLIBRI_HOME%"
  start "" "http://127.0.0.1:8765/"
  "%~dp0App\CartaColibri\cartacolibri.exe" serve --port 8765
)
