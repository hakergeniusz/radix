# Radix

**Radix** (`io.github.hakergeniusz.Radix`) is an independent desktop
calculator for Linux, written in Rust. It is an independently engineered
implementation of a documented behavioral specification (see `docs/`) —
no code, architecture, or branding is derived from any other calculator.

## Features

- **Six modes** — Basic, Advanced, Financial, Programming, Keyboard, and
  Conversion — with per-mode window widths and live switching
  (Ctrl+Alt+B/A/F/P/K/C).
- **History-centric workflow** — every evaluated equation becomes a history
  row; results re-render live when the result format, grouping, or number
  base changes.
- **Arbitrary-precision arithmetic** — exact rationals where possible
  (`1÷3×3 = 1` exactly), high-precision transcendentals, no overflow
  (`171!`, `2^1024` are fine), complex results (`sqrt(-1) = i`).
- **Percent grammar** with the documented relative-base model.
- **Bitwise operations** with a selectable word size (8/16/32/64-bit) and
  base re-rendering (binary/octal/decimal/hex).
- **Unit & currency conversion** — 14 categories, live exchange rates,
  temperature affine maps, number-base conversions.
- **Financial formulas** via form dialogs.
- **Four result formats** — Automatic, Fixed, Scientific, Engineering — with
  decimals/trailing-zero/grouping preferences.
- **Graceful persistence** — mode, history, conversion units, window size,
  color scheme, and preferences are restored on relaunch.
- Light/dark themes with live re-theme, keyboard shortcuts throughout,
  clipboard, character-level undo/redo.

## Building

Requirements: Rust ≥ 1.85, GTK4 ≥ 4.14, libadwaita ≥ 1.5, `pkg-config`.

```sh
cargo build --release
./target/release/radix
```

Run the test suite:

```sh
cargo test --locked
```

## Repository layout

| Path | Content |
|---|---|
| `docs/` | the approved behavioral specification (sole source of truth) |
| `crates/radix-engine/` | expression engine: parser, numerics, formatting |
| `src/` | the GTK4/libadwaita application |
| `data/` | desktop entry, AppStream metadata, original icon artwork |
| `packaging/` | Debian, Fedora, Arch, Flatpak packaging |
| `IMPLEMENTATION_PLAN.md` / `IMPLEMENTATION_DECISIONS.md` / `IMPLEMENTATION_QUESTIONS.md` | process documents |
| `THIRD_PARTY_LICENSES.md` | dependency licence inventory |

## Licence

Radix's own code and assets are MIT-licensed (see `LICENSE`). Third-party
dependencies have their own licences — see `THIRD_PARTY_LICENSES.md`.
