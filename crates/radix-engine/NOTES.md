# radix-engine — implementation notes

Clean-room implementation of the Radix expression engine from the approved
spec (`docs/00`–`docs/15`). No GNOME Calculator source, binaries, or
implementation material was consulted; the only behavioural sources are the
spec documents and the vectors of `docs/12` (T001–T055).

## Architecture

```
src/
  error.rs     ErrorKind, EvalError (kind + byte span), prose Display impls
  num.rs       Value (opaque): Exact(BigRational) | Approx(BigFloatNumber)
               | Complex(Box<(Value, Value)>); exact ↔ float conversions,
               raw-parts decomposition, lossless serialization
  ops.rs       arithmetic with exactness promotion; real + complex
               transcendental primitives (astro-float); rounding family;
               sqrt with exact perfect-square fast path
  parser.rs    tokenizer (per-base digits, hex-letter priority, superscript/
               subscript runs, E-notation rejection) + tiered recursive-
               descent parser producing spanned AST nodes
  eval.rs      Env (AngleUnit/WordSize/Base/variables/last_result),
               evaluate(), parse_value_text(), AST evaluator incl. the
               relative-percent rule and assignment
  functions.rs built-in dispatch, physical constants (exact decimals),
               Lanczos gamma, statistics, word-size bitwise ops, f(x)/x
               picker catalogs
  format.rs    FormatPrefs + the four result formats (Automatic/Fixed/
               Scientific/Engineering), base rendering, complex rendering
  echo.rs      normalize_echo (glyph substitution + optional literal grouping)
```

Evaluation pipeline: `evaluate` → trim / strip one trailing `=` → tokenize
(in `env.base`) → parse (`name = value` assignment detected at top level) →
eval AST → store `env.last_result` on success.

## Numeric model

* **Exact rationals** (`num-rational` over `num-bigint`) for `+ − × ÷(≠0)
  %`, integer-exponent `^`, integer `!`. `1÷3×3 = 1` exactly (T022).
* **High precision**: `astro-float` 0.3 `BigFloatNumber`, 200-bit working
  precision (+32 guard bits for transcendentals, rounded back down). Every
  astro error (overflow etc.) surfaces as `ErrorKind::OutOfDomain`; there
  are no NaN/Inf states in `Value`.
