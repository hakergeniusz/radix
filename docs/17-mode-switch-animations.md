# 17 — Mode-Switch Animations (fresh Xvfb session, black-box)

Investigation record. **Scope is deliberately narrow**: only animations observed
while switching calculator modes with `Ctrl+Alt+B / A / F / P / K / C`
(Basic / Advanced / Financial / Programming / Keyboard / Conversion).
Dialog, popover, carousel-page and text-caret animations are **excluded**, even
where they were incidentally visible in a capture.

Claim labels follow doc 00 §3: **[Observed]** (measured in captures),
**[Inferred]** (interpretation of observed frames), **[Unknown]**.

---

## 1. Environment & method

| Aspect | Value |
|---|---|
| Application | installed native `/usr/bin/gnome-calculator`, version **51.1** (`--version`) |
| Display | **new** `Xvfb :101`, 1280×800×24, `-nolisten tcp`; no window manager; the older `:98` rig was not reused or disturbed |
| Session | `dbus-run-session`, private XDG dirs under `/tmp/opencode/anim-explore/state`; `GDK_BACKEND=x11`, `WAYLAND_DISPLAY` unset; `GTK_IM_MODULE=xim` requested (module absent on host, GTK fell back) |
| Animation setting | read from the app's *own* session bus: `org.gnome.desktop.interface enable-animations = true` |
| Capture | `ffmpeg n9.1` `x11grab`, 800×700+0+0, **60 fps**, FFV1 intra (`-g 1`); shortcut injected ≈1.2 s into a ≈3.2 s clip; frames extracted with `-fps_mode passthrough` |
| Analysis | per-frame absolute pixel diff > 12 → changed-pixel count and bounding box; focus-ring alpha proxy = mean(R−G) over the header mode-button crop: `(78,5)–(212,51)` in 370 px windows, `(288,4)–(412,52)` in 700 px windows |
| Coverage | **18 successful transition captures**, covering every destination mode plus repeats of Keyboard→Conversion and Conversion→Advanced |
| Source policy | source-blind: no application source, package contents, binary inspection, upstream repository or source-derived documentation; GUI pixels only |

A `gsettings` disable-animations control was attempted, but the private session
has no writable dconf backend (`gsettings set … false` returned 0 while
`gsettings get` still reported `true`), so that control is **not** used as
evidence. The identification of the fade below rests on the measured smooth
alpha ramp itself.

Raw frame sequences were trimmed after curation; per-capture analysis files,
switch contact sheets, selected raw frames, ring traces and the capture scripts
are preserved under `notes/anim-mode-switch/` (§8).

---

## 2. Summary of findings

### 2.1 No window, keypad or stack animation on any mode switch — **[Observed]**

Every tested switch changes the window in **one or two frames** (16–33 ms at
60 fps). There are no intermediate window widths, no sliding or crossfading
keypads, and no blended pixels between the old and new layout. Width jumps
directly: **370 px** in Basic and Conversion, **700 px** in Advanced,
Financial, Programming and Keyboard.

| Capture (from → to) | Large-change frames | Extra frames | Verdict |
|---|---|---|---|
| `basic-adv` | 1 (f71) | — | instant |
| `adv-basic` | 2 (f70, f71) | f70 clears strip x≥370; f71 redraws remainder | 2-stage redraw, not tweened |
| `key-basic` | 2 (f71, f72) | same clear/redraw pattern | 2-stage redraw |
| `adv-prog` | 1 (f66) | — | instant |
| `prog-fin` | 2 (f73, f74) | f74 is a discrete keypad-page redraw (y≥274), not a blend | instant + page reset |
| `fin-key` | 1 (f70) | — | instant |
| `basic-conv` | 1 (f67) | + ring fade-in (§2.2) | instant + ring |
| `adv-conv` | 2 (f66, f67) | + ring fade-in | instant + ring |
| `prog-conv` | 2 (f70, f72) | + ring fade-in | instant + ring |
| `fin-conv` | 2 (f66, f67) | + ring fade-in | instant + ring |
| `key-conv`, `key-conv-2` | 2 (f70, f71) | + ring fade-in | instant + ring |
| `conv-basic` | 1 (f71) | — | instant |
| `conv-prog` | 1 (f72) | tooltip at f104–105 (pointer parked over pad) — excluded | instant |
| `conv-adv` | 1 (f71) | + ring fade-out | instant + ring |
| `conv-adv-2` | 1 (f67) | no ring lit at switch | instant |
| `conv-fin` | 1 (f57) | + ring fade-out | instant + ring |
| `conv-key` | 1 (f73) | + ring fade-out | instant + ring |

### 2.2 The one real animation: header mode-picker focus-ring alpha — **[Observed]**

The header pill button that displays the current mode name
("Basic ⌄", "Advanced ⌄", …) carries the accent-colored focus outline. When it
gains or loses focus as a side effect of a mode switch, that outline's **alpha
fades smoothly**; nothing else in the window animates.

- **Fade-in** — observed on **all five** switches into Conversion (from Basic,
  Advanced, Financial, Programming and Keyboard; 5/5 captures, plus a repeat
  from Keyboard). Starts **3–4 frames (50–67 ms) after** the switch frame and
  ramps over **11–12 intervals (≈185–200 ms)** to a steady pink outline
  (peak proxy ≈5.8 mean R−G; 576 accent pixels in the crop).
- **Fade-out** — observed on Conversion→Advanced (8 intervals ≈134 ms),
  Conversion→Financial (8 ≈134 ms) and Conversion→Keyboard (9 ≈150 ms),
  beginning on the switch frame.
