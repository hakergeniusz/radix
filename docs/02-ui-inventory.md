# 02 — UI Inventory

Implementation-facing. All statements **[Observed]** unless labeled otherwise.
Window/dialog structure and content; visual geometry/colors in doc 11.

---

## 1. Window frame & header strip

Single top-level window titled "Calculator", client-side-decorated
(CSD-style: the header strip is part of the window content; there is no
external title bar). **[Observed]**

Header strip content, left → right:

| Element | Narrow (370) | Wide (700) | Notes |
|---|---|---|---|
| Undo | ↩ icon | ↩ icon | **absent in Conversion mode** |
| Redo | ↪ icon | ↪ icon | absent in Conversion; pale/disabled until an undo exists |
| Mode button | mode name + ⌄ chevron | same | opens mode menu (§2) |
| Hamburger | ☰ | ☰ | opens primary menu (§3) |
| Minimize | − | − | in the 370 px window it sits next to ☰ |
| Maximize | ▢ | ▢ | no-op without a WM |
| Close | × | × | graceful close (persists state) |

**[Observed]** Header anchor positions are **width-relative** — e.g. the mode
button sits at ≈ x 344 in the 700 px window but ≈ x 100–150 in the 370 px
window, and the close button moves from ≈ x 672 to ≈ x 341 (measured; see doc
11 §2). In Financial/Programming/Advanced wide layouts the left header shows
undo/redo as filled/pale circular buttons; in Keyboard mode the same undo/redo
appear as plain arrows. **[Observed]**

## 2. Mode menu (from mode button)

Radio list; current mode shows a filled radio dot; hovering a row highlights
it with a rounded gray background. Items (label, accelerator):

- Basic — Ctrl+Alt+B
- Advanced — Ctrl+Alt+A
- Financial — Ctrl+Alt+F
- Programming — Ctrl+Alt+P
- Keyboard — Ctrl+Alt+K
- Conversion — Ctrl+Alt+C

**[Observed]** Choosing a mode closes the menu, switches the layout, and
retargets the window width (narrow modes: Basic, Conversion; wide: Advanced,
Financial, Programming, Keyboard) — measured in both directions.

## 3. Primary (hamburger) menu

Items, in order (Keyboard/Basic/Advanced/Financial/Programming modes):

1. New Window — Ctrl+N
2. Clear History — Ctrl+Escape *(absent in Conversion mode, which has no
   history)* **[Observed]**
3. *Section "Result Format"* — radio group: Automatic / Fixed / Scientific /
   Engineering
4. Preferences — Ctrl+,
5. Keyboard Shortcuts — Ctrl+?
6. Help — F1
7. About Calculator

**[Observed]** Section headers render as bold gray text; separators between
groups; accelerator labels right-aligned in pale text.

## 4. Preferences dialog

In-window dialog (a sheet over the window, rounded corners, with a close ×),
scrollable. Two sections. **[Observed]**

**General**

| Row | Control | Values seen |
|---|---|---|
| Number of Decimals | − / + stepper with value | 9 as-found; 6 set during tests |
| Trailing Zeroes | on/off toggle | off as-found; on during tests |
| Group Digits | on/off toggle | off as-found; on during tests |
| Angle Units | dropdown | Radians / Degrees / Gradians |
| Word Size | dropdown | 8-bit / 16-bit / 32-bit / 64-bit |

**Currency Conversion**

| Row | Control | Values seen |
|---|---|---|
| Exchange Rate Refresh Interval | dropdown | Weekly (other options not enumerated) |
| Currency Display | dropdown | Currency Code / Currency Name |
| Show Currencies in Auto-Completion List | dropdown | All ✓ (remaining options clipped off-screen in the test environment — unverified) |
| Favorite Currencies | → subpage | |

**Favorite Currencies subpage** — back chevron, title, close ×, "Search
currencies" field, alphabetical currency list with on/off toggles (observed
head: Algerian Dinar DZD, Argentine Peso ARS, Armenian Dram AMD, Australian
Dollar AUD, Bahraini Dinar BHD, Bangladeshi Taka BDT, Botswana Pula BWP,
Brazilian Real BRL, British Pound Sterling GBP; all off as-found). **[Observed]**

Decimals-stepper row shows a focused outline (pink accent in light theme)
when it has keyboard focus. **[Observed]**

## 5. Keyboard Shortcuts dialog

Searchable shortcuts window with pill section headers. Sections and entries
**[Observed]** (accelerators as displayed):

- **General**: New Window Ctrl+N · Close Current Window Ctrl+W · Quit the
  Application Ctrl+Q · Show Help F1 · Open Menu F10 · Show Preferences Ctrl+, ·
  Keyboard Shortcuts Ctrl+? · Clear History Ctrl+Escape
- **Switching Modes**: Ctrl+Alt+B / A / F / P / K / C (per mode)
- **Programming Mode**: Ctrl+B Binary · Ctrl+O Octal · Ctrl+D Decimal · Ctrl+H
  Hexadecimal
