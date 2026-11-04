# 04 — Arithmetic & Evaluation Semantics

Implementation-facing. Statements **[Observed]** unless labeled otherwise;
vectors in doc 12. Precedence/associativity tables live in doc 03 §3–4 and
are not repeated here.

---

## 1. Numeric domain

- **Arbitrary precision, no overflow**: `171! = 1.24101807×10³⁰⁹`, `2^1024 =
  1.797693135×10³⁰⁸`, `170!` fine — no clamping, no saturation, no error.
  **[Observed]**
- **Exact rational behavior where expected**: `1÷3×3 = 1` exactly (no
  floating-point residue). **[Observed]**. Whether *all* arithmetic is
  exact-rational until a transcendent step (vs. high-precision decimal) is
  **[Inferred]** from the observed exactness; not directly distinguished.
- **Complex results are produced, not errors**: `sqrt(-1) = i`. **[Observed]**. Complex arithmetic surface beyond this one vector (e.g. `i²`,
  complex trig) not probed — **[Unknown]**.
- Negative zero, signed infinity, NaN literals — not probed. **[Unknown]**

## 2. Basic operators

| Vector | Result | Note |
|---|---|---|
| `2+3*4` | 14 | precedence |
| `(2+3)*4` | 20 | grouping |
| `10-4-3` | 3 | left-assoc |
| `100/5/2` | 10 | left-assoc |
| `2^3^2` | 512 | right-assoc power |
| `-5+3` | −2 | unary minus |
| `5!` | 120 | factorial |
| `0!` | 1 | |
| `2.5!` | 3.32335097 | gamma extension to non-integers |
| `-4!` | −24 | unary minus looser than `!` |
| `50%` | 0.5 | percent postfix (doc 03 §4) |

All **[Observed]**.

## 3. Rounding family

| Construct | Semantics | Vectors |
|---|---|---|
| `round(x)` | half away from zero | `round(2.5)=3`, `round(-2.5)=-3` |
| `[x]` (button brackets) | same as round: nearest integer, half away from zero | `[2.7]=3`, `[2.3]=2`, `[2.5]=3`, `[-2.5]=-3` |
| `floor(x)` | toward −∞ | `floor(-2.5)=-3`, `floor(pi)=3` |
| `ceil(x)` | toward +∞ | `ceil(-2.5)=-2` |
| `int(x)` | truncate toward 0 | `int(-2.5)=-2` |
| `frac(x)` | fractional part, sign of x | `frac(-2.5)=-0.5` |

All **[Observed]**. `round(x)` is 1-argument only
(`round(2.567;2)` errors).

## 4. Trigonometry & angle units

- Angle-unit setting applies to trig and inverse trig: in Degrees,
  `sin(30)=0.5`, `cos(60)=0.5`, `tan(45)=1`, `sin(90)=1`, `asin(0.5)=30`,
  `acos(0.5)=60`, `atan(1)=45`. **[Observed]**
- The constant π is the *numeric* 3.14159… even in Degrees mode:
  `sin(pi/2) = 0.027412134` (i.e. sin of 1.5708 degrees — no degree/radian
  reinterpretation of π). **[Observed]**
- Hyperbolics independent of angle unit: `sinh(1)=1.17520119`. **[Observed]**
- `tan(90)` in Degrees → error with a specific prose message (doc 10 §2);
  history list *shrinks* while the error is displayed (transient state).
  **[Observed]**
- Angle unit cycle button: Deg → Rad → Grad → Deg, label and tooltip update
  live; cycling affects subsequent evaluations (earlier results unchanged).
  **[Observed]**
- Grad-mode vectors: not probed. **[Unknown]**

## 5. Logarithms & powers

- `log(100) = 2` (common log), `ln` natural log, `log2` binary logarithm,
  `logy(x;base)`: `log(8;2) = 3`. **[Observed]**
- `e^2 = 7.389056099`; `10^21 = 1×10²¹` (auto-scientific display, doc 07 §3).
  **[Observed]**
- Keypad power buttons: x² produces pretty superscript echo (`2²`); x^y
  inserts caret syntax (echoed literally); x⁻¹ postfix (`4⁻¹ = 0.25`); ↑n
  superscript-entry mode (`5³` echo, value 125). **[Observed]**

## 6. Bitwise operations (Programming mode)

| Vector | Result | Note |
|---|---|---|
| `12∧10` | 8 | bitwise AND, glyphs echoed |
| `12∨10` | 14 | bitwise OR |
| `12⊻10` | 6 | bitwise XOR |
| `¬5` | 1.844674407×10¹⁹ | bitwise NOT = 2⁶⁴−1−x (64-bit domain) |
| `twos 5` | 1.844674407×10¹⁹…B | twos complement = 2⁶⁴−x (differs from ¬ by 1) |
| `1<<10` | 1024 | left shift |
| `1024>>3` | 128 | right shift |
| `bswap 255` | 1.837468648×10¹⁹ | byte swap (prefix form) = 0xFF00000000000000 |

All **[Observed]**.

**Word size semantics** — word size (8/16/32/64-bit) constrains the *bitwise
domain* only: with 8-bit word size, `¬0 = 255` and the bit panel shows the
low 8 bits, while plain arithmetic stays unbounded (`255+1 = 256`, no
wraparound). **[Observed]**

**Bases** — base switches (Binary/Octal/Decimal/Hexadecimal; Ctrl+B/O/D/H)
re-render all history results in the new base; entry parses per current base.
Fractional values in non-decimal bases: `-26.85` displayed in binary as
`-11010.110110011` (minus + radix point, 9 fractional digits); `1010.1₂ =
10.5₁₀` exact. **[Observed]**

## 7. Financial mode formulas

Financial buttons open form dialogs (doc 02 §9) that compute and deposit a
numeric result into the entry (no history row). Only one formula family was
verified numerically:

- **Fv (Future Value)** with numeric inputs −100, 0.05, 10 (field labels for
  this dialog were not individually recorded) → entry received
  `-1257.789253555`. That value matches the standard end-of-period
  ordinary-annuity future value with the −100 acting as the *periodic
  payment* (100·((1.05¹⁰−1)/0.05) = 1257.789…), i.e. the first field is a
  payment, not a present value. **[Observed output; formula identification
  is Inferred from the single vector]** — general agreement across
  sign/direction conventions needs more vectors (doc 14).
- Remaining dialogs (Ctrm, Ddb, Gpm, Pmt, Pv, Rate, Sln, Syd, Term): field
  inventories observed; computational semantics **[Unknown]** (not exercised
  numerically).

## 8. Conversion arithmetic

Live conversion of the target row (doc 06); arithmetic is exact decimal with
the display rules of doc 07; temperature conversions are exact affine maps
(100 C = 212 F; −40 C = −40 F; −273.15 C = 0 K exactly). Below-absolute-zero
input is accepted without clamp or warning. **[Observed]**

## 9. Randomness

`rand` yields a fresh pseudo-random value in [0,1) per evaluation (observed
0.801359698; distribution/period not probed). **[Observed]**

## 10. Order of evaluation & re-evaluation

- Evaluation is triggered per Return/`=` (doc 03 §7); results are *values*,
  not formulas: editing a variable later does not recompute earlier history
  rows (rows store rendered text; format/base changes re-render *numbers* —
  doc 07 §5 — but variable redefinition does not). **[Inferred]** from
  stored-echo behavior + variable model; a direct
  redefine-and-inspect probe was not run. **[Unknown]** (doc 14)
