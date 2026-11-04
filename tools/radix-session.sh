#!/usr/bin/env bash
# Radix GUI test session — isolated Xvfb + private D-Bus, sandboxed XDG dirs.
# Never touches the user's real desktop session.
# Usage: tools/radix-session.sh <command...>   (run inside the session)
#        DISPLAY=:99 Xvfb 1280x800x24, no WM (matches the spec environment).
set -euo pipefail

DISP=${RADIX_TEST_DISPLAY:-:99}
BIN=${RADIX_BIN:-"$PWD/target/debug/radix"}
STATE=${RADIX_TEST_STATE:-/tmp/radix-gui-test-state}

mkdir -p "$STATE"/{data,config,cache,state}
export XDG_DATA_HOME="$STATE/data"
export XDG_CONFIG_HOME="$STATE/config"
export XDG_CACHE_HOME="$STATE/cache"
export XDG_STATE_HOME="$STATE/state"
export GDK_BACKEND=x11
unset GTK_IM_MODULE  # xim module often absent post-update; GTK falls back
# Match the owner-evidence environment: dark style (GTK_THEME forces
# libadwaita's StyleManager to dark; the private bus has no dconf daemon).
export GTK_THEME=Adwaita-dark
export RADIX_DARK=1
unset WAYLAND_DISPLAY 2>/dev/null || true

# Xvfb needs a moment to bind its socket; wait-loop avoids the bind race.
# NOTE: xdpyinfo is not installed on this host — use xrandr for the probe.
if ! xrandr --display "$DISP" >/dev/null 2>&1; then
  Xvfb "$DISP" -screen 0 1280x800x24 -nolisten tcp &
  XVFB_PID=$!
  for _ in $(seq 1 50); do
    xrandr --display "$DISP" >/dev/null 2>&1 && break
    sleep 0.2
  done
fi
export DISPLAY="$DISP"

exec dbus-run-session -- "$@"
