#!/usr/bin/env bash
# Fresh, detached Xvfb session for black-box mode-switch animation capture.
set -euo pipefail
R=/tmp/opencode/anim-explore
D=:101
S=$R/state
mkdir -p "$S"/{data,config,cache,state} "$R/evidence"

if [ -S /tmp/.X11-unix/X101 ]; then
  echo "X101 socket already exists - not starting a second Xvfb" >&2
  exit 1
fi

setsid nohup Xvfb "$D" -screen 0 1280x800x24 -nolisten tcp >"$R/xvfb.log" 2>&1 &
for _ in $(seq 1 40); do
  [ -S /tmp/.X11-unix/X101 ] && break
  sleep 0.25
done
[ -S /tmp/.X11-unix/X101 ] || { echo "Xvfb failed" >&2; exit 1; }

setsid nohup dbus-run-session -- bash -c '
  set -eu
  export DISPLAY=:101
  export GDK_BACKEND=x11
  export GTK_IM_MODULE=xim
  unset WAYLAND_DISPLAY
  S=/tmp/opencode/anim-explore/state
  export XDG_DATA_HOME="$S/data" XDG_CONFIG_HOME="$S/config"
  export XDG_CACHE_HOME="$S/cache" XDG_STATE_HOME="$S/state"
  exec /usr/bin/gnome-calculator
' >"$R/app.log" 2>&1 &

for _ in $(seq 1 60); do
  DISPLAY=$D xdotool search --class -- "calculator" >/dev/null 2>&1 && break
  sleep 0.25
done
echo "session up on $D"
DISPLAY=$D xdotool search --class -- "calculator" 2>/dev/null | while read -r w; do
  echo "win=$w name=$(DISPLAY=$D xdotool getwindowname "$w" 2>/dev/null)"
  DISPLAY=$D xdotool getwindowgeometry "$w" 2>/dev/null
done
