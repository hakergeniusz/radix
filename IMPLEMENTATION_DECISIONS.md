# IMPLEMENTATION_DECISIONS — Radix

Significant architecture decisions and where the spec was silent or
[Unknown], the resolution chosen. Decision IDs start at D1 (the project
repository was reset before implementation; any earlier decision series is
void). Append-only.

- **D1 — GUI toolkit: GTK4 + libadwaita (gtk4-rs / adw crates).**
  The spec (docs/02, 11) describes a CSD header, anchored popovers with modal
  grab, in-window sheets, a searchable shortcuts window, two complete palettes
  with live dark/light re-theme, and hover/press/focus states. libadwaita
  provides these natively; a custom canvas or foreign toolkit would re-derive
  them. Toolkit choice is explicitly free (brief §3); the visual contract is
  taken from docs/11 (geometry/colors), not from any widget hierarchy.

- **D2 — Numeric stack: pure Rust (num-bigint + num-rational for exact
  arithmetic, astro-float for high-precision reals; no GMP/MPFR/MPC, no
  C-linked crates).** Rationale: every numerical dependency must be audited
  and redistribution-compliant (brief §11); a pure permissively-licensed
  stack (MIT/Apache) removes LGPL static-linking obligations and simplifies
  AMD64/ARM64 builds. Observed precision vectors (9-decimal displays,
  171!, 2^1024, exact `1÷3×3=1`) are met with exact rationals plus ~200-bit
  working precision. Gamma for non-integer factorials is implemented in-tree
  (Lanczos approximation — public-domain mathematics).

- **D3 — Engine/application split.** `crates/radix-engine` owns grammar,
  evaluation, numerics, formatting, function/constant catalogs (docs/03–05,
  07); the GUI owns interaction, history, persistence, conversion data. The
  engine's public API was frozen before implementation to keep the two
  workstreams independent.

