#!/bin/bash
# tooltips.sh OUT "label:X Y" ... — hover each coordinate ~1.2 s, crop the tooltip zone
# (bottom band under the pointer), composite all into one labeled image.
set -u
ROOT=/home/hakergeniusz/Projects/radix
OUT="$1"; shift
STAGE="$ROOT/notes/shots/crops"
mkdir -p "$STAGE"
rm -f "$STAGE"/tt-*.png "$STAGE"/tt-labels.txt
i=0
for tok in "$@"; do
  label="${tok%%:*}"; xy="${tok#*:}"
  DISPLAY=:98 xdotool mousemove ${xy% *} ${xy#* }
  sleep 1.7
  DISPLAY=:98 import -window root "$STAGE/tt-full-$i.png"
  python3 - "$STAGE/tt-full-$i.png" "$STAGE/tt-$i.png" "${xy% *}" "${xy#* }" <<'PY'
import sys
from PIL import Image
src, dst, px, py = sys.argv[1], sys.argv[2], int(sys.argv[3]), int(sys.argv[4])
im = Image.open(src)
# crop a band around/below the pointer to catch the tooltip
x0, x1 = max(0, px - 260), min(im.width, px + 260)
y0, y1 = max(0, py - 20), min(im.height, py + 90)
im.crop((x0, y0, x1, y1)).resize(((x1 - x0) * 2, (y1 - y0) * 2), Image.LANCZOS).save(dst)
PY
  echo "$i=$label" >> "$STAGE/tt-labels.txt"
  i=$((i+1))
done
python3 - "$ROOT/notes/shots/$OUT.png" "$STAGE" <<'PY'
import sys, os
from PIL import Image, ImageDraw
dst, stage = sys.argv[1], sys.argv[2]
labels = dict(l.split("=", 1) for l in open(f"{stage}/tt-labels.txt").read().splitlines() if l)
crops = sorted((f for f in os.listdir(stage) if f.startswith("tt-") and f.endswith(".png") and "full" not in f),
               key=lambda f: int(f.split("-")[1].split(".")[0]))
W = max(Image.open(f"{stage}/{c}").width for c in crops) + 200
rows = []
for c in crops:
    i = int(c.split("-")[1].split(".")[0])
    im = Image.open(f"{stage}/{c}")
    row = Image.new("RGB", (W, im.height + 8), (12, 12, 14))
    d = ImageDraw.Draw(row)
    d.text((8, 16), labels.get(str(i), "?"), fill=(255, 210, 90))
    row.paste(im, (195, 4))
    rows.append(row)
out = Image.new("RGB", (W, sum(r.height for r in rows)), (12, 12, 14))
y = 0
for r in rows:
    out.paste(r, (0, y)); y += r.height
out.save(dst)
PY
rm -f "$STAGE"/tt-*.png "$STAGE"/tt-labels.txt
echo "saved $OUT"
