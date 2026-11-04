# 08 — Input & Interaction

Implementation-facing. Statements **[Observed]** unless labeled otherwise.
Keyboard-shortcut listings from the in-app shortcuts dialog; interaction
rules verified by driving the GUI.

---

## 1. Keyboard shortcuts (dialog-verified listing)

### General
| Shortcut | Action |
|---|---|
| Ctrl+N | New Window |
| Ctrl+W | Close Current Window |
| Ctrl+Q | Quit the Application |
| F1 | Help |
| F10 | Open (hamburger) Menu |
| Ctrl+, | Preferences |
| Ctrl+? | Keyboard Shortcuts dialog |
| Ctrl+Escape | Clear History |

### Modes
Ctrl+Alt+B Basic · Ctrl+Alt+A Advanced · Ctrl+Alt+F Financial ·
Ctrl+Alt+P Programming · Ctrl+Alt+K Keyboard · Ctrl+Alt+C Conversion.
**[Observed]** (B/A/K/C exercised live)

### Programming mode
Ctrl+B Binary · Ctrl+O Octal · Ctrl+D Decimal · Ctrl+H Hexadecimal.
**[Observed]** (all four exercised; base switches re-render results, doc 07 §9)

### Keyboard Entry
`*` `/` → insert ×/÷ · Ctrl+2 Square ² · Ctrl+R Square Root √ · Ctrl+I Inverse
⁻¹ · Ctrl+P Pi π · Ctrl+T Tau τ · Ctrl+' Degree ° · Alt+[ then ] Floor ⌊⌋ ·
Ctrl+[ then ] Ceiling ⌈⌉ · Ctrl+E Scientific Format · Alt+0–9 Subscript
digits · Ctrl+0–9 Superscript digits · Ctrl+− Superscript minus.
Listed from the dialog; individually exercised: Alt+[ (inserted ⌊ glyph),
Ctrl+[ (inserted a bracket pair postfix). Others not exercised one-by-one —
their *dialog-listed* effect is **[Observed]**, live behavior **[Unknown]**
where marked in doc 15.

### Equation
Ctrl+C Copy · Ctrl+V Paste · Ctrl+Z Undo · Ctrl+Shift+Z Redo · Alt+←/→
Previous/Next Equation. Exercised: Copy/Paste/Undo/Redo (§4–5). Alt+←/→
equation navigation not exercised (**[Unknown]**).

**[Observed]** unless noted.

## 2. Entry typing rules

- Free-form text; appends at caret; click positions the caret (doc 03 §1).
- **Fresh-start rule**: typing a digit into a result-showing entry starts a
  new expression; typing after an error appends. **[Observed]**
- Return / `=` evaluates (doc 03 §7).
- Escape clears the entry selection (and dismisses popovers); the C button /
  "Clear Display [Escape]" tooltip implies Escape also clears the display —
  observed as selection-clear in Keyboard mode; full-clear behavior
  **[Unknown]**.
- Ctrl+A selects all entry text. **[Observed]**

## 3. Keyboard-mode specific

- **No keypad**; full-height history; bare entry (doc 02 §12).
- **Up-arrow** on an empty entry recalls the last result into the entry;
  further Up/Down presses are inert (a one-step recall, not a ring).
  **[Observed]**
- Typing after a recall replaces (fresh-start rule). **[Observed]**
- History rows are **display-only**: single click, double click, right click
  all inert (hover highlight + full-equation tooltip only). **[Observed]**

## 4. Clipboard

- Copy (Ctrl+C) with a selection puts the selected entry text on the
  clipboard; Paste (Ctrl+v) inserts at the caret, replacing any selection.
  Verified round-trip: select-all → copy → type replacement → select-all →
  paste restores the copied text. Paste into an empty entry works. No paste
  dialog/special forms. **[Observed]**
- Clipboard is system-wide (X11 selection) — inter-application paste not
  probed beyond the app itself. **[Unknown]** for foreign clipboards.

## 5. Undo / Redo

- **Character-level edit undo** on the entry via header ↩ / ↪ and
  Ctrl+Z / Ctrl+Shift+Z: typing "224" then ↩ removes the last typed
  character; repeated ↩ keeps untyping; ↪ re-applies. Redo pale/disabled
  until an undo exists. **[Observed]**
- Undo covers **edit actions** (typing, deletion), not evaluated-history
  operations. Whether Clear History is undoable was not probed.
  **[Unknown]**
- Conversion mode has **no undo/redo in the header** (and no entry-level undo
  probed). **[Observed]**

## 6. Auto-completion (currency codes)

- Typing a currency-code prefix ("US D") in the entry opens an inline
  completion popover above the caret listing matching codes with a detail
  line ("Danish Krone - kr").
- **Tab** accepts the highlighted suggestion ("US D" → "US DKK|"); popover
  closes on acceptance.
- **Escape** dismisses the popover leaving the typed text. **[Observed]**
- The "Show Currencies in Auto-Completion List" preference gates this
  (option "All" as-found; other options unverified — doc 02 §4).

## 7. Mouse semantics

- **Caret placement** by click x-position inside entry/value text.
  **[Observed]**
- **Keypad buttons steal focus** from the entry; type after clicking a button
  requires re-focusing the entry (or type via keypad). **[Observed]**
- **Popovers have a modal grab**: an outside click only dismisses the
  popover — the click does not also reach the control beneath the pointer.
  **[Observed]**
- **Hover states**: keypad buttons darken on hover and darken further while
  pressed (doc 11 §4); menu rows highlight; history rows highlight with a
  tooltip. **[Observed]**
- **Tooltips** appear after a hover delay (~1–1.5 s observed) on keypad
  buttons, history rows, About rows (full URL on Website row). **[Observed]**
- Keypad **carousel paging** at narrow width: page dots and mouse-wheel over
  the keypad switch pages (observed dots; wheel behavior per the wide/narrow
  comparison). **[Observed]**
- Window **close button** performs a graceful close that persists state
  (doc 09 §4). **[Observed]**

## 8. Clear History

Ctrl+Escape (and the menu item) wipes **all** history rows **instantly, with
no confirmation dialog**. **[Observed]**

## 9. New Window

Menu item + Ctrl+N exist. Multi-window behavior (independent state? shared
history? window count limits) not probed. **[Unknown]** (doc 14 notes a
single-instance handoff question)
