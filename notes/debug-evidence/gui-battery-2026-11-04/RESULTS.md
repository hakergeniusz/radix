# GUI T-battery results — 2026-11-04 (post docs/16 §8 round)

Isolated Xvfb session (`tools/radix-session.sh`, Adwaita-dark, fresh state
unless noted), xdotool XTEST input, RADIX_TRACE assertions + screenshots.
Engine vectors T001–T049/T050/T051 etc. run headless in
`crates/radix-engine/tests/vectors.rs` — **63/63 pass**.

| Vector | Result | Evidence |
|---|---|---|
| T027/E087 display set (√2, 5÷6, 0.833333333×111, 3×111+(23÷60)×111) | PASS — exact original strings incl. `92.5` swallow (D23) | `t056-E087-advanced.png` vs E087 |
| T052 decimals 6 + trailing + grouping | PASS — `2+3 = 5.000000`, `1,234.00…` | `t052t053t055-formatting.png` |
| T053 significance destruction | PASS — `0.0000123 = 0.000012` | same |
| T055 grouped echo on new rows only | PASS — `1,234+0` echo grouped | same |
| T056 live target update | PASS — target follows source per keystroke | stage 3 captures |
| T071 `10 DKK in USD` free-entry | PASS — numeric row, plain formatting | stage 2 captures |
| T072 mode accels + width re-target | PASS — 410 narrow / 750 wide (owner-evidence widths) | header shots |
| T073 Up-recall in Keyboard | PASS — `46` recalled into empty entry | `t073-recall.png` |
| T074 fresh start after result | PASS — trace `state=ResultShown fresh=true` replaces | trace log |
| T075 append after error | PASS — `1/0` + `2` → `1/02` (trace `state=ErrorShown fresh=false`) | trace log |
| T077 Ctrl+Escape clear | PASS — history emptied instantly | stage 1 captures |
| T078 completion popover | PASS — D-codes + detail lines above caret | `t078-completion.png` |
| T079 Tab accepts first suggestion | PASS — trailing token replaced | stage 2 captures |
| T081 header undo/redo | PASS — `224` → `22` via Ctrl+Z | stage 1 captures |
| T089 graceful close + relaunch | PASS — mode Conversion, Length Kilometers→Miles, history intact, entry empty; **fixed zero-height value rows on restore** | `t089-restored-conversion.png` |
| T089 bit panel entry-follow (D24) | PASS — zeros on empty entry (E089), `2255` → `1000 1100 1111` live | `t089-bitpanel-empty.png`, `t084-bitpanel-2255.png` |
| T097 error styling | PASS — red text + token highlight + prose line, no history row | `t075t097-error.png` |
| T098 Preferences structure | PASS — matches E093/E094 groups and as-found values | `t098-prefs-set.png` (decimals 6, toggles driven live) |
| T075/T078 popover dismiss on eval | PASS after fix — completion closes on `=`/error | stage 3 |

Not driven this round (dialog/click-heavy or network): T060, T064–T067,
T068–T070 (live rates), T082/T083 (Financial dialog), T084 à-dialog,
T085/T086 (picker clicks), T088 (F1), T090 (SIGKILL), T091–T095/T096
(hover/theme/geometry), T099 (About). Several of these were visually
verified in the earlier same-day battery (`09-conversion.png` and the
E085–E090 sweep); the rest are marked unverified in the final report.

---

## Side-by-side comparison round (2026-11-04 18:5x owner batch, later same day)

Owner provided 11 side-by-side screenshots (mine left, original right).
Reproduced the same scenes in the Xvfb rig (fresh state, same four
evaluations: 2+2, 5÷6, 0.833333333×111, 3×111+(23÷60)×111) and fixed the
deltas; composites archived in `notes/comparison-2026-11-04/`.

| Scene | Result | Evidence |
|---|---|---|
| Basic 410×720, 4 rows | PASS — card+entry strip, row separators, = at 63.75%, "0.833333…" ellipsis, 19px fonts | sbs-basic.png |
| Advanced 750×720 | PASS — full grid, uniform function shade, bold digits, ⇧⁻¹ tint | sbs-advanced.png |
| Programming 750 | PASS — bit panel inside card under entry strip, A–F dim in Decimal, bits track entry (2255 → 100011001111) | sbs-programming.png, my-programming-2255.png |
| Keyboard 750 | PASS — card fills window, no ⌫ | sbs-keyboard.png |
| Financial 750 | PASS — 11-col grid, gap column, wide formula buttons | sbs-financial.png |
| Conversion 410 | PASS — Kilometers→Miles default, bright bold unit labels, 28px values | sbs-conversion.png |
| Primary menu | PASS — Clear History Ctrl+Escape present, section dividers, no duplicate header item | my-menu-open.png |
| Preferences | PASS — 370×660 fixed dialog (rig shows flat rows: GTK_THEME artifact; user's env renders boxed like the original) | sbs-preferences.png |
| Favorite Currencies | PASS — back arrow, inline search, two-line name-sorted rows (table superset noted as Q25) | sbs-favorites.png |

Fixes driven by this round (all in this repo):

1. Display area rebuilt as ONE card (entry strip inside, bit panel inside
   Programming); Basic's separate strip removed; top spacer removed.
2. CSS: stock window/header (no overrides), two-tone card surfaces,
   row separators (border-top per row + has-rows bottom line), 19px row
   fonts, entry strip 18px text, ⇧⁻¹ tint 5% accent, = follows
   @accent_bg_color.
3. History rows: 40-col homogeneous grid (echo 16 cols, "=" 24–26,
   result 27–38), CHAR wrap, max_width_chars natural caps (GTK4 windows
   grow to content natural — measured 410→474px without them).
4. Windows: NARROW_W 370→410, WIDE_W 700→750; gtk-decoration-layout
   ":close" (no min/max); outer calc/conversion stack hhomogeneous(false).
5. Keypads: spacing 8v/4h, height 40, uniform function shade (arith ops
   are NOT digit-colored — pixel-verified #38383b), function labels
   regular weight.
6. Menu: Clear History + sections; duplicate "Result Format" item removed.
7. Preferences: 370×660. Favorites: rewritten (back arrow, inline search,
   two-line rows sorted by name, filter-as-you-type).
8. Conversion: explicit fg color on unit dropdowns (was ~62% dim),
   28px values, wider spacing.

Gates after the round: engine 63/63, clippy 0 warnings, window stays at
mode default through the full evaluation bisect.