- **Keyboard Entry**: * / → ×/÷ · Ctrl+2 Square ² · Ctrl+R Square Root √ ·
  Ctrl+I Inverse ⁻¹ · Ctrl+P Pi π · Ctrl+T Tau τ · Ctrl+' Degree ° · Alt+[ then
  ] Floor ⌊⌋ · Ctrl+[ then ] Ceiling ⌈⌉ · Ctrl+E Scientific Format · Alt+0–9
  Subscript digits · Ctrl+0–9 Superscript digits · Ctrl+− Superscript minus
- **Equation**: Ctrl+C Copy · Ctrl+V Paste · Ctrl+Z Undo · Ctrl+Shift+Z Redo ·
  Alt+←/→ Previous/Next Equation

Verified interactively during the investigation: mode accelerators, Clear
History, Copy/Paste/Undo/Redo, programming base switches (Ctrl+B/O/D/H).
The remaining accelerators are listed here from the dialog itself; their
individual effects were not all separately exercised (see doc 15).

## 6. About dialog

In-window sheet: application icon, "Calculator", "The GNOME Project", version
chip **51.1**, then rows: Website ↗ (tooltip shows the full URL), Report an
Issue ↗, Credits ›, Legal ›. **[Observed]**

## 7. Help window (F1)

Separate window (~957 px wide) titled with the app name + "Help": search
field; sections "User Interface" (Using the History View / Using the Mouse /
Using the Keyboard / Superscript and Subscript) and "Equations" (Basic
Equations / Absolute Values / Bitwise Operations / Factorials / Factorization
/ Percentages / Powers and Roots / Rounding / Logarithms / Modulus Division /
Trigonometry / Variables and Constants). **[Observed]** (Content used only as
hypothesis source — doc 01 §3.)

## 8. Keypads

### 8.1 Basic (narrow)

5-column grid, 5 rows (plus a page-dot indicator when the function rows are
carousel-paged):

```
C    ↑n   ↓n   ÷    mod
7    8    9    ×    %
4    5    6    −    ( )
1    2    3    +    [ = ]   (tall, spans rows 4–5)
0.    x ⌄  f(x) ⌄
```

**[Observed]** At 700 px width the same grid renders with wider buttons;
`=` spans the last two rows in the rightmost column in both widths.

### 8.2 Advanced (wide)

Function rows above the Basic rows (page 2 of the carousel at narrow width;
full grid at ~1000 px width reveals a 10-column layout with the extra
function columns; at 700 px the visible grid is):

```
C    ↑n   ↓n   ÷    mod
7    8    9    ×    %
4    5    6    −    ( )
1    2    3    +    [ = ]
0.    x ⌄  f(x) ⌄
```

Page 2 (carousel):

```
[inverse-toggle pink]  sin  cos  tan  Deg
x²   sinh  cosh  tanh  π
x^y  logy  log   ln    e
conj Arg   Re    Im    i
x⁻¹  [x]   |x|   x!    axb
```

**[Observed]**. The pink first button of page 2 has no
captured tooltip; its effect was not isolated (identity Unknown — doc 14).
"Deg" is the live angle-unit cycle button. The carousel page dots appear
bottom-center; mouse-wheel over the keypad and the dots switch pages at narrow
width; at wide width all columns fit and no paging is needed.

### 8.3 Financial (wide)

Same grid as Advanced, but most function buttons open **formula dialogs**
instead of inserting tokens (doc 02 §9); the numeric core (C ↑n ↓n ÷ mod /
digits / operators / x f(x)) behaves as in Advanced. **[Observed]**

### 8.4 Programming (wide)

Advanced's grid plus (page 1 additions) bitwise operators and base digits:

- Row additions: ∧ ∨(?) ⊻ ¬ << >> log2 swap à(character code) — observed
  buttons with tooltips "Bitwise AND", "Bitwise Exclusive OR", "Bitwise NOT",
  "Left Shift [<<]", "Right Shift [>>]", "Binary Logarithm", "Swap
  Endianness", "Insert Character Code". **[Observed]**
- Hex digit buttons A–F (dimmed when invalid for the current base).
- Below the entry: base readout row and a **bit panel** (interactive-looking
  cells with index tooltips on hover; toggling not demonstrated — doc 14).
- Base dropdown (Binary/Octal/Decimal/Hexadecimal) and word-size dropdown
  (8/16/32/64-bit) are part of the programming UI near the entry/base row.
  **[Observed]**

### 8.5 Conversion (narrow)

Digit-only keypad:

```
C    ⇅    ⌫
7    8    9
4    5    6
1    2    3
0.    [category slot]
```

The bottom-right slot changes with category: empty (most), +/− (Temperature),
ⓘ (Currency). No operators, no `=`. **[Observed]** Number Base category
replaces the numeric grid with a hex grid:

```
C    ⇅    ⌫
B    C    D    E    F
6    7    8    9    A
1    2    3    4    5
0.   (+/− slot)
```

**[Observed]** (exact arrangement per screenshot).

## 9. Formula dialogs (Financial mode)

Each financial button opens a modal form dialog: title, description text,
labeled fields each with a pencil edit icon, first field pre-focused with
"0" selected, Cancel and Calculate buttons. Observed buttons → dialogs:

