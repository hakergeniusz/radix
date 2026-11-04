# INDEX — GNOME Calculator Black-Box Behavioral Specification

Clean-room handoff package. Produced by GUI-only black-box investigation of
the installed application (version 51.1) per the project brief: no source
code, packages, binaries, or source-derived documentation were consulted.
Version and environment facts: doc 01.

## Reading order

**Start here**
- **[00 — Overview & Scope](00-overview-scope.md)** — product model, deliverable map, statement-labeling convention (Observed / Inferred / Unknown).

**Implementation-facing specification**
- **[02 — UI Inventory](02-ui-inventory.md)** — window, header, all six modes, every menu/dialog/keypad/popover, tooltip table.
- **[03 — Expression Grammar](03-expression-grammar.md)** — tokens, echo normalization, precedence surface, percent grammar, functions/variables syntax, evaluation triggers.
- **[04 — Arithmetic](04-arithmetic.md)** — numeric domain, verified vectors, rounding family, trig/angle units, logs/powers, bitwise + word size + bases, financial formulas, conversion arithmetic.
- **[05 — Functions, Constants & Variables](05-functions-constants.md)** — full f(x) inventory, typed-name resolution, constants (math + physics), `_`, user variables, rand.
- **[06 — Unit & Currency Conversion](06-unit-currency-conversion.md)** — role model, swap semantics, categories/units, temperature affine rules, currency rates + ⓘ dialog + expressions, error/reset behavior.
- **[07 — Number Formatting](07-number-formatting.md)** — four result formats, decimals/trailing/grouping prefs, live re-render vs stored echoes, base re-rendering.
- **[08 — Input & Interaction](08-input-and-interaction.md)** — complete shortcut listings, typing/recall rules, clipboard, character undo/redo, completion, mouse semantics, popover grabs.
- **[09 — History, Persistence & Settings](09-history-persistence-settings.md)** — history model, full settings inventory with observed effects, graceful-vs-hard persistence contract.
- **[11 — Visual Specification](11-visual-spec.md)** — measured geometry (window, header anchors, keypad grids), sampled two-theme color tokens, interactive states; reference screenshots in `assets/`.

**Verification aids**
- **[12 — Test Catalog](12-test-catalog.md)** — 100 prioritized behavioral tests (P0/P1/P2) with expected outputs and evidence citations.
- **[10 — Error Handling](10-error-handling.md)** — error presentation surfaces, observed error cases with verbatim messages, side-effect rules.

**Process / internal records** (not part of the behavior contract)
- **[01 — Environment & Methodology](01-environment-methodology.md)** — test environment, incidents/fixes, method, evidence discipline, access policy & analyst disclosure, limitations.
- **[13 — Traceability](13-traceability.md)** — claim → evidence map; asset manifest.
- **[14 — Unresolved Questions](14-unresolved-questions.md)** — 43 open items with triage.
- **[15 — Coverage Report](15-coverage-report.md)** — tested / partial / untested / environment limitations / confidence statement.

## Package conventions

- Every behavioral claim in docs 02–09/11 is labeled **[Observed]**,
  **[Inferred]**, or **[Unknown]** (definitions in doc 00 §3).
- Docs 00, 02–09 are path-free (no internal file references); docs 01, 10–15
  carry evidence citations into the internal log.
- Reference images: `docs/assets/` (21 curated screenshots, dark + light).
- Test vectors: doc 12; harness recipe: doc 01 §1.
| 16 — UI Corrections from Owner-Provided Evidence (2026-11-04) | docs/16-ui-corrections.md | Supersedes the inaccurate parts of docs/02/§1,§7–9; evidence E085–E095. |
