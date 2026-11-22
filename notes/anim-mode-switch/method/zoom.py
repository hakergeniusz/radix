#!/usr/bin/env python3
"""zoom.py DIR START END [SCALE] - contact sheet of consecutive full-res frames."""
import sys, os, glob
from PIL import Image, ImageDraw
d, a, b = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])
scale = float(sys.argv[4]) if len(sys.argv) > 4 else 1.0
frames = sorted(glob.glob(os.path.join(d, "frames", "f*.png")))
sel = frames[max(0, a):min(len(frames), b + 1)]
# crop to window area (x 0..710, y 0..620) to drop black desktop
w, h = 710, 620
tw, th = int(w * scale), int(h * scale)
cols = 2
rows = (len(sel) + cols - 1) // cols
sheet = Image.new("RGB", (cols * tw, rows * (th + 20)), (20, 20, 20))
dr = ImageDraw.Draw(sheet)
for n, f in enumerate(sel):
    idx = a + n
    im = Image.open(f).convert("RGB").crop((0, 0, w, h)).resize((tw, th), Image.LANCZOS)
    x, y = (n % cols) * tw, (n // cols) * (th + 20) + 20
    sheet.paste(im, (x, y))
    dr.text((x + 4, y - 16), f"frame {idx}  t={(idx)*1000/60:.0f}ms", fill=(255, 255, 100))
out = os.path.join(d, f"zoom-{a}-{b}.png")
sheet.save(out)
print(out)