- **D4 — Boundary Unknowns resolved (docs/14 items 1–4, 7–11, 25, 26, 28,
  31, 32).** The spec deliberately leaves display thresholds open; the
  implementation pins them so behavior is deterministic. Each choice fits
  every recorded vector:
  - Automatic format → scientific when |x| ≥ 10¹² or 0 < |x| < 10⁻⁶
    (bracketed by 61917364224 plain and 1.844674407×10¹⁹ scientific).
  - Scientific small-integer exemption → integers with |x| < 10 stay plain
    (9, 6 plain; 99 → 9.9×10¹).
  - Fixed format → never scientific; integer parts longer than 13 digits
    truncate with `…` (matches 1844674407370…).
  - Number-of-Decimals range → 0..=15 (9 as-found, 6 exercised).
  - Display rounding → half away from zero (π → 3.141592654 at 9).
  - Implicit multiplication, ⌊⌋/⌈⌉ glyph brackets, standalone subscripts →
    parse errors (only errors were observed upstream; docs/14 #7, #8, #9).
  - Typed function/constant names resolve case-insensitively (lowercase
    verified; superset cannot contradict vectors; docs/14 #10).
  - `mod` = floor-mod (remainder convention unobserved).
  - Statistics: stdev sample (n−1), stdevp population (n), var = sample
    variance.
  - Escape clears the entry selection, else the entry (C-button tooltip says
    "Clear Display [Escape]"; docs/14 #28).
  - Alt+←/→ cycles the entry through stored history equations (listed
    shortcut; effect unobserved; docs/14 #26).
  - Grad-mode trig treated as gradians (1 grad = π/200 rad; no vectors).
  - No history cap (none observed; docs/14 #25).
  - Bit panel: implemented as display-only with index tooltips (two click
    attempts upstream produced no toggle — docs/14 #12).

- **D5 — Currency rates: free network provider, honest source label.**
  Rates refresh from a public no-key endpoint (frankfurter.dev, ECB-derived)
  with a bundled snapshot fallback; the ⓘ dialog's "Source …" line names the
  configured provider. This deviates from the observed verbatim "Source
  International Monetary Fund" (docs/06 §6) — recorded as an intentional,
  user-visible deviation rather than mislabeling the data source. Refresh
  interval/display/completion preferences are wired per docs/02 §4.

- **D6 — Unenumerated unit lists (docs/06 §4: 10 of 14 categories not
  enumerated).** Standard, verifiable unit sets with exact decimal factors
  are provided for Angle, Area, Digital Storage, Duration, Energy, Frequency,
  Mass, Pressure, Speed, Volume (SI-derived factors; temperature affine maps
  per docs/06 §5). Ordering follows largest→smallest by factor (the observed
  lists are "mostly" ordered this way; the exceptions' rule is Unknown, so a
  strict rule was chosen and documented).

- **D7 — Identity.** Window title "Radix", About "Radix", exec `radix`, app
  ID `io.github.hakergeniusz.Radix`, original icon (brief §8–9). The observed
  window title "Calculator" is the upstream product's identity and is
  deliberately not reproduced.

- **D8 — Persistence.** A single JSON state file under
  `XDG_STATE_HOME/io.github.hakergeniusz.Radix/state.json`, written atomically
  on graceful close, holds: history rows (echo text + serialized Value for
  live re-render), current mode, conversion category/units, window size,
  color-scheme choice, and the scalar preferences (docs/09 §3–4: restored on
  orderly exit, lost on hard kill, pending entry text not restored). A
  GSettings schema was considered but a single state file keeps the state
  transactional (one atomic write for everything, matching the observed
  graceful-vs-kill contract).

- **D9 — Financial formulas.** Field inventories per docs/02 §9; formulas
  are the standard finance forms: Fv = pmt·((1+r)ⁿ−1)/r (matches the single
  contract vector T083 with pmt=−100, r=0.05, n=10 → −1257.789253555); Pmt =
  P·r/(1−(1+r)⁻ⁿ); Pv = pmt·(1−(1+r)⁻ⁿ)/r; Ctrm n = ln(FV/PV)/ln(1+r); Rate
  r = (FV/PV)^(1/n)−1; Term n = ln(1+FV·r/pmt)/ln(1+r); Sln = (cost−salvage)/life;
  Syd = (cost−salvage)·(life−period+1)·2/(life·(life+1)); Ddb = min(2/life·book,
  remaining); Gpm price = cost/(1−margin). Only Fv is contract-verified
  (docs/14 #29); the rest are documented as standard-formula implementations.

- **D10 — History rows store the evaluated Value (compactly serialized)
  alongside the echo text** so format/base changes re-render results live
  (docs/07 §5) without re-evaluating (variable redefinition does not rewrite
  history — docs/04 §10).

- **D11 — `=` key evaluates** (docs/03 §7 "9=" convention) and is never
  inserted as a character; digits after a shown result start a new entry,
  operators append (fresh-start rule, docs/03 §1; E084-pattern behavior).

- **D12 — Multi-window.** Ctrl+N / New Window opens an additional window
  sharing the engine environment and history (behavior otherwise unobserved —
  docs/14 #20; minimal shared-state model chosen).

- **D13 — Sundry f(x) category and Electromagnetism constants (spec gaps —
  docs/14 #5, #6).** Sundry = abs, exp, sqrt (standard, independently
  verifiable). Electromagnetism = c, q (elementary charge), μ₀, ε₀, Z₀ with
  CODATA values. Physics constants use standard CODATA/public values.

- **D14 — Original icon.** In-house vector artwork (stylised √-radical over
  a decimal point, accent pink #d56199), SVG master + PNGs at 16/24/32/48/
  64/128/256/512, registered as `io.github.hakergeniusz.Radix` in hicolor;
  no upstream artwork or branding.

- **D15 — Packaging.** deb (debhelper), rpm (spec), Arch PKGBUILD, Flatpak
  (GNOME runtime); AMD64 + ARM64; only documented permissions (network for
  currency refresh is the sole non-default one, per brief §10 "required
  permissions only").

- **D16–D21 — Owner-evidence UI corrections (docs/16, 2026-11-04).** Eleven
  screenshots of the original (v51.1) supplied by the project owner
  supersede the inaccurate parts of docs/02: single full keypad grids at
  wide width (Advanced 10-col, Financial 11-col, Programming 12-col with
  base/word dropdowns in the pad; no carousel), bit panel with 63/47/32 /
  31/15/0 labels and no base readout row, no back arrow in Conversion,
  entry inside the display card except in Basic, no Clear History menu
  item, "About Calculator" label, Preferences currency group (Weekly /
  Currency Name / All defaults), Length default Kilometers. ⇧⁻¹ implemented
  as an inverse-functions toggle (Inferred — D16); logᵧ→`log(`, ⌈⌉/⌊⌋→
  `ceil(`/`floor(` (D17); Currency Conversion button jumps to the currency
  pane (D18).

- **D22 — Entry clears after `=`; operator continuation (docs/16 §8, 2026-11-04).**
  E085/E087 show a caret-only entry directly after evaluation, superseding the
  docs/03 §1 "shows a result" wording. The evaluated result is remembered
  invisibly: an operator typed next continues from it (`375.55+2` → `377.55`),
  a digit starts fresh (T074 preserved). Continuation itself [Inferred].
- **D23 — Automatic plain display: 9 fractional digits + 9-run swallow
  (docs/16 §8).** `sqrt(2)` → `1.414213562` (T027) and `0.833333333×111` →
  `92.5` (E087) cannot come from one significance cap; implemented rule = cap
  at 9 fractional digits, then re-round when the capped fraction contains a
  run of ≥5 nines. Mechanism [Inferred] from two observations.
- **D24 — Programming bit panel follows the entry (docs/16 §8, E089).** Bits
  of the current entry value (integer part, word-size mask); empty or
  unparseable → zeros; updates on every entry change. History rows do not
  feed the panel.

## D25 — One display card in every mode (2026-11-04, side-by-side batch)
The entry strip lives INSIDE the display card in all calc modes (Basic
included); Programming adds the bit panel below it inside the same card.
Supersedes the earlier Basic-separate-strip reading of E085. [Observed]

## D26 — Stock libadwaita window palette
Window background, headerbar and accent come from the system libadwaita
palette (no CSS overrides); app CSS styles only the card surfaces
(#3d3d41 / #333337 dark), keypad buttons (#38383b / #4e4e51), row
separators and text sizes. The = button and ⇧⁻¹ accents follow
@accent_bg_color. [Observed]

## D27 — History row grid: 40 homogeneous columns
echo spans columns 0–15 (40%), the dim "=" centers across columns 24–26
(63.75%), the result right-aligns across 27–38 (30%) and ellipsizes when
narrow. Labels: echo regular 19px CHAR-wrap (mid-number wraps like E085),
result bold 19px. max_width_chars caps keep the window's natural width at
or below the mode default (GTK4 windows grow to content natural — measured
410→474px without the caps). [Observed]

## D28 — Header decoration layout ":close"
The original shows only the close button in every mode; Radix pins
gtk-decoration-layout to ":close". Window defaults: 410 (Basic/Conversion),
750 (wide modes), height 720. [Observed]

## D29 — Conversion unit dropdown color
libadwaita's flat menubutton chain renders the unit labels at ≈62%
brightness; the original shows full foreground. Radix pins
`color: @window_fg_color` on label.conv-unit and the menubutton button.
[Observed]
