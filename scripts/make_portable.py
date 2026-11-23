#!/usr/bin/env python3
"""Builds dist/CartaColibriPortable-<version>.zip in the PortableApps.com layout:

  CartaColibriPortable/
    CartaColibriPortable.bat  .sh      launchers (Data/ is the state directory: nothing is written outside this folder)
    App/AppInfo/appinfo.ini            PortableApps metadata
    App/CartaColibri/                  the tracked tree (code, UI, docs)
    Data/                              your index, groups, outputs, models (survives upgrades)

Usage: python3 scripts/make_portable.py --version 0.1.0-alpha.1"""
import argparse, os, subprocess, sys, zipfile
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SKIP = ("site/", "site-carta/", "docs/media/", ".github/", "docs/experiments/", "docs/baselines/")

BAT = r"""@echo off
setlocal
rem CartaColibri Portable — everything stays inside this folder (Data\)
set "HERE=%~dp0"
set "CARTACOLIBRI_HOME=%HERE%Data\cartacolibri"
if not exist "%CARTACOLIBRI_HOME%" mkdir "%CARTACOLIBRI_HOME%"
set "APP=%HERE%App\CartaColibri"
where py >nul 2>nul && (set "PY=py -3") || (set "PY=python")
rem Opens the Library / RAG-fetch / RAG-CARTA page. For the AI tasks start the engine too:
rem   %PY% "%APP%\c\coli" web --model "%HERE%Data\models\<your-model>"
start "" "http://127.0.0.1:8790/carta.html"
%PY% "%APP%\ui\server.py" --port 8790 --backend http://127.0.0.1:8000
"""
SH = """#!/usr/bin/env bash
# CartaColibri Portable — everything stays inside this folder (Data/)
HERE="$(cd "$(dirname "$0")" && pwd)"
export CARTACOLIBRI_HOME="$HERE/Data/cartacolibri"; mkdir -p "$CARTACOLIBRI_HOME"
APP="$HERE/App/CartaColibri"
# For the AI tasks start the engine too:  python3 "$APP/c/coli" web --model "$HERE/Data/models/<your-model>"
( sleep 1.5; (xdg-open http://127.0.0.1:8790/carta.html || open http://127.0.0.1:8790/carta.html) >/dev/null 2>&1 ) &
exec python3 "$APP/ui/server.py" --port 8790 --backend http://127.0.0.1:8000
"""
INI = """[Format]
Type=PortableApps.comFormat
Version=3.0

[Details]
Name=CartaColibri Portable
AppID=CartaColibriPortable
Publisher=Alexio (chiuta)
Homepage=https://github.com/chiuta/CartaColibri
Category=Utilities
Description=Local AI library on the Colibri engine: index, search, summarize, extract, synthesize.
Language=Multilingual
Trademarks=
InstallType=Portable

[License]
Shareable=true
OpenSource=true
Freeware=true
CommercialUse=true
EULAVersion=1

[Version]
PackageVersion={v}
DisplayVersion={v}

[Control]
Icons=1
Start=CartaColibriPortable.bat
"""


def main():
    ap = argparse.ArgumentParser(); ap.add_argument("--version", default="dev"); a = ap.parse_args()
    files = subprocess.check_output(["git", "ls-files"], cwd=ROOT, text=True).splitlines()
    os.makedirs(os.path.join(ROOT, "dist"), exist_ok=True)
    out = os.path.join(ROOT, "dist", f"CartaColibriPortable-{a.version}.zip")
    top = "CartaColibriPortable/"
    with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as z:
        for f in files:
            if f.startswith(SKIP) or not os.path.isfile(os.path.join(ROOT, f)): continue
            z.write(os.path.join(ROOT, f), f"{top}App/CartaColibri/{f}")
        z.writestr(top + "CartaColibriPortable.bat", BAT.replace("\n", "\r\n"))
        info = zipfile.ZipInfo(top + "CartaColibriPortable.sh"); info.external_attr = 0o755 << 16
        z.writestr(info, SH)
        z.writestr(top + "App/AppInfo/appinfo.ini", INI.format(v=a.version))
        z.writestr(top + "Data/README.txt", "Your index, groups, outputs and models live here. Safe to keep across upgrades.\n")
    print(out, os.path.getsize(out) // 1024, "KB")


if __name__ == "__main__":
    main()
