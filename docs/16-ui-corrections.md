# 16 — UI Corrections from Owner-Provided Evidence (2026-11-04)

**Provenance.** On 2026-11-04 the project owner provided eleven screenshots of
the original application (self-reported version **51.1**, About dialog;
developer field redacted by the owner and treated as redacted here) covering
all six modes, the primary menu, the Preferences dialog and the About dialog.
The files are archived in `notes/evidence-2026-11-04/` as evidence batch
**E085–E095**. Per the owner's instruction these screenshots supersede the
partially inaccurate parts of docs/00–15 (chiefly docs/02 §1, §8.2–8.4, §9
and the docs/02 §7 Conversion header entry). Everything below is **[Observed]
(E085–E095)** unless marked **[Inferred]**.

## 1. Evidence index

| ID | File (notes/evidence-2026-11-04/) | Content |
|----|-----------------------------------|---------|
| E085 | …15-44-14.webp | Basic, 410 px, history with rows, entry strip |
| E086 | …15-44-23.webp | Basic, same, caret in entry |
| E087 | …15-44-27.webp | Advanced, 750 px, full 10-column grid |
| E088 | …15-44-32.webp | Financial, 750 px, full 11-column grid |
| E089 | …15-44-35.webp | Programming, 750 px, bit panel + 12-column grid |
| E090 | …15-44-42.webp | Keyboard, 750 px, no keypad, entry inside card |
| E091 | …15-45-06.webp | Conversion, 410 px, Length Kilometers → Miles |
| E092 | …15-45-14.webp | Conversion with primary menu open |
| E093 | …15-45-31.webp | Preferences (General + Currency Conversion) |
| E094 | …15-45-37.webp | Preferences (scrolled; Favorite Currencies row) |
| E095 | …15-45-51.webp | About dialog (developer redacted by owner) |

## 2. Keypad layouts (supersedes docs/02 §8.2–8.4 carousel model)

At wide width there is **no carousel and no page dots**: each wide mode is a
single full grid with the function columns LEFT of the digit block.

- **Advanced — 10 equal columns** (E087):
  `⇧⁻¹ x² xʸ conj x⁻¹ C ↑n ↓n ÷ mod / sin sinh logᵧ Arg [x] 7 8 9 × % /
  cos cosh log Re |x| 4 5 6 − ( ) / tan tanh ln Im x! 1 2 3 + [=span2] /
  Deg π e i a×b 0 . x⌄ f(x)⌄`.
- **Financial — 11 columns** (E088): R1 `⇧⁻¹ x² xʸ log ln · C ↑n ↓n ÷ mod`
  (gap column between ln and C); R2–R4 `Ctrm Ddb Fv / Gpm Pmt Pv / Rate Sln
  Syd` as 2-column-wide buttons; R5 `Term(2) Currency Conversion(4) 0 . x⌄
  f(x)⌄`. `Currency Conversion` is a wide button in the pad.
- **Programming — 12 columns** (E089): 5 function columns, 5 narrow digit
  columns, 2 operator columns; R1 `⇧⁻¹ x² xʸ twos ⌈x⌉ · C ↑n ↓n ÷ mod`;
  R2 `∧ ∨ logᵧ swap ⌊x⌋ B C D E F × %`; R3 `¬ ⊻ log int |x| 6 7 8 9 A − ( )`;
  R4 `≪ ≫ log₂ frac x! 1 2 3 4 5 + [=span2]`; R5 `Decimal⌄(2) 64-bit⌄(2) á
  0 . x⌄ f(x)⌄(2)`. The base and word-size dropdowns live **in the pad's
  last row** (not in a strip above).
- **Basic — unchanged** (5-column grid, E085; matches docs/02 §8.1).
- **Conversion — unchanged** digit pad `C ⇅ ⌫ / digits / 0 .` (E091).

### 2.1 Bit panel (supersedes docs/02 §8.4 "base readout row")

E089 shows **no base readout row**: between the entry card and the keypad
there is only the display-only bit panel — two rows of eight nibble groups
(4 bits each) with dim index labels `63 … 47 … 32` (top row, MSB first) and
`31 … 15 … 0` (bottom row). Cells beyond the word size blank; digit buttons
for values ≥ the current radix render insensitive (A–F dim in Decimal).