* **Complex** = (re, im) pair of `Value`s (parts may be exact;
  `sqrt(-1)`'s imaginary part is exactly 1). Zero imaginary demotes to a
  real. Implemented: +−×÷, ^, sqrt, exp, ln, log/log2, sin, cos, tan, sinh,
  cosh, tanh, conj, arg, re, im, abs. asin/acos/atan of complex arguments
  → `OutOfDomain` (doc 14 #30 left open; correctness over exotica).
* **Display rounding is exact**: the formatter converts values to their
  exact rational (floats are exact dyadic rationals m·2ⁿ) and rounds with
  BigInt arithmetic, half away from zero, at the prefs' decimals count —
  so all 9-decimal vectors match exactly. astro-float is never asked to
  round for display.
* **rand**: fresh exact rational k/2⁶⁴ ∈ [0,1) per evaluation (≥ 19
  significant digits; a draw with < 30 bits set is redrawn).
* **Gamma** for non-integer factorial: Lanczos approximation (g = 7, n = 9
  public-domain coefficient set), reflection for z < ½, computed as
  √(2π)·t^(z−0.5)·e^(−t)·A(z) on the big float. Negative non-integer
  factorials are defined (Γ); negative *integer* factorials are poles
  (`OutOfDomain`). Exact integer factorials use BigInt products (171! fine).
* **tan poles**: exact rational angles get the exact test (Degrees: arg/90
  an odd integer; Gradians: arg/100; the argument node's span is reported);
  approx angles (incl. computed π/2) get a numeric check (|cos| < 1e-18).
  Exact near-pole angles are *not* flagged and keep their huge values.

## Chosen crates

`num-bigint 0.4`, `num-rational 0.4`, `num-traits 0.2`, `astro-float 0.3`,
`rand 0.9` — all pure Rust, MIT/Apache-2.0, as pre-approved. astro-float
provided everything needed (sqrt, exp, ln, trig, inverse trig, hyperbolics,
pow, π/e constant cache, raw mantissa/exponent access); **no swap to
dashu-float was necessary**. Notes on astro-float 0.3.2 quirks that shaped
the code:

* The ergonomic `BigFloat` wrapper (`src/ext.rs`) is compiled out of the
  published crate; only `BigFloatNumber` is public. All ops take an
  explicit precision + `RoundingMode` and return `Result` (no NaN/Inf
  values — errors are `Err`).
* `Clone` is fallible (allocation only); `Value`'s `Clone` unwraps it
  (OOM aborts the process like any other allocation failure).
* Mantissa words are **little-endian** and represent a **fraction in
  [0.5, 1)**: value = m·2^(e − 64·words). `num::bf_parts` normalizes to
  |v| = m·2^e for exact BigInt math.

## Boundary decisions implemented (unobserved in the investigation)

Mirroring the brief:

1. **Automatic threshold**: |x| ≥ 10¹² → scientific; 0 < |x| < 10⁻⁶ →
   scientific (pre-rounding magnitudes).
2. **Scientific exemption**: integers with |x| < 10 render plain
   (9, 6 plain; 99 → `9.9×10¹`).
3. **Fixed ellipsis**: integer parts longer than 13 digits truncate to 13
   digits + `…` (`1844674407370…`), never scientific, fraction dropped.
4. **Decimals pref** range 0..=15 accepted; caps plain fractional digits
   and scientific/engineering mantissa digits (significance destruction at
   low settings is expected, T053).
5. **Engineering**: magnitudes in [0.001, 1000) plain; exponent multiple of
   3 (`1.23×10⁻⁵ → 12.3×10⁻⁶`; bswap → `18.37468648×10¹⁸`).
6. **Complex rendering**: `i`, `−i`, `bi`, `a+bi`, `a−bi` forms; parts
   formatted with the current prefs.
7. **Precedence** (tightest → loosest): `!`/superscripts, `^` (right-assoc,
   `10%^2 = (10²)% = 1` — the `%^` interleave is handled in the parser),
   postfix `%`, prefix `− ¬ √ twos bswap`, `× ÷ mod`, `+ −`, `<< >>`,
   `∧ ∨ ⊻` (bitwise placement unobserved; chosen loosest, C-like shifts
   above additive — revisit if vectors appear).
8. **Unary minus binds looser than postfix**: `-4! = −24`; `2^-3` accepted
   in the exponent position.
9. **Word-size bitwise domain**: operands masked into [0, 2^W) (two's
   complement for negatives), results masked (`5<<63` wraps);
   `¬5 = 2⁶⁴−1−5`, `twos 5 = 2⁶⁴−5`, `bswap 255 = 0xFF…00`; shifts ≥ word
   width yield 0; fractional/complex operands → `InvalidBitwiseOperand`.
   Plain arithmetic stays unbounded (8-bit `255+1 = 256`).
10. **`modexp(x;y;p)`** via BigInt modpow; y ≥ 0 and p > 0 integers
    required, else `InvalidBitwiseOperand`.
11. **Hex entry**: a letter-run token parses as a hex number only when the
    *entire* maximal alphanumeric run is hex digits (`abc` → 0xABC, `ceil`
    stays a function); digits followed by letters split (`2pi`, `65bswap`
    → number + identifier → adjacent-token parse error, span at the
    identifier). In Hex, hex digits win over the `e`/`c`… constant names.
12. **Case handling**: functions and `e`/`pi`/`tau` match
    case-insensitively; the ASCII physics glyphs (`h k R G g c q NA`) match
    case-**sensitively** (G ≠ g) and exist so picker-inserted glyphs parse;
    `phi` errors (T086); variables stored lowercase.
13. **Constants-first name resolution**: built-in constants shadow user
    variables of the same name; assignments to reserved names are
    ParseErrors.
14. **Assignment**: `name = value` (identifier LHS only) evaluates, stores
    (lowercased) and yields the value; `evaluate` strips one trailing `=`
    evaluation marker; on success `env.last_result` is set (the app may
    override).
15. **`0^0 = 1`** (calculator convention); `0^negative` → DivisionByZero.
16. **Superscript runs** are postfix powers with the run as exponent
    (`5³`, `4⁻¹`, `(2+3)²`); subscript runs outside glyphs are ParseErrors
    (`5₃`); glyph floor/ceiling brackets ⌊⌋⌈⌉ always ParseError (doc 14 #7);
    E-notation (`1E5`, `1.5e3`, `10e+5`) is ParseError with the literal's
    span; no implicit multiplication (`2pi`, `2(3)` → ParseError, doc 14 #9).
17. **Nesting depth** of brackets/calls is capped (64) → ParseError instead
    of a stack overflow on adversarial input.
18. **Negative zero** never renders (`−0` normalizes to `0`).
19. **rand** is exact (k/2⁶⁴) — a harmless superset of the observed double
    behaviour.

## Deviations / notes for the integrator

* Public API matches the brief's frozen signatures exactly, plus small
  additive helpers: `Value::is_exact()`, `Value::from_i64()`,
  `Value::real_and_imaginary()`, `WordSize::bits()`, `Env: Default`,
  `ErrorKind/EvalError: Eq`, `EvalError: Display + Error`,
  `ErrorKind` prose strings matching the two verbatim doc-10 messages.
* `evaluate` sets `env.last_result` on success (including for `x=…`
  assignments); the app can override after the call.
* `sinh(1)` renders `1.175201194` at 9 decimals (mathematically correct
  half-away rounding); doc 04 records the truncated capture
  `1.17520119`. Same class: `sin(200 grad)` is `4.4×10⁻⁷¹` (≈0 at 200-bit
  precision), not exactly `0` — the double-precision app would have shown
  `1.2×10⁻¹⁶`; both are "sin ≈ 0".
* `parse_value_text` accepts optional sign + whitespace + one literal per
  base; `-e.5` → `None` (conversion "Error" state, T061 family).
* Astro-float exponent overflow is mapped to `OutOfDomain`; astronomically
  large exact powers/exponents are capped (`^` exponent ≤ 10⁶ and result
  ≤ 8 Mbit; `!` ≤ 50000) — beyond that `OutOfDomain`.
* Currency / unit conversion (`10 DKK in USD`) is intentionally NOT
  handled here — such input tokenizes to an adjacent-token ParseError; the
  app must intercept before calling `evaluate`.

## Test coverage

`cargo test -p radix-engine` — 63 tests, all passing:
* engine-applicable vectors T001–T055 (T036/T046/T054 are UI actions; their
  numeric halves are covered via `render_in_base`/`format_value` purity),
* doc 04 edge battery, rounding family, percent battery,
* doc 10 error cases with exact byte spans (`1/0` → span of `0`, `tan(90)`
  → span of `90`, `65bswap` → span of `bswap`),
* formatting battery T048–T055 with explicit `FormatPrefs`,
* complex battery (`i² = −1`, `sqrt(-4) = 2i`, `exp(iπ) ≈ −1`,
  `(-8)^(1/3)` principal value),
* serialization round-trips for every value shape,
* catalogs (all picker entries + φ typed_name None),
* adversarial input (deep nesting → error, not crash).

`cargo clippy -p radix-engine --all-targets` — clean (0 warnings).
