#!/usr/bin/env python3
"""cropzoom.py DIR START END X0 Y0 X1 Y1 [SCALE] - tiled crop of consecutive frames."""
import sys, os, glob
from PIL import Image, ImageDraw
d = sys.argv[1]; a = int(sys.argv[2]); b = int(sys.argv[3])
box = tuple(int(v) for v in sys.argv[4:8])
scale = float(sys.argv[8]) if len(sys.argv) > 8 else 2.0
frames = sorted(glob.glob(os.path.join(d, "frames", "f*.png")))
sel = [(a + n, frames[a + n]) for n in range(max(0, a), min(len(frames), b + 1))]
w, h = box[2] - box[0], box[3] - box[1]
tw, th = int(w * scale), int(h * scale)
cols = 3
rows = (len(sel) + cols - 1) // cols
sheet = Image.new("RGB", (cols * tw, rows * (th + 18)), (20, 20, 20))
dr = ImageDraw.Draw(sheet)
for n, (idx, f) in enumerate(sel):
    im = Image.open(f).convert("RGB").crop(box).resize((tw, th), Image.LANCZOS)
    x, y = (n % cols) * tw, (n // cols) * (th + 18) + 18
    sheet.paste(im, (x, y))
    dr.text((x + 3, y - 15), f"f{idx}", fill=(255, 255, 100))
out = os.path.join(d, f"cropzoom-{a}-{b}.png")
sheet.save(out)
print(out)
