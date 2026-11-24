#!/usr/bin/env python3
"""Fails when the documentation no longer covers the program: every CLI command must be in the command
reference of all 8 READMEs and of the site, and every tour screenshot must exist in all 8 languages.
Usage: python3 scripts/check_docs_complete.py [path/to/cartacolibri]"""
import os, re, subprocess, sys
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
LANGS = ["en", "ro", "fr", "it", "es", "pt", "de", "hu"]
binp = sys.argv[1] if len(sys.argv) > 1 else os.path.join(ROOT, "target", "release", "cartacolibri")
h = subprocess.run([binp, "--help"], capture_output=True, text=True, check=True).stdout
cmds = [m.group(1) for m in re.finditer(r"^  ([a-z][a-z-]*)\s", h.split("Commands:")[1].split("Options:")[0], re.M) if m.group(1) != "help"]
bad = []
site = open(f"{ROOT}/site-carta/index.html", encoding="utf-8").read()
for l in LANGS:
    md = open(f"{ROOT}/docs/cartacolibri/README.{l}.md", encoding="utf-8").read()
    for c in cmds:
        if not re.search(r"`[^`\n]*\b" + re.escape(c) + r"\b", md): bad.append(f"README.{l}.md lacks command `{c}`")
for c in cmds:
    if c not in site: bad.append(f"site lacks command {c}")
n_tour = int(re.search(r"\[1,2,3,[0-9,]+\]\.map", site).group(0).count(",")) + 1
for l in LANGS:
    for n in range(1, n_tour + 1):
        for p in (f"docs/cartacolibri/img/{l}-{n}.png", f"site-carta/img/{l}-{n}.webp"):
            if not os.path.exists(f"{ROOT}/{p}"): bad.append(f"missing {p}")
    for p in (f"docs/cartacolibri/CartaColibri-Tutorial.{l}.pdf", f"docs/cartacolibri/video/{l}-demo.mp4"):
        if not os.path.exists(f"{ROOT}/{p}"): bad.append(f"missing {p}")
if bad:
    print("\n".join(bad)); sys.exit(f"{len(bad)} documentation gap(s)")
print(f"docs complete: {len(cmds)} commands x 8 languages, {n_tour} tour screens x 8 languages")
