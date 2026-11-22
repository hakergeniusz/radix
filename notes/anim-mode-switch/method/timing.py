#!/usr/bin/env python3
"""Per-capture focus-ring alpha trace around the switch frame."""
import os, re, glob
import numpy as np
from PIL import Image

R = "/home/hakergeniusz/Projects/radix/notes/anim-mode-switch/evidence"
# capture name -> destination mode (narrow: basic/conversion, wide: rest)
TO = {
    "basic-adv": "advanced", "adv-basic": "basic", "adv-prog": "programming",
    "prog-fin": "financial", "fin-key": "keyboard", "key-basic": "basic",
    "adv-conv": "conversion", "basic-conv": "conversion", "prog-conv": "conversion",
    "fin-conv": "conversion", "key-conv": "conversion", "key-conv-2": "conversion",
    "key-conv-3": "conversion", "conv-basic": "basic", "conv-adv": "advanced",
    "conv-adv-2": "advanced", "conv-prog": "programming", "conv-fin": "financial",
    "conv-key": "keyboard", "noanim-conv": "conversion",
}
for name in sorted(TO):
    d = os.path.join(R, name)
    ap = os.path.join(d, "analysis.txt")
    if not os.path.exists(ap):
        continue
    switch = None
    for line in open(ap):
        m = re.match(r"\s*(\d+)\s+(\d+)", line)
        if m and int(m.group(2)) >= 5000:
            switch = int(m.group(1)); break
    if switch is None:
        continue
    crop = (78, 5, 212, 51) if TO[name] in ("basic", "conversion") else (288, 4, 412, 52)
    vals = []
    frames = sorted(glob.glob(os.path.join(d, "frames", "f*.png")))
    for i in range(max(0, switch - 2), min(len(frames), switch + 22)):
        px = np.asarray(Image.open(frames[i]).convert("RGB").crop(crop), dtype=np.int16)
        vals.append(round(float((px[:, :, 0] - px[:, :, 1]).mean()), 2))
    print(f"{name:12s} switch=f{switch:3d} alpha: {vals}")
