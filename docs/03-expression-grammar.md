# 03 — Expression Grammar

Implementation-facing. Statements **[Observed]** unless labeled **[Inferred]**
or **[Unknown]**. Test vectors for every claim: doc 12.

---

## 1. Input model

The entry is a **free-form text field**: any character can be typed or pasted;
validation happens at evaluation time (Conversion-mode value rows behave the
same — doc 06 §7). There is no keystroke filtering and no input-mask. Typing
appends at the caret; clicking inside the text places the caret at the
click's x-position (mid-text editing like a normal text field). **[Observed]**

**Fresh-start rule** — typing a digit when the entry currently shows a
*result* (after evaluating, or after Up-arrow recall in Keyboard mode) starts
a new expression, discarding the shown text. Typing after an *error* appends
to the erroneous text instead. **[Observed]**

## 2. Tokens

| Token | Typed as | Echoed as (history row) | Notes |
|---|---|---|---|
| Digits | 0–9 | same | |
| Decimal point | `.` | `.` | |
| Plus / minus | `+` `-` | `+` `−` (true minus glyph) | `-5+3` parses as unary minus |
| Multiply | `*` | `×` | also keypad × |
| Divide | `/` | `÷` | also keypad ÷ |
| Power | `^` | `^` (literal caret) | right-associative |
| Superscript power | ↑n button (or Ctrl+digits) | pretty superscript, e.g. `5³` | second power syntax; keypad ↑n |
| Subscript | ↓n button (or Alt+digits) | pretty subscript, e.g. `5₃` | alone = parse error; purpose unresolved (doc 14) |
| Percent | `%` | `%` | postfix; grammar in §4 |
| Factorial | `!` | `!` | postfix |
| Modulus | `mod ` (infix) | `mod` | also `mod(x;y)` function form |
| Grouping | `( )` | `( )` | also keypad pair button |
| Absolute value | `\|x\|` (bars via button) | `\|…\|` | `\|5\|` = 5 |
| Round brackets | `[x]` (button) | `[…]` | rounds to nearest integer, half away from zero — **not** ceiling/floor |
| Floor/ceiling glyphs | ⌊ ⌋ ⌈ ⌉ via Alt+[ / Ctrl+[ | as typed | closing-glyph syntax unresolved; errors observed (doc 14) |
| Function call | `name(arg;arg…)` | as typed | semicolon separates arguments |
| Constants | glyphs π τ e φ (via x menu / Ctrl+P / Ctrl+T) or typed names `pi` `tau` `e` | glyph or name | typed `phi` does **not** resolve |
| Variables | `name=value` defines; `name` evaluates; `_` = last result | `x=5` echoes with result 5; `_` echoes as `_` | `_×2` echo observed |
| Unit expression | `<amount> <unit-code> in <unit-code>` | as typed | Keyboard mode: `10 DKK in USD` → numeric result |
| `rand` | typed or x menu | as typed | fresh random value each evaluation |

Scientific E-notation literals (`1E5`, `1.5e3`) are **parse errors** —
scientific input is only available through ×10^n constructs (`10^21`,
`e^2`, keypad ↑n / x^y). **[Observed]**

## 3. Operator precedence and associativity

From the percent-resolution battery and associativity vectors:

```
1.  postfix:  !  %  ²  ⁻¹  (button postfixes)
2.  ^  (power)              — right-associative: 2^3^2 = 512
3.  ×  ÷  mod               — left-associative
4.  +  −                    — left-associative
```

Verified vectors: `10-4-3 = 3`, `100/5/2 = 10` (left-assoc); `2^3^2 = 512`
(right-assoc); `10%^2 = 1` (power binds tighter than percent); `100+10%*2 =
100.2` (percent binds tighter than ×). **[Observed]** Unary minus binds
looser than postfix factorial: `-4! = -24`. **[Observed]**

Implicit multiplication (e.g. `2pi`, `2(3)`) was not probed — **[Unknown]**
(doc 14).

## 4. Percent grammar

Model (fits every recorded vector):

- `%` is a postfix operator dividing by 100, **except** when it terminates
  the right operand of `+`/`−`, where it is *relative to the accumulated
  left side*: `a + b% = a + a·(b/100)`.
- Vectors: `50% = 0.5`; `100+10% = 110`; `100*10% = 10`; `200-50% = 100`;
  `10%% = 0.001`; `1+100+10% = 111.1` (relative to 101, the accumulated
  left side — not to 100); `(100+10)% = 1.1`; `50%+50% = 0.75`
  (0.5 + 50%·0.5); `10%+1 = 1.1`. **[Observed]**

## 5. Function call syntax

- `name(arg)` single-argument forms; `name(a;b)` semicolon-separated
  multi-argument forms (`log(8;2) = 3` — second argument is the base).
  **[Observed]**
- Wrong arity is a parse/evaluation error: `round(2.567;2)` errors (round is
  1-argument). **[Observed]**
- Function-picker templates insert `name(` ready for arguments. **[Observed]**
- Prefix function `bswap 255` (space form) works; postfix placement
  (`65bswap`) is malformed. **[Observed]**
- Typed function names are lowercase-resolved (`sqrt(2)`, `log(100)`,
  `sin(30)`, `asin(0.5)`, `round(2.5)`, `floor(-2.5)`, `twos 5`). **[Observed]**

## 6. Variables

- `x=5` defines variable x with value 5 (history row "x=5 = 5"); afterwards
  `x*2` evaluates with it. **[Observed]**
- `_` always holds the last evaluated result (`_*2` after a result of 7 →
  14). **[Observed]**
- Undefined variable names (`y`, `phi`) produce an error (see doc 10). Names
  accepted as variables overlap with function names per observation (`x`
  fine); full name-collision rules **[Unknown]**.
- Variables/constants defined via the x-popover assign row (field + assign
  button) — UI exists; assignment flow not exercised end-to-end. **[Unknown]**
  for its exact behavior (doc 14).

## 7. Evaluation trigger

- **Return/Enter** (and `=` key) evaluates the entry; the keypad `=` button
  does the same. Typing `9=` mid-entry evaluates too (trailing `=`
  convention observed: "9=" produced row "9 = 9"). **[Observed]**
- After evaluation the entry shows the **formatted result string**
  (doc 07 §6). A new history row is appended for every successful
  evaluation; errors add none (doc 10 §3).
- In Programming mode, base-switch shortcuts (Ctrl+B/O/D/H) do not evaluate:
  they re-render existing results in the chosen base. **[Observed]**

## 8. Case and whitespace

- Typed lowercase function/constant names resolve (`pi`, `tau`, `sqrt`);
  uppercase typed names were not probed. **[Unknown]** (doc 14)
- Whitespace tolerance: `10 DKK in USD` parses with single spaces; spaces
  around operators accepted in all typed vectors (e.g. `10 - 4 - 3`);
  exhaustive whitespace grammar **[Unknown]**.

## 9. Editing interactions (grammar-adjacent)

- Ctrl+A select-all, Delete/Backspace, clipboard copy/paste round-trip —
  doc 08 §4.
- Character-level undo/redo of entry edits via Ctrl+Z / Ctrl+Shift+Z /
  header buttons — doc 08 §5.
- Currency-code auto-completion with Tab acceptance — doc 08 §6.
