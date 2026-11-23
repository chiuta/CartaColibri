#!/usr/bin/env bash
# macOS: double-click to start CartaColibri. Data stays in this folder (Data/).
HERE="$(cd "$(dirname "$0")" && pwd)"
xattr -dr com.apple.quarantine "$HERE" 2>/dev/null   # unsigned build: lift the download quarantine
"$HERE/cartacolibri.sh"
sleep 2
