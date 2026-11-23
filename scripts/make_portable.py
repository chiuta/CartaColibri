#!/usr/bin/env python3
"""Builds dist/CartaColibriPortable-<version>-<target>.zip in the PortableApps.com layout from a compiled binary.

  CartaColibriPortable/
    CartaColibriPortable.bat | .sh     launcher (Data/ is the state directory; nothing is written outside this folder)
    App/AppInfo/appinfo.ini            PortableApps metadata
    App/CartaColibri/cartacolibri[.exe], LICENSE, README.md, web-grab/
    Data/                              your index, groups, outputs (survives upgrades)

Usage: python3 scripts/make_portable.py --version 0.2.0-alpha.1 --binary target/release/cartacolibri --target linux-x86_64"""
import argparse, os, zipfile
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BAT = r"""@echo off
setlocal
set "HERE=%~dp0"
set "CARTACOLIBRI_HOME=%HERE%Data\cartacolibri"
if not exist "%CARTACOLIBRI_HOME%" mkdir "%CARTACOLIBRI_HOME%"
rem For the AI tasks also start the Colibri engine (coli web --model ...) — default gateway http://127.0.0.1:8000
start "" "http://127.0.0.1:8765/"
"%HERE%App\CartaColibri\cartacolibri.exe" serve --port 8765
"""
SH = """#!/usr/bin/env bash
HERE="$(cd "$(dirname "$0")" && pwd)"
export CARTACOLIBRI_HOME="$HERE/Data/cartacolibri"; mkdir -p "$CARTACOLIBRI_HOME"
( sleep 1.5; (xdg-open http://127.0.0.1:8765/ || open http://127.0.0.1:8765/) >/dev/null 2>&1 ) &
exec "$HERE/App/CartaColibri/cartacolibri" serve --port 8765
"""
INI = """[Format]
Type=PortableApps.comFormat
Version=3.0

[Details]
Name=CartaColibri Portable
AppID=CartaColibriPortable
Publisher=Alexandru-Ionut Chiuta
Homepage=github.com/chiuta/CartaColibri
Category=Utilities
Description=Local AI library: RAG-fetch index and RAG-CARTA summaries with citations. 100% local.
Language=Multilingual

[License]
Shareable=true
OpenSource=true
Freeware=true
CommercialUse=true

[Version]
PackageVersion={v}.0
DisplayVersion={v}
"""


def main():
    ap = argparse.ArgumentParser(); ap.add_argument("--version", default="dev"); ap.add_argument("--binary", required=True); ap.add_argument("--target", default="local")
    a = ap.parse_args()
    win = a.binary.endswith(".exe")
    os.makedirs(os.path.join(ROOT, "dist"), exist_ok=True)
    out = os.path.join(ROOT, "dist", f"CartaColibriPortable-{a.version}-{a.target}.zip")
    top = "CartaColibriPortable/"
    with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as z:
        def add(src, dst, mode=0o644):
            zi = zipfile.ZipInfo(top + dst); zi.external_attr = mode << 16; zi.compress_type = zipfile.ZIP_DEFLATED
            with open(src, "rb") as f: z.writestr(zi, f.read())
        def put(text, dst, mode=0o644):
            zi = zipfile.ZipInfo(top + dst); zi.external_attr = mode << 16; z.writestr(zi, text)
        put(BAT.replace("\n", "\r\n"), "CartaColibriPortable.bat") if win else put(SH, "CartaColibriPortable.sh", 0o755)
        put(INI.format(v=a.version.split("-")[0]), "App/AppInfo/appinfo.ini")
        add(a.binary, "App/CartaColibri/" + ("cartacolibri.exe" if win else "cartacolibri"), 0o755)
        for f in ("LICENSE", "README.md"): add(os.path.join(ROOT, f), "App/CartaColibri/" + f)
        for dp, _, fn in os.walk(os.path.join(ROOT, "web-grab")):
            for f in fn: add(os.path.join(dp, f), "App/CartaColibri/" + os.path.relpath(os.path.join(dp, f), ROOT).replace(os.sep, "/"))
        put("", "Data/.keep")
    print(out)

if __name__ == "__main__":
    main()
