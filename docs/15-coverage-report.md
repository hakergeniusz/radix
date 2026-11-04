# 15 — Coverage Report

What the investigation covered, partially covered, or could not cover.
Process-facing. Session: 2026-11-03 → 2026-11-04, one app version (51.1),
81 evidence entries (E000–E081), ~200 screenshots.

---

## 1. Tested (behavior verified with vectors + screenshots)

| Area | Depth |
|---|---|
| Operator semantics & precedence (+ − × ÷ ^ % ! mod, unary minus, associativity) | full battery incl. 12 disambiguating percent vectors (E016–E018) |
| Arbitrary precision / overflow / exactness (171!, 2^1024, 1÷3×3) | verified (E019, E026) |
| Rounding family (round/floor/ceil/int/frac/[x], incl. sign and half cases) | verified (E019–E020, E033–E035) |
| Trig + angle units (Deg; inverse; hyperbolic; π semantics; cycle button) | verified in Degrees; Rad/Grad switch verified, Grad vectors not run (E020, E038–E039) |
| Logs/powers (log/ln/log2/logy; e^x; 10^x; button syntaxes) | verified (E017, E019, E040) |
| Bitwise ops + word-size semantics + bases (AND OR XOR NOT shifts bswap twos; 8/64-bit; binary/octal/hex re-render; fractional bases) | verified (E040–E043, E045, E058) |
| Functions inventory (5 of 6 f(x) categories captured; typed-name resolution for 15+ functions) | captured; statistics/complex families listed-not-evaluated (E010–E012) |
| Constants & variables (math constants; physics categories; `_`; user vars; rand) | verified (E021–E022, E026) |
| Six modes: layouts, keypad inventories, tooltips, mode menu, per-mode window widths | full (E002–E003, E030, E040, E046–E047, E058, E065–E066, E080) |
| Conversion: role model, swap, C/⌫, category switch semantics, 3 unit lists, search dropdowns, error/reset rules, temperature affine + below-zero, number-base dimming | full (E047–E061) |
| Currency: live rates, search, ⓘ dialog, refresh action, Keyboard-mode conversion expressions, completion popover | verified (E062–E064, E072–E073) |
| Financial: 10 formula dialogs' field inventories + Fv numeric vector | dialogs verified; 9 of 10 formulas unverified numerically (E046) |
| Formatting: 4 formats, decimals/trailing/grouping interactions, live re-render vs stored echo, base re-render | full battery (E067–E069, E042) |
| Errors: presentation surfaces, 10 case studies, side effects (no row; append-after-error; live recovery; reset rule) | full (E017, E019–E020, E026, E031, E041, E045, E053, E061) |
| Input: shortcuts (dialog + exercised subset), typing rules, recall, clipboard, character undo/redo, completion, clear-history, focus/steal rules, popover grabs, tooltips, hover/press states | full (E013, E023, E027–E028, E031, E042, E045, E056, E060, E065, E072, E074–E076) |
| History: model, display-only behavior, wrap, cross-mode sharing, clear, panel button | full (E002, E065, E070, E075) |
| Persistence: graceful close (×3 data points incl. mode/category/units/width/scheme/history) vs kill −9 | verified (E025, E077–E078, E081) |
| Visual: two full palettes (sampled), live re-theme both directions, keypad geometry (measured), header anchors per width, selection/hover/press/focus/error styles, About/prefs/menus both themes | full (E079 + measurement sessions) |

## 2. Partially tested (observed but with gaps)

| Area | Gap |
|---|---|
| Formatting boundaries | Automatic threshold, Scientific exemption bound, Fixed ellipsis, decimals range (doc 14 #1–4) |
| Financial formulas | only Fv verified; other 9 dialogs unverified numerically (doc 14 #29) |
| Statistics/complex/functions | listed but never evaluated (doc 05 §1) |
| Shortcuts | dialog content complete; ~12 of ~35 not individually exercised (doc 14 #26–27) |
| Angle units | Rad switch verified; Grad vectors missing (doc 14 #31) |
| Unit lists | Length (18), Temperature (4), Number Base, Currency enumerated; 10 other categories not (doc 14 #18) |
| Preferences | Show-Currencies options clipped; refresh-interval options not enumerated (doc 14 #15) |
| History | no cap probing; auto-scroll inferred (doc 14 #25, #43) |

## 3. Untested / out of reach

| Area | Reason |
|---|---|
| Fresh-profile first launch | would require wiping app state (destructive to test env state) — doc 14 #33 |
| Command-line interface | out of brief — doc 14 #34 |
| Localization, RTL | single-locale session — #35 |
| Accessibility tree | not probed — #36 |
| WM behaviors (maximize, resize, modal parenting) | no window manager — #37 |
| IME/IBus text input | deliberately bypassed (XIM) after the keystroke-swallowing incident (E024) |
| Real network refresh result (new rates) | refresh showed no observable change during session — #17 |
| Multi-window | not probed — #20 |
| Custom function builder | not probed — #38 |

## 4. Environment-caused limitations (Xvfb / harness)

1. **No window manager** — maximize no-op, no drag-resize, no alt-tab, no
   real focus-follows; CSD shadows present. Window-level behaviors beyond
   move-free single-window use are unverified (E068).
2. **Portal fidelity** — theme source resolution order ambiguous (E079);
   portals present but the real desktop's full stack was not replicated.
3. **Synthetic input caveats** — three input-path incidents (stuck modifier,
   IBus swallowing, popover grabs) required workarounds; remaining flake:
   occasional first-keystroke loss after dialog/popover close (E056 pattern).
   Mitigated by the select-all-then-type pattern; residual risk: a
   mis-keyed vector could mislabel an output — all critical vectors were
   re-run or re-verified via multiple routes.
4. **Popover clipping** — one preferences dropdown extended past the screen
   edge without scrolling (E068); its hidden options are unknown.
5. **Single display scale** (1×) — high-DPI rendering not assessed.
6. **Screenshot-only capture** — no video; micro-timing behaviors
   (animation durations) not measured.

## 5. Confidence statement

- **High confidence** (multi-vector, re-verified, screenshot-backed):
  arithmetic/percent models, rounding family, bitwise/word-size model,
  conversion role/swap/dimming models, formatting live-vs-stored split,
  persistence contrast, error presentation model, keypad geometry.
- **Medium confidence** (single-vector or pattern-inferred): financial Fv
  convention, Financial/Programming width forcing, variables-not-recomputed
  inference, unit-list ordering pattern.
- **Open** (doc 14): 43 enumerated questions; none block the core behavioral
  contract of docs 02–09 — they refine edges.

## 6. Reproducibility

All vectors are re-runnable per doc 12 against any implementation of the
same contract; harness recipe in doc 01 §1 (Xvfb + private D-Bus + XTEST +
screenshot pipeline; app launched detached with the XIM input-method
override). Currency-dependent vectors must assert format shape unless rates
are pinned.
