# 11 — Visual Specification

Geometry, color, and interactive-state specification, measured from this
investigation's own screenshots (assets referenced below live in
`docs/assets/`; raw captures under `notes/shots/`). Toolkit-independent:
surfaces are described as rectangles, text, and state changes, not widget
classes. Statements **[Observed]** (pixel-sampled/edge-scanned) unless
labeled otherwise.

Reference screenshots:
- `reference-basic-dark-370.png` / `reference-basic-light-370.png`
- `reference-advanced-dark-700.png` / `reference-advanced-light-700.png`
- `reference-keyboard-light-700.png`, `reference-financial-light-700.png`,
  `reference-programming-light-700.png`, `reference-conversion-light-370.png`
- dialogs: `dialog-mode-menu-{dark,light}.png`, `dialog-hamburger-{dark,light}.png`,
  `dialog-preferences-{dark,light}.png`, `dialog-about-{dark,light}.png`,
  `dialog-unit-dropdown-search-light.png`
- states: `state-errors-dark.png`, `state-error-division-light.png`,
  `state-error-tangent-dark.png`, `state-entry-selection-light.png`

---

## 1. Window

- Default (narrow) size ≈ **370 × 639 px** at 1× scale; wide keypad modes
  re-target ≈ **700 px** wide; height unchanged (~660 px incl. CSD shadow).
  Window has **rounded corners** and a soft drop shadow (CSD; no WM).
  **[Observed]** (E001, E030, E080)
- Per-mode widths: Basic 370 · Conversion 370 · Keyboard 700 · Advanced 700 ·
  Financial 700 · Programming 700 (measured; each observed switching from a
  differently-sized mode except Financial/Programming — inferred by pattern).
  **[Observed]** (E080)

## 2. Regions & geometry (measured by edge scan on the 1× captures)

### 2.1 Header strip
Full-width strip ≈ **44–48 px** tall (baseline grid: controls centered at
y≈28). Contents left→right: undo, redo, mode button, (flexible space),
hamburger, minimize, maximize, close. In the 370 px window the same controls
compress toward the left; measured anchors (px, 1×): narrow — mode ≈ 100–150,
☰ ≈ 228, − ≈ 266, ▢ ≈ 305, × ≈ 341; wide — mode ≈ 344 (center), ☰ ≈ 557,
− ≈ 596, ▢ ≈ 634, × ≈ 672. **[Observed]** (E081 header anchor note; measured)

### 2.2 History list
- Occupies the body area above the entry (full height in Keyboard mode).
- Rows: equation text left-aligned, "=" centered, result **right-aligned
  bold**; row height ≈ 28–36 px (wraps to two lines for long equations);
  hairline separators; no zebra striping in light theme. **[Observed]**
- Wide Advanced/Financial layouts inset the history as a **rounded card**
  (clamped ≈ 675 px wide) floating over the body background. **[Observed]**
  (E030, E070)

### 2.3 Entry line
Bottom strip ≈ 50–65 px tall; text left-aligned with generous left padding
(≈ 16–20 px); caret present when focused; result echo bold. **[Observed]**

### 2.4 Keypad grid (measured, 1×)
| Window width | Button size | Gaps | Side margins | Rows |
|---|---|---|---|---|
| 370 px | ≈ 62–64 × 44 px | ≈ 4–7 px | ≈ 18 px | 5 |
| 700 px | ≈ 130 × 44 px | ≈ 4–5 px | ≈ 17–18 px | 5 |

- Buttons are rounded rectangles, **corner radius ≈ 6 px** (corner-scan on
  `reference-advanced-light-700.png`).
- **`=` is double-height** (~44×2 + gap ≈ 93 px) in the rightmost column,
  white "=" glyph; accent-colored (pink) in both themes.
- Keypad starts at y≈399 (700-px Advanced; above it sit the history and the
  keypad-visibility toggle row). Page dots (carousel) sit bottom-center below
  the grid. **[Observed]** (E030, E079, edge scans)