## 3. Display layout (supersedes docs/02 §1 notes and docs/11 card notes)

- Header: `[undo] [redo] ‹mode ⌄ (centered title widget)› [hamburger] [close]`
  for every calc mode **including Keyboard**; undo/redo dim when the stacks
  are empty. **Conversion has no undo/redo and no back arrow** (E091) — the
  "back-arrow ‹" in docs/02 §7 is wrong.
- The history area is one rounded card. Wide modes and Keyboard keep the
  **entry inside the card** (entry row: textview + small ⌫ icon button at the
  right; Keyboard shows no ⌫). Basic keeps the entry as a separate strip
  between the card and the keypad, with an empty gap above the card (E085).
- History rows: echo left (wraps to two lines when narrow), dim `=` column,
  bold right-aligned result that **ellipsizes with … when narrow**
  (`5÷6 = 0.8333…` at 410 px vs `0.833333333` at 750 px, E085 vs E087).
- Keyboard mode: no keypad; the card fills the window, history rows sit
  directly above the entry at the bottom (E090).

## 4. Primary menu (supersedes docs/08 §1 items)

E092: `New Window Ctrl+N` · section **Result Format** (radio: Automatic •
Fixed · Scientific · Engineering) · `Preferences Ctrl+,` · `Keyboard
Shortcuts Ctrl+?` · `Help F1` · `About Calculator`. **No Clear History
item.** The About dialog (E095) is the standard libadwaita about layout:
logo, name, version badge (51.1), developer (redacted), Website / Report an
Issue / Credits / Legal.

## 5. Preferences (supersedes docs/02 §3 omissions)

E093/E094: General group — `Number of Decimals` (spin, **value 9**),
`Trailing Zeroes`, `Group Digits`, `Angle Units` (Degrees), `Word Size`
(64-bit). **Currency Conversion group** — `Exchange Rate Refresh Interval`
(**Weekly**), `Currency Display` (**Currency Name**), `Show Currencies in
Auto-Completion List` (**All**), `Favorite Currencies` (sub-page, chevron).
The option lists behind the closed dropdowns were not captured [Unknown →
decided below].

## 6. Conversion pane defaults

E091: Length defaults to **Kilometers → Miles** (docs/06's unit order is
otherwise confirmed; Parsecs is merely the first list entry, not the
default). Source and result numbers render large and bold; unit dropdowns
are flat text buttons.

## 7. Decisions taken on the new evidence (continuing IMPLEMENTATION_DECISIONS)

- **D16 — ⇧⁻¹ toggle (pink, first button of every wide pad).** Identity was
  Unknown in docs/14; E085–E095 confirm placement and styling only. [Inferred]
  semantics: an *inverse functions* toggle that swaps registered buttons to
  their inverses — sin↔sin⁻¹ (asin), sinh/cosh/tanh likewise, x²↔√, log↔10ˣ,
  ln↔eˣ, log₂↔2ˣ. Pads without targets (Financial beyond x²/log/ln,
  Programming likewise) simply have fewer registered pairs. Flagged in
  IMPLEMENTATION_QUESTIONS.
- **D17 — logᵧ inserts `log(`** (docs/04 §: `logy(x;base)` is spelled
  `log(8;2)`); ⌈x⌉/⌊x⌋ insert `ceil(`/`floor(` (the glyph syntax errored in
  observation — docs/03 §floor/ceiling); `[x]` inserts `[]` with the caret
  between; `|x|` inserts `||`.
- **D18 — Financial "Currency Conversion" button** switches to the
  Conversion pane with the Currency category [Inferred; effect not isolated
  in the evidence].
- **D19 — Length default unit is Kilometers** (E091); other categories keep
  the docs/06 defaults pending evidence.
- **D20 — Currency preference option lists** (dropdowns closed in E093):
  Interval = Never/Daily/Weekly/Monthly; Display = Currency Name/Currency
  Code; Show = All/Favorites. Favorites filter the completion list;
  Currency Display switches the completion detail line between
  "Name - symbol" and the symbol [Inferred].
- **D21 — Mode title dropdown is the centered header title widget** (all
  modes; E085–E092); `About Calculator` is the menu label (E092).

## 8. Corrections from the 750×720 reproduction round (2026-11-04, later)

Rebuilding the E087/E089 scenes in the isolated session (same four history
rows, same window size) surfaced five further deltas, all fixed and
re-verified against the evidence:

