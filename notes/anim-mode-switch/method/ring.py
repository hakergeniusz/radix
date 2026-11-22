#!/usr/bin/env python3
"""ring.py DIR X0 Y0 X1 Y1 [START END] - pink focus-ring pixel count per frame."""
import sys, os, glob
import numpy as np
from PIL import Image
d = sys.argv[1]
x0, y0, x1, y1 = (int(v) for v in sys.argv[2:6])
a, b = (int(v) for v in sys.argv[6:8]) if len(sys.argv) >= 8 else (0, 10**9)
frames = sorted(glob.glob(os.path.join(d, "frames", "f*.png")))
print("frame pinkpx meanR-G")
for i in range(max(0, a), min(len(frames), b + 1)):
    px = np.asarray(Image.open(frames[i]).convert("RGB").crop((x0, y0, x1, y1)), dtype=np.int16)
    rg = px[:, :, 0] - px[:, :, 1]
    count = int(((rg > 35) & (px[:, :, 0] > 90)).sum())
    print(f"{i:4d} {count:7d} {float(rg.mean()):7.2f}")
