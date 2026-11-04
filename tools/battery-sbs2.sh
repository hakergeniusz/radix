#!/usr/bin/env bash
set -u
cd /home/hakergeniusz/Projects/radix
export PATH="$PATH:/usr/bin"
S=/tmp/rig-shots
RUST_BACKTRACE=1 target/debug/radix 2>$S/app3.err &
sleep 2.5
realwin() {
  local BEST=0 WID=""
  for w in $(xdotool search --class radix 2>/dev/null); do
    GW=$(xdotool getwindowgeometry "$w" 2>/dev/null | awk '/Geometry/{print $2}' | cut -dx -f1)
    if [ "${GW:-0}" -gt "$BEST" ]; then BEST=$GW; WID=$w; fi
  done
  echo "$WID"
}
WID=$(realwin)
xdotool windowfocus "$WID" 2>/dev/null; sleep 0.3
for e in "2+2" "5÷6" "0.833333333×111" "3×111+(23÷60)×111"; do
  xdotool type --delay 20 "$e"; xdotool key Return; sleep 0.35
done
sleep 0.5
import -window root $S/m01-basic.png
xdotool key --clearmodifiers ctrl+alt+c; sleep 1
echo AFTER_CONV_SWITCH alive=$(pgrep -f "target/debug/radix" | head -1)
WID=$(realwin); xdotool windowfocus "$WID" 2>/dev/null
xdotool windowsize "$WID" 360 610; xdotool windowmove "$WID" 0 0; sleep 0.5
import -window root $S/m03-conversion.png
xdotool mousemove 296 27 click 1; sleep 0.7
echo AFTER_MENU_CLICK alive=$(pgrep -f "target/debug/radix" | head -1)
import -window root $S/m02-conv-menu.png
xdotool key Escape; sleep 0.4
xdotool key --clearmodifiers ctrl+alt+a; sleep 1
echo AFTER_ADV alive=$(pgrep -f "target/debug/radix" | head -1)
WID=$(realwin); xdotool windowfocus "$WID" 2>/dev/null
xdotool windowsize "$WID" 700 615; xdotool windowmove "$WID" 0 0; sleep 0.5
import -window root $S/m04-advanced.png
xdotool key --clearmodifiers ctrl+alt+f; sleep 1
WID=$(realwin); xdotool windowfocus "$WID" 2>/dev/null
xdotool windowsize "$WID" 700 615; xdotool windowmove "$WID" 0 0; sleep 0.5
import -window root $S/m05-financial.png
xdotool key --clearmodifiers ctrl+alt+p; sleep 1
WID=$(realwin); xdotool windowfocus "$WID" 2>/dev/null
xdotool windowsize "$WID" 700 615; xdotool windowmove "$WID" 0 0; sleep 0.5
xdotool type --delay 20 "11111111"; sleep 0.4
import -window root $S/m06-programming.png
xdotool key --clearmodifiers ctrl+alt+k; sleep 1
WID=$(realwin); xdotool windowfocus "$WID" 2>/dev/null
xdotool windowsize "$WID" 700 630; xdotool windowmove "$WID" 0 0; sleep 0.5
import -window root $S/m07-keyboard.png
echo BATTERY3_OK
