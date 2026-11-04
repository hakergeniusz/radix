#!/bin/bash
# shot.sh NAME [sleep_s] [geom WxH+X+Y] — screenshot the isolated Xvfb display.
# Usage: shot.sh basic-start            -> notes/shots/basic-start.png (full screen)
#        shot.sh disp-crop 0 300x120+40+80  -> crops after capture
set -u
ROOT=/home/hakergeniusz/Projects/radix
NAME="$1"; S="${2:-0}"; CROP="${3:-}"
[ "$S" != "0" ] && sleep "$S"
OUT="$ROOT/notes/shots/$NAME.png"
DISPLAY=:98 import -window root "$OUT"
if [ -n "$CROP" ]; then
  python3 - "$OUT" "$CROP" <<'PY'
import sys
from PIL import Image
p, crop = sys.argv[1], sys.argv[2]
w, h, x, y = (int(v) for v in crop.replace("x", " ").replace("+", " +").split())
im = Image.open(p).crop((x, y, x + w, y + h))
im = im.resize((im.width * 3, im.height * 3), Image.LANCZOS)
im.save(p)
PY
fi
echo "saved notes/shots/$NAME.png"