- **History rows anchor to the BOTTOM of the card**, directly above the
  entry; spare space accumulates above the rows (E085, E087, E089 — all
  show top gap, no bottom gap).
- **The dim `=` sits in a constant column** at ≈62% of the row width,
  independent of echo/result widths (E087: `=` at x≈465 for all four rows
  at 750 px; E085: x≈250 at 410 px). Implemented as a 4-column homogeneous
  grid: echo spans cols 1–2, `=` centers in col 3 (62.5%), result
  right-aligns in col 4 and ellipsizes when narrow.
- **The entry is EMPTY after evaluation** (E085 and E087 both show a
  caret-only entry directly after the last evaluation). This supersedes
  docs/03 §1's "when the entry currently shows a result" wording: the
  fresh-start rule still applies to the *recall* case (text in buffer);
  after `=` the value is remembered invisibly.
- **The entry backspace button is the ⌫-key glyph** (zoom of E087 shows the
  backspace-key outline with ✕), not a circled-✕.
- **The bit panel tracks the entry, not the last history row** (E089: all
  zeros while the entry is empty despite four history rows).

- **D22 — Entry clears after `=`; operator continuation [Inferred].** After
  a successful evaluation the entry shows only the caret (E085/E087). The
  result is remembered: typing an operator afterwards continues from it
  (`375.55+2` → `377.55`), typing a digit starts fresh — preserving
  docs/03 §1 / T074 semantics. The continuation rule itself is [Inferred]:
  no evidence shows the post-`=` operator case.
- **D23 — Automatic plain display = 9 fractional digits + 9-run swallow
  [Inferred mechanism].** Observed constraints: `sqrt(2)` renders
  `1.414213562` (T027/E019 — 9 fractional digits) while
  `0.833333333×111` renders `92.5` (E087 — the exact value
  `92.499999963` collapses). No single significance cap satisfies both;
  the implemented rule — cap at 9 fractional digits, then re-round if the
  capped fraction contains a run of ≥5 nines — reproduces every observed
  vector (`5÷6` → `0.833333333`, `92.499999963` → `92.5`, `375.55`,
  `1.844674407×10¹⁹`). Flagged in IMPLEMENTATION_QUESTIONS.
- **D24 — Bit panel follows the entry** (E089): current entry value's
  integer part, masked to the word size; empty or unparseable entry → all
  zeros; cells beyond the word size blank. Updates on every entry change.

## 9. Corrections from the side-by-side evidence batch (2026-11-04 18:54–18:56)

The owner provided eleven side-by-side screenshots (mine left, original right;
archived as the source of this section; my reproduction composites in
`notes/comparison-2026-11-04/`) covering Basic/Advanced/Financial/Programming/
Keyboard/Conversion, the primary menu, Preferences and Favorite Currencies.
Pixel scans of the originals (window #2 of each pair) supersede parts of §3,
§4 and §8:

- **The display card exists in EVERY mode, including Basic** — §3's "Basic
  keeps the entry as a separate strip between the card and the keypad" is
  WRONG. One rounded card (radius ≈12px, ≈12px margins on all sides) holds:
  bottom-anchored history rows with 1px separators (above the first row,
  between rows, and below the last row only while rows exist), then the
  **entry strip as the card's bottom section** (caret + ⌫ right, no ⌫ in
  Keyboard), and in Programming a separator + the bit panel below it.
