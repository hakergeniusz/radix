#!/usr/bin/env python3
"""Keep compact evidence: per-capture switch contact sheets + key raw frames."""
import os, re, glob, shutil, subprocess
from PIL import Image, ImageDraw

R = "/home/hakergeniusz/Projects/radix/notes/anim-mode-switch"
E = os.path.join(R, "evidence")
KEY = {"key-conv", "conv-adv", "conv-fin", "basic-adv", "prog-fin", "noanim-conv2"}

for d in sorted(glob.glob(os.path.join(E, "*"))):
    if not os.path.isdir(d):
        continue
    name = os.path.basename(d)
    ap = os.path.join(d, "analysis.txt")
    frames = sorted(glob.glob(os.path.join(d, "frames", "f*.png")))
    if not os.path.exists(ap) or not frames:
        print("skip (no frames/analysis):", name)
        continue
    switch = None
    for line in open(ap):
        m = re.match(r"\s*(\d+)\s+(\d+)", line)
        if m and int(m.group(2)) >= 5000:
            switch = int(m.group(1)); break
    if switch is None:
        print("skip (no switch):", name)
        continue
    idxs = list(range(max(0, switch - 2), min(len(frames), switch + 23), 2))
    tw, th = 355, 310
    cols = 4
    rows = (len(idxs) + cols - 1) // cols
    sheet = Image.new("RGB", (cols * tw, rows * (th + 18)), (20, 20, 20))
    dr = ImageDraw.Draw(sheet)
    for n, i in enumerate(idxs):
        im = Image.open(frames[i]).convert("RGB").crop((0, 0, 710, 620)).resize((tw, th), Image.LANCZOS)
        x, y = (n % cols) * tw, (n // cols) * (th + 18) + 18
        sheet.paste(im, (x, y))
        dr.text((x + 4, y - 15), f"f{i} {(i-switch)*1000/60:+.0f}ms", fill=(255, 255, 120))
    sheet.save(os.path.join(d, "switch-contact.png"))

    if name in KEY:
        sel = os.path.join(d, "selected")
        os.makedirs(sel, exist_ok=True)
        for i in range(max(0, switch - 2), min(len(frames), switch + 21)):
            shutil.copy2(frames[i], os.path.join(sel, f"f{i:04d}.png"))
    print("curated", name, "switch", switch)

# trash the bulky raw frame directories (keep selected/ and contact sheets)
for d in sorted(glob.glob(os.path.join(E, "*", "frames"))):
    subprocess.run(["trash", "-f", d], check=False)
print("done")
