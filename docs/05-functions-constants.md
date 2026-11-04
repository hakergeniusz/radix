# 05 — Functions, Constants & Variables

Implementation-facing. Statements **[Observed]** unless labeled otherwise.
Function inventory from the f(x) picker submenus; typed-name resolution
verified separately (vectors in doc 12).

---

## 1. Function inventory (f(x) picker)

The picker is organized in six categories. Listed functions were each *seen
as a menu template*; those marked ✓ were additionally evaluated
numerically.

### Trigonometry
sin(x) ✓ · cos(x) ✓ · tan(x) ✓ · sin⁻¹(x) ✓ (typed `asin`) · cos⁻¹(x) ✓
(typed `acos`) · tan⁻¹(x) ✓ (typed `atan`) · sinh(x) ✓ · cosh(x) ✓ · tanh(x)
— (listed; not evaluated) **[Observed list]**

### Complex
conj(x) · arg(x) · re(x) · im(x) — listed; not evaluated (only `sqrt(-1) = i`
demonstrates complex output). **[Observed list]**

### Programming
twos(x) ✓ · bswap(x) ✓ (prefix form `bswap 255`) · mod(x;y) (infix `mod`
verified) · modexp(x;y;p) — listed; not evaluated. **[Observed list]**

### Rounding
round(x) ✓ · floor(x) ✓ · ceil(x) ✓ · int(x) ✓ · frac(x) ✓. **[Observed]**

### Statistics (all listed with semicolon-separated variadic templates)
sum(x₁;x₂;…) · sumsq(x₁;x₂;…) · average(x₁;x₂;…) · median(x₁;x₂;…) ·
min(x₁;x₂;…) · max(x₁;x₂;…) · stdev(x₁;x₂;…) · stdevp(x₁;x₂;…) · var(x₁;x₂;…)
— none evaluated numerically. **[Observed list]**

### Sundry
Category exists; its contents were **not captured**. **[Unknown]** (doc 14)

## 2. Typed-name resolution (verified)

| Typed | Resolves | Result |
|---|---|---|
| `sqrt(2)` | ✓ | 1.414213562 |
| `sqrt(-1)` | ✓ | i |
| `log(100)` | ✓ | 2 |
| `log(8;2)` | ✓ | 3 |
| `sin(30)`, `asin(0.5)`, … | ✓ | per doc 04 §4 |
| `round`, `floor`, `ceil`, `int`, `frac` | ✓ | per doc 04 §3 |
| `twos 5` (prefix) | ✓ | 2⁶⁴−5 |
| `bswap 255` (prefix) | ✓ | 0xFF00000000000000 |
| `12 mod 10` (infix) | ✓ | 2 |

**[Observed]**. Function names are matched lowercase;
mixed-case typed names not probed (**[Unknown]**).

## 3. Constants

Constants live in the x-popover under category headings; clicking one inserts
its **glyph** into the entry (e.g. π); typed names also resolve for e, pi
(π), tau (τ). **[Observed]**

### Mathematics
| Name | Glyph | Typed name | Value |
|---|---|---|---|
| Euler's Number | e | `e` | 2.718281828… (`e^2 = 7.389056099`) |
| Pi | π | `pi` | 3.141592654 (`tau÷2 = 3.141592654`; numeric even in Degrees mode) |
| Tau | τ | `tau` | 6.283185307 |
| Golden Ratio | φ | — | typed `phi` **errors** (no typed name) |

**[Observed]**

### Physics (categories + members; values not individually verified)
- **Atomic & Nuclear**: Planck's Constant (h), Fine-Structure Constant (α),
  Electron Mass (mₑ), Proton Mass (mₚ)
- **Thermodynamics**: Boltzmann (k), Atomic Mass Unit (mᵤ), Molar Gas (R),
  Stefan-Boltzmann (σ), Avogadro (NA)
- **Gravitation**: Gravitational Constant (G), Earth Acceleration (g)
- **Electromagnetism**: category heading observed; member list not captured.
  **[Unknown]**

## 4. Variables

- `_` — last-result variable; its listed value in the x-popover updates after
  each evaluation. `_` echoes as `_` in equation rows. **[Observed]**.
- **User variables**: `name=value` in the entry defines a variable (history
  row "x=5 = 5"); subsequent uses evaluate (`x*2 = 10`). **[Observed]**
- Undefined names error (doc 10 §2). Variable name rules (allowed
  characters, collisions with function names) **[Unknown]**.
- The x-popover's assign row ("Variable name" field + assign button) provides
  UI-driven assignment; end-to-end flow not exercised. **[Unknown]**
- **Persistence of variables across restart**: not probed. **[Unknown]**

## 5. rand

Available from the x-popover and by typed name; produces a fresh value in
[0,1) per evaluation. **[Observed]**

## 6. Custom function builder

The f(x) popover's bottom row exposes "1 [−] [+]" (an argument-count stepper)
and an add (⊕) button — a custom-function construction affordance. Its flow
(naming, body, storage, invocation) was not explored. **[Unknown]** (doc 14)
