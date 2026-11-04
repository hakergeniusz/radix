# 13 — Traceability

Maps specification docs → evidence entries (E###) in `notes/evidence-log.md`
and key screenshots under `notes/shots/`. Process-facing.

---

## 1. Doc → evidence index

| Doc | Primary evidence |
|---|---|
| 00 overview | synthesis; window/mode facts E001–E003, E047, E065, E080 |
| 01 environment | E000–E001, E007, E013, E024, E056, E068, E079 |
| 02 UI inventory | E001–E005, E008–E012, E027–E029, E030, E040, E044, E046, E047, E054, E058, E062–E066, E068, E072, E080, E081 |
| 03 grammar | E015–E018, E021, E026, E031, E033–E035, E042, E045, E053, E072, E073 |
| 04 arithmetic | E016–E020, E026, E030–E033, E038–E039, E040–E043, E045, E046, E055, E057–E058 |
| 05 functions/constants | E010–E012, E017, E019–E022, E026, E041, E045 |
| 06 conversion | E047–E059, E061–E064, E073 |
| 07 formatting | E016–E017, E031–E032, E042, E051, E057, E063, E067–E069 |
| 08 input/interaction | E013, E023, E027–E028, E031, E036–E037, E042, E045, E049, E053, E056, E059–E060, E065, E072, E074–E076, E078 |
| 09 history/persistence/settings | E002, E005, E008, E015, E025, E046, E065, E068–E071, E075, E077–E081 |
| 10 errors | E017, E019–E020, E026, E031, E041, E045, E053, E057, E059, E061 |
| 11 visual | E001–E002, E016–E017, E023, E030, E031–E032, E036–E037, E050, E063, E065, E069, E079 + edge-scan/pixel-probe sessions (2026-11-03/04) + assets in `docs/assets/` |
| 12 test catalog | every vector cites its E-entry inline |
| 14 unresolved | per-entry references inline |
| 15 coverage | session totals E000–E081 |

## 2. Key behavioral rules → their evidence

| Rule (spec doc §) | Evidence chain |
|---|---|
| Percent relative-to-accumulated-left model (03 §4) | E017 vectors + E018 disambiguation battery (`1+100+10%`, `10%^2`, `100+10%*2`) |
| `^` right-assoc (03 §3) | E016 `2^3^2` |
| `[x]` = round-half-away (03 §2, 04 §3) | E033 (ceiling hypothesis) → E034 (`[2.3]=2` refutes ceiling) → E035 (`[-2.5]=-3` confirms half-away) |
| twos(x) vs ¬x differ by 1 (04 §6) | E041 (`¬5` vs `twos 5`) |
| Word size constrains bitwise only (04 §6) | E042 (`255+1=256` at 8-bit) + E043 (`¬0=255`, bit panel 8 bits) |
| π numeric in Degrees (04 §4) | E020 `sin(pi/2)` |
| Conversion roles move, positions don't (06 §2) | E049 (row edits) + E051 (unit change makes target) |
| Swap preserves TOP number (06 §2) | E052 (two cases incl. converted-value case) |
| Digit dimming follows TOP base; typing ungated (06 §8) | E059 (three configurations + typed-`2` probe) |
| Error target reset rule (06 §7) | E061 |
| Results re-render live; echoes stored (07 §5) | E067 (format switch), E069 (grouping on old rows), E042 (base switch) |
| Trailing padding + significance destruction (07 §2–3) | E069 battery |
| Scientific small-integer exemption (07 §1) | E067 (9, 6 plain; 99 scientific) |
| Fresh-start rule (03 §1, 08 §2) | E031 (result) + E065 (recall) + append-after-error E031 |
| One-step Up recall (08 §3) | E065 |
| Popover modal grab (08 §7) | E023 (batch eaten) + E060 (controlled re-test) |
| Character-level undo (08 §5) | E074 |
| Clear-history instant/no-confirm (08 §8) | E075 |
| Clipboard round-trip (08 §4) | E076 |
| Completion Tab/Escape (08 §6) | E072 |
| Graceful persistence vs kill (09 §4) | E077 (pre-close state) → E078 (relaunch compare) + E025 (kill contrast) + E081 (second data point: units/width/scheme) |
| Live color-scheme re-theme (11 §6) | E079 (prefer-light→prefer-dark→default live) |
| Per-mode window widths (11 §1) | E080 (edge-scan measurements, both directions) + E081 (narrow header anchors) |
| Keypad geometry (11 §2.4) | edge scans on `reference-advanced-light-700.png` / `light-basic.png` (2026-11-04) |
| Financial dialog → entry, no row (04 §7, 09 §1) | E046 (Fv worked example) |
| History display-only (08 §3) | E065 |
| Category switch carries target's number (06 §3) | E054 |
| Currency expression in Keyboard mode (06 §10) | E073 |
| Rate dialog refresh behavior (06 §6) | E062 |
| Insert Character Code (02 §10) | E044 |
| Function templates insert `name(` (03 §5) | E012 |
| Constant glyph insertion (05 §3) | E021 |
| Popover popover anchoring varies with list length (05 §3) | E022 |
| Insert-Character-Code / tooltips / hotkeys listing (02) | E044, E023, E036–E040, E027–E028 |

## 3. Asset manifest (docs/assets/)

| File | Content |
|---|---|
| reference-basic-dark-370.png | first-render Basic, dark, as-found history (E002) |
| reference-advanced-dark-700.png | Advanced wide, dark (E030) |
| dialog-mode-menu-dark/light | mode radio menu (E003, E080) |
| dialog-hamburger-dark/light | primary menu incl. Result Format (E004, E066) |
| dialog-preferences-dark/light | prefs incl. Currency section (E005, E068) |
| dialog-about-dark/light | About w/ version chip (E029) |
| dialog-unit-dropdown-search-light.png | unit dropdown + search + checkmark column (E063) |
| state-errors-dark.png | inline error styling, dark (E019) |
| state-error-tangent-dark.png | tan(90) prose error (E020) |
| state-error-division-light.png | 1÷0 error, light colors (E019/E079) |
| state-entry-selection-light.png | selection colors (E079) |
| reference-*-light | per-mode light-theme references (E079 battery) |

## 4. Coverage of the label

Every **[Observed]** claim in docs 02–09/11 traces to at least one E-entry
with a screenshot; **[Inferred]** claims list their supporting observations
in-section; **[Unknown]** items are collected in doc 14 with their
blocking reason.
