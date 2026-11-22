#!/usr/bin/env bash
set -uo pipefail
R=/home/hakergeniusz/Projects/radix/notes/anim-mode-switch
while read -r name from to; do
  [ -z "$name" ] && continue
  echo "=== $name ($from -> $to) ==="
  bash "$R/capture.sh" "$name" "$from" "$to" 1.2 2.0 || { echo "CAPTURE FAIL $name"; continue; }
  python3 "$R/analyze.py" "$R/evidence/$name" > "$R/evidence/$name/analysis.txt" 2>&1 || echo "ANALYZE FAIL $name"
  head -1 "$R/evidence/$name/analysis.txt"
  awk '$2+0 >= 20 {print "   ",$0}' "$R/evidence/$name/analysis.txt" | head -8
done <<'EOF'
adv-prog ctrl+alt+a ctrl+alt+p
prog-fin ctrl+alt+p ctrl+alt+f
fin-key ctrl+alt+f ctrl+alt+k
key-basic ctrl+alt+k ctrl+alt+b
adv-basic ctrl+alt+a ctrl+alt+b
key-conv ctrl+alt+k ctrl+alt+c
conv-basic ctrl+alt+c ctrl+alt+b
conv-adv ctrl+alt+c ctrl+alt+a
EOF
echo BATCH_DONE
