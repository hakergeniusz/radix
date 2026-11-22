#!/usr/bin/env python3
"""cropzoom2.py DIR X0 Y0 X1 Y1 SCALE - tiled crop of curated selected/ frames."""
import sys, os, glob
from PIL import Image, ImageDraw
d = sys.argv[1]
box = tuple(int(v) for v in sys.argv[2:6])
scale = float(sys.argv[6])
frames = sorted(glob.glob(os.path.join(d, "selected", "f*.png")))
w, h = box[2] - box[0], box[3] - box[1]
tw, th = int(w * scale), int(h * scale)
cols = 3
rows = (len(frames) + cols - 1) // cols
sheet = Image.new("RGB", (cols * tw, rows * (th + 18)), (20, 20, 20))
dr = ImageDraw.Draw(sheet)
for n, f in enumerate(frames):
    idx = int(os.path.basename(f)[1:5])
    im = Image.open(f).convert("RGB").crop(box).resize((tw, th), Image.LANCZOS)
    x, y = (n % cols) * tw, (n // cols) * (th + 18) + 18
    sheet.paste(im, (x, y))
    dr.text((x + 3, y - 15), f"f{idx}", fill=(255, 255, 100))
out = os.path.join(d, "ring-zoom.png")
sheet.save(out)
print(out)
