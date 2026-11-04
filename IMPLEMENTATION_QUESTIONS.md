# IMPLEMENTATION_QUESTIONS — Radix

Ambiguities, gaps, and risks found while reading the approved specification,
each with the provisional resolution being used (see
`IMPLEMENTATION_DECISIONS.md`). Items marked RESOLVED-BY-DECISION are pinned
locally; items marked OPEN need specification owner input or a future
spec-side experiment (the investigation itself is closed).

None of the specification material showed signs of contamination; the docs
are internally consistent, and every [Unknown] is triaged in docs/14.

| # | Question | Where | Status / provisional resolution |
|---|---|---|---|
| 1 | Automatic-format threshold, Scientific exemption bound, Fixed ellipsis rule, decimals range | docs/07 §10, docs/14 #1–4 | RESOLVED-BY-DECISION D4 (pinned to fit all vectors) |
| 2 | Sundry f(x) category contents; Electromagnetism constants | docs/14 #5–6 | RESOLVED-BY-DECISION D13 (standard sets) |
| 3 | ⌊⌋/⌈⌉ glyph syntax; subscript purpose; implicit multiplication | docs/14 #7–9 | RESOLVED-BY-DECISION D4 (parse errors — only errors observed) |
| 4 | Case-insensitivity, whitespace grammar | docs/14 #10–11 | RESOLVED-BY-DECISION D4 (case-insensitive; lenient whitespace) |
| 5 | Bit panel interactivity | docs/14 #12 | RESOLVED-BY-DECISION D4 (display-only + tooltips) |
| 6 | Currency source label vs honest provider naming | docs/06 §6 | INTENTIONAL DEVIATION, D5 (provider named truthfully) |
| 7 | 10 of 14 unit categories unenumerated; ordering rule | docs/06 §4, docs/14 #18 | RESOLVED-BY-DECISION D6 (standard sets, strict large→small) |
| 8 | Financial formula conventions beyond the Fv vector | docs/04 §7, docs/14 #29 | PARTIALLY OPEN — standard formulas implemented (D9); sign conventions for the 9 unverified dialogs may differ from the original |
| 9 | Multi-window state sharing | docs/14 #20 | RESOLVED-BY-DECISION D12 (shared engine/history) |
| 10 | History cap, auto-scroll, Clear-History undoability | docs/14 #21, #25, #43 | RESOLVED-BY-DECISION (no cap; auto-scroll to newest; clear not undoable) |
| 11 | Color-scheme source resolution order | docs/14 #16 | OPEN (ENV) — libadwaita's native resolution used; both palettes + live re-theme implemented per docs/11 |
| 12 | Exchange-refresh feedback (success vs cached) | docs/14 #17 | PARTIALLY OPEN — refresh shows a spinner + updates the date on success, error notice on failure (a refinement of the silent observed behavior; UI needs *some* failure mode) |
| 13 | Popover flip/scroll policy, search-field threshold | docs/14 #19, #41 | RESOLVED-BY-DECISION (search field shown for lists > ~10 items; popovers scroll when needed — a superset of observed behavior) |
| 14 | Custom function builder (arg stepper + ⊕) | docs/14 #38 | OPEN — UI affordance present, flow not exercised upstream; Radix implements define-and-invoke via the variables mechanism if feasible, else ships it disabled; will be documented |
| 15 | WM-dependent behaviors (maximize, drag-resize, modal parenting), IME, a11y tree, localization | docs/14 #35–37 | OPEN (ENV/OOS per brief scope) — standard toolkit behavior, untested against the spec |
| 16 | Command-line interface | docs/14 #34 | OPEN (OOS) — not specified, not implemented |
| 17 | First-launch fresh-profile defaults | docs/14 #33 | RESOLVED-BY-DECISION — empty history, defaults: Basic mode, decimals 9, trailing off, grouping off, Automatic, Degrees, 64-bit (the as-found values are explicitly not guaranteed defaults — docs/09 §3) |
| 18 | Variable name rules / collisions with function names | docs/14 #23 | RESOLVED-BY-DECISION — identifiers [A-Za-z_][A-Za-z0-9_]*; user variables shadow nothing; function names take precedence in call position; redefinition allowed |
| 19 | Whether variable redefinition re-renders history | docs/04 §10, docs/14 #40 | RESOLVED-BY-DECISION — history stores values (D10); redefinition does not recompute old rows |
| 20 | Persistence of variables/undo across restart | docs/14 #22 | RESOLVED-BY-DECISION — variables persist with state (useful superset, unobserved upstream); undo stack does not |
- **Q21 — ⇧⁻¹ toggle semantics (docs/16 §7/D16).** The owner evidence
  (E085–E095) confirms the button's placement, pink styling and toggle look
  but not its effect. Implemented as an inverse-functions toggle (sin ↔
  sin⁻¹/asin family, x² ↔ √, log ↔ 10ˣ, ln ↔ eˣ, log₂ ↔ 2ˣ). If the original
  toggles a different set (e.g. also hyperbolic inverses of log variants),
  only the registered-pair table in `src/keypads.rs` needs updating.
- **Q22 — Currency preference option lists (docs/16 §5/D20).** The
  Preferences dropdowns were captured closed (E093). Chosen lists: refresh
  interval Never/Daily/Weekly/Monthly (default Weekly), display
  Currency Name/Currency Code (default Name), completion All/Favorites
  (default All). Adjust if the original differs.

- **Q23 — Automatic-format swallow mechanism (docs/16 §8/D23).** The rule
  reproducing `92.499999963 → 92.5` while keeping `sqrt(2) → 1.414213562`
  is [Inferred]: only two inputs pin it. Alternatives (10-sig cap, f64
  cleanup) contradict one observation each. More owner inputs with long
  9-runs (e.g. `2−1e-10`, `1/9999999`) would confirm or refine.
- **Q24 — Operator typed right after `=` (docs/16 §8/D22).** E085/E087 show
  the entry empties after evaluation but no evidence covers what the next
  keystroke yields. Implemented: operator continues from the remembered
  result; digit starts fresh. If the original instead treats operators as
  fresh starts (`+2` alone), only this differs.

## Q25 — Currency table contents
The original's Favorite Currencies list starts at "Algerian Dinar" and does
not show AFN/ALL/AOA/AWG/AZN; Radix's table is a superset (all ISO codes it
documents). Should the table be reduced to the original's exact set?
(Owner evidence 2026-11-04 18:56:15, right pane.)

## Q26 — History font metrics
All evidence shows ≈19px history text at both 410px and 750px widths (digit
glyph height 14px). Radix matches this, but the owner's font renders the
17-glyph echo ≈5% wider than Cantarell 19px, so the "3×111+(23÷60)×111"
wrap point at 410px differs by one glyph. Confirm the owner's font
configuration (or accept the residual).
