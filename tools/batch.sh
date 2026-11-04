#!/bin/bash
# batch.sh OUTNAME "expr1" "expr2" ... — run N expressions (N<=6), each:
#   keyup modifiers -> click C -> sleep -> type expr -> Return -> sleep
# then crop each result region and composite into one labeled image.
set -u
ROOT=/home/hakergeniusz/Projects/radix
OUT="$1"; shift
STAGE="$ROOT/notes/shots/crops"
mkdir -p "$STAGE"
rm -f "$STAGE"/crop-*.png
i=0
for e in "$@"; do
  for m in ctrl shift alt super meta; do DISPLAY=:98 xdotool keyup $m 2>/dev/null; done
  DISPLAY=:98 xdotool mousemove 185 325
  sleep 0.15
  DISPLAY=:98 xdotool click 1
  sleep 0.2
  DISPLAY=:98 xdotool mousemove 44 405
  sleep 0.15
  DISPLAY=:98 xdotool click 1
  sleep 0.3
  DISPLAY=:98 xdotool type --delay 40 -- "$e"
  DISPLAY=:98 xdotool key Return
  sleep 0.55
  DISPLAY=:98 import -window root "$STAGE/full-$i.png"
  python3 - "$STAGE/full-$i.png" "$STAGE/crop-$i.png" <<'PY'
import sys
from PIL import Image
src, dst = sys.argv[1], sys.argv[2]
im = Image.open(src).crop((0, 240, 370, 350))
im = im.resize((im.width * 3, im.height * 3), Image.LANCZOS)
im.save(dst)
PY
  i=$((i+1))
done
python3 - "$ROOT/notes/shots/$OUT.png" "$STAGE" "$@" <<'PY'
import sys
from PIL import Image, ImageDraw
dst, stage = sys.argv[1], sys.argv[2]
exprs = sys.argv[3:]
crops = [f"{stage}/crop-{i}.png" for i in range(len(exprs))]
W = max(Image.open(c).width for c in crops) + 380
rows = []
for i, (c, e) in enumerate(zip(crops, exprs)):
    im = Image.open(c)
    row = Image.new("RGB", (W, im.height + 14), (12, 12, 14))
    row.paste(im, (370, 12))
    d = ImageDraw.Draw(row)
    d.text((8, 20), f"#{i}", fill=(255, 210, 90))
    d.text((40, 20), e[:44], fill=(200, 200, 200))
    rows.append(row)
H = sum(r.height for r in rows)
out = Image.new("RGB", (W, H), (12, 12, 14))
y = 0
for r in rows:
    out.paste(r, (0, y)); y += r.height
out.save(dst)
PY
rm -f "$STAGE"/full-*.png
echo "saved $OUT"
