# 06 — Unit & Currency Conversion

Implementation-facing. Statements **[Observed]** unless labeled otherwise.
Conversion mode is entered via the mode menu (Ctrl+Alt+C) or the Financial
mode's "Currency Conversion" button.

---

## 1. Layout model

A narrow (~370 px) window with:

1. **Category dropdown** at top (e.g. "Length ⌄").
2. **Two stacked rows**, each a large editable value with a unit dropdown
   beneath it.
3. **Digit-only keypad** (doc 02 §8.5): C, ⇅ (swap), ⌫, digits, decimal
   point, and a category-dependent slot (empty / +/− / ⓘ). No operators, no
   `=` — conversion is **live**: the counterpart updates on every keystroke.
   **[Observed]**

No history list in this mode; the hamburger menu drops its "Clear History"
item here. **[Observed]**

## 2. Roles: source and target

- Both value rows are editable. The **last-edited row is the source**; the
  other row is the live target. **Row positions never move** — only the roles
  do. **[Observed]**
- Changing either row's **unit** makes that row the target (the other row
  keeps its value and becomes the source). **[Observed]**
- Keypad digits insert into the caret's field without clicking the field
  first; clicking into a value places the caret at the click position.
  **[Observed]**
- **C** zeroes both fields and moves the caret to the bottom field (bottom
  becomes source). **[Observed]**
- **⇅ swap** exchanges the two unit assignments and **preserves the top
  field's number**, re-interpreting that number in its new unit; the bottom is
  recomputed. Consequence: the physical quantity described changes (observed
  twice, including when the top number was itself a previously converted
  value). **[Observed]**
- **⌫** deletes one character per click from the caret field; recovery from
  the error state is live (§7). **[Observed]**

## 3. Categories (14)

Angle · Area · Currency · Digital Storage · Duration · Energy · Frequency ·
Length · Mass · Number Base · Pressure · Speed · Temperature · Volume.
Current category is check-marked in the dropdown. **[Observed]**

**Category switch semantics**: values are **not** reset — the target row's
number is carried into both rows (the source row's number is lost); units
reset to the category defaults. Observed: Length→Temperature carried "2.734"
into both Celsius rows. **[Observed]**

## 4. Unit lists

- **Length (18, in dropdown order)**: Parsecs, Light Years, Astronomical
  Units, Rack Units, Nautical Miles, Miles, Kilometers, Cables, Fathoms,
  Meters, Yards, Feet, Inches, Centimeters, Millimeters, Micrometers,
  Nanometers, Desktop Publishing Points. Ordering is mostly largest→smallest
  but not strict (Rack Units after AU; DPP after nm) — ordering rule
  **[Unknown]**. **[Observed]**
- **Temperature (4)**: Celsius, Fahrenheit, Kelvin, Rankine. **[Observed]**
- **Number Base**: unit dropdowns become base selectors Binary / Octal /
  Decimal / Hexadecimal (no search field). **[Observed]**
- **Currency**: full ISO-code list with live rates (§6). **[Observed]**
- Other categories' unit lists: not enumerated. **[Unknown]**
- **Search field in dropdowns**: long lists (Currency) show an auto-focused
  search field (with clear ⊗ once text is entered); short lists (Temperature
  4, Number Base 4) show none. Threshold count **[Unknown]**. Filtering hides
  the checkmark column; unfiltered lists mark the current unit with a
  checkmark. **[Observed]**

## 5. Temperature specifics

Exact affine conversions: 100 C = 212 F; −40 C = −40 F (crossover);
−273.15 C = −459.67 F and = 0 K exactly. **Below-absolute-zero input is
accepted** (−300 C → −26.85 K) with no clamp and no warning. The +/− button
toggles the source's sign in place, retaining the caret. **[Observed]**

## 6. Currency

- Rates are live (network-backed): 1 Algerian Dinar → 0.007379646 US Dollar
  at test time. **[Observed]**
- **ⓘ (Exchange Rate Details)**: in-window dialog, one card per row unit —
  currency-name heading, "Updated <date>", "Source International Monetary
  Fund" with an external-link affordance; "Refresh Rates" button at the
  bottom closes the dialog. After clicking it, the displayed date was
  unchanged and no error/progress feedback appeared (success-vs-cached
  **[Unknown]**). **[Observed]**
- Refresh-interval and currency-display preferences exist in Preferences
  (doc 02 §4); the display setting's effect on conversion UI was not isolated.
  **[Unknown]**
- Favorite-currency toggles (Preferences subpage) — effect on the conversion
  dropdowns not probed. **[Unknown]**

## 7. Input & error behavior

- Value rows are **free-form text** (no keystroke filtering): `-e.5` was
  accepted while typing; `-` renders as a true minus glyph (−). **[Observed]**
- Unparseable source text → **target displays bold "Error"** in the value
  position (no colored styling, no prose message). Recovery is live when the
  text becomes parseable again (⌫ deletes one char per click). **[Observed]**
- Changing a unit while the source text is unparseable **resets both values
  to 0** (an Error target cannot become source). **[Observed]**
- Typed (keyboard) input is not gated by the keypad's digit dimming: typing
  `2` into a binary-base source was accepted and produced an Error target
  until fixed. **[Observed]**

## 8. Number Base category specifics

- Keypad becomes a hex grid (B C D E F / 6 7 8 9 A / 1 2 3 4 5 over the
  standard C ⇅ ⌫ and 0. rows). **[Observed]**
- **Digit dimming follows the TOP row's base**: both rows Binary → only 0/1
  enabled; top Decimal → 2–9 enabled, A–F dim; top Binary + bottom
  Hexadecimal → only 0/1 enabled. Dimming is a **visual affordance only** —
  keyboard input is not gated. **[Observed]**
- Fractional bases supported: −26.85 (decimal) rendered in binary as
  −11010.110110011; 1010.1₂ = 10.5₁₀ exact. **[Observed]**

## 9. Display rules in conversion results

- Target values format with the Number-of-Decimals preference (≤ that many
  fractional digits), trailing zeros trimmed, trailing "." trimmed
  (e.g. "212" not "212.0"); large magnitudes use scientific notation
  (1.687281277×10¹⁶ for 1 parsec in fathoms). **[Observed]**
- Conversion values honor the same formatting prefs as results (doc 07);
  currency results in *Keyboard-mode expressions* are plain numbers, not
  currency-symbol-formatted (`10 DKK in USD = 1.494877167`). **[Observed]**

## 10. Currency expressions outside Conversion mode

In Keyboard mode's free entry, `<amount> <CODE> in <CODE>` evaluates as a
conversion: `10 DKK in USD` → history row "10 DKK in USD = 1.494877167"
(9 decimals under as-found prefs; Automatic format). Unit-code tokens
auto-complete (doc 08 §6). Other unit words (e.g. "10 meters in feet") were
not probed in Keyboard mode. **[Unknown]** for non-currency units (doc 14).