- **Two-tone card surfaces**: history rows sit on the lighter card surface
  (#3d3d41 in dark), the entry/bit strip on a darker inset (#333337).
- **Window palette is stock libadwaita** — the window background is
  #222226 and the headerbar #2e2e32 (dark); no app-level overrides. The
  earlier #444448/#3d3d41-everywhere palette came from misreading the card
  surface as the window surface.
- **Keypad geometry**: buttons span the full card inset (12px margins),
  4px column gaps, 8px row gaps, ≈40px tall, radius ≈9px. Digits +
  decimal point use the lighter shade (#4e4e51); every other button
  (functions, C, ↑n ↓n, ÷ × − + % ( ), mod, Deg π e i a×b) uses #38383b —
  operators are NOT digit-colored and NOT bold. Digit labels bold ≈17px,
  function labels regular ≈14px, `mod` regular. ⇧⁻¹: #38383b tinted with
  ≈5% accent (#3b2f38), glyph in light accent (#f08aba).
- **History row geometry**: 40-column homogeneous grid — echo 0–15 (40%),
  dim "=" centered across columns 24–26 (center 63.75%; measured
  63.1–63.6% at both 410px and 750px), result right-aligned over the last
  12 columns. History text ≈19px at every width (digit glyph height 14px
  in all screenshots); result bold; result ellipsizes at 410px
  ("0.8333…") and renders fully at 750px; the long echo
  "3×111+(23÷60)×111" wraps mid-number (CHAR wrap) to "…×11 / 1" at 410px.
- **Window defaults**: Basic/Conversion 410px, wide modes 750px
  (supersedes the 370/700 constants). Header shows ONLY the close button —
  the decoration layout is forced to ":close" (no minimize/maximize).
- **Primary menu (supersedes §4's "No Clear History item")**: sections are
  `New Window` | `Clear History Ctrl+Escape` | Result Format header + 4
  radios | Preferences / Keyboard Shortcuts / Help | About Calculator —
  with divider lines between sections and NO duplicate "Result Format"
  item.
- **Preferences dialog** is a fixed narrow window (≈370×660), independent
  of the parent window (confirmed against both the 410px and the old 750px
  evidence).
- **Favorite Currencies** page: back-arrow header, always-visible inline
  "Search currencies" entry below the header, two-line rows (currency name
  + dim code subtitle) sorted by NAME, switch per row. The original's
  table starts at "Algerian Dinar" — it omits AFN/ALL/AOA/AWG/AZN which
  Radix's superset table includes [data-level delta, accepted].
- **Conversion pane**: unit dropdowns render in the full foreground color,
  bold ≈15px (libadwaita's flat menubutton otherwise dims them to ≈62% —
  Radix pins `color: @window_fg_color`); values ≈28px bold.
- **[Inferred] 19px row font is width-independent**: measured digit glyph
  height 14px in the 410px and 750px evidence alike.

- **D25 — One card per mode [Observed]** (supersedes the §3 Basic-strip
  reading): the display card contains history rows + entry strip (+ bit
  panel in Programming) in every calc mode.
- **D26 — Stock window palette [Observed]**: window/header/accent follow
  the system libadwaita palette; only the card surfaces, keypad buttons
  and row geometry are app-styled.
- **D27 — History row grid [Observed]**: 40-col homogeneous grid, echo 40%,
  "=" centered at 63.75%, result right-aligned 30%, ≈19px text, CHAR wrap.
- **D28 — Decoration layout [Observed]**: `:close` only.

## 10. Corrections from the 2026-11-04 20:16–20:18 side-by-side batch (round 2)

Twelve new owner screenshots (`/tmp/ref-radix-2`, composites in
`notes/comparison-2026-11-04/sbs2-*.png`) re-measured pixel-exact; the
§9 constants that survive are kept, the rest are corrected here.

### 10.1 Windows and chrome
- **[Observed] Default windows are 360 px (Basic/Conversion) and 700 px
  (wide modes) tall ≈ 615** — supersedes §9's 410/750 reading (those were
  user-resized windows). Basic renders **full-bleed**: the history band
  `#3c3c40` runs edge-to-edge under the header, rows bottom-anchored with
  1px top separators, pitch 37, row text padding 20/20/7/7 (supersedes
  D25's "one inset card" for Basic; wide modes keep the inset card).
- **[Observed] Entry strip** `#333337` = 40px entry row (text centered,
  19px font) + 28px radix line; ⌫ top-aligned (margin-top 4, end 12).
  In Programming the radix line shows `{octal}₈ {hex}₁₆` of the entry
  value at 13px dim; empty in every other mode.

### 10.2 Keypads
- **[Observed] Button metrics**: 64×44 everywhere except Programming
  (64×53, every batch), radius ≈7, gaps 4, pad margins 12; `=` spans the
  last two rows in the rightmost column (92px; 110px in Programming).
  Hex digits are 36–37 wide; Programming base/word-size buttons 130;
  Financial formula buttons 109, Term 109, Currency Conversion 221.
- **[Observed] Programming rows**: r0 = 10×64 (C bold); r1/r2 = 5×64 ops
  + 5×36 hex + 64 tail; r3 = 5×64 + 5×36 + 64; the two bottom rows share
  the 110px `=` (so the pad is `r0,r1,r2, [r3;r4 | =]`). Bit panel:
  `#333337`, digit pitch 15 (11px bold glyphs), nibble groups of 4,
  row pitch ≈26, indices 63/47/32/31/15/0 dim 11px.
- **[Observed] Financial rows**: r0 = 10×64; formula rows 3×109 + digits;
  rows 1–3 are full-width; only r4–r5 share the 92px `=`.
- **[Implementation note]**: uniform pads use a homogeneous GtkGrid; mixed
  pads use hbox-per-row. GtkMenuButton's internal toggle keeps the
  theme's ~11px side padding — its natural width must be zeroed via
  `menubutton.radix-btn > button { padding: 0 }` or the homogeneous
  columns inflate (window grew to 385px).

### 10.3 History and scrolling
- **[Observed] Result column ≈10/40 cols**; at 360 the widget ellipsizes
  the engine string ("0.8333…"), at 700 the full "0.833333333" shows —
  the engine string itself is correct (round_scaled(9)).
- **[Observed] Mode switch resets history scroll to top** (orig
  Programming shows the oldest rows with row 4 clipped below); row
  appends still scroll to bottom. Keyboard mode hides the keypad stack
  entirely (card reaches 12px from the window bottom).

### 10.4 Menus and dialogs
- **[Observed] Conversion primary menu has NO Clear History** (sections:
  New Window | Result Format radios | Prefs/Shortcuts/Help | About);
  the calc-mode menu keeps Clear History (§4 stands for calc modes).
- **[Observed] Preferences** ≈363×593; "Show Currencies in Auto-
  Completion List" title wraps to two lines.
- **[Deferred] Primary-menu popover ≈273px wide in the original; Radix's
  truncates accelerators at ≈200px — mechanism unresolved, cosmetic.

### 10.5 Post-evaluation entry echo (2026-11-05 correction)
- **[Observed] After `=` the result stays in the entry** as a bold echo
  with the caret at the end (owner side-by-side: entry shows "4" after
  2+2 =) — supersedes the E085/E087 caret-only reading (D22 → D36).
  Operators append to the visible result; digits start a fresh entry.

### 10.6 Verification
- 63/63 engine tests, clippy clean, GUI battery
  (`tools/radix-session.sh` + Xvfb) re-shot all 7 scenes; composites
  `notes/comparison-2026-11-04/sbs2-{basic,convmenu,conversion,advanced,
  financial,programming,keyboard}.png`. Known accepted deltas: echo wrap
  point at 360 (font metrics), +10px AdwDialogHost chrome in wide modes,
  rig renders accent blue (user's system accent is pink).

## 11. Keyboard shortcuts (2026-11-05, owner Ctrl+? screenshots)
- **[Observed]** The shortcuts window is an AdwShortcutsDialog with search,
  section pill row and five sections: General (New Window Ctrl+N, Close
  Window Ctrl+W, Quit Ctrl+Q, Help F1, Open Menu F10, Preferences Ctrl+,,
  Shortcuts Ctrl+?, Clear History Ctrl+Escape), Switching Modes (6 modes +
  Keyboard-alt Ctrl+Alt+T), Programming Mode (Ctrl+B/O/D/H base switch),
  Keyboard Entry (below) and Equation (Copy/Paste/Undo/Redo/Prev/Next
  Alt+←→/Clear Display Escape/Factorize Ctrl+F).
- **[Observed] Keyboard Entry rows**: `*` → ×, `/` → ÷, Square Ctrl+2,
  Square Root Ctrl+R, Inverse Ctrl+I, Pi Ctrl+P, Tau Ctrl+T, Degree Ctrl+',
  Floor Alt+[ then ], Ceiling Ctrl+[ then ], Scientific Format Ctrl+E,
  Subscript Alt+0–9, Superscript Ctrl+0–9, Superscript minus Ctrl+−.
- **[Implementation]** Accelerator display strings follow the
  AdwShortcutLabel format (sequences joined with `+`, ranges with `...`);
  behavior wired in `wire_entry_keys` + `win.open-menu`/`win.factorize`
  actions. Factorize replaces the entry with the prime product (12 →
  2×2×3); the a×b button triggers it (docs/02 tooltip).
