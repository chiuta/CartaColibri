#!/usr/bin/env bash
# CartaColibri per-user installer (Linux / macOS). No sudo; everything goes under ~/.local.
#   ./install.sh              install (run it from the unzipped package)
#   ./install.sh --uninstall  remove the program, links and menu entry (your library in ~/.cartacolibri is kept)
set -eu
PREFIX="${PREFIX:-$HOME/.local}"
APPDIR="$PREFIX/share/cartacolibri"
BIN="$PREFIX/bin"
DESK="${XDG_DATA_HOME:-$HOME/.local/share}/applications/cartacolibri.desktop"

if [ "${1:-}" = "--uninstall" ]; then
  [ -f "$APPDIR/.cartacolibri-install" ] || [ ! -e "$APPDIR" ] || { echo "Refusing to remove $APPDIR (not created by this installer)"; exit 1; }
  rm -rf "$APPDIR"; rm -f "$BIN/cartacolibri" "$BIN/cartacolibri-ui" "$BIN/cartacolibri-browser" "$DESK"
  command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$(dirname "$DESK")" 2>/dev/null || true
  echo "CartaColibri removed. Your library (~/.cartacolibri) was left untouched; delete it by hand if you want."
  exit 0
fi

SRC="$(cd "$(dirname "$0")" && pwd)"
[ -x "$SRC/App/CartaColibri/cartacolibri" ] || SRC="$(dirname "$SRC")"
[ -x "$SRC/App/CartaColibri/cartacolibri" ] || { echo "Run this from the unzipped CartaColibri package (App/CartaColibri/cartacolibri not found)."; exit 1; }

rm -rf "$APPDIR"; mkdir -p "$APPDIR" "$BIN" "$(dirname "$DESK")"
cp -R "$SRC/App/CartaColibri/." "$APPDIR/"
[ -d "$SRC/Other" ] && cp -R "$SRC/Other/." "$APPDIR/" || true
touch "$APPDIR/.cartacolibri-install"
ln -sf "$APPDIR/cartacolibri" "$BIN/cartacolibri"
[ -x "$APPDIR/cartacolibri-browser" ] && ln -sf "$APPDIR/cartacolibri-browser" "$BIN/cartacolibri-browser"

# one-click starter: server (if needed) + browser; the library lives in ~/.cartacolibri
cat > "$BIN/cartacolibri-ui" <<UI
#!/usr/bin/env bash
PORT="\${CARTACOLIBRI_PORT:-8765}"
up() { (exec 3<>"/dev/tcp/127.0.0.1/\$PORT") 2>/dev/null; }
if [ "\${1:-}" = stop ]; then pkill -u "\$(id -u)" -f "$APPDIR/cartacolibri serve" && echo stopped; exit 0; fi
if ! up; then
  mkdir -p "\$HOME/.cartacolibri"
  nohup "$APPDIR/cartacolibri" serve --port "\$PORT" >"\$HOME/.cartacolibri/server.log" 2>&1 &
  for _ in \$(seq 1 50); do up && break; sleep 0.2; done
fi
if [ "\$(uname -s)" = Darwin ]; then open "http://127.0.0.1:\$PORT/"; else xdg-open "http://127.0.0.1:\$PORT/" >/dev/null 2>&1 || echo "Open http://127.0.0.1:\$PORT/"; fi
UI
chmod +x "$BIN/cartacolibri-ui"

if [ "$(uname -s)" != Darwin ]; then
  ICON="$APPDIR/cartacolibri-mark.png"; [ -f "$ICON" ] || ICON=cartacolibri
  cat > "$DESK" <<DESKTOP
[Desktop Entry]
Type=Application
Name=CartaColibri
Comment=Local AI library - 100% on your machine
Exec=$BIN/cartacolibri-ui
Icon=$ICON
Terminal=false
Categories=Office;Utility;
DESKTOP
  command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$(dirname "$DESK")" 2>/dev/null || true
fi

echo "Installed in $APPDIR"
echo "Start it: open 'CartaColibri' from your application menu, or run: cartacolibri-ui"
case ":$PATH:" in *":$BIN:"*) ;; *) echo "Hint: add $BIN to your PATH, e.g.  echo 'export PATH=\"$BIN:\$PATH\"' >> ~/.profile" ;; esac
echo "Uninstall: $0 --uninstall"
