#!/usr/bin/env bash
# capture.sh NAME FROM_KEY TO_KEY [PRE_SEC] [POST_SEC] [GEOM WxH+X+Y]
# Sets a known source mode, then records the mode-switch transition losslessly.
set -euo pipefail
R=/home/hakergeniusz/Projects/radix/notes/anim-mode-switch
D=:101
NAME=${1:?name}; FROM=${2:?from key}; TO=${3:?to key}
PRE=${4:-1.2}; POST=${5:-2.0}; GEOM=${6:-800x700+0+0}
OUT="$R/evidence/$NAME"
mkdir -p "$OUT/frames"

W=$(DISPLAY=$D xdotool search --name '^Calculator$' 2>/dev/null | head -1)
[ -n "$W" ] || { echo "no Calculator window" >&2; exit 1; }
DISPLAY=$D xdotool windowfocus --sync "$W"
DISPLAY=$D xdotool key --clearmodifiers "$FROM"
sleep 0.8
echo "capture '$NAME': $FROM -> $TO | win=$W | $(DISPLAY=$D xdotool getwindowgeometry --shell "$W" | tr '\n' ' ')"

WIDTH=${GEOM%%+*}; XY=${GEOM#*+}; X=${XY%%+*}; Y=${XY#*+}
# drop any previous clip for this name (trash, never rm)
[ -f "$OUT/raw.mkv" ] && trash -f "$OUT/raw.mkv" 2>/dev/null || true
find "$OUT/frames" -name 'f*.png' -delete 2>/dev/null || true

DISPLAY=$D ffmpeg -hide_banner -loglevel error -y \
  -f x11grab -framerate 60 -video_size "$WIDTH" -i "$D.0+$X,$Y" \
  -c:v ffv1 -level 3 -g 1 -threads 2 "$OUT/raw.mkv" >"$OUT/ffmpeg.log" 2>&1 &
FF=$!
sleep "$PRE"
date +%s.%N > "$OUT/key.time"
DISPLAY=$D xdotool key --clearmodifiers "$TO"
sleep "$POST"
kill -INT "$FF" 2>/dev/null || true
wait "$FF" 2>/dev/null || true
ffmpeg -hide_banner -loglevel error -y -i "$OUT/raw.mkv" -fps_mode passthrough "$OUT/frames/f%04d.png"
trash -f "$OUT/raw.mkv" 2>/dev/null || true
echo "frames=$(find "$OUT/frames" -name 'f*.png' | wc -l) -> $OUT/frames"
