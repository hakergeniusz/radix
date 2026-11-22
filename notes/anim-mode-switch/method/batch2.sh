#!/usr/bin/env bash
set -uo pipefail
R=/home/hakergeniusz/Projects/radix/notes/anim-mode-switch
while read -r name from to; do
  [ -z "$name" ] && continue
  echo "=== $name ($from -> $to) ==="
  bash "$R/capture.sh" "$name" "$from" "$to" 1.2 2.0 >/dev/null || { echo "CAPTURE FAIL $name"; continue; }
  python3 "$R/analyze.py" "$R/evidence/$name" > "$R/evidence/$name/analysis.txt" 2>&1 || echo "ANALYZE FAIL $name"
  python3 "$R/summary1.py" "$R/evidence/$name/analysis.txt"
done <<'EOF'
basic-conv ctrl+alt+b ctrl+alt+c
adv-conv ctrl+alt+a ctrl+alt+c
prog-conv ctrl+alt+p ctrl+alt+c
fin-conv ctrl+alt+f ctrl+alt+c
conv-prog ctrl+alt+c ctrl+alt+p
conv-fin ctrl+alt+c ctrl+alt+f
conv-key ctrl+alt+c ctrl+alt+k
key-conv-2 ctrl+alt+k ctrl+alt+c
conv-adv-2 ctrl+alt+c ctrl+alt+a
EOF
echo BATCH2_DONE
