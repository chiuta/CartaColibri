#!/usr/bin/env python3
"""Builds dist/CartaColibriPortable-<version>-<target>.zip (PortableApps.com layout + one-click launchers) from compiled binaries.

  CartaColibriPortable/
    START-HERE.txt                      8 languages, <=10 lines each
    Windows:  CartaColibri.bat, CartaColibri-hidden.vbs, Stop-CartaColibri.bat, [CartaColibri-Browser.bat], install.ps1
    Linux:    cartacolibri.sh, CartaColibri.desktop, install.sh
    macOS:    CartaColibri.command, cartacolibri.sh, install.sh
    App/AppInfo/appinfo.ini (+ appicon.png)   PortableApps metadata (CUFAR manager)
    App/CartaColibri/cartacolibri[.exe], [cartacolibri-browser[.exe]], LICENSE, README.md, web-grab/
    Data/cartacolibri/                  index, groups, outputs: CARTACOLIBRI_HOME points here, survives upgrades
    Other/                              icon, release notes template

Also: --sums writes dist/SHA256SUMS for every zip in dist/.
Usage: python3 scripts/make_portable.py --version 0.2.0-alpha.9 --binary target/release/cartacolibri --target linux-x86_64 [--browser PATH]
       python3 scripts/make_portable.py --sums"""
import argparse, glob, hashlib, os, re, zipfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PK = os.path.join(ROOT, "packaging")
DIST = os.path.join(ROOT, "dist")


def rd(name, mode="r"):
    with open(os.path.join(PK, name), mode, **({} if "b" in mode else {"encoding": "utf-8", "newline": ""})) as f:
        return f.read()


def sums():
    zs = sorted(glob.glob(os.path.join(DIST, "*.zip")))
    if not zs:
        raise SystemExit("no zip files in dist/ to checksum")

    def digest(path):
        h = hashlib.sha256()
        with open(path, "rb") as f:
            for chunk in iter(lambda: f.read(1 << 20), b""):
                h.update(chunk)
        return h.hexdigest()

    lines = [f"{digest(z)}  {os.path.basename(z)}" for z in zs]
    out = os.path.join(DIST, "SHA256SUMS")
    with open(out, "w", newline="\n") as f:
        f.write("\n".join(lines) + "\n")
    print(out)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--version", default="dev")
    ap.add_argument("--binary")
    ap.add_argument("--target", default="local")
    ap.add_argument("--browser", help="optional: compiled cartacolibri-browser binary to include")
    ap.add_argument("--sums", action="store_true", help="only (re)write dist/SHA256SUMS")
    a = ap.parse_args()
    if a.sums:
        return sums()
    if not a.binary:
        ap.error("--binary is required")
    # the version and target end up in a file name: no path separators, no surprises
    if not re.fullmatch(r"[0-9A-Za-z][0-9A-Za-z.+-]*", a.version) or not re.fullmatch(r"[0-9A-Za-z][0-9A-Za-z_.-]*", a.target):
        ap.error("--version / --target may only contain letters, digits, . + - _")
    for label, path in (("--binary", a.binary), ("--browser", a.browser)):
        if path and not os.path.isfile(path):
            ap.error(f"{label}: file not found: {path}")
    win = a.binary.endswith(".exe") or a.target.startswith("windows")
    mac = a.target.startswith("macos")
    os.makedirs(DIST, exist_ok=True)
    out = os.path.join(DIST, f"CartaColibriPortable-{a.version}-{a.target}.zip")
    top = "CartaColibriPortable/"
    with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as z:
        def put_bytes(data, dst, mode=0o644):
            zi = zipfile.ZipInfo(top + dst, date_time=(2026, 1, 1, 0, 0, 0))
            zi.external_attr = mode << 16
            zi.compress_type = zipfile.ZIP_DEFLATED
            z.writestr(zi, data)

        def add(src, dst, mode=0o644):
            with open(src, "rb") as f:
                put_bytes(f.read(), dst, mode)

        def pk(name, dst=None, mode=0o644, crlf=False):
            t = rd(name).replace("\r\n", "\n")
            if crlf:
                t = t.replace("\n", "\r\n")
            put_bytes(t.encode("utf-8"), dst or name, mode)

        pk("START-HERE.txt", crlf=win)
        if win:
            for n in ("CartaColibri.bat", "Stop-CartaColibri.bat"):
                pk(n, crlf=True)
            pk("CartaColibri-hidden.vbs", crlf=True)
            if a.browser:
                pk("CartaColibri-Browser.bat", crlf=True)
            pk("install.ps1", crlf=True)
        else:
            pk("cartacolibri.sh", mode=0o755)
            pk("install.sh", mode=0o755)
            if mac:
                pk("CartaColibri.command", mode=0o755)
            else:
                pk("CartaColibri.desktop", mode=0o755)
        v = a.version
        pk_ini = rd("appinfo.ini").replace("{v}", v.split("-")[0]).replace("{version}", v).replace("Start=CartaColibri.bat", "Start=" + ("CartaColibri.bat" if win else "CartaColibri.command" if mac else "cartacolibri.sh")).replace("\r\n", "\n")
        put_bytes(pk_ini.replace("\n", "\r\n").encode(), "App/AppInfo/appinfo.ini")
        icon = os.path.join(ROOT, "assets", "cartacolibri-mark.png")
        add(icon, "App/AppInfo/appicon.png")
        add(icon, "Other/cartacolibri-mark.png")
        pk("release-notes.md", "Other/release-notes.md")
        exe = ".exe" if win else ""
        add(a.binary, f"App/CartaColibri/cartacolibri{exe}", 0o755)
        if a.browser:
            add(a.browser, f"App/CartaColibri/cartacolibri-browser{exe}", 0o755)
        for f in ("LICENSE", "README.md"):
            add(os.path.join(ROOT, f), "App/CartaColibri/" + f)
        for dp, dn, fn in os.walk(os.path.join(ROOT, "web-grab")):
            dn.sort()
            for f in sorted(fn):
                p = os.path.join(dp, f)
                add(p, "App/CartaColibri/" + os.path.relpath(p, ROOT).replace(os.sep, "/"))
        put_bytes(b"", "Data/cartacolibri/.keep")
    print(out)


if __name__ == "__main__":
    main()
