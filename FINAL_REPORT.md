# FINAL_REPORT — Radix clean-room implementation (Team B)

Status: **implementation complete, verification complete to the extent the
host allows**. Date: 2026-11-04. App version 0.1.0, app-id
`io.github.hakergeniusz.Radix`, exec `radix`.

---

## 1. What was built

An independent GTK4/libadwaita desktop calculator in Rust reproducing the
documented behavior of GNOME Calculator 51.1 from the black-box spec
(`docs/00`–`docs/16`) and owner-supplied evidence screenshots only.

- **`crates/radix-engine`** — expression engine: exact rational arithmetic
  (`astro-float`-free; num-rational BigInt rationals), the full operator
  set (+ − × ÷ ^ % ! mod, unary minus, implicit multiplication, right-assoc
  `^`), rounding family (`round` `floor` `ceil` `int` `frac` `[x]`),
  trig/hyperbolic/inverse with Deg/Rad/Grad, logs (`log` `ln` `log2`
  `log(8;2)` form), bitwise domain with word-size masks (8/16/32/64) and
  bases (bin/oct/dec/hex input + re-render), `conj` `arg` `re` `im`
  complex family, twos/bswap/shifts, statistics and financial function
  families, constants (π e τ φ physics categories, `_`, user variables,
  `rand`), result formats (automatic/scientific/engineering/fixed).
- **`src/`** — the application: six modes (Basic, Advanced, Financial,
  Programming, Keyboard, Conversion), history card model with bottom-
  anchored rows and the fixed `=` column, entry controller (fresh-start
  rule, undo/redo, superscript/subscript modes, currency completion with
  Tab/Escape, character-map dialogs), conversion pane (role model, swap,
  live recalculation, category semantics, searchable unit dropdowns,
  currency rates via `ureq`+rustls with refresh-interval preferences),
  programming bit panel following the entry, preferences & favorites
  dialogs, keyboard shortcuts, graceful-close persistence.
- **`data/`** — desktop file, appstream metainfo (validated), hicolor icons.
- **`packaging/`** — Arch PKGBUILD, debian/, fedora spec, flatpak manifest.
- **`tools/radix-session.sh`** — hermetic Xvfb+dbus test session harness.
- **`docs/16`** — the corrections addendum produced from owner evidence;
  `docs/00`–`15` are Team A's frozen spec.

## 2. Verification matrix

| Area | Status | How |
|---|---|---|
| Engine behavior vectors (docs/12 T001–T055) | **63/63 PASS** | `cargo test -p radix-engine` (vectors.rs) |
| Formatting vs original output | **exact match** on the E087 scene incl. the `92.5` display-correction case (D23) | `0.833333333×111 → 92.5`, `5÷6 → 0.833333333`, `sqrt(2) → 1.414213562` verified through the real engine |
| Clippy (workspace) | **0 warnings** | `cargo clippy --workspace` |
| GUI T-vectors (interactive) | 19 driven **PASS**, remainder visually verified earlier or marked unverified (see §5) | `notes/debug-evidence/gui-battery-2026-11-04/RESULTS.md` |
| E087/E089 scene reproduction | **PASS** — same history rows, same window size, layout matches (bottom-anchored rows, `=` column ≈62%, ⌫ entry glyph, keypad grids, bit panel) | `10-adv-e087-match.png`, `11-prg-e089-match.png` |
| Mode menu radio sync | **PASS** (fixed this round) | `12-adv-menu-radio-sync.png` |
| Persistence roundtrip (T089) | **PASS** (fixed this round: zero-height value rows on restore) | `t089-restored-conversion.png` |
| Arch package | **VERIFIED** — `makepkg` full cycle incl. `check()`; contents `/usr/bin/radix`, desktop, icons | `/tmp/radix-pkgtest/radix-0.1.0-1-x86_64.pkg.tar.zst` |
| Debian package | **UNVERIFIED** — no dpkg toolchain on this host (Arch) | — |
| RPM package | **UNVERIFIED** — no rpmbuild on this host | — |
| Flatpak | manifest JSON valid; desktop + appstream validate; **full builder run UNVERIFIED** (requires GNOME SDK download) | `appstreamcli validate`, `desktop-file-validate` |
| ARM64 (both deb/rpm and Arch) | **UNVERIFIED** — no cross toolchain; marked per brief §15 | — |
| Currency live-rate path | shape verified; live network values **UNVERIFIED** (rate table ships static defaults; refresh wired) | stage-2 battery |

## 3. Bugs found and fixed in this final round

1. **Mode/format/base/word radio desync** — stateful actions never moved
   their state on activation; menus showed stale radios (visible in the
   owner-comparison screenshots). Fixed in all four actions + programmatic
   `set_mode` sync.
2. **Entry backspace icon** — was circled-✕ (`edit-delete-symbolic`);
   evidence zoom shows the ⌫-key glyph → `edit-clear-symbolic`.
3. **History rows top-anchored** → bottom-anchored per E085/E087/E089.
4. **`=` column** — hugged the result; now a constant ≈62% column
   (4-column homogeneous grid), constant across widths as in E085/E087.
5. **Entry kept the result after `=`** — E085/E087 show a caret-only
   entry. Now clears after evaluation with invisible operator
   continuation (D22); fresh-start on digits preserved (T074).
