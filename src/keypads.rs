//! Keypad construction: the button grids per mode (docs/02 §8, docs/16 §2).
//!
//! Builders return the grid plus explicit handles; the window owns all
//! behavior wiring (insert/evaluate/popovers/mode buttons).
//!
//! Layouts follow the owner-provided screenshots of the original
//! (notes/evidence-2026-11-04, 2026-11-04), which supersede the carousel
//! model of docs/02 §8.2–8.4: at wide width each mode is a single full grid
//! with the function columns LEFT of the digit block (docs/16 §2).

use std::cell::Cell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{Grid, MenuButton};

/// A keypad button description.
pub struct Btn {
    pub label: &'static str,
    pub tooltip: &'static str,
    /// Token inserted at the caret (None for behavior buttons).
    pub token: Option<&'static str>,
    /// Move the caret back N chars after insert (pair buttons).
    pub caret_back: usize,
    pub css: &'static str,
}

const fn ins(label: &'static str, tooltip: &'static str, token: &'static str) -> Btn {
    Btn { label, tooltip, token: Some(token), caret_back: 0, css: "radix-btn" }
}
const fn digit(label: &'static str) -> Btn {
    Btn { label, tooltip: "", token: Some(label), caret_back: 0, css: "radix-btn digit" }
}
const fn sp(label: &'static str, tooltip: &'static str) -> Btn {
    Btn { label, tooltip, token: None, caret_back: 0, css: "radix-btn" }
}
const fn pair(label: &'static str, tooltip: &'static str, token: &'static str) -> Btn {
    Btn { label, tooltip, token: Some(token), caret_back: 1, css: "radix-btn" }
}

/// A button whose label and token swap when the ⇧⁻¹ toggle is active.
#[derive(Clone)]
pub struct Invertible {
    pub button: gtk::Button,
    pub direct: (&'static str, &'static str),
    pub inverse: (&'static str, &'static str),
    pub token_cell: Rc<Cell<&'static str>>,
}

