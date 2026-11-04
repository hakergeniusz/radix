#!/usr/bin/env bash
# Scripted GUI verification battery (docs/12 subset) against the Radix binary
# in an isolated Xvfb session. Produces screenshots + a pass/fail summary
# from screen OCR-free checks (window geometry, pixel probes are advisory;
# behavioral vectors are primarily verified by engine unit tests).
#
# Usage: tools/radix-gui-test.sh [path-to-radix-binary]
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN=${1:-"$ROOT/target/debug/radix"}
OUT=${RADIX_TEST_OUT:-/tmp/radix-gui-test}
rm -rf "$OUT"; mkdir -p "$OUT"

run_in_session() {
  RADIX_BIN="$BIN" RADIX_TEST_OUT_STATE=/tmp/radix-gui-test-state \
    "$ROOT/tools/radix-session.sh" "$@"
}

shot() { # shot <name>
  env -i DISPLAY="${RADIX_TEST_DISPLAY:-:99}" PATH=/usr/bin:/bin \
    import -window root "$OUT/$1.png" 2>/dev/null
}

# --- launch -----------------------------------------------------------------
run_in_session bash -c '
  export RADIX_TEST_STATE=/tmp/radix-gui-test-state
  mkdir -p "$RADIX_TEST_STATE"/{data,config,cache,state}
  XDG_DATA_HOME="$RADIX_TEST_STATE/data" XDG_CONFIG_HOME="$RADIX_TEST_STATE/config" \
  XDG_CACHE_HOME="$RADIX_TEST_STATE/cache" GDK_BACKEND=x11 GTK_IM_MODULE=xim \
  '"$BIN"' & echo $! > /tmp/radix-gui-test/pid
' &
sleep 2.5
shot 001-launch-basic

# --- basic evaluate via xdotool ---------------------------------------------
type_into_entry() {
  xdotool search --name '^Radix$' windowactivate --sync 2>/dev/null || true
  xdotool type --delay 40 "$1"
  xdotool key --delay 60 Return
  sleep 0.3
}

type_into_entry "2+3*4"        ; shot 010-t001-2p3x4
type_into_entry "2^3^2"        ; shot 011-t005-pow-rightassoc
type_into_entry "100+10%"      ; shot 012-t008-percent-rel
type_into_entry "1+100+10%"    ; shot 013-t012-percent-accum
type_into_entry "171!"         ; shot 014-t021-factorial-171
type_into_entry "1/3*3"        ; shot 015-t022-exact
type_into_entry "2^1024"       ; shot 016-t023-bigpow
type_into_entry "sqrt(2)"      ; shot 017-t027-sqrt2
type_into_entry "log(8;2)"     ; shot 018-t028-logy
type_into_entry "round(2.5)"   ; shot 019-t029-round
type_into_entry "sin(30)"      ; shot 020-t032-sin30
type_into_entry "tan(90)"      ; shot 021-t-tan90-error

# --- mode switches (T072) ---------------------------------------------------
for k in a f p k c b; do
  xdotool key --clearmodifiers "ctrl+alt+$k"
  sleep 0.4
  shot "030-mode-$k"
done

echo "battery done; screenshots in $OUT"
