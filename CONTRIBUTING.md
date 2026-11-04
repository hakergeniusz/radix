# Contributing to Radix

Thanks for your interest in contributing.

## Ground rules

- The approved specification in `docs/` is the behavioral contract. Changes
  to observable behavior must be justified against the spec (or recorded as
  a deliberate deviation in `IMPLEMENTATION_DECISIONS.md`).
- Keep the clean-room discipline: do not consult other calculator
  implementations for behavioral or architectural answers.
- Behaviour fixes need a test (engine vectors in `crates/radix-engine`, UI
  behaviors in the verification battery).

## Workflow

1. Fork / branch from `master`.
2. `cargo fmt` and `cargo clippy` must be clean.
3. `cargo test --locked` must pass.
4. Keep commits small and conventional
   (`feat:`, `fix:`, `docs:`, `test:`, `chore:`, `refactor:`).
5. Version bumps must update: root `Cargo.toml`, `packaging/debian/changelog`,
   `packaging/fedora/radix.spec`, `packaging/arch/PKGBUILD`, and the
   AppStream `releases` section in `data/io.github.hakergeniusz.Radix.metainfo.xml`.

## Architecture notes

- `crates/radix-engine` is dependency-light and UI-free; its public API is
  the frozen contract documented in its `lib.rs`.
- The GUI lives in `src/` and talks to the engine only through that API.
- Unit and currency data tables live in `src/data/`.
- Persistence is a single atomic JSON state file (see
  `IMPLEMENTATION_DECISIONS.md` D8).
