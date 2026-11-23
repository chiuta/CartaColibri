#!/usr/bin/env bash
# Run from Git Bash in the CartaColibri folder, after creating the EMPTY repo github.com/chiuta/CartaColibri.
set -euo pipefail
git config user.name  "Alexio"
git config user.email "alexio@trom.tf"
git branch -M main
git remote get-url origin >/dev/null 2>&1 || git remote add origin https://github.com/chiuta/CartaColibri.git
git remote get-url upstream >/dev/null 2>&1 || git remote add upstream https://github.com/JustVugg/colibri.git
git push -u origin main
echo "Pushed. Next: Settings → Pages → Source: GitHub Actions; then Actions → 'Tag a CartaColibri release' (0.1.0-alpha.1)."
