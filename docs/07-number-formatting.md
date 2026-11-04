# 07 — Number Formatting & Display

Implementation-facing. Statements **[Observed]** unless labeled otherwise.
Formatting was exercised through the Result-Format radios (hamburger menu)
and the Preferences rows (Number of Decimals, Trailing Zeroes, Group Digits);
vectors in doc 12.

---

## 1. Result formats (four radio options)

| Format | Rule | Verified vectors |
|---|---|---|
| **Automatic** | plain positional until large magnitude, then scientific with superscript exponent (e.g. 1.844674407×10¹⁹) | 61917364224 stayed plain; 1.844674407×10¹⁹ went scientific; 10^21 → 1×10²¹ |
| **Fixed** | never scientific; wide integers truncate with a trailing ellipsis | 1844674407370… (truncated) |
| **Scientific** | always mantissa×10ⁿ, mantissa digit count = Number of Decimals preference; **small integers exempt** (rendered plain) | 9 and 6 plain; 99 → 9.9×10¹ |
| **Engineering** | exponent a multiple of 3; magnitudes in [0.001, 1000) stay plain | 1.23×10⁻⁵ → 12.3×10⁻⁶; bswap result → 18.37468648×10¹⁸; 333.333333333 and 128 stayed plain |

**[Observed]** — except: the Automatic switch-over threshold, the
Scientific small-integer exemption's upper bound (somewhere in 10–98), and
the exact ellipsis position in Fixed are **[Unknown]** (doc 14).

## 2. Number of Decimals preference

Stepper with − / + (value 9 as-found; 6 exercised). Effects **[Observed]**:

- Caps **fractional digits** in plain results (`333.333333` at 6 decimals).
- Caps **mantissa digits** in scientific/engineering mantissas
  (`3.333333×10²`).
- Truncation at low settings can destroy significance entirely
  (`0.0000123 → 0.000012` at 6 decimals). The preference's
  minimum/maximum range is **[Unknown]**.

## 3. Trailing Zeroes preference

When ON, plain and scientific mantissas are **padded to exactly the decimals
count**: `6.000000`, `1.230000×10⁻⁵` (decimals 6). When OFF, trailing zeros
are trimmed (and a trailing "." is dropped: "212" not "212.0" — also observed
in conversion results). **[Observed]**

## 4. Group Digits preference

When ON, **integer-part** separators appear (thousands grouping in en-US:
`1,234,567.000000`). Grouping also applies to the **typed-input echo** of a
history row (`1,234,567` in the equation column) — but only for rows created
*after* the preference changed; earlier rows keep their stored ungrouped
echo (`1000÷3` stayed ungrouped). **[Observed]**

## 5. Live re-render vs stored echo

- **Results re-render live** when the format radios, decimals, grouping, or
  (in Programming mode) the base changes — all history results update at
  once, including conversion values. **[Observed]**
- **Equation echoes are stored text**: switching Result Format or grouping
  does *not* rewrite the equation column (only results). Operator glyphs in
  echoes are normalized at row creation: typed `*` → ×, `/` → ÷, `-` → −;
  `^` stays literal; button-produced superscripts echo pretty (`5³`, `2²`).
  **[Observed]**

## 6. Entry/result echo behavior

- After evaluating, the **entry shows the formatted result string**
  (obeying current decimals/trailing/grouping prefs — e.g. `5.000000|` after
  Trailing ON + decimals 6 + solving `2+3`). **[Observed]**
- Keyboard-mode Up-arrow recall loads the last result *text* into the entry
  (doc 08 §3). **[Observed]**

## 7. Magnitude conventions

- Scientific notation renders with a **superscript exponent** in results and
  history (1.844674407×10¹⁹); the equation column never auto-scientifies. —
  **[Observed]**
- Leading zero present for |x|<1 with no grouping applied to the fractional
  part beyond the decimals cap (`0.007379646`). **[Observed]**

## 8. Currency display

Conversion targets and conversion-expression results are **plain numbers**
(no currency symbol injected). The Currency Display preference
(Currency Code / Currency Name) exists; its observable effect (unit dropdown
labels vs completion detail lines) was not isolated. **[Unknown]**

## 9. Base re-rendering (Programming)

Base changes re-render **results** in the new base across history (Ctrl+H
turned all results hexadecimal); equations are not re-interpreted. Bases
interact with the Conversion mode's Number Base category identically
(observed plain digit strings with radix point; no 0x/0b prefixes seen).
**[Observed]**

## 10. What was not determined

- Automatic-format threshold; Scientific exemption bound; Fixed ellipsis
  rule; decimals preference bounds; negative-zero rendering; infinity/NaN
  rendering (no vector produced them); grouping shape under other locales —
  **[Unknown]** (doc 14).