- **Easing is decelerating (ease-out), not linear.** Fade-in alpha increments
  in `key-conv`: 0.40, 0.70, 0.85, 0.70, 0.74, 0.60, 0.53, 0.43, 0.34, 0.22,
  0.14 → 0. Fade-out decrements in `conv-adv`: 1.28, 0.97, 0.69, 0.51, 0.24,
  0.15, 0.10, 0.05 → 0.

Visual proof: `notes/anim-mode-switch/evidence/key-conv/ring-zoom.png`
(fade-in) and `.../conv-adv/ring-zoom.png`, `.../conv-fin/ring-zoom.png`
(fade-out).

---

## 3. Measured timing table

`S` = first large-change frame of the switch; `α` = mean(R−G) proxy over the
header-button crop. Durations are at 60 fps (16.7 ms/frame), so ±1 frame
(±17 ms) is the sampling resolution.

| Capture | S | first α>0 | plateau / zero | Ramp |
|---|---:|---:|---:|---|
| `key-conv` | 70 | S+3 | +14 | ≈184 ms in |
| `key-conv-2` | 70 | S+3 | +14 | ≈184 ms in (reproduced) |
| `basic-conv` | 67 | S+3 | +15 | ≈200 ms in |
| `adv-conv` | 66 | S+4 | +15 | ≈184 ms in |
| `fin-conv` | 66 | S+4 | +16 | ≈200 ms in |
| `prog-conv` | 70 | S+4 | +16 | ≈200 ms in |
| `conv-adv` | 71 | α=3.99 at S | zero at S+8 | ≈134 ms out |
| `conv-fin` | 57 | α=4.62 at S | zero at S+8 | ≈134 ms out |
| `conv-key` | 73 | α=2.13 at S | zero at S+9 | ≈150 ms out |
| all other edges | — | 0 throughout | — | no ring animation |

---

## 4. What triggers the fade — **[Inferred]**

The animation is **focus-driven, not mode-driven**. It is not tied to a
particular pair of modes, only to whether the mode button gains or loses
keyboard focus when the old mode's UI is torn down:

- every switch **into** Conversion removes the calculator UI and leaves focus
  on the mode button → fade-in (5/5);
- switches **out of** Conversion fade the ring out **only when it was lit at
  switch time** (`conv-adv`, `conv-fin`, `conv-key`);
- on repeats where the ring was already unlit at switch time (`conv-basic`,
  `conv-prog`, `conv-adv-2`) no fade occurs.

Synthetic key injection does not deterministically force widget focus, so the
per-edge mapping in §2.1 is observational rather than a guarantee; the
fade-in/out mechanism itself is fully reproducible.

---

## 5. Explicitly excluded (not mode-switch animations)

- **Caret blink** — 50-pixel changes at x≈18, y≈238/291 in several captures.
- **Two-frame width redraws** (§2.1) — GTK clearing the vacated strip and
  redrawing; no interpolated geometry.
- **`prog-fin` f74** — discrete keypad-page change below y=274.
- **`conv-prog` f104–105** — tooltip raised under the stationary pointer.
- **About/Preferences dialog slide, popover and carousel animations** — known
  to exist (doc 16 §13) but out of the requested scope.

---

## 6. Implication for Radix

- `src/win.rs` uses `StackTransitionType::None` for both stacks and
  `window.set_default_size(mode.width(), …)` in one step. That already matches
  the observed absence of window/keypad/stack animation — **do not add
  synthetic mode-switch animations**.
- The only animated detail, the header focus-ring fade, is supplied by GTK
  itself whenever the mode button actually receives focus; no custom code is
  needed to reproduce it.
- Any window-resize animation a real compositor/WM might add is out of reach of
  this no-WM rig and remains an ENV item (cf. doc 14 #37).

---

## 7. Limitations

- 60 fps sampling (≈16.7 ms); an animation shorter than two frames would be
  invisible. All reported fades span 8–12 frames.
- No window manager, single display scale (1×), dark theme, en-US locale only.
- Focus state before a switch could not be forced deterministically (§4).
- The `enable-animations=false` control could not be applied in the private
  session; the fade identification relies on the pixel-level ramp.
- Only keyboard shortcuts were used to switch modes; the header dropdown path
  was not re-tested (both routes invoke the same mode action).

---

## 8. Evidence index

All paths relative to the repository root.

- `notes/anim-mode-switch/method/` — `setup.sh` (fresh :101 session),
  `capture.sh` (60 fps FFV1 + key injection), `analyze.py` (frame diffs),
  `ring.py` / `timing.py` (alpha traces), `cropzoom2.py`, `curate.py`,
  `batch.sh` / `batch2.sh`, `control2.sh` (inconclusive control, retained for
  transparency).
- `notes/anim-mode-switch/evidence/<capture>/analysis.txt` — full per-frame
  change log for all 18 captures.
- `notes/anim-mode-switch/evidence/<capture>/switch-contact.png` — contact
  sheet of frames around each switch, labelled in ms relative to `S`.
- `key-conv/ring-zoom.png`, `conv-adv/ring-zoom.png`,
  `conv-fin/ring-zoom.png` — magnified header-button fade sequences.
- `key-conv/ring.txt`, `conv-adv/ring.txt` — raw alpha-proxy numbers used for
  §3.
- `key-conv/selected/`, `conv-adv/selected/`, `conv-fin/selected/`,
  `basic-adv/selected/`, `prog-fin/selected/` — original unscaled frames for
  the canonical examples.
