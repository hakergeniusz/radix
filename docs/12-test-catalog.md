# 12 — Prioritized Behavioral Test Catalog

Behavioral acceptance tests for a clean-room implementation. Every expected
value below was observed in the investigation (citations E###). Priority:
**P0** = core contract (must match exactly), **P1** = major features,
**P2** = edges/refinements. Tests are implementation-agnostic: given the
inputs, the observable outputs must match.

Setup conventions: en-US locale, defaults as noted; "row" = history row
echo→result.

---

## P0 — Arithmetic & grammar core

| # | Input (entry) | Expected observable | Evid |
|---|---|---|---|
| T001 | `2+3*4` ⏎ | row `2+3×4 = 14` | E016 |
| T002 | `(2+3)*4` ⏎ | 20 | E016 |
| T003 | `10-4-3` ⏎ | 3 (left-assoc) | E016 |
| T004 | `100/5/2` ⏎ | 10 (left-assoc) | E016 |
| T005 | `2^3^2` ⏎ | 512 (right-assoc) | E016 |
| T006 | `-5+3` ⏎ | −2 | E016 |
| T007 | `50%` ⏎ | 0.5 | E017 |
| T008 | `100+10%` ⏎ | 110 | E017 |
| T009 | `100*10%` ⏎ | 10 | E017 |
| T010 | `200-50%` ⏎ | 100 | E017 |
| T011 | `10%%` ⏎ | 0.001 | E017 |
| T012 | `1+100+10%` ⏎ | 111.1 (% of accumulated left side 101) | E018 |
| T013 | `(100+10)%` ⏎ | 1.1 | E018 |
| T014 | `10%^2` ⏎ | 1 (^ binds tighter than %) | E018 |
| T015 | `50%+50%` ⏎ | 0.75 | E018 |
| T016 | `100+10%*2` ⏎ | 100.2 (% binds tighter than ×) | E018 |
| T017 | `5!` ⏎ | 120 | E019 |
| T018 | `0!` ⏎ | 1 | E019 |
| T019 | `2.5!` ⏎ | 3.32335097 (gamma) | E019 |
| T020 | `-4!` ⏎ | −24 | E019 |
| T021 | `171!` ⏎ | 1.24101807×10³⁰⁹ (no overflow) | E019 |
| T022 | `1÷3×3` ⏎ | 1 exactly | E026 |
| T023 | `2^1024` ⏎ | 1.797693135×10³⁰⁸ | E026 |
| T024 | `_*2` (after result 7) ⏎ | 14 | E026 |
| T025 | `x=5` ⏎ then `x*2` ⏎ | rows `x=5 = 5`, `x×2 = 10` | E026 |
| T026 | `1E5` ⏎ | error (E-notation rejected) | E017 |
| T027 | `sqrt(2)` ⏎ | 1.414213562 | E019 |
| T028 | `log(8;2)` ⏎ | 3 | E019 |
| T029 | `round(2.5)` / `round(-2.5)` | 3 / −3 (half away) | E020 |
| T030 | `[2.5]` via bracket button | 3 (round, not ceil/floor) | E034/E035 |
| T031 | `[x]` display | `[2.3]=2`, `[2.7]=3` | E034 |

## P0 — Trigonometry & angle units (Degrees default)

| # | Input | Expected | Evid |
|---|---|---|---|
| T032 | `sin(30)` | 0.5 | E020 |
| T033 | `tan(45)` | 1 | E020 |
| T034 | `sin(pi/2)` (Degrees) | 0.027412134 (π numeric) | E020 |
| T035 | `asin(0.5)` | 30 | E020 |
| T036 | Deg-button cycle | Deg→Rad→Grad→Deg; label+tooltip update | E038/E039 |

## P0 — Programming mode

| # | Input | Expected | Evid |
|---|---|---|---|
| T037 | `12∧10` | 8 | E041 |
| T038 | `12∨10` | 14 | E041 |
| T039 | `12⊻10` | 6 | E041 |
| T040 | `¬5` (64-bit) | 1.844674407×10¹⁹ | E041 |
| T041 | `twos 5` | 0xFFFFFFFFFFFFFFFB (differs from ¬ by 1) | E041 |
| T042 | `1<<10` / `1024>>3` | 1024 / 128 | E041 |
| T043 | `bswap 255` | 0xFF00000000000000 = 1.837468648×10¹⁹ | E045 |
| T044 | 8-bit word: `255+1` | 256 (no arithmetic wrap) | E042 |
| T045 | 8-bit word: `¬0` | 255 | E043 |
| T046 | Ctrl+H after decimal rows | all results re-render hex | E042 |
| T047 | `1010.1` in binary base → decimal | 10.5 | E058 |

## P1 — Formatting

| # | Setup | Action | Expected | Evid |
|---|---|---|---|---|
| T048 | Automatic | solve `10^21` | `1×10²¹` | E017 |
| T049 | Scientific | solve `99` | `9.9×10¹`; `9` and `6` stay plain | E067 |
| T050 | Engineering | value 1.23×10⁻⁵ | `12.3×10⁻⁶`; [0.001,1000) plain | E067 |
| T051 | Fixed | wide integer | truncated with "…" (never scientific) | E067 |
| T052 | decimals 6, Trailing ON, Grouping ON | solve `2+3` | entry echo `5.000000`; row result `5.000000`; grouping `1,234,567.000000` on suitable values | E069 |
| T053 | decimals 6 | `0.0000123` | `0.000012` (significance destruction) | E069 |
| T054 | change Result Format | observe history | all *results* re-render; equation echoes unchanged | E067 |
| T055 | grouping ON then new row `1234+0` | echo | new row's equation echoes grouped; older rows ungrouped | E069 |

## P1 — Conversion mode

| # | Action | Expected | Evid |
|---|---|---|---|
| T056 | type `1` in source (Length, Meters→?) | target updates live per keystroke | E047 |
| T057 | edit bottom row | bottom becomes source; roles move, positions don't | E049 |
| T058 | change target unit | that row becomes target; other keeps value | E051 |
| T059 | ⇅ with top `5.23×10¹⁷` Fathoms | units exchange; top number preserved & re-read in new unit | E052 |
| T060 | C then type | both zero; caret at bottom; bottom = source | E049 |
| T061 | type `-e.5` | accepted while typing (free-form); minus renders − | E053 |
| T062 | unparseable source | target shows bold "Error"; clears live on fix | E053/E057 |
| T063 | change unit while source invalid | both values reset to 0 | E061 |
| T064 | Temperature `100` C→F | 212; `-40` C→F = −40; `-273.15` C→K = 0 exactly | E055 |
| T065 | Temperature `-300` C→K | −26.85 accepted, no clamp | E055 |
| T066 | Category switch Length→Temperature | target's number carried into both rows; units default | E054 |
| T067 | Number Base: top Binary bottom Hexadecimal | keypad dims all but 0/1 (follows TOP base); typed `2` still enters | E059 |
| T068 | 1 DZD → USD | 0.007379646 (live-rate dependent; verify format shape) | E063 |
| T069 | Currency dropdown search "US Do" | filtered to US Dollar; checkmark column hidden while filtering | E063/E064 |
| T070 | Currency ⓘ dialog | cards per unit w/ Updated date + IMF source; Refresh Rates closes dialog | E062 |
| T071 | Keyboard mode `10 DKK in USD` ⏎ | numeric row (≈1.49 at test rate); plain number formatting | E073 |

## P1 — Modes & UI behavior

| # | Action | Expected | Evid |
|---|---|---|---|
| T072 | Ctrl+Alt+B/A/F/P/K/C | mode switches; window re-targets 370/700 per mode | E080 |
| T073 | Keyboard mode: Up on empty entry | last result recalled into entry | E065 |
| T074 | type digit after result | fresh start (new expression) | E031/E065 |
| T075 | type digit after error | appends to erroneous text | E031 |
| T076 | history row click / double / right-click | inert (tooltip + hover only) | E065 |
| T077 | Ctrl+Escape | history cleared instantly, no confirm | E075 |
| T078 | type `US D` in entry | completion popover w/ codes + detail line | E072 |
| T079 | Tab in completion | accepts ("US DKK"); Escape dismisses keeping text | E072 |
| T080 | entry: select-all copy; replace; select-all paste | clipboard round-trip | E076 |
| T081 | header ↩ after typing `224` | removes last character; ↪ re-applies | E074 |
| T082 | Financial button (e.g. Pv) | form dialog; Calculate fills entry; NO history row | E046 |
| T083 | Financial Fv dialog with numeric inputs −100, 0.05, 10 | entry `-1257.789253555` | E046 |
| T084 | Programming à dialog: 'A' + Insert | entry gets `65` in current base | E044 |
| T085 | function picker click (e.g. median) | inserts `median(` template | E012 |
| T086 | constant click π / typed `pi`, `tau` | glyph insert; names evaluate; `phi` errors | E021/E026 |
| T087 | hamburger in Conversion mode | no Clear History item | E066/menu obs |
| T088 | F1 | help window opens (structure per doc 02 §7) | E014 |

## P1 — Persistence

| # | Action | Expected | Evid |
|---|---|---|---|
| T089 | set mode Conversion + units; add history rows; close via ×; relaunch | mode/category/units/width/scheme restored; history intact; entry EMPTY | E078/E081 |
| T090 | kill -9; relaunch | history since last orderly exit lost | E025 |

## P2 — Visual & interaction states

| # | Action | Expected | Evid |
|---|---|---|---|
| T091 | hover keypad button | shade changes; darker on press | E079 |
| T092 | click outside open dropdown | dropdown dismisses only (underlying control untouched) | E060 |
| T093 | hover history row | highlight + full-equation tooltip | E065 |
| T094 | color-scheme toggle live | instant re-theme both directions | E079 |
| T095 | keypad button tooltips | per doc 02 §11 table (incl. live Deg tooltip) | E036–E040 |
| T096 | window close in narrow window (× ≈ x341) | graceful close + persist | E081 |
| T097 | error `1/0` styling | red text + token highlight; colors per doc 11 §3 | E019/light capture |
| T098 | Preferences rows | controls & as-found values per doc 02 §4 | E005/E008/E068 |
| T099 | About dialog | icon, name, 51.1 chip, Website/Issue/Credits/Legal rows; URL tooltip | E029/E079 |
| T100 | shortcuts dialog content | per doc 02 §5 | E027/E028 |

## Notes for test execution

- Currency-vector expectations (T068, T071) depend on live rates: assert
  *format shape* and monotone reasonableness, not exact digits, unless rates
  are mocked.
- Floating display values assert the **shown digits** as recorded (9-decimal
  default).
- T089/T090 require process-level control of the app (orderly close vs
  SIGKILL) — same harness pattern as the investigation (doc 01 §1).
