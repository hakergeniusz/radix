#!/bin/bash
# tmpls.sh OUT "label:X Y" ... — for each pair: clear entry (ctrl+a+del via entry click),
# click button at X Y, crop the full entry line; composite all into one labeled image.
set -u
ROOT=/home/hakergeniusz/Projects/radix
OUT="$1"; shift
STAGE="$ROOT/notes/shots/crops"
mkdir -p "$STAGE"
rm -f "$STAGE"/tmpl-*.png
i=0
for tok in "$@"; do
  label="${tok%%:*}"; xy="${tok#*:}"
  for m in ctrl shift alt super meta; do DISPLAY=:98 xdotool keyup $m 2>/dev/null; done
  DISPLAY=:98 xdotool mousemove 185 325; sleep 0.15; DISPLAY=:98 xdotool click 1; sleep 0.2
  DISPLAY=:98 xdotool key ctrl+a; sleep 0.1; DISPLAY=:98 xdotool key Delete; sleep 0.2
  DISPLAY=:98 xdotool mousemove ${xy% *} ${xy#* }; sleep 0.15; DISPLAY=:98 xdotool click 1
  sleep 0.45
  DISPLAY=:98 import -window root "$STAGE/tmpl-full-$i.png"
  python3 - "$STAGE/tmpl-full-$i.png" "$STAGE/tmpl-$i.png" <<'PY'
import sys
from PIL import Image
src, dst = sys.argv[1], sys.argv[2]
im = Image.open(src).crop((140, 312, 900, 366))
im = im.resize((int(im.width * 1.6), int(im.height * 1.6)), Image.LANCZOS)
im.save(dst)
PY
  echo "$i=$label" >> "$STAGE/tmpl-labels.txt"
  i=$((i+1))
done
python3 - "$ROOT/notes/shots/$OUT.png" "$STAGE" <<'PY'
import sys, os
from PIL import Image, ImageDraw
dst, stage = sys.argv[1], sys.argv[2]
labels = dict(l.split("=", 1) for l in open(f"{stage}/tmpl-labels.txt").read().splitlines() if l)
crops = sorted((f for f in os.listdir(stage) if f.startswith("tmpl-") and f.endswith(".png") and "full" not in f),
               key=lambda f: int(f.split("-")[1].split(".")[0]))
W = max(Image.open(f"{stage}/{c}").width for c in crops) + 240
rows = []
for c in crops:
    i = int(c.split("-")[1].split(".")[0])
    im = Image.open(f"{stage}/{c}")
    row = Image.new("RGB", (W, im.height + 10), (12, 12, 14))
    d = ImageDraw.Draw(row)
    d.text((8, 18), labels.get(str(i), "?"), fill=(255, 210, 90))
    row.paste(im, (230, 5))
    rows.append(row)
out = Image.new("RGB", (W, sum(r.height for r in rows)), (12, 12, 14))
y = 0
for r in rows:
    out.paste(r, (0, y)); y += r.height
out.save(dst)
PY
rm -f "$STAGE"/tmpl-*.png "$STAGE"/tmpl-labels.txt
echo "saved $OUT"