| Button | Dialog title | Fields |
|---|---|---|
| Ctrm | Compounding Term | Periodic Interest Rate, Future Value, Present Value |
| Ddb | Double-Declining Depreciation | Cost, Life, Period |
| Fv | Future Value | field labels not individually recorded (worked example used numeric inputs −100, 0.05, 10) |
| Gpm | Gross Profit Margin | Cost, Margin |
| Pmt | Periodic Payment | Principal, Periodic Interest Rate, Term |
| Pv | Present Value | Periodic Payment, Periodic Interest Rate, Number of Periods |
| Rate | Periodic Interest Rate | Future Value, Present Value, Term |
| Sln | Straight-Line Depreciation | Cost, Salvage, Life |
| Syd | Sum-of-the-Years'-Digits Depreciation | Cost, Salvage, Life, Period |
| Term | Payment Period | Periodic Payment, Future Value, Periodic Interest Rate |

**[Observed]** Calculate fills the **entry** with the numeric result and
closes the dialog; **no history row is added**. Cancel closes without
effect.

## 10. Other dialogs & floating surfaces

- **Insert Character Code** (Programming, à button): "Character" field +
  "Convert to: Unicode Code Point" dropdown + Cancel/Insert; inserting 'A'
  enters the code point (65) in the current base. **[Observed]**
- **Exchange Rate Details** (Conversion, ⓘ in Currency): one card per row
  unit — currency-name heading, "Updated <date>", "Source International
  Monetary Fund" with external-link icon; bottom "Refresh Rates" button which
  closes the dialog when clicked. **[Observed]**
- **Unit dropdowns** (Conversion): anchored popover; long lists (Currency)
  include a search field (auto-focused, pink focus ring, clear ⊗ when
  non-empty) — short lists (Temperature: 4 items; Number Base) show no search
  field (size-conditional — threshold Unknown). Current unit marked with a
  checkmark in unfiltered lists; the checkmark column is omitted while
  filtering. **[Observed]**
- **f(x) popover** (Basic/Advanced/Financial): search field ("Function
  name"); categories Trigonometry, Complex, Programming, Rounding, Statistics,
  Sundry; each category a submenu of function templates; clicking a template
  inserts `name(` into the entry. Bottom row: arg-count stepper "1 [−] [+]"
  and an add (⊕) button — a custom-function builder (behavior not explored —
  doc 14). **[Observed]**
- **x / variables popover**: list "_ = <last result>" (live value), "rand",
  Constants categories (Mathematics / Electromagnetism / Atomic & Nuclear /
  Thermodynamics / Gravitation), bottom row "Variable name" field + assign
  button. Clicking a constant inserts its glyph. Popover anchors/grows from
  the button; short lists anchor lower. **[Observed]**
- **Entry auto-completion** (Keyboard/other free entry): typing a currency
  code prefix ("US D") opens an inline popover above the caret listing
  matching codes with a detail line ("Danish Krone - kr"); Tab accepts the
  highlighted suggestion; Escape dismisses leaving typed text. **[Observed]**

## 11. Tooltip inventory (button labels)

Captured tooltips (button = tooltip): C = "Clear Display [Escape]" · ↑n =
"Superscript Mode [Ctrl]" · ÷ = "Divide [/]" · mod = "Modulus Divide" · x² =
"Square [Ctrl+2]" · conj = "Complex Conjugate" · x⁻¹ = "Inverse [Ctrl+I]" ·
axb = "Factorize [Ctrl+F]" · sinh/cosh/tanh = "Hyperbolic Sine/Cosine/
Tangent" · cos = "Cosine" · tan = "Tangent" · ln = "Natural Logarithm" · log =
"Common Logarithm" · logy = "Logarithm" · Arg = "Complex Argument" · |x| =
"Absolute Value [| |]" · x! = "Factorial [!]" · % = "Percentage [%]" · () =
"Group [()]" · Deg = "Angle Unit: Degrees/Radians/Gradians" (live) · ∧ =
"Bitwise AND" · ⊻ = "Bitwise Exclusive OR" · ¬ = "Bitwise NOT" · << = "Left
Shift [<<]" · >> = "Right Shift [>>]" · log2 = "Binary Logarithm" · swap =
"Swap Endianness" · à = "Insert Character Code". **[Observed]**. History rows show a tooltip with the full equation on
hover. **[Observed]**

## 12. Layout regions per mode

| Region | Basic | Advanced | Financial | Programming | Keyboard | Conversion |
|---|---|---|---|---|---|---|
| Header w/ undo/redo | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ (back-arrow ‹ instead) |
| History list | ✓ | ✓ (inset card at ≥~675 px) | ✓ (+ clear-history button bottom-right of panel) | ✓ | ✓ full-height | ✗ |
| Entry line | ✓ | ✓ | ✓ | ✓ (higher, above bit panel) | ✓ bare | two value rows |
| Keypad | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ digit-only |
| Extra | page dots | carousel + dots | formula buttons | base/word dropdowns, bit panel | — | category + unit dropdowns |
