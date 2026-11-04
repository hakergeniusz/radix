# 14 — Unresolved Questions Register

Open items, blocking reason, and impact. Process-facing; each item names the
spec sections it touches. Status: **OPEN** (not determinable this session) /
**ENV** (blocked by the Xvfb/no-WM/portal environment) / **OOS** (out of
scope per brief or deliberately not probed to avoid destructive setup).

| # | Question | Touches | Status | Impact if answered differently |
|---|---|---|---|---|
| 1 | Automatic-format switch-over threshold (when plain becomes scientific) | 07 §1 | OPEN | display-only; affects T048 acceptance band |
| 2 | Scientific small-integer exemption bound (9, 6 plain; 99 scientific; 10–98 boundary) | 07 §1 | OPEN | display-only |
| 3 | Fixed-format wide-integer ellipsis rule (position, digit count) | 07 §1 | OPEN | display-only |
| 4 | Number-of-Decimals preference min/max (only 6 and 9 exercised) | 07 §2 | OPEN | formatting edge behavior |
| 5 | Sundry f(x) category contents | 05 §1 | OPEN | function inventory gap |
| 6 | Electromagnetism constants member list | 05 §3 | OPEN | constants inventory gap |
| 7 | ⌊⌋/⌈⌉ floor/ceiling glyph syntax (typed closing forms errored) | 03 §2 | OPEN | grammar for glyph brackets |
| 8 | Purpose/semantics of subscript entry (↓n / Alt+digits) | 03 §2 | OPEN | grammar completeness |
| 9 | Implicit multiplication (`2pi`, `2(3)`) support | 03 §3 | OPEN | grammar acceptance set |
| 10 | Case-insensitivity of typed names (mixed/upper) | 03 §8, 05 §2 | OPEN | parser detail |
| 11 | Whitespace grammar boundaries | 03 §8 | OPEN | parser detail |
| 12 | Bit panel interactivity (cells look interactive; two click attempts didn't toggle) | 02 §8.4 | OPEN | programming-mode UI behavior |
| 13 | Currency Display pref observable effect | 06 §6, 07 §8 | OPEN | settings wiring |
| 14 | Favorite Currencies effect on dropdowns/completion | 06 §6 | OPEN | settings wiring |
| 15 | Show-Currencies options beyond "All" (clipped off-screen; popover wouldn't scroll) | 02 §4 | ENV | settings completeness |
| 16 | Color-scheme source resolution order (dark at launch under 'default'; live 'default' renders light) | 11 §6, 01 §6 | ENV | theme-plumbing parity |
| 17 | Exchange-rate refresh feedback (success vs cached; interval mechanics) | 06 §6 | ENV | rates UX parity |
| 18 | Unit-list ordering rule (mostly large→small w/ exceptions) | 06 §4 | OPEN | list order fidelity |
| 19 | Search-field threshold in unit dropdowns (4 items none; long lists yes) | 06 §4 | OPEN | dropdown UX |
| 20 | Multi-window behavior (New Window/Ctrl+N; single-instance handoff hint in E025) | 08 §9 | OPEN | window model |
| 21 | Clear-History undo-ability | 08 §5 | OPEN | undo scope |
| 22 | Variables/constants persistence across restart | 05 §4 | OPEN | persistence scope |
| 23 | Variable name rules & name collisions | 03 §6 | OPEN | parser detail |
| 24 | Result-Format setting persistence across restart (direct check) | 09 §3 | OPEN | persistence scope |
| 25 | History length cap | 09 §2 | OPEN | memory/UI behavior at scale |
| 26 | Alt+←/→ equation navigation (listed in shortcuts) | 08 §1 | OPEN | shortcut parity |
| 27 | Un-exercised Keyboard-Entry shortcuts (Ctrl+2/R/I/P/T/E/', Alt+digits, Ctrl+digits, Ctrl+−) | 08 §1 | OPEN | shortcut parity |
| 28 | Escape on entry: selection-clear observed; full display-clear behavior | 08 §2 | OPEN | input detail |
| 29 | Financial formulas beyond the single Fv vector (sign/direction conventions) | 04 §7 | OPEN | financial correctness |
| 30 | Complex-arithmetic surface beyond sqrt(-1)=i | 04 §1 | OPEN | numeric domain |
| 31 | Grad-mode trig vectors | 04 §4 | OPEN | angle-unit completeness |
| 32 | Negative zero / infinity / NaN rendering | 04 §1, 07 §10 | OPEN | display of degenerate values |
| 33 | First-launch fresh-profile shape (empty history? default prefs? default mode?) | 09 §6 | OOS | onboarding defaults; would require wiping app state |
| 34 | Command-line interface (e.g. solve flags) | 00 §4 | OOS | out of brief scope |
| 35 | Localization (grouping shape, translated strings, RTL) | 00 §4 | OOS | single-locale session |
| 36 | Accessibility semantics (AT-SPI tree, announcements) | 00 §4 | OOS | not probed |
| 37 | WM-dependent behaviors (maximize, drag-resize, modal parenting) | 01 §6 | ENV | windowing parity |
| 38 | Custom function builder flow (arg stepper + ⊕) | 05 §6 | OPEN | feature surface |
| 39 | rand distribution/period | 04 §9 | OPEN | non-deterministic function detail |
| 40 | Whether variable redefinition re-renders history (formula vs value semantics) | 04 §10 | OPEN | evaluation model |
| 41 | Popover flip/scroll policy (long popover clipped, no scroll observed) | 01 §6, 02 §4 | ENV | popover sizing behavior |
| 42 | Keypad carousel wheel-vs-dots page flipping at wide width (dots hidden?) | 02 §8 | OPEN | carousel behavior detail |
| 43 | History auto-scroll-to-newest rule | 09 §2 | OPEN | list behavior detail |

## Triage summary

- **Display-boundary items (1–4)**: resolvable with a handful of targeted
  vectors (binary search over magnitudes); recommended first follow-up.
- **Inventory gaps (5, 6)**: one popover walk each.
- **Grammar gaps (7–11)**: small typed-probe battery.
- **ENV items (15–17, 37, 41)**: need a WM-equipped or portal-fidelity
  environment; the no-WM Xvfb session cannot decide them.
- **OOS (33–36)**: outside this investigation's constraints by design.
