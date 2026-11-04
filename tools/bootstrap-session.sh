#!/bin/bash
# bootstrap-session.sh — isolated Xvfb + dbus session for black-box GUI testing
# of org.gnome.Calculator, fully DETACHED from the calling shell (setsid+nohup)
# so harness task management cannot kill it. Idempotent: safe to re-run.
#
# Usage: bootstrap-session.sh [DISPLAY] [WxHxDEPTH] [dark]
set -u
DISP="${1:-:98}"
GEOM="${2:-1280x800x24}"
DARK="${3:-0}"
ROOT=/home/hakergeniusz/Projects/radix
SBX="$ROOT/notes/sandbox-home"
ENVF="$SBX/session-env"
READY="$SBX/session-ready"
LOG="$ROOT/notes/session-current.log"
mkdir -p "$SBX"

# --- already up? -----------------------------------------------------------
if [ -f "$READY" ] && [ -S "/tmp/.X11-unix/X${DISP#:}" ] && pgrep -f "dbus-daemon.*$SBX" >/dev/null 2>&1; then
  echo "session already running on $DISP"
  exit 0
fi
rm -f "$READY"

# --- stop leftovers --------------------------------------------------------
pkill -f "Xvfb $DISP" 2>/dev/null
pkill -f "dbus-daemon.*session-current" 2>/dev/null
sleep 0.5

# --- Xvfb (detached) -------------------------------------------------------
setsid nohup Xvfb "$DISP" -screen 0 "$GEOM" -nolisten tcp >>"$LOG" 2>&1 &
# wait for the X socket (first run raced this: app started before bind)
for i in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20; do
  [ -S "/tmp/.X11-unix/X${DISP#:}" ] && break
  sleep 0.5
done
[ -S "/tmp/.X11-unix/X${DISP#:}" ] || { echo "FATAL: Xvfb socket never appeared"; exit 1; }

# --- session bus + portals + app (detached) --------------------------------
setsid nohup dbus-run-session -- bash -c '
  set -u
  DISP="'"$DISP"'"; DARK="'"$DARK"'"
  SBX="'"$SBX"'"; ENVF="'"$ENVF"'"; READY="'"$READY"'"
  unset WAYLAND_DISPLAY
  export GDK_BACKEND=x11 DISPLAY="$DISP"
  export XDG_DATA_HOME="$SBX" XDG_CONFIG_HOME="$SBX/config"
  mkdir -p "$XDG_CONFIG_HOME"
  [ "$DARK" = "1" ] && gsettings set org.gnome.desktop.interface color-scheme "prefer-dark" 2>/dev/null
  /usr/lib/xdg-desktop-portal-gtk >/dev/null 2>&1 &
  /usr/lib/xdg-desktop-portal >/dev/null 2>&1 &
  sleep 1.2
  printf "export DBUS_SESSION_BUS_ADDRESS=%q\nexport DISPLAY=%q\nexport XDG_DATA_HOME=%q\nexport XDG_CONFIG_HOME=%q\n" \
    "$DBUS_SESSION_BUS_ADDRESS" "$DISP" "$SBX" "$SBX/config" > "$ENVF"
  touch "$READY"
  echo "session ready bus=$DBUS_SESSION_BUS_ADDRESS display=$DISP dark=$DARK"
  # keep the bus alive forever; app is launched separately via launch-app.sh
  sleep infinity
' >>"$LOG" 2>&1 &

# wait for ready file
for i in 1 2 3 4 5 6 7 8 9 10; do
  [ -f "$READY" ] && break
  sleep 0.5
done
[ -f "$READY" ] && echo "session up on $DISP (env: $ENVF)" || { echo "FATAL: session did not become ready"; exit 1; }
