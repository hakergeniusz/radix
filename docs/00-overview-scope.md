# 00 — Overview & Scope

Deliverable 00 of 16 · GNOME Calculator black-box behavioral specification.
Status of statements: every factual claim in docs 02–09 and 11 is labeled
**[Observed]**, **[Inferred]**, or **[Unknown]** (defined in §3). Docs 01, 10,
12–15 additionally carry process and evidence citations.

---

## 1. The product in one page

The application is a desktop calculator presented as a single top-level window
with a client-side-decorated header (no external title bar): a header strip
containing undo/redo buttons (except in Conversion mode), a mode switcher
button showing the current mode name with a chevron, a hamburger (primary
menu) button, and the standard minimize/maximize/close trio. **[Observed]**

Below the header, the window always shows a scrollable **history list** of
past equations (each row: equation text left, "=" center, result right) with a
free-form **entry line** at the bottom, except in Conversion mode where the
history is hidden and two value/unit rows take its place. **[Observed]**

The application has **six modes**, switchable from the mode menu (radio list
with keyboard accelerators Ctrl+Alt+B/A/F/P/K/C):

| Mode | Layout family | Window width |
|---|---|---|
| Basic | history + entry + 5-column function keypad | narrow (~370 px) |
| Advanced | history + entry + 5-column keypad (function rows above the basic rows, paged carousel when narrow) | wide (~700 px) |
| Financial | like Advanced; most function buttons open formula dialog windows | wide (~700 px) |
| Programming | like Advanced; adds base selector, bit panel, hex/binary digits | wide (~700 px) |
| Keyboard | history + bare free-form entry, **no keypad at all** | wide (~700 px) |
| Conversion | unit category + two role-based value/unit rows + digit-only keypad | narrow (~370 px) |

Switching modes re-targets the window width automatically (narrow ↔ wide).
**[Observed]**

The product is a *calculator with history as the central object*: every
evaluated equation becomes a history row; result-format and base changes
re-render all stored results live; history persists across restarts (on
orderly exit).

## 2. Deliverable map

| Doc | Content | Audience |
|---|---|---|
| 00 | overview & scope (this doc) | both |
| 01 | test environment & methodology (internal record) | process |
| 02 | UI inventory: windows, menus, dialogs, keypads, tooltips | implementation |
| 03 | expression grammar (tokens, precedence surface, entry editing) | implementation |
| 04 | arithmetic & evaluation semantics (incl. bitwise, bases) | implementation |
| 05 | functions, constants, variables | implementation |
| 06 | unit & currency conversion behavior | implementation |
| 07 | number formatting & display rules | implementation |
| 08 | input & interaction (keyboard, mouse, clipboard, undo) | implementation |
| 09 | history, persistence, settings | implementation |
| 10 | error handling catalog (cited) | both |
| 11 | visual specification (geometry, color, states; cited) | implementation |
| 12 | prioritized behavioral test catalog (cited) | implementation/QA |
| 13 | traceability: spec claim → evidence | process |
| 14 | unresolved questions register | process |
| 15 | coverage report (tested / partial / untested / limitations) | process |

## 3. Statement labeling convention

- **[Observed]** — directly produced and seen in the GUI during the
  investigation, with a recorded input sequence and screenshot-backed output.
  Reproducible from the test vectors in doc 12.
- **[Inferred]** — a general rule proposed to explain observed cases; fits all
  recorded observations but was not itself directly demonstrated. Flagged
  wherever used; test vectors for it appear in doc 12 as verification targets.
- **[Unknown]** — the behavior was not determined (not probed, not
  determinable through the GUI, or environment-limited). Enumerated in doc 14
  with impact assessment.

Nothing in this specification is derived from source code, binaries, packages,
or upstream documentation. The investigation used only the running
application's user-visible behavior (see doc 01 §5 for the access policy and
the analyst disclosure).

## 4. Scope

**In scope** — the behavior of the installed application (version 51.1, read
from its About dialog) under a single locale (en_US.UTF-8), single window
size families (~370 px / ~700 px), both color schemes (light and dark), with
mouse and keyboard interaction, on an X11 display without a window manager.
All user-visible surfaces: header, history, entry, keypads, menus, dialogs,
tooltips, completion popovers, help window (structure only), clipboard
behavior, persistence behavior, error presentation.

**Out of scope / not determined** (see doc 14 for the full register):
- Command-line interface behavior (e.g. any solve/evaluate flags) — not
  probed; treated as Unknown and excluded from the behavioral spec.
- Networking mechanics of exchange-rate retrieval (endpoint, protocol) — only
  the user-visible result of a refresh is specified.
- Localization beyond en_US (number grouping shape, translated strings) —
  single-locale observation only.
- Accessibility-tree semantics and screen-reader announcements — not probed.
- Behavior under a real window manager (tiling, maximize semantics, modal
  parenting quirks) — the test environment had no WM; WM-dependent behaviors
  are flagged.

## 5. Terminology

- **Header strip** — the client-side-decorated top bar of the window.
- **History row** — one entry in the history list: stored equation text,
  "=" separator, stored result text.
- **Equation echo** — the equation text as stored/displayed in a history row
  (operator glyphs normalized; superscripts rendered pretty where produced by
  dedicated buttons).
- **Entry** — the editable input line at the bottom (Keyboard/Basic/Advanced/
  Financial/Programming) or the editable value rows (Conversion).
- **Result echo** — the formatted result string that replaces the entry's
  content after evaluating.
- **Keypad** — the grid of buttons below the entry (absent in Keyboard mode).
- **Popover** — a transient floating surface anchored to a button or text
  position (menus, dropdowns, completion lists, function/variable pickers).
- **Source / Target (Conversion)** — the last-edited value row and the
  computed counterpart; roles, not fixed positions (see doc 06).
- **Narrow / wide** — window width targets ~370 px and ~700 px.