/// Handles into a keypad grid.
#[derive(Default)]
pub struct PadHandles {
    pub clear: Option<gtk::Button>,
    pub equals: Option<gtk::Button>,
    pub variables: Option<MenuButton>,
    pub functions: Option<MenuButton>,
    pub super_mode: Option<gtk::Button>,
    pub sub_mode: Option<gtk::Button>,
    pub angle_cycle: Option<gtk::Button>,
    pub char_code: Option<gtk::Button>,
    /// (button, current token, caret-back) triples for token buttons.
    pub tokens: Vec<(gtk::Button, Rc<Cell<&'static str>>, usize)>,
    /// Contextual slot buttons in conversion keypads (swap/clear/backspace,
    /// plus/minus, info).
    pub swap: Option<gtk::Button>,
    pub backspace: Option<gtk::Button>,
    pub context_slot: Option<gtk::Button>,
    /// Financial formula buttons: (button, formula id).
    pub formulas: Vec<(gtk::Button, &'static str)>,
    /// ⇧⁻¹ inverse-functions toggle (wide pads).
    pub inverse_toggle: Option<gtk::ToggleButton>,
    /// Buttons that swap label/token with the toggle.
    pub invertibles: Vec<Invertible>,
    /// Financial "Currency Conversion" mode button.
    pub currency_conv: Option<gtk::Button>,
    /// Programming digit buttons: (button, digit value) for base dimming.
    pub base_digits: Vec<(gtk::Button, u32)>,
}

/// Keypad button height: 44 px measured on every pad of the original
/// (2026-11-04 20:1x batch) EXCEPT Programming, whose 5-row pad renders
/// 53 px buttons in both evidence batches (see `PROGRAMMING_H`).
pub const BTN_H: i32 = 44;
pub const PROGRAMMING_H: i32 = 53;

fn mk(btn: &Btn, handles: &mut PadHandles) -> gtk::Button {
    let b = gtk::Button::new();
    b.set_label(btn.label);
    if !btn.tooltip.is_empty() {
        b.set_tooltip_text(Some(btn.tooltip));
    }
    for c in btn.css.split_whitespace() {
        b.add_css_class(c);
    }
    b.set_hexpand(true);
    b.set_vexpand(true);
    b.set_height_request(BTN_H);
    match btn.label {
        "C" if handles.clear.is_none() => {
            handles.clear = Some(b.clone());
        }
        "=" => {
            handles.equals = Some(b.clone());
        }
        "↑n" => {
            handles.super_mode = Some(b.clone());
        }
        "↓n" => {
            handles.sub_mode = Some(b.clone());
        }
        "Deg" => {
            handles.angle_cycle = Some(b.clone());
        }
        "á" => {
            handles.char_code = Some(b.clone());
        }
        _ => {}
    }
    if let Some(tok) = btn.token {
        handles
            .tokens
            .push((b.clone(), Rc::new(Cell::new(tok)), btn.caret_back));
    }
    if let Ok(v) = u32::from_str_radix(btn.label, 36) {
        if btn.css.contains("digit") {
            handles.base_digits.push((b.clone(), v));
        }
    }
    b
}

fn pad_grid() -> Grid {
    // Measured off the original (2026-11-04 20:1x batch): 4px row and column
    // gaps, 44px buttons (pitch 48); the 12px window margins come from the
    // keypad stack.
    Grid::builder()
        .row_spacing(4)
        .column_spacing(4)
        .column_homogeneous(true)
        .build()
}

/// The pink ⇧⁻¹ inverse-functions toggle (first button of every wide pad).
fn inverse_toggle(handles: &mut PadHandles) -> gtk::ToggleButton {
    let t = gtk::ToggleButton::builder()
        .label("⇧⁻¹")
        .tooltip_text("Inverse Functions")
        .css_classes(["radix-btn", "inverse-toggle"])
        .hexpand(true)
        .vexpand(true)
        .height_request(BTN_H)
        .build();
    handles.inverse_toggle = Some(t.clone());
    t
}

/// A keypad row box: fixed-width buttons with the measured 4px gaps.
/// The original's Programming/Financial pads mix 64px buttons with narrow
/// (37px hex) and wide (109/132/221px) ones per row — no single GtkGrid
/// column template can express that, so each row is its own box.
fn pad_row(spacing: i32) -> gtk::Box {
    gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(spacing)
        .hexpand(true)
        .build()
}

fn fixed_btn<W: gtk::prelude::IsA<gtk::Widget>>(b: &W, w: i32, h: i32) {
    b.set_hexpand(false);
    b.set_vexpand(false);
    b.set_width_request(w);
    b.set_height_request(h);
    b.set_halign(gtk::Align::Fill);
}

/// An insert button registered as invertible (⇧⁻¹ swaps label + token).
fn inv_btn(
    handles: &mut PadHandles,
    direct: (&'static str, &'static str),
    inverse: (&'static str, &'static str),
    tooltip: &'static str,
) -> gtk::Button {
    let b = mk(&ins(direct.0, tooltip, direct.1), handles);
    let token_cell = handles
        .tokens
        .iter()
        .find(|(btn, _, _)| btn == &b)
        .map(|(_, cell, _)| cell.clone())
        .expect("inv_btn token registered");
    handles.invertibles.push(Invertible { button: b.clone(), direct, inverse, token_cell });
    b
}

/// Menu buttons occupying the x / f(x) slots (no behavior here).
fn menu_button(label: &str, handles: &mut PadHandles, vars: bool) -> MenuButton {
    let b = MenuButton::builder().label(label).hexpand(true).vexpand(true).height_request(BTN_H).build();
    b.add_css_class("radix-btn");
    if vars {
        handles.variables = Some(b.clone());
    } else {
        handles.functions = Some(b.clone());
    }
    b
}

/// Basic-mode grid (docs/02 §8.1; matches the owner screenshots exactly).
pub fn basic() -> (Grid, PadHandles) {
    let mut handles = PadHandles::default();
    let grid = pad_grid();
    let rows: [[Btn; 4]; 5] = [
        [
            sp("C", "Clear Display [Escape]"),
            sp("↑n", "Superscript Mode [Ctrl]"),
            sp("↓n", "Subscript Mode"),
            ins("÷", "Divide [/]", "÷"),
        ],
        [
            digit("7"),
            digit("8"),
            digit("9"),
            ins("×", "Multiply [*]", "×"),
        ],
        [
            digit("4"),
            digit("5"),
            digit("6"),
            ins("−", "Subtract [-]", "−"),
        ],
        [
            digit("1"),
            digit("2"),
            digit("3"),
            ins("+", "Add [+]", "+"),
        ],
        [digit("0"), ins(".", "Decimal Point [.]", "."), sp("", ""), sp("", "")],
    ];
    // Column 4: mod % ( ) = and row 5: x f(x) — assembled explicitly to keep
    // the docs/02 §8.1 shape (5 columns, = spanning rows 4–5).
    let col4: [Btn; 5] = [
        ins("mod", "Modulus Divide", "mod "),
        ins("%", "Percentage [%]", "%"),
        pair("( )", "Group [()]", "()"),
        Btn { label: "=", tooltip: "Calculate [=]", token: None, caret_back: 0, css: "radix-btn equals" },
        sp("", ""),
    ];
    for (r, row) in rows.iter().enumerate() {
        for (c, btn) in row.iter().enumerate() {
            if btn.label.is_empty() {
                continue;
            }
            let b = mk(btn, &mut handles);
            grid.attach(&b, c as i32, r as i32, 1, 1);
        }
    }
    for (r, btn) in col4.iter().enumerate() {
        if btn.label.is_empty() {
            continue; // the = bottom half occupies row 5 col 4 — no filler
        }
        let rowspan = if btn.label == "=" { 2 } else { 1 };
        let b = mk(btn, &mut handles);
        grid.attach(&b, 4, r as i32, 1, rowspan);
    }
    // x and f(x) in row 5, columns 2–3.
    let xv = menu_button("x", &mut handles, true);
    let fx = menu_button("f(x)", &mut handles, false);
    grid.attach(&xv, 2, 4, 1, 1);
    grid.attach(&fx, 3, 4, 1, 1);
    (grid, handles)
}

/// Advanced mode: one 10-column grid, function columns left of the digit
/// block (docs/16 §2; owner evidence 2026-11-04 — no carousel paging).
pub fn advanced() -> (Grid, PadHandles) {
    let mut handles = PadHandles::default();
    let grid = pad_grid();

    let inv = inverse_toggle(&mut handles);
    grid.attach(&inv, 0, 0, 1, 1);
    let r0 = [
        ins("x²", "Square [Ctrl+2]", "²"),
        ins("xʸ", "Power", "^"),
        ins("conj", "Complex Conjugate", "conj("),
        ins("x⁻¹", "Inverse [Ctrl+I]", "⁻¹"),
        sp("C", "Clear Display [Escape]"),
        sp("↑n", "Superscript Mode [Ctrl]"),
        sp("↓n", "Subscript Mode"),
        ins("÷", "Divide [/]", "÷"),
        ins("mod", "Modulus Divide", "mod "),
    ];
    for (c, btn) in r0.iter().enumerate() {
        let b = mk(btn, &mut handles);
        grid.attach(&b, (c + 1) as i32, 0, 1, 1);
    }

    // Row 1: sin sinh logᵧ Arg [x] | 7 8 9 × %
    let sin = inv_btn(&mut handles, ("sin", "sin("), ("sin⁻¹", "asin("), "Sine");
    let sinh = inv_btn(&mut handles, ("sinh", "sinh("), ("sinh⁻¹", "asinh("), "Hyperbolic Sine");
    let logy = mk(&ins("logᵧ", "Logarithm log(x;base)", "log("), &mut handles);
    let arg = mk(&ins("Arg", "Complex Argument", "arg("), &mut handles);
    let round = mk(&pair("[x]", "Round to Nearest Integer [x]", "[]"), &mut handles);
    grid.attach(&sin, 0, 1, 1, 1);
    grid.attach(&sinh, 1, 1, 1, 1);
    grid.attach(&logy, 2, 1, 1, 1);
    grid.attach(&arg, 3, 1, 1, 1);
    grid.attach(&round, 4, 1, 1, 1);
    for (c, d) in ["7", "8", "9"].iter().enumerate() {
        let b = mk(&digit(d), &mut handles);
        grid.attach(&b, (c + 5) as i32, 1, 1, 1);
    }
    let mul = mk(&ins("×", "Multiply [*]", "×"), &mut handles);
    let pct = mk(&ins("%", "Percentage [%]", "%"), &mut handles);
    grid.attach(&mul, 8, 1, 1, 1);
    grid.attach(&pct, 9, 1, 1, 1);

    // Row 2: cos cosh log Re |x| | 4 5 6 − ( )
    let cos = inv_btn(&mut handles, ("cos", "cos("), ("cos⁻¹", "acos("), "Cosine");
    let cosh = inv_btn(&mut handles, ("cosh", "cosh("), ("cosh⁻¹", "acosh("), "Hyperbolic Cosine");
    let log = inv_btn(&mut handles, ("log", "log("), ("10ˣ", "10^("), "Common Logarithm");
    let re = mk(&ins("Re", "Real Part", "re("), &mut handles);
    let abs = mk(&pair("|x|", "Absolute Value [| |]", "||"), &mut handles);
    grid.attach(&cos, 0, 2, 1, 1);
    grid.attach(&cosh, 1, 2, 1, 1);
    grid.attach(&log, 2, 2, 1, 1);
    grid.attach(&re, 3, 2, 1, 1);
    grid.attach(&abs, 4, 2, 1, 1);
    for (c, d) in ["4", "5", "6"].iter().enumerate() {
        let b = mk(&digit(d), &mut handles);
        grid.attach(&b, (c + 5) as i32, 2, 1, 1);
    }
    let sub = mk(&ins("−", "Subtract [-]", "−"), &mut handles);
    let grp = mk(&pair("( )", "Group [()]", "()"), &mut handles);
    grid.attach(&sub, 8, 2, 1, 1);
    grid.attach(&grp, 9, 2, 1, 1);

    // Row 3: tan tanh ln Im x! | 1 2 3 + [=]
    let tan = inv_btn(&mut handles, ("tan", "tan("), ("tan⁻¹", "atan("), "Tangent");
    let tanh = inv_btn(&mut handles, ("tanh", "tanh("), ("tanh⁻¹", "atanh("), "Hyperbolic Tangent");
    let ln = inv_btn(&mut handles, ("ln", "ln("), ("eˣ", "e^("), "Natural Logarithm");
    let im = mk(&ins("Im", "Imaginary Part", "im("), &mut handles);
    let fact = mk(&ins("x!", "Factorial [!]", "!"), &mut handles);
    grid.attach(&tan, 0, 3, 1, 1);
    grid.attach(&tanh, 1, 3, 1, 1);
    grid.attach(&ln, 2, 3, 1, 1);
    grid.attach(&im, 3, 3, 1, 1);
    grid.attach(&fact, 4, 3, 1, 1);
    for (c, d) in ["1", "2", "3"].iter().enumerate() {
        let b = mk(&digit(d), &mut handles);
        grid.attach(&b, (c + 5) as i32, 3, 1, 1);
    }
    let add = mk(&ins("+", "Add [+]", "+"), &mut handles);
    grid.attach(&add, 8, 3, 1, 1);
    let eq = mk(
        &Btn { label: "=", tooltip: "Calculate [=]", token: None, caret_back: 0, css: "radix-btn equals" },
        &mut handles,
    );
    grid.attach(&eq, 9, 3, 1, 2);

    // Row 4: Deg π e i a×b | 0 . x⌄ f(x)⌄
    let deg = mk(&sp("Deg", "Angle Unit: Degrees"), &mut handles);
    let pi = mk(&ins("π", "Pi [Ctrl+P]", "π"), &mut handles);
    let e = mk(&ins("e", "Euler's Number", "e"), &mut handles);
    let i = mk(&ins("i", "Imaginary Unit", "i"), &mut handles);
    let axb = mk(&ins("a×b", "Factorize [Ctrl+F]", "fact("), &mut handles);
    grid.attach(&deg, 0, 4, 1, 1);
    grid.attach(&pi, 1, 4, 1, 1);
    grid.attach(&e, 2, 4, 1, 1);
    grid.attach(&i, 3, 4, 1, 1);
    grid.attach(&axb, 4, 4, 1, 1);
    let zero = mk(&digit("0"), &mut handles);
    let dot = mk(&ins(".", "Decimal Point [.]", "."), &mut handles);
    grid.attach(&zero, 5, 4, 1, 1);
    grid.attach(&dot, 6, 4, 1, 1);
    let xv = menu_button("x", &mut handles, true);
    let fx = menu_button("f(x)", &mut handles, false);
    grid.attach(&xv, 7, 4, 1, 1);
    grid.attach(&fx, 8, 4, 1, 1);
    (grid, handles)
}

/// Financial mode: per-row boxes — the formula buttons render 109px wide
/// against 64px digit buttons in the original (2026-11-04 20:17:34 batch),
/// which no uniform grid expresses. Ten formula buttons open form dialogs
/// (docs/02 §9); "Currency Conversion" jumps to the currency pane.
pub fn financial() -> (gtk::Box, PadHandles) {
    let mut handles = PadHandles::default();
    let pad = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(4)
        .build();

    // Row 1: ten equal 64px buttons — ⇧⁻¹ x² xʸ log ln C ↑n ↓n ÷ mod.
    let r0 = pad_row(4);
    let inv = inverse_toggle(&mut handles);
    fixed_btn(&inv, 64, BTN_H);
    r0.append(&inv);
    let x2 = inv_btn(&mut handles, ("x²", "²"), ("√", "√("), "Square [Ctrl+2]");
    fixed_btn(&x2, 64, BTN_H);
    r0.append(&x2);
    let xw = mk(&ins("xʸ", "Power", "^"), &mut handles);
    fixed_btn(&xw, 64, BTN_H);
    r0.append(&xw);
    let log = inv_btn(&mut handles, ("log", "log("), ("10ˣ", "10^("), "Common Logarithm");
    fixed_btn(&log, 64, BTN_H);
    r0.append(&log);
    let ln = inv_btn(&mut handles, ("ln", "ln("), ("eˣ", "e^("), "Natural Logarithm");
    fixed_btn(&ln, 64, BTN_H);
    r0.append(&ln);
    let clear = mk(&sp("C", "Clear Display [Escape]"), &mut handles);
    fixed_btn(&clear, 64, BTN_H);
    r0.append(&clear);
    let up = mk(&sp("↑n", "Superscript Mode [Ctrl]"), &mut handles);
    fixed_btn(&up, 64, BTN_H);
    r0.append(&up);
    let down = mk(&sp("↓n", "Subscript Mode"), &mut handles);
    fixed_btn(&down, 64, BTN_H);
    r0.append(&down);
    let div = mk(&ins("÷", "Divide [/]", "÷"), &mut handles);
    fixed_btn(&div, 64, BTN_H);
    r0.append(&div);
    let md = mk(&ins("mod", "Modulus Divide", "mod "), &mut handles);
    fixed_btn(&md, 64, BTN_H);
    r0.append(&md);
    pad.append(&r0);

    // Rows 2-5: 109px formula buttons + 64px digits; = spans rows 4-5 in
    // its own 64px column beside the row4/row5 stack.
    let lower = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(4)
        .hexpand(true)
        .build();
    let rows45 = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(4)
        .hexpand(true)
        .build();

    let formula_row = |row: &gtk::Box, names: [&'static str; 3],
                           handles: &mut PadHandles| {
        for name in names {
            let b = mk(&sp(name, name), handles);
            b.set_tooltip_text(Some(name));
            handles.formulas.push((b.clone(), name));
            fixed_btn(&b, 109, BTN_H);
    row.append(&b);
        }
    };

    let r1 = pad_row(4);
    formula_row(&r1, ["Ctrm", "Ddb", "Fv"], &mut handles);
    for d in ["7", "8", "9"] {
        let b = mk(&digit(d), &mut handles);
        fixed_btn(&b, 64, BTN_H);
    r1.append(&b);
    }
    let mul = mk(&ins("×", "Multiply [*]", "×"), &mut handles);
    fixed_btn(&mul, 64, BTN_H);
    r1.append(&mul);
    let pct = mk(&ins("%", "Percentage [%]", "%"), &mut handles);
    fixed_btn(&pct, 64, BTN_H);
    r1.append(&pct);

    let r2 = pad_row(4);
    formula_row(&r2, ["Gpm", "Pmt", "Pv"], &mut handles);
    for d in ["4", "5", "6"] {
        let b = mk(&digit(d), &mut handles);
        fixed_btn(&b, 64, BTN_H);
    r2.append(&b);
    }
    let sub = mk(&ins("−", "Subtract [-]", "−"), &mut handles);
    fixed_btn(&sub, 64, BTN_H);
    r2.append(&sub);
    let grp = mk(&pair("( )", "Group [()]", "()"), &mut handles);
    fixed_btn(&grp, 64, BTN_H);
    r2.append(&grp);

    let r3 = pad_row(4);
    formula_row(&r3, ["Rate", "Sln", "Syd"], &mut handles);
    for d in ["1", "2", "3"] {
        let b = mk(&digit(d), &mut handles);
        fixed_btn(&b, 64, BTN_H);
    r3.append(&b);
    }
    let add = mk(&ins("+", "Add [+]", "+"), &mut handles);
    fixed_btn(&add, 64, BTN_H);
    r3.append(&add);

    // Row 5: Term 109 · Currency Conversion 221 · 0 . x f(x) 64.
    let r4 = pad_row(4);
    let term = mk(&sp("Term", "Term"), &mut handles);
    term.set_tooltip_text(Some("Term"));
    handles.formulas.push((term.clone(), "Term"));
    fixed_btn(&term, 109, BTN_H);
    r4.append(&term);
    let cc = gtk::Button::builder()
        .label("Currency Conversion")
        .css_classes(["radix-btn"])
        .height_request(BTN_H)
        .build();
    handles.currency_conv = Some(cc.clone());
    fixed_btn(&cc, 221, BTN_H);
    r4.append(&cc);
    let zero = mk(&digit("0"), &mut handles);
    fixed_btn(&zero, 64, BTN_H);
    r4.append(&zero);
    let dot = mk(&ins(".", "Decimal Point [.]", "."), &mut handles);
    fixed_btn(&dot, 64, BTN_H);
    r4.append(&dot);
    let xv = menu_button("x", &mut handles, true);
    xv.set_width_request(64);
    xv.set_height_request(BTN_H);
    xv.set_hexpand(false);
    r4.append(&xv);
    let fx = menu_button("f(x)", &mut handles, false);
    fx.set_width_request(64);
    fx.set_height_request(BTN_H);
    fx.set_hexpand(false);
    r4.append(&fx);

    // Rows 2-3 span the full pad width (no = column beside them); only
    // rows 4-5 share the tall equals.
    pad.append(&r1);
    pad.append(&r2);
    rows45.append(&r3);
    rows45.append(&r4);
    lower.append(&rows45);
    let eq = mk(
        &Btn { label: "=", tooltip: "Calculate [=]", token: None, caret_back: 0, css: "radix-btn equals" },
        &mut handles,
    );
    eq.set_hexpand(false);
    eq.set_width_request(64);
    eq.set_height_request(BTN_H * 2 + 4);
    eq.set_vexpand(false);
    lower.append(&eq);

    pad.append(&lower);
    (pad, handles)
}


/// Programming mode: per-row boxes — hex digit buttons render 36px wide
/// against 64px function buttons (2026-11-04 20:17:09 + old-batch evidence,
/// identical in both), and the pad's buttons are 53px tall (every other pad
/// is 44px). Rows 1-3 are independent; rows 4-5 share the 2-row `=`.
pub fn programming(base_btn: &MenuButton, word_btn: &MenuButton) -> (gtk::Box, PadHandles) {
    let mut handles = PadHandles::default();
    let pad = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(4)
        .build();
    let h = PROGRAMMING_H;

    // Row 1: ⇧⁻¹ x² xʸ twos [x] C ↑n ↓n ÷ mod — ten 64px buttons; the C
    // renders bold like the digits in the original.
    let r0 = pad_row(4);
    let inv = inverse_toggle(&mut handles);
    fixed_btn(&inv, 64, h);
    r0.append(&inv);
    let x2 = inv_btn(&mut handles, ("x²", "²"), ("√", "√("), "Square [Ctrl+2]");
    fixed_btn(&x2, 64, h);
    r0.append(&x2);
    let xw = mk(&ins("xʸ", "Power", "^"), &mut handles);
    fixed_btn(&xw, 64, h);
    r0.append(&xw);
    let twos = mk(&ins("twos", "Two's Complement", "twos("), &mut handles);
    fixed_btn(&twos, 64, h);
    r0.append(&twos);
    let round = mk(&pair("[x]", "Round to Nearest Integer [x]", "[]"), &mut handles);
    fixed_btn(&round, 64, h);
    r0.append(&round);
    let clear = mk(&sp("C", "Clear Display [Escape]"), &mut handles);
    fixed_btn(&clear, 64, h);
    clear.add_css_class("bold-label");
    r0.append(&clear);
    let up = mk(&sp("↑n", "Superscript Mode [Ctrl]"), &mut handles);
    fixed_btn(&up, 64, h);
    r0.append(&up);
    let down = mk(&sp("↓n", "Subscript Mode"), &mut handles);
    fixed_btn(&down, 64, h);
    r0.append(&down);
    let div = mk(&ins("÷", "Divide [/]", "÷"), &mut handles);
    fixed_btn(&div, 64, h);
    r0.append(&div);
    let md = mk(&ins("mod", "Modulus Divide", "mod "), &mut handles);
    fixed_btn(&md, 64, h);
    r0.append(&md);
    pad.append(&r0);

    // Rows 2-3: five 64px ops, five 36px base digits, two 64px operators.
    let op_digit_row = |ops: [gtk::Button; 5], digits: [&'static str; 5],
                        tail: Vec<gtk::Button>, handles: &mut PadHandles| {
        let row = pad_row(4);
        for b in ops {
            fixed_btn(&b, 64, h);
            row.append(&b);
        }
        for d in digits {
            let b = mk(&digit(d), handles);
            fixed_btn(&b, 36, h);
            row.append(&b);
        }
        for b in tail {
            fixed_btn(&b, 64, h);
            row.append(&b);
        }
        row
    };
    let and = mk(&ins("∧", "Bitwise AND", "∧"), &mut handles);
    let or = mk(&ins("∨", "Bitwise OR", "∨"), &mut handles);
    let logy = mk(&ins("logᵧ", "Logarithm log(x;base)", "log("), &mut handles);
    let swap = mk(&ins("swap", "Swap Endianness", "bswap("), &mut handles);
    let round2 = mk(&pair("[x]", "Round to Nearest Integer [x]", "[]"), &mut handles);
    let mul = mk(&ins("×", "Multiply [*]", "×"), &mut handles);
    let pct = mk(&ins("%", "Percentage [%]", "%"), &mut handles);
    let r1 = op_digit_row(
        [and, or, logy, swap, round2],
        ["B", "C", "D", "E", "F"],
        vec![mul, pct],
        &mut handles,
    );
    pad.append(&r1);

    let not = mk(&ins("¬", "Bitwise NOT", "¬"), &mut handles);
    let xor = mk(&ins("⊻", "Bitwise Exclusive OR", "⊻"), &mut handles);
    let log = inv_btn(&mut handles, ("log", "log("), ("10ˣ", "10^("), "Common Logarithm");
    let int = mk(&ins("int", "Integer Part", "int("), &mut handles);
    let abs = mk(&pair("|x|", "Absolute Value [| |]", "||"), &mut handles);
    let sub = mk(&ins("−", "Subtract [-]", "−"), &mut handles);
    let grp = mk(&pair("( )", "Group [()]", "()"), &mut handles);
    let r2 = op_digit_row(
        [not, xor, log, int, abs],
        ["6", "7", "8", "9", "A"],
        vec![sub, grp],
        &mut handles,
    );
    pad.append(&r2);

    // Rows 4-5 share the tall = column: a vertical pair of rows plus the
    // 110px equals button beside them.
    let lower = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(4)
        .hexpand(true)
        .build();
    let rows45 = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(4)
        .hexpand(true)
        .build();
    let shl = mk(&ins("«", "Left Shift [<<]", "<<"), &mut handles);
    let shr = mk(&ins("»", "Right Shift [>>]", ">>"), &mut handles);
    let log2 = inv_btn(&mut handles, ("log₂", "log2("), ("2ˣ", "2^("), "Binary Logarithm");
    let frac = mk(&ins("frac", "Fractional Part", "frac("), &mut handles);
    let fact = mk(&ins("x!", "Factorial [!]", "!"), &mut handles);
    let add = mk(&ins("+", "Add [+]", "+"), &mut handles);
    let r3 = op_digit_row(
        [shl, shr, log2, frac, fact],
        ["1", "2", "3", "4", "5"],
        vec![add],
        &mut handles,
    );
    pad.append(&r3);

    let r4 = pad_row(4);
    base_btn.set_css_classes(&["radix-btn"]);
    base_btn.set_width_request(130);
    base_btn.set_height_request(h);
    base_btn.set_hexpand(false);
    r4.append(base_btn);
    word_btn.set_css_classes(&["radix-btn"]);
    word_btn.set_width_request(130);
    word_btn.set_height_request(h);
    word_btn.set_hexpand(false);
    r4.append(word_btn);
    let charcode = mk(&sp("á", "Insert Character Code"), &mut handles);
    fixed_btn(&charcode, 64, h);
    r4.append(&charcode);
    let zero = mk(&digit("0"), &mut handles);
    fixed_btn(&zero, 64, h);
    r4.append(&zero);
    let dot = mk(&ins(".", "Decimal Point [.]", "."), &mut handles);
    fixed_btn(&dot, 64, h);
    r4.append(&dot);
    let xv = menu_button("x", &mut handles, true);
    xv.set_width_request(64);
    xv.set_height_request(h);
    xv.set_hexpand(false);
    r4.append(&xv);
    let fx = menu_button("f(x)", &mut handles, false);
    fx.set_width_request(64);
    fx.set_height_request(h);
    fx.set_hexpand(false);
    r4.append(&fx);
    rows45.append(&r4);
    lower.append(&rows45);
    let eq = mk(
        &Btn { label: "=", tooltip: "Calculate [=]", token: None, caret_back: 0, css: "radix-btn equals" },
        &mut handles,
    );
    eq.set_hexpand(false);
    eq.set_width_request(64);
    eq.set_height_request(h * 2 + 4);
    lower.append(&eq);
    pad.append(&lower);
    (pad, handles)
}


/// Conversion digit keypad (docs/02 §8.5); `hex` selects the Number-Base
/// grid; `slot` fills the bottom-right contextual cell (empty / "+/−" / "ⓘ").
/// The empty slot renders as a dimmed empty tile in the original.
pub fn conversion(hex: bool, slot: &str) -> (Grid, PadHandles) {
    let grid = Grid::builder()
        .row_spacing(4)
        .column_spacing(4)
        .margin_start(12)
        .margin_end(12)
        .margin_top(12)
        .margin_bottom(12)
        .build();
    let mut handles = PadHandles::default();

    let top_rows: Vec<Vec<Btn>> = if hex {
        vec![
            vec![digit("B"), digit("C"), digit("D"), digit("E"), digit("F")],
            vec![digit("6"), digit("7"), digit("8"), digit("9"), digit("A")],
            vec![digit("1"), digit("2"), digit("3"), digit("4"), digit("5")],
        ]
    } else {
        vec![
            vec![digit("7"), digit("8"), digit("9")],
            vec![digit("4"), digit("5"), digit("6")],
            vec![digit("1"), digit("2"), digit("3")],
        ]
    };

    let ctrl = [
        ("C", "Clear", "clear"),
        ("⇅", "Swap Units", "swap"),
        ("⌫", "Backspace", "backspace"),
    ];
    for (c, (label, tip, role)) in ctrl.iter().enumerate() {
        let b = gtk::Button::builder().label(*label).tooltip_text(*tip).css_classes(["radix-btn"]).hexpand(true).vexpand(true).height_request(BTN_H).build();
        match *role {
            "clear" => handles.clear = Some(b.clone()),
            "swap" => handles.swap = Some(b.clone()),
            _ => handles.backspace = Some(b.clone()),
        }
        grid.attach(&b, c as i32, 0, 1, 1);
    }
    for (r, row) in top_rows.iter().enumerate() {
        for (c, btn) in row.iter().enumerate() {
            let b = mk(btn, &mut handles);
            grid.attach(&b, c as i32, (r + 1) as i32, 1, 1);
        }
    }
    let b0 = mk(&digit("0"), &mut handles);
    let bdot = mk(&ins(".", "Decimal Point", "."), &mut handles);
    grid.attach(&b0, 0, 4, 1, 1);
    grid.attach(&bdot, 1, 4, 1, 1);
    if !slot.is_empty() {
        let bs = gtk::Button::builder().label(slot).css_classes(["radix-btn"]).hexpand(true).vexpand(true).height_request(BTN_H).build();
        handles.context_slot = Some(bs.clone());
        grid.attach(&bs, 2, 4, 1, 1);
    } else {
        // The empty bottom-right slot still renders as a (dimmed) tile in
        // the original (2026-11-04 20:18:13: ~50%-opacity square).
        let empty = gtk::Button::builder()
            .css_classes(["radix-btn", "empty-slot"])
            .hexpand(true)
            .vexpand(true)
            .height_request(BTN_H)
            .sensitive(false)
            .build();
        grid.attach(&empty, 2, 4, 1, 1);
    }
    (grid, handles)
}
