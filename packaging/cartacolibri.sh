#!/usr/bin/env bash
# CartaColibri one-click launcher (Linux / macOS). Data stays inside this folder (Data/cartacolibri).
#   ./cartacolibri.sh           start the local server (if not running) and open the browser
#   ./cartacolibri.sh stop      stop the server
#   ./cartacolibri.sh --desktop add an application-menu entry (Linux, ~/.local/share/applications)
#   ./cartacolibri.sh --browser start the web-grab browser instead (if included)
# Nothing here uses the network except 127.0.0.1.
set -u
SELF="$0"; [ -L "$SELF" ] && SELF="$(readlink -f "$SELF")"
HERE="$(cd "$(dirname "$SELF")" && pwd)"
APP="$HERE/App/CartaColibri"
PORT="${CARTACOLIBRI_PORT:-8765}"
URL="http://127.0.0.1:$PORT/"
export CARTACOLIBRI_HOME="${CARTACOLIBRI_HOME:-$HERE/Data/cartacolibri}"
mkdir -p "$CARTACOLIBRI_HOME"
PIDF="$CARTACOLIBRI_HOME/server.pid"
LOG="$CARTACOLIBRI_HOME/server.log"

up() { (exec 3<>"/dev/tcp/127.0.0.1/$PORT") 2>/dev/null; }
open_url() {
  if [ "$(uname -s)" = Darwin ]; then open "$1" >/dev/null 2>&1 && return 0
  else xdg-open "$1" >/dev/null 2>&1 && return 0; fi
  echo "Open this address in your browser: $1"
}

case "${1:-}" in
  stop)
    if [ -f "$PIDF" ] && kill "$(cat "$PIDF")" 2>/dev/null; then echo "CartaColibri stopped."; else echo "CartaColibri was not running."; fi
    rm -f "$PIDF"; exit 0 ;;
  --desktop)
    D="${XDG_DATA_HOME:-$HOME/.local/share}/applications"; mkdir -p "$D"
    ICON="$HERE/Other/cartacolibri-mark.png"; [ -f "$ICON" ] || ICON=cartacolibri
    cat > "$D/cartacolibri-portable.desktop" <<DESK
[Desktop Entry]
Type=Application
Name=CartaColibri (portable)
Comment=Local AI library - 100% on your machine
Exec="$HERE/cartacolibri.sh"
Icon=$ICON
Terminal=false
Categories=Office;Utility;
DESK
    echo "Added: $D/cartacolibri-portable.desktop"; exit 0 ;;
  --browser)
    [ -x "$APP/cartacolibri-browser" ] || { echo "cartacolibri-browser is not included in this package."; exit 1; }
    exec "$APP/cartacolibri-browser" --home "$CARTACOLIBRI_HOME" ;;
esac

[ -x "$APP/cartacolibri" ] || chmod +x "$APP/cartacolibri" "$APP/cartacolibri-browser" 2>/dev/null
if ! up; then
  nohup "$APP/cartacolibri" serve --port "$PORT" >"$LOG" 2>&1 &
  echo $! >"$PIDF"
  for _ in $(seq 1 50); do up && break; sleep 0.2; done
  up || { echo "The server did not start; see $LOG"; exit 1; }
fi
open_url "$URL"
echo "CartaColibri is running at $URL  (stop it with: $SELF stop)"
