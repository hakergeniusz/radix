#!/usr/bin/env bash
# Control v2: disable GTK animations in the app's OWN private session env.
set -uo pipefail
SRC=/tmp/opencode/anim-explore
R=/home/hakergeniusz/Projects/radix/notes/anim-mode-switch
DBUS=$(tr '\0' '\n' < /proc/333640/environ | rg '^DBUS_SESSION_BUS_ADDRESS=' | cut -d= -f2-)
export DBUS_SESSION_BUS_ADDRESS="$DBUS"
export XDG_CONFIG_HOME=/tmp/opencode/anim-explore/state/config
export XDG_DATA_HOME=/tmp/opencode/anim-explore/state/data
gsettings set org.gnome.desktop.interface enable-animations false
sleep 0.6
echo "in app session: enable-animations=$(gsettings get org.gnome.desktop.interface enable-animations)"
bash "$SRC/capture.sh" noanim-conv2 ctrl+alt+k ctrl+alt+c 1.2 2.0
python3 "$SRC/analyze.py" "$R/evidence/noanim-conv2" > "$R/evidence/noanim-conv2/analysis.txt" 2>&1
python3 "$SRC/ring.py" "$R/evidence/noanim-conv2" 78 5 212 51 65 105 > "$R/evidence/noanim-conv2/ring.txt"
python3 "$SRC/summary1.py" "$R/evidence/noanim-conv2/analysis.txt"
gsettings set org.gnome.desktop.interface enable-animations true
echo "restored: $(gsettings get org.gnome.desktop.interface enable-animations)"
