# Evidence Log — org.gnome.Calculator black-box investigation

Internal investigation record. NOT part of the implementation-facing spec.

Entry format:

    [E###] date-time | window/state | action sequence | observed result | confidence

Confidence: O = directly observed (screenshot/quoted pixels), O* = observed with
minor reading uncertainty (e.g. dense digits), I = inference only.

Analyst prior-exposure disclosure (recorded up front per methodology §limitations):
the analyst has prior experience with the GNOME desktop family on this host, and a
prior project on this machine concerned a calculator reimplementation. NO source
code of org.gnome.Calculator (or any derivative) was read for this specification,
per the brief's absolute prohibition. Every factual claim in the spec is grounded
in a dated, screenshot-backed experiment from this log and is independently
reproducible from the test vectors.

---

## Session 1 — 2026-11-03, light theme

- [E000] env: Xvfb :98 1280x800x24, scale 1.0, locale en_US.UTF-8, Adwaita light,
  flatpak org.gnome.Calculator from Flathub (runtime org.gnome.Platform//51),
  sandboxed XDG dirs, no WM (CSD shadows visible), portal running.
- [E001] 16:20 | first launch attempt failed (Xvfb socket bound ~1.8 s after spawn; app exited
  "Failed to open display"). Relaunch 16:21 OK. Window "Calculator" 370x639 @(0,0); helper 1x1
  window named "gnome-calculator" also present (xdotool --pid targets the wrong one; use --name).
- [E002] 16:21 | initial render: DARK theme (as-found; sandboxed session dconf had no
  color-scheme set — dark source unconfirmed, likely app-private dconf from earlier host use).
  Header: undo, redo, "Basic ⌄", hamburger, minimize, maximize, close (CSD, no WM).
  History (as-found, NOT fresh): 2+2 = 4 | 5÷6 = 0.8333… | 0.8333333333×111 = 92.5 |
  3×111+(23÷60)×111 = 375.551 (wraps to 2 lines). Entry row below history with caret + clear icon.
  Keypad (Basic): row1 C ↑n ↓n ÷ mod; row2 7 8 9 × %; row3 4 5 6 − ( ); row4 1 2 3 + [=pink tall];
  row5 0 . x⌄ f(x)⌄. [=] spans rows 4-5, pink accent. Screenshot s1-001-initial.png.
- [E003] 16:23 | click "Basic ⌄" header button -> radio menu: Basic (Ctrl+Alt+B, selected),
  Advanced (Ctrl+Alt+A), Financial (Ctrl+Alt+F), Programming (Ctrl+Alt+P), Keyboard (Ctrl+Alt+K),
  Conversion (Ctrl+Alt+C). Six modes. s1-002-mode-menu.png
- [E004] 16:24 | hamburger menu: New Window (Ctrl+N) | Clear History (Ctrl+Escape) |
  "Result Format" group radios: Automatic(sel)/Fixed/Scientific/Engineering |
  Preferences (Ctrl+,) | Keyboard Shortcuts (Ctrl+?) | Help (F1) | About Calculator.
  s1-003-hamburger.png
- [E005] 16:26 | Preferences dialog (370 wide, scrollable): General: Number of Decimals = 9
  (-/+ steppers), Trailing Zeroes toggle OFF, Group Digits toggle OFF, Angle Units dropdown
  "Degrees", Word Size dropdown "64-bit". Currency Conversion: Exchange Rate Refresh Interval
  "Weekly", Currency Display "Currency Name", "Show Currencies in Auto-Completion List" "All"
  (clipped). s1-004b-preferences.png. NOTE as-found values may not be defaults (prior host use).
- [E007] 16:20-16:40 | INCIDENT: harness-side background-task kills destroyed the dbus session
  twice; the app hung (rendered but unresponsive — clicks/keys had no visible effect) then died
  (black screen). Root cause: session was a child of harness-managed background tasks.
  Fix: bootstrap-session.sh rewritten — Xvfb + dbus-run-session + app all setsid/nohup-detached,
  X-socket wait loop added (E001 race), idempotent relaunch; launch-app.sh added. Prefs dialog
  observations E005/E006 were captured before the freeze; unchanged values re-verified on relaunch.
- [E008] 16:47 | Prefs scrolled: Currency Conversion section: Exchange Rate Refresh Interval
  (Weekly ⌄), Currency Display (Currency Name ⌄), Show Currencies in Auto-Completion List (All ⌄),
  Favorite Currencies (→ subpage). Angle Units options: Radians, Degrees ✓, Gradians.
  s1-011-prefs-bottom2.png, s1-012-angle-units.png
- [E009] 16:49 | Favorite Currencies subpage: back chevron, title, close X, "Search currencies"
  search field, alphabetical currency list w/ toggles (observed: Algerian Dinar DZD, Argentine
  Peso ARS, Armenian Dram AMD, Australian Dollar AUD, Bahraini Dinar BHD, Bangladeshi Taka BDT,
  Botswana Pula BWP, Brazilian Real BRL, British Pound Sterling GBP; all toggles OFF as-found).
  Full enumeration deferred to conversion-mode testing. s1-013-fav-currencies.png
- [E010] 16:52 | f(x) popover: categories Trigonometry, Complex, Programming, Rounding,
  Statistics, Sundry; "Function name" search field; bottom row "1 [-] [+] [⊕]" (custom function
  builder, arg count stepper + add). Trigonometry submenu: sin(x), cos(x), tan(x), sin⁻¹(x),
  cos⁻¹(x), tan⁻¹(x), sinh(x), cosh(x), tanh(x) [scroll check pending].
  s1-014-fx-menu.png, s1-015-fx-trig.png
- [E011] 16:55 | f(x) submenus: Complex: conj(x), arg(x), re(x), im(x). Programming: twos(x),
  bswap(x), mod(x;y), modexp(x;y;p). Rounding: round(x), floor(x), ceil(x), int(x), frac(x).
  s1-016-fx-complex.png, s1-017-fx-programming.png, s1-018-fx-rounding.png
- [E012] 16:57 | f(x) submenus: Statistics: sum(x₁;x₂;…), sumsq(x₁;x₂;…), average(x₁;x₂;…),
  median(x₁;x₂;…), min(x₁;x₂;…), max(x₁;x₂;…), stdev(x₁;x₂;…), stdevp(x₁;x₂;…), var(x₁;x₂;…).
  ACCIDENTAL DISCOVERY: clicking a function row inserts a "name(" call template into the entry
  (entry showed "median(9)" after misclicks — trailing digit was my own click contamination).
  s1-019-fx-statistics.png
- [E013] 17:05 | KEYBOARD FIX: XTEST keys were silently dropped due to a stuck modifier from an
  earlier ctrl+a (every key arrived as Ctrl+key). Fix: keyup ctrl/shift/alt/super/meta before
  input batches. Keyboard now delivered. NOTE: clicks always worked (pointer XTEST unaffected).
- [E014] 17:05 | F1 opens "Calculator Help" window (separate window, ~957px wide): search field,
  sections "User Interface" (Using the History View / Using the Mouse / Using the Keyboard /
  Superscript and Subscript) and "Equations" (Basic Equations / Absolute Values / Bitwise
  Operations / Factorials / Factorization / Percentages / Powers and Roots / Rounding /
  Logarithms / Modulus Division / Trigonometry / Variables and Constants). Help text will be
  used ONLY as hypotheses to verify experimentally, never as standalone evidence.
  s1-026-after-keyup.png
- [E015] 17:10 | C button: clears ENTRY only, history untouched. Solve convention: typed
  "9=" added history row "9 = 9", entry then showed result "9". Row "51+2 = 53" appended earlier
  by typing "1+2=" (leftover "5" from a click prepended — typing appends to current entry content).
  Result replaces entry content after solve. s1-027/028/029.
- [E016] 17:15 | batch b1 (typed via keyboard, Return solves; history echoes * as ×, / as ÷,
  - as −; ^ stays literal): 2+3*4=14 | (2+3)*4=20 | 10-4-3=3 (left-assoc) | 100/5/2=10 (left-assoc)
  | 2^3^2=512 (RIGHT-assoc powers) | -5+3=-2 (unary minus OK). b1-arith.png
- [E017] 17:22 | b3 percent: 50%=0.5 | 100+10%=110 | 100*10%=10 | 200-50%=100 | 10%%=0.001 |
  50%*2=1. b4 notation: e=2.718281828 | 1E5,1e5,1.5e3 = parse errors (salmon inline) |
  10^21=1×10²¹ (auto scientific w/ superscript) | e^2=7.389056099.
- [E018] 17:26 | b5 percent-resolution: 1+100+10%=111.1 (% relative to ACCUMULATED left side
  101, not immediate 100) | (100+10)%=1.1 (group postfix ÷100) | 10%^2=1 (^ binds TIGHTER than %:
  (10^2)%=100%) | 50%+50%=0.75 (0.5+50%·0.5) | 100+10%*2=100.2 (% binds TIGHTER than ×: 0.1×2)
  | 10%+1=1.1. Model: precedence ^ > % > ×/÷ > +/−; % = postfix ÷100 EXCEPT as the terminating
  right operand of +/- where it is relative to the accumulated left side. b3/b4/b5 pngs.
- [E019] 17:31 | b6 factorial: 5!=120 | 0!=1 | 2.5!=3.32335097 (gamma extension) | -4!=-24
  (unary minus looser than !) | 170!=7.257415615×10³⁰⁶ | 171!=1.24101807×10³⁰⁹ (no overflow,
  arbitrary precision). b7: 1/0, 0/0 = salmon error, divisor highlighted, NO history row |
  sqrt(-1)=i (complex) | sqrt(2)=1.414213562 (typed function names work) | log(100)=2 |
  log(8;2)=3 (semicolon separator, 2nd arg = base).
- [E020] 17:36 | b8 trig (Degrees): sin(30)=0.5, cos(60)=0.5, tan(45)=1, sin(90)=1 |
  tan(90)=ERROR with prose line below entry: "Tangent is undefined for angles that are multiples
  of π (180°) from π/2 (90°)" (history shrinks; entry text salmon w/ "90" highlighted) |
  sin(pi/2)=0.027412134 (pi numeric radians-value in Degrees mode). b9: asin(0.5)=30,
  acos(0.5)=60, atan(1)=45 (inverse returns Degrees), sinh(1)=1.17520119, round(2.5)=3,
  round(-2.5)=-3 (half away from zero). b10: floor(-2.5)=-3, ceil(-2.5)=-2, int(-2.5)=-2
  (truncate toward 0), frac(-2.5)=-0.5, floor(pi)=3, round(2.567;2)=ERROR (arity: 1 arg only).
- [E021] 17:40 | x menu: "_ = 3" (last-result variable, value updates), "rand", Constants
  categories: Mathematics / Electromagnetism / Atomic & Nuclear / Thermodynamics / Gravitation;
  bottom: "Variable name" field + assign (↓) button (custom variables). Mathematics constants:
  Euler's Number (e), Pi (π), Tau (τ), Golden Ratio (φ). Clicking a constant inserts its GLYPH
  (entry got "π"). Typed name "pi" also resolves. s1-031/032/033.
- [E022] 17:47 | Constants enumerated: Atomic & Nuclear: Planck's Constant (h), Fine-Structure
  Constant (α), Electron Mass (mₑ), Proton Mass (mₚ). Thermodynamics: Boltzmann (k), Atomic Mass
  Unit (mᵤ), Molar Gas (R), Stefan-Boltzmann (σ), Avogadro (NA). Gravitation: Gravitational
  Constant (G), Earth Acceleration (g). Popover re-anchors by list length (grows up from the x
  button, 2-item lists anchor lower). s1-038/039/040.
- [E023] 17:55 | POPOVER STICKINESS LESSON: with the x/variables popover open, rapid
  mousemove+click (no delay) outside the popover did NOT dismiss it and keys were swallowed —
  a whole batch was eaten. With mousemove;sleep;click the popover dismisses normally. batch.sh
  now sleeps 0.15 s between move and click. Tooltip discovered: C button = "Clear Display
  [Escape]". s1-041..043.
- [E024] 18:05 | KEYBOARD ROOT CAUSE #2: app.log shows 68x "IBUS-WARNING: Events queue growing
  too big, will start to drop" — the app's GTK ibus IM module connected to the USER'S REAL
  ibus-daemon (found via shared $HOME config; daemon predates the sandbox, --panel disable) and
  the context stalled, silently dropping ALL XTEST keystrokes (digits too, after ~17:50).
  Fix: relaunch app with --env=GTK_IM_MODULE=xim (launch-app.sh updated). Typing verified:
  "1+2*3=" -> row "1+2×3 = 7".
- [E025] 18:05 | PERSISTENCE OBSERVATION (incidental, to be re-tested controlled): after
  kill -9 + relaunch, history shows ONLY the 4 as-found rows + the new "1+2×3 = 7" — all ~60
  batch-session rows were LOST. Hypothesis: history is saved on orderly exit, not per-row;
  restored from last save. Also: X window id 4194309 observed on the new instance (id reuse or
  single-instance handoff — verify). b1-b11 evidence lives in screenshots/log, unaffected.
- [E026] 18:12 | b11d variables: _*2=14 (underscore = last result; echoes as "_×2") | rand=0.801359698
  | tau=6.283185307 (typed name) | phi=ERROR (no typed name for golden ratio; error region =
  dark-red box behind salmon text) | x=5 assigns variable (row "x=5 = 5") — b12: x*2=10 (user
  variable evaluates) | y=ERROR (undefined variable) | _+1=11 | tau÷2=3.141592654 | 1÷3×3=1
  EXACT (no float residue — exact/high-precision arithmetic) | 2^1024=1.797693135×10³⁰⁸ (no
  overflow). b11c/b11d/b12 pngs.
- [E027] 18:16 | Keyboard Shortcuts dialog (searchable, pill sections: General / Switching Modes
  / Programming Mode / Keyboard Entry / Equation). General: Open a New Window Ctrl+N, Close
  Current Window Ctrl+W, Quit the Application Ctrl+Q, Show Help F1, Open Menu F10, Show
  Preferences Ctrl+,, Keyboard Shortcuts Ctrl+?, Clear History Ctrl+Escape. Switching Modes:
  Ctrl+Alt+B/A/F(+P,K,C presumably). s1-048/049.
- [E028] 18:24 | Shortcuts complete. Keyboard Entry: * / (×/÷), Ctrl+2 Square ², Ctrl+R Square
  Root √, Ctrl+I Inverse ⁻¹, Ctrl+P Pi π, Ctrl+T Tau τ, Ctrl+' Degree °, Alt+[ then ] Floor ⌊⌋,
  Ctrl+[ then ] Ceiling ⌈⌉, Ctrl+E Scientific Format, Alt+0..9 Subscript digits, Ctrl+0..9
  Superscript digits, Ctrl+- Superscript minus. Equation: Ctrl+C Copy, Ctrl+V Paste, Ctrl+Z
  Undo, Ctrl+Shift+Z Redo, Alt+←/→ Previous/Next Equation. Programming: Ctrl+B/O/D/H base
  switches. s1-048..056.
- [E029] 18:28 | About dialog: app icon, "Calculator", "The GNOME Project", version chip "51.1",
  Website ↗, Report an Issue ↗, Credits >, Legal >. VERSION VIA UI = 51.1. s1-057.
- [E030] 18:35 | Ctrl+Alt+A -> Advanced: window GREW 370->~697 (mode min width), keypad carousel
  with 2 dots at 697 (dots/swipe/hwheel do NOT flip at this width); resizing window to 1000px
  reveals full 10-col grid (5 function cols + basic core): [inverse-toggle?](pink), sin cos tan
  Deg | x² sinh cosh tanh π | x^y logy log ln e | conj Arg Re Im i | x⁻¹ [x] |x| x! axb | C ↑n ↓n
  ÷ mod | 7 8 9 × % | 4 5 6 − () | 1 2 3 + = | 0 . x f(x). History card inset+rounded at wide
  width, clamped ~675px. s1-058..062.
- [E031] 18:44 | Advanced buttons: ↑n = SUPERSCRIPT power entry (5 ↑n 3 displays "5³" = 125;
  second power syntax besides ^; caret echoes literally, superscript echoes pretty) |
  ↓n = SUBSCRIPT digit entry (5 ↓n 3 -> "5₃" = parse error alone; suspected identifier/notation
  use — test pending) | TYPING-DIGIT FRESH-START RULE: typing a digit when entry shows a RESULT
  starts a new entry (t1 worked despite failed C-click); typing after an ERROR appends.
  seq.sh clear switched to Ctrl+A+Delete (C button coords are mode-dependent).
- [E032] 18:48 | Advanced: x² button -> "2²"=4 (superscript echo); x^y button inserts CARET
  syntax ("2^10"=1024, echoed literally); x⁻¹ postfix -> "4⁻¹"=0.25.
- [E033] 18:55 | |x| button inserts "|" pair: |5|=5. [x] button inserts "[" pair: [2.7]=3
  (ceiling OR round — disambiguating). Alt+[ keyboard inserts ⌊ glyph (floor-open); plain ]
  behavior unclear (got "2.7⌊⌊"-ish salmon). x! postfix = 5! = 120.
- [E034] 18:59 | [2.3]=2 AND [2.7]=3 -> [x] button brackets = ROUND to nearest integer (not
  ceiling/floor). Ctrl+[ inserted square-bracket-looking pair postfix (order mistake mine).
- [E035] 19:05 | [ ] brackets = ROUND half away from zero: [2.7]=3 [2.3]=2 [2.5]=3 [-2.5]=-3
  (same as round()). Glyph floor via Alt+[ inserted ⌊ but ⌊2.7<close> attempts errored — glyph
  bracket syntax UNRESOLVED (open question; typed floor()/ceil() work).
- [E036] 19:20 | Tooltips (Advanced): C="Clear Display [Escape]", ↑n="Superscript Mode [Ctrl]",
  ÷="Divide [/]", mod="Modulus Divide", x²="Square [Ctrl+2]", conj="Complex Conjugate",
  x⁻¹="Inverse [Ctrl+I]", axb="Factorize [Ctrl+F]" (!). mod button inserts infix "mod ".
- [E037] 19:35 | Tooltips: sinh="Hyperbolic Sine", cosh="Hyperbolic Cosine", tanh="Hyperbolic
  Tangent", cos="Cosine", tan="Tangent", ln="Natural Logarithm", log="Common Logarithm",
  logy="Logarithm", Arg="Complex Argument", |x|="Absolute Value [| |]", x!="Factorial [!]",
  %="Percentage [%]", ()="Group [()]", Deg="Angle Unit: Degrees" (live). No tooltip captured
  for sin/[x]/Re/Im/×/↓n/x^y (likely present; not rendered in sweeps). axb renders "a×b".
- [E038] 19:42 | Factorize (a×b button): 360 -> history "2×2×2×3×3×5 = 360" (equation replaced
  by ascending prime factorization, result unchanged). Deg button CLICK cycles angle unit:
  Deg->Rad (label + tooltip "Angle Unit: Radians"); cycling Deg→Rad→Grad→Deg assumed, verify
  below. s1-065/066.
- [E039] 19:47 | Angle-unit button cycles Deg→Rad→Grad→Deg; label + live tooltip update
  ("Angle Unit: Radians/Gradians/Degrees"). Advanced inventory COMPLETE.
- [E040] 19:55 | Programming tooltips: ∧="Bitwise AND", ⊻(underlined V)="Bitwise Exclusive OR",
  ¬="Bitwise NOT", <<="Left Shift [<<]", >>="Right Shift [>>]", log2="Binary Logarithm",
  swap="Swap Endianness", à-button="Insert Character Code". Bit panel bits show hover state
  (interactive). Entry row sits higher in wide programming layout (bit panel below).
- [E041] 20:05 | Programming: 12∧10=8, 12∨10=14, 12⊻10=6 (glyphs echoed), ¬5=1.844674407×10¹⁹
  (unsigned 64-bit: 0xFFFFFFFFFFFFFFFA), 1<<10=1024, 1024>>3=128, "5twos"=ERROR "Malformed
  expression" (prose line), twos 5=1.844674407×10¹⁹ (0xFFFFFFFFFFFFFFFB) — twos(x)=2⁶⁴−x vs
  ¬x=2⁶⁴−1−x (differ by one). Base readout under entry updates per result. p1..p7b pngs.
- [E042] 20:15 | Base dropdown: Binary/Octal/Decimal/Hexadecimal (Ctrl+B/O/D/H shortcuts work;
  Ctrl+H re-rendered ALL history results in hex). Word size dropdown: 8/16/32/64-bit.
  8-bit + 255+1 = 256 (NO arithmetic wraparound; word size affects bitwise domain only —
  verifying). NOTE: entry-click caret placement is position-sensitive (click x maps into text);
  ctrl+a+delete clear proved unreliable once — use C button (535,428 wide / 44,405 narrow).
- [E043] 20:24 | Word size semantics: bitwise ops masked to word size (8-bit: ¬0=255, bit panel
  low 8 bits only) while plain arithmetic is UNBOUNDED (255+1=256). Bit panel cells show INDEX
  tooltips on hover ("40"); two click attempts did not toggle any bit (display-only or
  Unknown). C-button (535,428) is the reliable wide-layout clear.
- [E044] 20:31 | Insert Character Code dialog: "Character" field + "Convert to: Unicode Code
  Point" dropdown + Cancel/Insert. 'A' + Insert -> entry "65" (code point in current base).
- [E045] 20:40 | bswap is PREFIX ("65bswap" malformed; "bswap 255"=1.837468648×10¹⁹ =
  0xFF00000000000000). FOCUS RULE: keypad button clicks steal focus from the entry; always
  re-click the entry before typing. Programming inventory COMPLETE.
- [E046] 21:00 | FINANCIAL MODE: buttons open FORM DIALOGS (Cancel/Calculate, description text,
  labeled fields w/ pencil edit icons, first field pre-focused with "0" selected). Captured:
  Ctrm="Compounding Term" (Periodic Interest Rate, Future Value, Present Value) |
  Ddb="Double-Declining Depreciati…" (Cost, Life, Period) | Gpm="Gross Profit Margin"
  (Cost, Margin) | Pmt="Periodic Payment" (Principal, Periodic Interest Rate, Term) |
  Pv="Present Value" (Periodic Payment, Periodic Interest Rate, Number of Periods) |
  Rate="Periodic Interest Rate" (Future Value, Present Value, Term) |
  Sln="Straight-Line Depreciation" (Cost, Salvage, Life) |
  Syd="Sum-of-the-Years'-Digits De…" (Cost, Salvage, Life, Period) |
  Term="Payment Period" (Periodic Payment, Future Value, Periodic Interest Rate).
  WORKED EXAMPLE Fv: -100/0.05/10 -> entry "-1257.789253555" (end-of-period annuity;
  NO history row added — dialog result fills entry). s1-093..104.

- [E047] 21:20 | ENTERING CONVERSION MODE: Financial "Currency Conversion" button switches the
  window to a 370px-wide Conversion UI (back-arrow header "Conversion ⌄" + hamburger). Layout:
  category dropdown top ("Length ⌄"), two stacked rows [large value + unit dropdown beneath],
  keypad C | ⇅ | ⌫ / 7 8 9 / 4 5 6 / 1 2 3 / 0 . (empty). NO operators, NO = — conversion is
  LIVE (target updates on every keystroke). s1-110.
- [E048] 21:22 | Categories (14): Angle, Area, Currency (✓ checkmark = current), Digital
  Storage, Duration, Energy, Frequency, Length, Mass, Number Base, Pressure, Speed,
  Temperature, Volume. Category switch resets values (Length entry: both 0, units default
  Parsecs/Parsecs). ⓘ button exists ONLY in Currency category. s1-108.
- [E049] 21:24 | BOTH value rows editable; last-edited row = SOURCE, other = live TARGET; row
  positions never move, only roles. Keypad digits insert into caret field WITHOUT needing a
  prior field click. Click C → zeroes fields, caret goes to BOTTOM field (bottom becomes
  source). Clicking into a field places caret at click x-position (mid-text edit like normal
  text entry). s1-111,116,117,122,123.
- [E050] 21:25 | Unit dropdowns: popover w/ search field (focused on open, salmon border),
  checkmark on current unit, scrollable list. Length units IN ORDER (18): Parsecs, Light
  Years, Astronomical Units, Rack Units, Nautical Miles, Miles, Kilometers, Cables, Fathoms,
  Meters, Yards, Feet, Inches, Centimeters, Millimeters, Micrometers, Nanometers, Desktop
  Publishing Points. Order mostly largest→smallest but NOT strict (Rack Units after AU;
  DPP after nm) — ordering rule Unknown. s1-112..115.
- [E051] 21:27 | Changing a row's UNIT makes that row the TARGET (other row becomes source,
  keeps its value). 1 Parsec → Fathoms: 1.687281277×10¹⁶ (10 sig digits shown, 9 after
  decimal point = as-found decimals pref; trailing zeros trimmed). pc→m 31 Parsecs =
  9.56567×10¹⁷ Meters EXACTLY — consistent w/ parsec constant 3.0857×10¹⁶ m (5 sig digits,
  Inferred; direct computation consistent). s1-116,119.
- [E052] 21:29 | ⇅ SWAP: exchanges the two unit assignments, PRESERVES the TOP field's number
  (reinterpreted in its new unit!), recomputes bottom. Quirk: physical quantity changes
  (5.23e17 Fathoms became 5.23e17 Meters). Verified twice incl. case where top number was
  itself a converted value (3.6576 = 2 fathoms→meters, then re-read as fathoms → 6.68901888).
  s1-120,121.
- [E053] 21:31 | Conversion input is FREE-FORM TEXT (no keystroke filtering): typing "-e.5"
  accepted; "-" renders as true minus (−). Unparseable source → TARGET shows bold "Error"
  (value position, no salmon styling, no prose). Error clears live when text becomes valid
  again (⌫ one char/click). s1-124,125.

- [E054] 21:40 | KEYPAD ADAPTS PER CATEGORY: Temperature keypad has "+/−" button (slot empty
  in Length). Category switch does NOT reset values: Length→Temperature carried the TARGET
  row's number into both rows (2.734033246 Celsius/Celsius); source row's "5" lost. Units
  reset to category defaults (Celsius/Celsius). Temperature units: Celsius, Fahrenheit,
  Kelvin, Rankine — NO search field in this dropdown (only 4 items; search seems size-
  conditional). s1-128,129.
- [E055] 21:41 | Temperature conversions EXACT affine: 100 C=212 F; -40 C=-40 F (crossover);
  -273.15 C=-459.67 F and =0 K exactly; -300 C=-26.85 K (BELOW absolute zero accepted, NO
  clamp, NO warning). Typed "-" renders as true minus and parses. +/− button toggles source
  sign in place (caret retained). s1-131..137.
- [E056] 21:42 | CONVERSION-MODE FOCUS FLAKINESS: after C, typed input sometimes does not
  land (100 typed → both fields stayed 0 once); reliable pattern = click field, ctrl+a,
  type. Cause Unknown (possibly focus timing after C or window focus). Also: unit-dropdown
  click works even while a field has caret; popover Escape returns focus ring to dropdown.
- [E057] 21:43 | Conversion Error display: unparseable source text → target shows bold
  "Error" (no salmon, no prose line); recovers live. Conversion arithmetic: exact decimal,
  display ≤9 decimals (decimals pref), trailing zeros trimmed, trailing "." trimmed
  ("212" not "212.0").

- [E058] 21:52 | NUMBER BASE CATEGORY: keypad becomes hex grid (5 cols: B C D E F / 6 7 8 9
  A / 1 2 3 4 5; C ⇅ ⌫ row and 0 . +/− row unchanged). Base dropdowns: Binary/Octal/Decimal/
  Hexadecimal (no search field). Fractional binary supported: -26.85 carried in as
  -11010.110110011 (minus + radix point, 9 fractional digits = decimals pref?).
  1010.1₂ = 10.5₁₀ exact. s1-139..145.
- [E059] 21:54 | KEYPAD DIGIT DIMMING follows the TOP row's base (3 observations): both
  Binary → only 0/1 enabled; top Decimal → 2-9 enabled A-F dim; top Binary + bottom
  Hexadecimal → only 0/1 enabled. Visual affordance ONLY: keyboard typing is NOT gated
  (typed "2" into binary source → "1010.12", target "Error").
- [E060] 21:55 | Popover modal grab: clicking outside an open dropdown only dismisses it
  (the outside click does not reach the button under the pointer; C tooltip appeared but C
  did not fire). Dropdown option Y-positions are ANCHOR-relative — re-verify per dropdown
  before clicking options.
- [E061] 21:56 | Changing a unit while source text is unparseable ("1010.12") resets both
  values to 0 (top Error target cannot become source). s1-147.

- [E062] 22:05 | CURRENCY CATEGORY: numeric keypad + ⓘ button (in +/− slot). Units default
  Algerian Dinar/Algerian Dinar. ⓘ opens "Exchange Rate Details" in-window dialog: one
  section per row unit (currency-name heading; card "Updated <date>" + "Source International
  Monetary Fund" with external-link icon), bottom "Refresh Rates" button. Clicking Refresh
  Rates closes the dialog. Date shown 10/30/2026 unchanged after refresh; no error, no
  visible network feedback (success-vs-cached Unknown). app.log silent. s1-148..151.
- [E063] 22:07 | Unit dropdown SEARCH: long lists (currency) show search field (auto-focused,
  clear ⊗ button when non-empty); typing "US Do" filtered to "US Dollar". Selecting it made
  top=US Dollar. 1 Algerian Dinar → 0.007379646 US Dollar (9 decimals, leading zero, no
  grouping). Live rates WORKING. s1-152,153.
- [E064] 22:08 | Currency search list omits the checkmark column (filtered single result had
  no ✓; current-unit checkmark appears in unfiltered lists).

- [E065] 22:35 | KEYBOARD MODE: NO keypad — full-height history list + bare free-form entry
  line at bottom. "2+2*2" -> row "2+2×2 = 6" (glyph normalization to real minus/times; typed
  "/" rendered "÷" in echo). Up-arrow on EMPTY entry recalls last result into entry; second
  Up/Down inert. Typing after result/recall REPLACES (fresh-start rule holds here too).
  History rows: single-click/double-click/right-click all inert (display-only; hover
  highlight + tooltip with full equation only). s1-156..163.
- [E066] 22:38 | Hamburger menu (full): New Window Ctrl+N | Clear History Ctrl+Escape |
  Result Format radios (Automatic/Fixed/Scientific/Engineering) | Preferences Ctrl+, |
  Keyboard Shortcuts Ctrl+? | Help F1 | About Calculator. Mode dropdown (title): Basic
  Ctrl+Alt+B / Advanced Ctrl+Alt+A / Financial Ctrl+Alt+F / Programming Ctrl+Alt+P /
  Keyboard Ctrl+Alt+K / Conversion Ctrl+Alt+C (radio, current dotted).
- [E067] 22:42 | RESULT FORMATS (live re-render ALL history results on switch; equation
  echoes NOT re-rendered — stored text): Automatic: plain until large magnitude then
  scientific (e.g. 1.844674407×10¹⁹; 61917364224 was plain). Fixed: NEVER scientific —
  wide integers truncate with "…" (1844674407370...). Scientific: mantissa(≤9 digits via
  decimals pref)×10^n; SMALL-INTEGER EXEMPTION: 9 and 6 stay plain, 99 -> 9.9×10¹ (boundary
  between 10-98 Unknown). Engineering: exponent multiple of 3 (1.23e-5 -> 12.3×10⁻⁶,
  bswap -> 18.37468648×10¹⁸); magnitudes in [0.001,1000) stay plain (333.333333333, 128).
- [E068] 22:45 | Preferences dialog (in-window, General): Number of Decimals stepper (−/+),
  Trailing Zeroes toggle, Group Digits toggle, Angle Units dropdown (Degrees), Word Size
  dropdown (64-bit); Currency Conversion section: Exchange Rate Refresh Interval (Weekly),
  Currency Display (Currency Code/Currency Name✓), Show Currencies in Auto-Completion List
  (All✓; other options CLIPPED below 660px window edge — popover doesn't scroll/flip;
  maximize is a NO-OP because the session has no WM; options beyond All UNVERIFIED).
- [E069] 22:50 | FORMATTING BATTERY (Trailing ON + decimals 6 + Grouping ON): trailing
  zeroes pad plain AND scientific mantissa to exactly `decimals` digits (6.000000,
  1.230000×10⁻⁵); decimals caps mantissa (3.333333×10²) AND plain (333.333333); decimals
  truncation can destroy significance (0.0000123 -> 0.000012). Grouping: integer part
  separators (1,234,567.000000) AND applies to typed-input echo in history ("1,234,567"
  row) but NOT to rows whose equation was entered before the pref change (1000÷3 stayed
  ungrouped) — equations are stored text, results re-render live. Entry shows the
  formatted result string after Return ("5.000000|").
- [E070] 22:52 | Financial-mode window at 700px shows history panel w/ clear-history
  (⌫-icon) button bottom-right of the panel; undo ↩ (filled circle) + redo ↪ (disabled)
  in header left. Conversion back-arrow in same position (‹) is mode-dependent header.
  History SHARED across modes (programming rows visible in Keyboard/Financial), survives
  mode switches.
- [E071] 22:55 | Prefs restored to as-found (decimals 9, Trailing OFF, Group OFF, Automatic,
  Degrees, 64-bit). NOTE decimals stepper "+" at (582,162) in 700-wide prefs (clicking
  (605,162) misses).
- [E072] 23:05 | AUTO-COMPLETION: typing a currency code in Keyboard entry ("US D") opens
  inline completion popover listing matching codes (DKK, DZD seen) with detail line
  "Danish Krone - kr"; Tab accepts highlighted suggestion (entry becomes "US DKK|");
  popover closes on acceptance and on Escape (Escape leaves typed text). Works in
  Keyboard-mode free entry; popover floats above entry, anchored to text position.
- [E073] 23:06 | CURRENCY EXPRESSION in Keyboard mode: "10 DKK in USD" + Return → history
  row "10 DKK in USD = 1.494877167" (9 decimals, Automatic format, as-found prefs);
  result is a NUMBER (not formatted as currency with symbol); rate implies
  1 USD ≈ 6.68954 DKK at test time. NOTE: first Return didn't register (input flake,
  second Return worked — see E056 pattern).
- [E074] 23:10 | CHARACTER-LEVEL UNDO/REDO in Keyboard mode: header ↩/↪ (↩ filled=enabled,
  ↪ pale=disabled at start). After typing "224" and pressing ↩, last character removed;
  repeated ↩ untypes further characters; ↪ re-applies. Undoes EDIT actions (typing,
  deletion), NOT evaluated history. No undo of history clear observed (undo disabled
  after clear? — UNVERIFIED: not probed explicitly).
- [E075] 23:12 | CLEAR HISTORY: Ctrl+Escape in Keyboard mode wipes ALL history rows
  INSTANTLY, NO confirmation dialog. History panel shows empty. (Equation/result rows
  unrecoverable except via ↩? — UNVERIFIED whether undo restores cleared rows.)
- [E076] 23:14 | CLIPBOARD ROUND-TRIP: select-all (ctrl+a) + copy (ctrl+c) in entry puts
  entry text on clipboard; paste (ctrl+v) replaces selection with clipboard content
  (verified: entry "10 DKK in USD" copied, typed "999", selected, pasted → entry shows
  "10 DKK in USD"). Also paste into empty entry works. No paste-special/dialog.
- [E077] 23:58 | PRE-CLOSE STATE (persistence setup): Keyboard mode, 700px window, history
  exactly 2 rows ("111+1 = 112", "222+2 = 224"), entry "224|" cursor at end. Screenshot
  crops/pre-close2.png. (Earlier state wiped via Ctrl+Escape E075 before adding rows.)
- [E078] 00:05 | GRACEFUL-CLOSE PERSISTENCE: window × clicked (672,28) → app exited
  (verified no bwrap/gnome-calculator process; note pgrep -f self-match pitfall: inline
  zsh -c command line matches its own pattern — check with pgrep -af | grep -v zsh).
  Relaunch via launch-app.sh → restored: Keyboard mode ✓, history rows "111+1 = 112"
  and "222+2 = 224" ✓ (both, in order), entry EMPTY ✗ (in-progress "224" NOT restored,
  just cursor). Screenshot crops/restart-small.png + restart-entry.png. CONTRAST: earlier
  kill -9 test LOST history (session memory). So: mode+history persist on graceful exit;
  pending entry text does not; hard kill loses history (persistence timing likely
  shutdown-triggered write — INFERRED mechanism, observed behavior only).

- [E079] 00:40 | COLOR-SCHEME LIVE SWITCH (both directions): setting the app's own
  interface color-scheme (gsettings keyfile store) prefer-light → whole UI re-themes
  INSTANTLY, no restart (Keyboard, history, entry all light). prefer-dark → back to
  dark instantly. Setting 'default' live → renders LIGHT (at original launch the app
  rendered dark under the same 'default' keyfile value — dark likely came from the
  system Settings portal at launch; live 'default' resolves light. Source-resolution
  order UNKNOWN). Light palette (sampled): header/body contrast #ffffff vs #f2f2f2,
  history rows #ffffff on #f2f2f2, selection bg #f3d0e1 w/ text #644155,
  error text/messages #c30000, error token highlight #f6bbbe, keypad buttons #e6e6e7
  (hover #c8c8ca, press #b4b4b7), keypad digit-class buttons darker #d2d2d4, "="
  double-height accent #d56199 w/ white text, prefs focus ring pinkish #d199b6,
  search-field focus ring #c78fac (accent family), dimmed keypad text #929295, active
  base label #333338. About dialog: white sheet, version badge pink pill, hover on
  "Website" row shows tooltip w/ full URL https://apps.gnome.org/Calculator.
  Screenshots light-*.png + crops/light-*.
- [E080] 00:47 | WINDOW WIDTH PER MODE (measured by right-edge scan): Basic 370,
  Conversion 370, Keyboard 700, Advanced 700 (all four observed switching FROM a
  differently-sized mode → mode forces its width); Financial/Programming observed at
  700 (reached from 700-wide modes; forcing direction INFERRED by pattern). Window
  resizes automatically on every mode switch; height unchanged (~660 incl. decorations).
  MODE MENU (light): options Basic/Advanced/Financial/Programming/Keyboard/Conversion
  w/ radio selected, accelerators Ctrl+Alt+B/A/F/P/K/C, hover = gray rounded row.
- [E081] 00:52 | ACCIDENTAL GRACEFUL CLOSE #2 (click hit narrow-window × — narrow
  window header: mode button ~(97..150,28), ☰ (228,28), − (266,28), □ (305,28),
  × (341,28)): relaunch restored Conversion mode, category Currency, source unit
  US Dollar, target Algerian Dinar, AND the ~370px window width + light/dark scheme
  choice. → persistence covers mode, category, units, window size, color scheme.
  History rows persisted too (111+1, 222+2). NOTE: 700px-window mode-button anchor
  (344,28) is the × button in the 370px window — header anchors are width-relative.
- [E082] 01:40 | DELIVERABLE PACKAGE WRITTEN: docs/INDEX.md + docs 00–15 (16
  deliverables) + docs/assets/ (21 curated screenshots). Path-free docs 00, 02–09
  (evidence citations stripped; labels only); citations retained in 01, 10–15 per
  handoff convention. Leak audit clean (no E###/paths/tool names in 00, 02–09).
  Fv field-label mislabel caught and fixed in docs 02/04/12 (recorded output
  −1257.789253555 matches ordinary-annuity Fv with −100 as the PERIODIC PAYMENT,
  not present value; field labels for that dialog were not individually recorded).
- [E083] 01:45 | STATE RESTORATION (end of session): history cleared (Ctrl+Escape)
  and the 4 as-found rows re-entered via Keyboard mode ("2+2", "5/6",
  "0.8333333333*111", "3*111+(23/60)*111" each + Return, entry ctrl+a+Delete
  between). Mode restored to Basic (370px), scheme keyfile value back to as-found
  'default' (renders dark). Full state backup taken first:
  notes/app-state-backup-final (copy of ~/.var/app/org.gnome.Calculator).
  Final-state screenshot crops/final-state-small.png — narrow-window result
  truncation reproduces the as-found "0.8333…"/"92.499…" echoes naturally.
- [E084] 01:48 | HARNESS FLAKE OBSERVATION (restoration attempts 1–2, not app
  behavior): typing "<expr>=" then Return sometimes produced extra junk rows of
  the form "<result echo>+<partial operand>" (e.g. "375.55+42.55 = 418.1");
  with trailing "=" removed and explicit ctrl+a+Delete between solves, four
  consecutive solves were clean. Attribution: synthetic-input flake
  (first-keystroke loss after focus transitions, E056 pattern) interacting with
  the fresh-start/append rules — NOT claimed as app behavior. Practical rule:
  never end automated typed input with '='.
