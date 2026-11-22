#!/usr/bin/env python3
import sys, re
p = sys.argv[1]
rows = []
for line in open(p):
    m = re.match(r"\s*(\d+)\s+(\d+)\s+\(([^)]*)\)", line)
    if m:
        rows.append((int(m.group(1)), int(m.group(2)), m.group(3)))
big = [(r[0], r[1]) for r in rows if r[1] >= 5000]
small = [(r[0], r[1], r[2]) for r in rows if 100 <= r[1] < 5000]
print(f"  big={big[:6]}  small={[r[0] for r in small][:16]}{'...' if len(small)>16 else ''}")
if small:
    print(f"  small bbox: {small[0][2]} .. {small[-1][2]}")