6. **Automatic formatter** — `0.833333333×111` printed `92.499999963`;
   the original prints `92.5` while `sqrt(2)` keeps `1.414213562`.
   Implemented 9-fractional-digit cap + ≥5-nine-run swallow (D23) —
   reproduces every observed vector, engine tests still 63/63.
7. **Programming bit panel** fed from history; E089 shows zeros with an
   empty entry → panel now follows the entry, live on every change (D24).
8. **Completion popover lingered after evaluation** → dismissed on `=`
   and on errors.
9. **Conversion Length default** was Kilometers→Kilometers; E091/D19 say
   Kilometers→Miles (`Category::default_pair`).
10. **Restored Conversion pane rendered blank** — value textviews got a
    zero allocation when the page was visible from first map; pinned
    fixed-height rows (E091-faithful).
11. **PKGBUILD** — makepkg's default LTO breaks `ring`'s asm at release
    link; `options=('!lto')` added; full makepkg cycle now green.

## 4. Clean-room statement

Only `docs/00`–`docs/16` (Team A's black-box spec + corrections) and
owner-supplied screenshots (`notes/evidence-2026-11-04/`, E085–E095,
developer field redacted) were used. GNOME Calculator's source, binaries
beyond the owner's screenshots, and Team A's private analysis notes were
never accessed. `~/Projects/radix-gpl` was never read. The engine is an
independent implementation; the only third-party numerics are
num-rational/num-bigint (MIT/Apache-2.0).

## 5. Known gaps / unverified (honest list)

- GUI vectors not driven this round: T060, T064–T067 (conversion
  dropdown/category interactions), T068–T070 (live currency rates),
  T082–T084 (Financial + char-code dialogs — forms verified structurally
  in the earlier same-day battery), T085/T086 (picker clicks), T088 (F1),
  T090 (SIGKILL semantics), T091–T095 (hover/theme/tooltip polish),
  T096 (narrow-close), T099 (About dialog). Earlier visual battery:
  `notes/debug-evidence/gui-battery-2026-11-04/09-conversion.png` + the
  E085–E090 six-mode sweep.
- Spec-level [Unknown]s inherited from docs/14 (#1–4 formatting bounds,
  #26–27 shortcut subset, #29 financial formulas beyond Fv, #31 Grad).
- deb/rpm/flatpak-builder/ARM64: unverified (see §2).
- Metainfo intentionally ships no `<screenshots>` yet — no hosted
  screenshot asset exists; add one at first release.
- About-dialog developer credit: the owner redacted it in E095; Radix
  credits "hakergeniusz" (this project's maintainer), not the original.

## 6. Licensing

Radix is MIT (`LICENSE`). Dependency scan of `Cargo.lock` (2026-11-04):
151 packages, 16 SPDX expressions, **all permissive**; no copyleft
attaches to the binary (`r-efi`'s LGPL alternative is one of three
OR-options, MIT satisfies it). Details in `THIRD_PARTY_LICENSES.md`.

## 7. Build / run / test

```sh
cargo build --release          # app (GTK4 + libadwaita 0.8 required)
cargo test -p radix-engine     # 63 engine vectors
cargo clippy --workspace       # 0 warnings
tools/radix-session.sh target/debug/radix   # hermetic GUI session
cd packaging/arch && makepkg -f             # Arch package (verified)
```

`RADIX_TRACE=1` enables behavioral tracing; `RADIX_DARK=1` forces dark
scheme for test rigs.

## 8. Open questions to the owner

- Q23: display-correction swallow mechanism (D23) is inferred from two
  observations — inputs with long 9-runs would confirm.
- Q24: operator-after-`=` semantics (D22) — implemented as continuation
  from the remembered result; unobserved in evidence.
- Q21: ⇧⁻¹ inverse-toggle semantics — implemented per D16 [Inferred].
- Q22: currency preference option lists behind closed dropdowns (D20).

---

## 9. Addendum — side-by-side comparison round (2026-11-04 evening)

The owner supplied eleven side-by-side screenshots (Radix left, original
right). A full re-verification round against them produced the fixes in
docs/16 §9 and decisions D25–D29; reproduction composites in
`notes/comparison-2026-11-04/`, results in the gui-battery RESULTS.md.
Highlights: one display card in every mode (entry strip and Programming bit
panel inside it), stock libadwaita window palette, 40-column history row
grid (echo 40% / "=" 63.75% / result right), 410/750 window defaults,
":close"-only header, menu gained Clear History (Ctrl+Escape) and lost a
duplicate header item, keypad spacing 8/4 with uniform function shade,
Preferences fixed at 370×660, Favorite Currencies rewritten (back arrow,
inline search, two-line name-sorted rows), conversion unit dropdowns
full-brightness. Gates re-run after the round: engine 63/63, clippy 0,
window-size bisect stable at mode defaults.

Note: the Radix flatpak installed on the owner's machine (`radix-origin`
remote, version 51.1.3) predates this round — several deltas in the
owner's screenshots (Kilometers→Kilometers default pair, single-line
favorites, separate Basic entry strip) are already fixed in the workspace
tree; the flatpak needs a rebuild to pick them up.
