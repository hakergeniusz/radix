#!/bin/bash
# launch-app.sh — launch org.gnome.Calculator inside the isolated session.
# Detached (setsid+nohup); refuses to start a second instance.
set -u
ROOT=/home/hakergeniusz/Projects/radix
ENVF="$ROOT/notes/sandbox-home/session-env"
[ -f "$ENVF" ] || { echo "FATAL: no session env — run bootstrap-session.sh first"; exit 1; }
if pgrep -f "bwrap.*gnome-calculator" >/dev/null 2>&1; then
  echo "app already running"
  exit 0
fi
setsid nohup bash -c '
  . "'"$ENVF"'"
  unset WAYLAND_DISPLAY XDG_DATA_HOME XDG_CONFIG_HOME
  export GDK_BACKEND=x11
  exec flatpak run --nosocket=wayland --socket=x11 \
    --env=WAYLAND_DISPLAY= --env=GDK_BACKEND=x11 \
    --env=GTK_IM_MODULE=xim org.gnome.Calculator
' >>"$ROOT/notes/app.log" 2>&1 &
sleep 3
pgrep -f "bwrap.*gnome-calculator" >/dev/null && echo "app launched" || { echo "FATAL: app did not start"; tail -5 "$ROOT/notes/app.log"; }
