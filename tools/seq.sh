#!/bin/bash
# seq.sh OUT "t:text" | "c:X Y" | "k:keysym" ... — build an entry via clicks/typing,
# then press Return and crop the entry area. t: type, c: click at coords, k: key.
set -u
ROOT=/home/hakergeniusz/Projects/radix
OUT="$1"; shift
STAGE="$ROOT/notes/shots/crops"
mkdir -p "$STAGE"
for m in ctrl shift alt super meta; do DISPLAY=:98 xdotool keyup $m 2>/dev/null; done
DISPLAY=:98 xdotool mousemove 185 325; sleep 0.15; DISPLAY=:98 xdotool click 1; sleep 0.2
DISPLAY=:98 xdotool key ctrl+a; sleep 0.1; DISPLAY=:98 xdotool key Delete; sleep 0.25
for tok in "$@"; do
  case "$tok" in
    t:*) DISPLAY=:98 xdotool type --delay 40 -- "${tok#t:}" ;;
    c:*) xy="${tok#c:}"; DISPLAY=:98 xdotool mousemove ${xy% *} ${xy#* }; sleep 0.15; DISPLAY=:98 xdotool click 1 ;;
    k:*) DISPLAY=:98 xdotool key "${tok#k:}" ;;
  esac
  sleep 0.25
done
DISPLAY=:98 xdotool key Return
sleep 0.6
DISPLAY=:98 import -window root "$STAGE/full-tmp.png"
python3 - "$STAGE/full-tmp.png" "$ROOT/notes/shots/$OUT.png" <<'PY'
import sys
from PIL import Image
src, dst = sys.argv[1], sys.argv[2]
im = Image.open(src).crop((0, 240, 900, 350))
im = im.resize((int(im.width * 1.8), int(im.height * 1.8)), Image.LANCZOS)
im.save(dst)
PY
rm -f "$STAGE/full-tmp.png"
echo "saved $OUT"
