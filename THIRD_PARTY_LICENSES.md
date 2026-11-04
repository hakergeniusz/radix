# THIRD_PARTY_LICENSES — Radix

Radix's own code and assets are MIT-licensed (`LICENSE`). This file inventories
every third-party dependency, its licence, linkage, and distribution
obligations. The authoritative list of Rust crates (including transitive
dependencies) is `Cargo.lock`; the table below lists the direct dependencies
and the notable system libraries.

Radix deliberately uses **no GPL/LGPL-licensed Rust crates and no C-linked
numerics** (see `IMPLEMENTATION_DECISIONS.md` D2): the numeric stack is pure
Rust under MIT/Apache-2.0.

**Verification scan (2026-11-04, `cargo metadata` over `Cargo.lock`):** 151
packages, 16 distinct SPDX expressions — all permissive (MIT, MIT OR
Apache-2.0, Apache-2.0 OR MIT, Unicode-3.0, ISC, BSD-2/3-Clause,
Apache-2.0 WITH LLVM-exception, CDLA-Permissive-2.0, Unlicense OR MIT,
Apache-2.0 AND ISC for `ring`, plus OR/AND combinations). The single
LGPL-mentioning expression (`r-efi`: MIT OR Apache-2.0 OR LGPL-2.1-or-later)
is a disjunction satisfied by its MIT alternative. No copyleft obligation
attaches to the `radix` binary.

## Rust crates (statically linked into the `radix` binary)

| Crate | Version | Licence | Role |
|---|---|---|---|
| `radix-engine` (path) | 0.1.0 | MIT (own code) | expression engine |
| `gtk4` | 0.10 | MIT | GTK4 Rust bindings |
| `libadwaita` (`adw`) | 0.8 | MIT | libadwaita Rust bindings |
| `ureq` | 2 | MIT OR Apache-2.0 | HTTP client (currency rates) |
| `serde_json` | 1 | MIT OR Apache-2.0 | persistence state file |
| `num-bigint` | 0.4 | MIT OR Apache-2.0 | exact big integers |
| `num-rational` | 0.4 | MIT OR Apache-2.0 | exact rationals |
| `num-traits` | 0.2 | MIT OR Apache-2.0 | numeric traits |
| `astro-float` | 0.3 | MIT OR Apache-2.0 | high-precision floats |
| `rand` | 0.9 | MIT OR Apache-2.0 | `rand` function |

All transitive Rust dependencies of the above are permissively licensed
(MIT, Apache-2.0, BSD-style, or ISC — e.g. the gtk-rs/glib-rs family: MIT;
rustls/webpki (via ureq with rustls): Apache-2.0/ISC/BSD). Verify at release
time with a licence scan over `Cargo.lock` (see `packaging` CI job).

## System libraries (dynamically linked at runtime)

| Library | Licence | Obligations |
|---|---|---|
| GTK 4 | LGPL-2.1+ | dynamic linking; ship licence notice (this file) |
| libadwaita | LGPL-2.1+ | dynamic linking; ship licence notice (this file) |
| GLib/GIO/Pango/Cairo/gdk-pixbuf | LGPL-2.1+ | dynamic linking; ship licence notice |
| glib-networking (TLS backend for rate refresh) | LGPL-2.1+ | runtime dependency, not linked into our binary |

Dynamic linking against LGPL-2.1+ system libraries carries no
source-redistribution obligation for Radix itself; the packages ship the
standard licence notices (Debian `copyright`, RPM `%license`, Arch
`/usr/share/licenses`, Flatpak metainfo `project_license`).

## Bundled assets

| Asset | Licence | Origin |
|---|---|---|
| `data/icons/**` (icon artwork) | MIT (Radix) | original in-house artwork |
| Currency fallback snapshot (`src/data/currency.rs`) | MIT (Radix) | approximate USD rates, replaced by live data at first refresh |

## License texts

MIT / Apache-2.0 texts are included in the upstream crates themselves and
are reproduced in source distributions via `vendor/`. The MIT text for Radix
is in `LICENSE`.
