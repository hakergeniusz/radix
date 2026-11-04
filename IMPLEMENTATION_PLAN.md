# IMPLEMENTATION_PLAN — Radix

Team B implementation plan for Radix (`io.github.hakergeniusz.Radix`), built
from the approved behavioral specification in `docs/` (00–15) only. Created
2026-11-03. See also `IMPLEMENTATION_DECISIONS.md` (D-IDs referenced below)
and `IMPLEMENTATION_QUESTIONS.md`.

## 1. Architecture overview

| Component | Location | Owner | Notes |
|---|---|---|---|
| Engine (parser, evaluation, formatting) | `crates/radix-engine` | engine workstream | Pure-Rust numerics; public API frozen at kickoff (D3) |
| GUI application | `src/` (crate `radix`) | app workstream | GTK4 + libadwaita (D1) |
| Unit/currency data | `src/data/` | app workstream | Categories, unit factors, currency provider (D5, D6) |
| Persistence | `src/persist.rs` | app workstream | GSettings prefs + state file (D8) |
| Icon + desktop integration | `assets/`, `data/` | app workstream | Original artwork (D14) |
| Packaging | `packaging/{debian,fedora,arch,flatpak}` | app workstream | AMD64 + ARM64 (D15) |

Numeric stack: exact rationals (BigInt) + ~200-bit inexact reals + complex
pairs — chosen independently, no upstream architecture is reproduced (D2).

## 2. Requirements traceability matrix (spec → implementation → tests)

Requirement sources are spec docs; verification vectors are `docs/12` IDs.
Status vocabulary: **planned / partial / done / verified** (verified = test
or scripted GUI check exists and passes).

| Area | Spec source | Vectors | Status |
|---|---|---|---|
| Expression grammar (tokens, precedence, %, functions, variables) | 03 | T001–T031 | planned → engine |
| Percent relative-base model | 03 §4 | T007–T016 | planned → engine |
| Arbitrary precision, exactness, complex | 04 §1 | T021–T023, T027, T047 | planned → engine |
| Rounding family | 04 §3 | T029–T031 | planned → engine |
| Trig + angle units | 04 §4 | T032–T036 | planned → engine + UI |
| Logs/powers | 04 §5 | T028, T048 | planned → engine |
| Bitwise + word size + bases | 04 §6 | T037–T047 | planned → engine + UI |
| Financial dialogs | 04 §7, 02 §9 | T082–T083 | planned → UI |
| Conversion arithmetic | 04 §8, 06 | T056–T067 | planned → UI + data |
| Currency (rates, ⓘ, expressions, completion) | 06 §6, §10, 08 §6 | T068–T071, T078–T079 | planned |
| Functions/constants/variables catalog | 05 | T024–T025, T085–T086 | planned → engine |
| Formatting (4 formats, prefs, re-render) | 07 | T048–T055 | planned → engine |
| Errors (3 surfaces, side effects) | 10 | T026 + doc-10 §2 battery | planned |
| History model + clear | 09 §1–2, 08 §8 | T076–T077 | planned |
| Input rules (fresh-start, recall, clipboard, undo) | 08 §1–7 | T073–T081 | planned |
| Persistence (graceful vs kill) | 09 §4 | T089–T090 | planned |
| Visual system (geometry, palette, states) | 11 | T091–T100 | planned |
| Modes + window widths | 00 §1, 11 §1 | T072 | planned |

## 3. Roadmap (brief §14 phases)

1. **Requirements** — done (this file + decisions + questions).
2. **Foundation** — workspace scaffold, engine API freeze, app shell (window,
   header, history, entry). ← current
3. **Engine** — delegated to the engine workstream: grammar, numerics,
   formatting, catalogs, tests against T-vectors.
4. **Parser/formatting integration** — engine wired into the entry flow
   (evaluate on Return/`=`, fresh-start rule, echo normalization, error
   highlighting spans).
5. **UI** — six modes, keypads, popovers (f(x), x, mode, hamburger,
   completion), dialogs (prefs, shortcuts, about, financial forms, char code,
   exchange details), help window.
6. **Remaining functionality** — conversion mode + unit data, currency rates,
   settings wiring, persistence.
7. **Packaging** — deb/rpm/arch/flatpak, AMD64 + ARM64.
8. **Verification** — full docs/12 battery (engine-level + scripted GUI under
   Xvfb), packaging smoke tests.
9. **Final review** — licensing audit, coverage report, final report.

## 4. Verification strategy

- Engine: `cargo test -p radix-engine` — every engine-applicable T-vector is
  a unit test (T001–T055 subset), plus doc-10 error battery.
- App logic (unit-convertible, persistence codec): `cargo test`.
- GUI: scripted battery under Xvfb (own virtual display, isolated session per
  the session-isolation rules) covering T072–T099 minus environment-bound
  items; visual comparison against `docs/assets/` references is advisory
  (geometry/palette per docs/11), since the original application must not be
  run for implementation-time comparison.
- Currency vectors (T068, T071): assert format shape + reasonableness, not
  exact digits (rates are live).
- Every unverified or environment-bound item stays marked (e.g. WM-dependent
  behaviors, IME, a11y-tree semantics, localization — docs/14, docs/15).