### 2.5 Conversion layout
Category dropdown top-left (y≈80); value rows: large bold numbers (≈ 28–32 px
text) with unit dropdowns beneath (y≈209 / y≈317 at 1×); keypad per §2.4
narrow. **[Observed]** (E047–E049)

## 3. Color specification

Sampled (sRGB hex) from the investigation's captures. Two themes; both
re-themed **live** when the interface color-scheme changes (no restart).
**[Observed]** (E079)

| Token | Dark | Light |
|---|---|---|
| Window body background | `#3d3d41` | `#f2f2f2` |
| Header strip background | `#444448`* | `#ffffff` |
| History row background | dark body tone | `#ffffff` |
| Entry background | `#222226` | `#ffffff` |
| Standard keypad button | `#38383b` | `#e6e6e7` |
| Digit-class keypad button (valid digits / darker class) | `#4e4e51` | `#d2d2d4` |
| Dimmed (invalid) keypad button text | gray on standard | `#929295` |
| Accent (=? pink; selection; focus rings; version chip) | `#d56199` | `#d56199` (=), `#f3d0e1` selection bg / `#644155` selection text, ring ≈ `#d199b6`–`#c78fac` |
| Error text | salmon family | `#c30000` |
| Error token highlight | dark-red box under salmon text | `#f6bbbe` |
| Hover on keypad button | lighter than rest state | `#c8c8ca` |
| Press on keypad button | darker still | `#b4b4b7` |

\* dark header sampled on the mode-button area; treat as the header chrome
tone. **[Observed]**

Dark-theme accent is the same pink family (e.g. `=` button `#d56199`); the
light-theme selection/hover values above come from
`state-entry-selection-light.png` / hover-press probes (E079).

Text: primary text near-white on dark / near-black on light; results in
history **bold**; disabled (redo, dimmed digits) reduced contrast.
**[Observed]**

## 4. Interactive states

- **Keypad button**: rest → hover (lighter/darker per theme table) → pressed
  (darkest) → focus ring (pink accent outline) when keyboard-focused.
  **[Observed]** (E079 hover/press samples)
- **Menu rows**: hover = rounded gray highlight (both themes).
  **[Observed]** (mode menu light capture)
- **Toggles (preferences)**: off = gray knob left; on = accent knob right
  (observed during formatting battery when toggled on). **[Observed]** (E069)
- **Stepper (decimals)**: − / + circular buttons; focused row outlined with
  the accent ring. **[Observed]** (light prefs capture)
- **Search fields**: auto-focused with accent-pink focus ring; clear (⊗)
  button appears when non-empty. **[Observed]** (E050, E063)
- **Selection in entry**: accent-tinted background with contrasting text
  (light: `#f3d0e1`/`#644155`). **[Observed]**
- **Tooltips**: dark pill with light text (light theme) after ~1–1.5 s hover.
  **[Observed]** (E023, E079)

## 5. Typography

- History equation column: regular weight; "=" separator: muted; result
  column: bold. Entry/result echo: bold, larger than history rows.
  Conversion values: large bold numerals. Headers/section labels: bold gray
  (menus, preferences). Accelerators: muted, right-aligned.
- Scientific notation uses **real superscript exponent glyphs** in results
  and history (e.g. ×10¹⁹); equation echoes keep typed syntax (^) or button
  pretty-forms (5³). **[Observed]** (E016, E017, E031)

## 6. Theming rules (toolkit-independent statement of the observed system)

1. Two palettes; every surface has a dark and light value; the accent pink
   and accent-tinted selection/focus/error-highlight tokens are shared in
   role across themes (exact hex differs per theme where sampled).
2. Elevation/layering: header and history rows sit on distinct backgrounds
   from the body (light: white on gray; dark: tone-on-tone darker entry).
3. State colors derive from the accent: selection, focus rings, toggles-on,
   version chip, = button. Error colors are a red family distinct from the
   accent.
4. Live re-theme on color-scheme change, both directions, no restart.
   **[Observed]** (E079)
