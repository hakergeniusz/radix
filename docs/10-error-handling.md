# 10 — Error Handling Catalog

Statements **[Observed]** unless labeled otherwise; evidence citations
included (E###). Complements docs 03 (grammar), 06 (conversion), 15
(coverage).

---

## 1. Error presentation surfaces

Three distinct user-visible error treatments were observed:

1. **Inline entry error (evaluation)** — the entry text turns salmon/red with
   the offending token highlighted (a selection-like rounded highlight on
   the exact substring); the history list **shrinks** to make room for an
   optional message line under the entry. Light theme: text ≈ `#c30000`,
   token highlight ≈ `#f6bbbe`; dark theme: salmon text with a dark-red box
   behind it for some error kinds. **[Observed]** (E019, E020, E026; assets
   `state-errors-dark.png`, `state-error-division-light.png`,
   `state-error-tangent-dark.png`)
2. **Prose message line** — under the entry, for errors with a human
   explanation (observed for undefined-tangent and malformed expressions).
   **[Observed]** (E020, E041)
3. **Conversion target "Error"** — unparseable source text renders the word
   "Error" bold in the *target value position* (no color styling, no prose);
   recovers live. **[Observed]** (E053, E057)

No error dialogs, no notifications, no crash/blank states observed across
the whole battery. **[Observed]**

## 2. Observed error cases (verbatim where captured)

| Input / situation | Presentation | Message text |
|---|---|---|
| `1/0`, `0/0` | inline; the **divisor token** highlighted; NO history row | "Division by zero is undefined" |
| `tan(90)` (Degrees) | inline; "90" highlighted; history shrinks | "Tangent is undefined for angles that are multiples of π (180°) from π/2 (90°)" |
| `1E5`, `1e5`, `1.5e3` | inline (parse error) | *(no prose captured)* |
| `5₃` (subscript digits alone) | parse error | — |
| `5twos` (postfix prefix-function) | inline + prose | "Malformed expression" |
| `phi` (no typed name) | inline; error region has a **dark-red box** behind salmon text (dark theme) | — |
| `y` (undefined variable) | inline error | — |
| `round(2.567;2)` (wrong arity) | error | — |
| `65bswap` (prefix function postfix) | malformed | — |
| `1010.12` as binary source (typed 2) | conversion target "Error" | — |

**[Observed]** (E017, E019, E020, E026, E031, E041, E045, E053, E059)

## 3. Side effects of errors

- **No history row** is created for a failed evaluation. **[Observed]** (E019)
- **Typing after an error appends** to the erroneous text (fresh-start rule
  does not apply). **[Observed]** (E031)
- Error display is **live/transient**: fixing the input (e.g. one ⌫ in
  conversion) clears the error immediately; in the tangent case the history
  list re-expands. **[Observed]** (E020, E053)
- **Conversion reset rule**: changing a unit while the source text is
  unparseable resets both values to 0 (an Error target cannot become
  source). **[Observed]** (E061)

## 4. Failure-free degradation

- Below-absolute-zero temperatures: accepted silently (no warning) — doc 06 §5.
- Word-size arithmetic overflow (255+1 at 8-bit): **not** an error (bitwise
  domain only) — doc 04 §6.
- Very large results (10³⁰⁹): normal scientific rendering, no warning —
  doc 04 §1.

## 5. Not determined

- Full taxonomy of prose messages (only two captured verbatim).
- Parse-error highlighting rules (which token gets highlighted when several
  are plausible).
- Behavior on unparseable *stored* history (none producible through the UI).
- Any error-state persistence across restart (none expected; untested).
- Double-error input (error while error displayed) edge flows.
