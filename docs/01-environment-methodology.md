# 01 — Test Environment & Methodology

Internal investigation record. Process-facing; contains evidence citations
(`notes/evidence-log.md`, entries E000–E081) and tool paths. Not part of the
implementation-facing behavior contract.

---

## 1. Environment

| Aspect | Value | How established |
|---|---|---|
| Application | org.gnome.Calculator from Flathub, user install | install record |
| Version | **51.1** | About dialog (E029, E079-light) |
| Display | Xvfb `:98`, 1280×800×24, scale 1.0, no window manager | bootstrap script (E000) |
| Session | private D-Bus (`dbus-run-session`) + xdg-desktop-portal-gtk for appearance | bootstrap script |
| Locale | en_US.UTF-8 | environment (E000) |
| Theme as-found | dark rendering at launch; color-scheme switchable live (E079) | screenshots |
| Window management | none (no WM): CSD shadows visible, maximize a no-op, no resize handles | E001, E068 |
| Input | XTEST synthetic mouse/keyboard (xdotool), 0.15–0.25 s move→click delays | tooling |
| Capture | full-screen PNG per step + crop/zoom composites for evidence | tooling |

Sandboxing: the app ran with detached XDG data/config directories inside the
private session so the host's real user settings were never read or written;
the app's own per-app state directory (created by the app under the user's
flatpak state tree) is where its settings/history persisted during testing.
The user's real desktop session was never touched (app never launched there;
one pre-existing user-owned calculator process on the real display was left
alone).

## 2. Stability incidents and their fixes (methodology-relevant)

1. **X-socket race** — first launch failed (~1.8 s socket bind delay); fixed
   with a wait loop (E001).
2. **Harness task kills destroyed the session twice** — app froze then died;
   fixed by fully detaching Xvfb/D-Bus/app from the driving shell (E007).
3. **Stuck Ctrl modifier** — every keystroke arrived as Ctrl+key after a
   ctrl+a; fixed by releasing all modifiers before input batches (E013).
4. **IBus keystroke swallowing** — the app's input-method module connected to
   the host's real ibus daemon and silently dropped all synthetic keystrokes
   (68× "events queue growing" in the app log); fixed by forcing the simple
   XIM input method for the app (E024). *Consequence: the investigation's
   keyboard path is XIM, not IBus; IME-composition behavior is out of scope.*
5. **Popover modal grab** — clicking outside an open dropdown only dismissed
   it; the click did not reach what was under the pointer (E023, E060).
   Tooling therefore always dismissed popovers as a separate step.
6. **Focus flakiness** — after certain transitions (post-C in Conversion,
   dialog closes) typed input occasionally did not land; the reliable pattern
   was: click field → select-all → type (E042, E056).

## 3. Method

Discovery-driven, source-blind GUI exploration:

1. **Inventory first** — enumerate every menu, dialog, keypad button and
   tooltip before semantic testing (E002–E012, E030, E040, E044, E046).
2. **Behavior batteries** — systematic input vectors per area (arithmetic
   batches b1–b12; formatting battery; conversion category walk; persistence
   cycle), each with a full-screen capture.
3. **Boundary probes** — overflow (170!/171!, 2^1024), division by zero,
   below-absolute-zero temperatures, word-size limits, percent adjacency
   combinations, malformed input, empty/degenerate cases.
4. **Contrast tests** — graceful close vs kill -9 for persistence (E025,
   E078); format-pref change vs stored equation echoes (E067–E069); light vs
   dark palettes (E079).
5. **Live re-verification** — every inferred rule was re-tested against at
   least one disconfirming candidate before being promoted (e.g. percent
   relative-base model E018; swap semantics E052; role model E049).

The in-app Help window (F1) was opened once to *harvest hypotheses*
(E014); every behavior claimed in this specification was then verified
through the GUI itself. Help text was never used as standalone evidence.

## 4. Evidence discipline

- Every entry in `notes/evidence-log.md` records: timestamp, window/state,
  action sequence, observed result, confidence (O / O* / I).
- Screenshots under `notes/shots/` (raw) and `notes/shots/crops/` (annotated
  crops); deliverable reference images copied to `docs/assets/`.
- Colors were sampled from our own screenshots with a pixel probe; geometry
  measured by edge scans on the same captures (E079, doc 11 §2).
- No source code, repository, package content, binary, or source-derived
  documentation of the application was accessed at any time.

## 5. Access policy & analyst disclosure (recorded per brief)

The brief prohibits reading/inspecting/searching the application's source
code, upstream repositories, tests, build scripts, package internals, or
source-derived documentation, and prohibits decompilation. This was honored
end-to-end: only the running GUI was exercised. The analyst declares prior
general familiarity with the GNOME desktop family on this host and prior
calculator-reimplementation work on this machine; to prevent contamination,
no earlier project artifacts were consulted (an unrelated older project
directory on disk was deliberately not read), and all spec claims are grounded
exclusively in this investigation's dated, screenshot-backed experiments.

## 6. Known environment limitations (affecting interpretation)

- **No WM** — maximize button is a no-op (E068); window resize only occurred
  via mode switches; drag-resize, tiling, and modal parenting under a real WM
  are untested.
- **Single locale** — grouping shape ("1,234,567"), decimal point, and all
  strings are en-US observations.
- **Popover clipping** — with the window near its minimum height, one
  preferences dropdown extended below the screen edge and could not be
  scrolled; its remaining options are unverified (E068).
- **Theme plumbing ambiguity** — dark-at-launch vs live-'default'-renders-
  light discrepancy (E079 §end) leaves the color-scheme source-resolution
  order Unknown; both palettes themselves are fully documented.
- **Rates freshness** — currency refresh produced no visible network feedback;
  success-vs-cache indistinguishable (E062).
