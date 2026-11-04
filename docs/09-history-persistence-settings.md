# 09 — History, Persistence & Settings

Implementation-facing. Statements **[Observed]** unless labeled otherwise.

---

## 1. History model

- Every successful evaluation appends a row: **equation echo** left, "="
  center, **result** right (doc 07 §5 for echo/format rules). Rows stack
  newest at the bottom; the list scrolls. **[Observed]**
- Long equation echoes **wrap to two lines** within the row. **[Observed]**
- History is **shared across modes** (programming rows visible in
  Keyboard/Financial; conversion mode hides the list but does not clear it).
  Survives mode switches. **[Observed]**
- Rows are **display-only** (doc 08 §3): no click actions; hover highlight +
  tooltip with the full equation. **[Observed]**
- **Clear History** (Ctrl+Escape / menu item): instant, no confirmation,
  removes all rows; absent in Conversion mode's menu. **[Observed]**
- In wide Financial layout the history panel carries a **clear-history
  (backspace-icon) button** at its bottom-right. **[Observed]**
- Errors add **no** rows (doc 10 §3). Financial dialog results add **no**
  rows (they fill the entry). **[Observed]**

## 2. History limits

No cap observed in testing (≈70 rows accumulated without pruning). An
upper bound may exist — **[Unknown]**. Scrollback position/behavior on new
rows (auto-scroll to newest) **[Inferred]** from always-latest-visible
observations; not explicitly manipulated.

## 3. Settings inventory (Preferences dialog; doc 02 §4)

| Setting | Type | As-found | Observed effect |
|---|---|---|---|
| Number of Decimals | stepper (−/+) | 9 | caps fractional digits & mantissa (doc 07 §2) |
| Trailing Zeroes | toggle | off | pads to decimals count (doc 07 §3) |
| Group Digits | toggle | off | integer-part separators; applies to new row echoes (doc 07 §4) |
| Angle Units | dropdown | Degrees | trig semantics (doc 04 §4) |
| Word Size | dropdown | 64-bit | bitwise domain only (doc 04 §6) |
| Exchange Rate Refresh Interval | dropdown | Weekly | refresh scheduling (mechanics unobserved) |
| Currency Display | dropdown | Currency Name | effect not isolated **[Unknown]** |
| Show Currencies in Auto-Completion List | dropdown | All | gates completion list; options beyond All unverified |
| Favorite Currencies | subpage | all off | effect not probed **[Unknown]** |

As-found values are **not** necessarily factory defaults (prior host use).
**[Observed]**

Result Format (Automatic/Fixed/Scientific/Engineering) is a menu radio
group, also persisted (format survived restart implicitly via as-found
observations — direct restart check of the format setting **[Unknown]**).

## 4. Persistence (restart behavior)

**Graceful close** (window × / presumably Ctrl+W/Ctrl+Q) — on relaunch:
- **Restored**: current mode; category + both unit assignments (Conversion);
  window width (per-mode target); color-scheme choice; **complete history
  (all rows, in order)**. **[Observed]**
- **Not restored**: the in-progress entry text (entry comes up empty).
  **[Observed]**
- **Hard kill** (SIGKILL): history changes since the last orderly save are
  **lost** (≈60 batch-session rows vanished; only earlier as-found rows +
  one pre-kill row survived). **[Observed]**
- **Inferred mechanism**: state is written on orderly exit (not per-row /
  per-change). Fits both observations. **[Inferred]**
- Variables, undo stack, scroll position across restart: not probed.
  **[Unknown]**

## 5. Window state

- Per-mode target widths (narrow ~370 / wide ~700); the window resizes
  itself on mode switches; height stable (~660 with decorations). On relaunch
  the width matched the restored mode's target. **[Observed]**
- Window position: not manipulated (no WM); position persistence
  **[Unknown]**.
- Maximize is a no-op without a WM (environment limitation, doc 01 §6).

## 6. First-launch / fresh-profile shape

The investigation's first launch carried pre-existing history rows (prior
host use), so a controlled fresh-profile first-run (empty history? default
prefs? default mode?) was **not** observed. **[Unknown]** — flagged as a
deliberately unverified area (doc 14) since establishing it would require
wiping the app's state, which was out of bounds for this session.
