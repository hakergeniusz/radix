#!/usr/bin/env python3
"""Frame-diff analysis of a mode-switch capture (source-blind, pixels only)."""
import sys, os, glob
import numpy as np
from PIL import Image, ImageDraw

d = sys.argv[1]
frames = sorted(glob.glob(os.path.join(d, "frames", "f*.png")))
if not frames:
    sys.exit(f"no frames in {d}")

def load(i):
    return np.asarray(Image.open(frames[i]).convert("RGB"), dtype=np.int16)

rows = []
prev = load(0)
for i in range(1, len(frames)):
    cur = load(i)
    mask = np.abs(cur - prev).max(axis=2) > 12
    cnt = int(mask.sum())
    bbox = None
    if cnt:
        ys, xs = np.where(mask)
        bbox = (int(xs.min()), int(ys.min()), int(xs.max()), int(ys.max()))
    rows.append((i, cnt, bbox))
    prev = cur

onset = next((i for i, c, b in rows if c >= 200), None)
print(f"frames={len(frames)} fps=60 onset_frame={onset}")
print(" idx  changed  bbox(x0,y0,x1,y1)")
for i, c, b in rows:
    if c >= 20 or (onset is not None and onset - 4 <= i <= onset + 100):
        print(f" {i:4d} {c:8d}  {b}")

if onset is None:
    sys.exit("no significant change detected")

# contact sheet: 3 frames before onset .. 90 after, sampled every 3 frames
idxs = list(range(max(0, onset - 3), min(len(frames), onset + 91), 3))
tw, th = 280, 245
cols = 4
rcount = (len(idxs) + cols - 1) // cols
sheet = Image.new("RGB", (cols * tw, rcount * (th + 18)), (25, 25, 25))
draw = ImageDraw.Draw(sheet)
for n, i in enumerate(idxs):
    im = Image.open(frames[i]).convert("RGB").resize((tw - 4, th - 4), Image.LANCZOS)
    x = (n % cols) * tw + 2
    y = (n // cols) * (th + 18) + 16
    sheet.paste(im, (x, y))
    ms = int(round((i - onset) * 1000 / 60))
    draw.text((x + 2, y - 14), f"f{i} {ms:+d}ms", fill=(255, 255, 120))
out = os.path.join(d, "contact.png")
sheet.save(out)
print("contact sheet ->", out)
