//! Main window: header, history, entry, keypads, mode switching, and the
//! evaluation glue between the UI and the engine (docs/00 §1, docs/02, docs/09).

use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use gtk::glib;
use gtk::gdk;
use gtk::gio;

use crate::entry::{EntryController, EntryState};
use crate::keypads;

use radix_engine::{
    evaluate, format_value, normalize_echo, AngleUnit, Base, ErrorKind, FormatPrefs, ResultFormat,
    Value, WordSize,
};

pub const NARROW_W: i32 = 360;
pub const WIDE_W: i32 = 700;
pub const DEFAULT_H: i32 = 615;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Basic,
    Advanced,
    Financial,
    Programming,
    Keyboard,
    Conversion,
}

impl Mode {
    pub fn key(self) -> &'static str {
        match self {
            Mode::Basic => "basic",
            Mode::Advanced => "advanced",
            Mode::Financial => "financial",
            Mode::Programming => "programming",
            Mode::Keyboard => "keyboard",
            Mode::Conversion => "conversion",
        }
    }
    pub fn from_key(k: &str) -> Option<Mode> {
        Some(match k {
            "basic" => Mode::Basic,
            "advanced" => Mode::Advanced,
            "financial" => Mode::Financial,
            "programming" => Mode::Programming,
            "keyboard" => Mode::Keyboard,
            "conversion" => Mode::Conversion,
            _ => return None,
        })
    }
    pub fn label(self) -> &'static str {
        match self {
            Mode::Basic => "Basic",
            Mode::Advanced => "Advanced",
            Mode::Financial => "Financial",
            Mode::Programming => "Programming",
            Mode::Keyboard => "Keyboard",
            Mode::Conversion => "Conversion",
        }
    }
    pub fn width(self) -> i32 {
        match self {
            Mode::Basic | Mode::Conversion => NARROW_W,
            _ => WIDE_W,
        }
    }
    pub fn all() -> [Mode; 6] {
        [
            Mode::Basic,
            Mode::Advanced,
            Mode::Financial,
            Mode::Programming,
            Mode::Keyboard,
            Mode::Conversion,
        ]
    }
}

/// One stored history row: the equation echo plus the value needed to
/// re-render the result live (docs/07 §5). Engine rows store the evaluated
/// `Value`; conversion-expression rows (docs/06 §10) store the f64 result.
#[derive(Debug, Clone)]
pub enum RowValue {
    Engine(Value),
    Conv(f64),
}

pub struct HistoryRow {
    pub echo: String,
    pub value: RowValue,
}

impl HistoryRow {
    pub fn render(&self, fmt: &FormatPrefs) -> String {
        match &self.value {
            RowValue::Engine(v) => format_value(v, fmt),
            RowValue::Conv(f) => crate::fmt::automatic(*f, fmt.decimals, fmt.trailing_zeroes, fmt.group),
        }
    }
}

pub struct RadixWindow {
    pub window: adw::ApplicationWindow,
    // Header
    pub undo_btn: gtk::Button,
    pub redo_btn: gtk::Button,
    pub mode_label: gtk::Label,
    pub hamburger: gtk::MenuButton,
    // Body (calc modes share one layout; conversion is separate)
    pub stack: gtk::Stack,
    pub backspace_btn: gtk::Button,
    pub history_scroll: gtk::ScrolledWindow,
    pub history_list: gtk::ListBox,
    pub message: gtk::Label,
    pub entry: Rc<EntryController>,
    /// Currency-conversion preferences shared with the entry completion.
    pub currency: crate::data::currency::SharedCurrencyPrefs,
    pub keypad_stack: gtk::Stack,
    /// History+entry card (full-bleed in Basic — no inset, no radius).
    pub display_card: gtk::Box,
    /// Octal/hex readout line inside the entry strip (Programming only;
    /// reserves its 28px in every mode so the strip is 68px tall).
    pub radix_line: gtk::Label,
    // Programming extras: bit panel area inside the display card (below the
    // entry strip), shown only in Programming mode.
    pub bit_area: gtk::Box,
    pub base_btn: gtk::MenuButton,
    pub word_btn: gtk::MenuButton,
    pub bit_cells: Vec<gtk::Label>,
    /// Programming pad digit buttons for base dimming (docs/02 §8.4).
    pub prg_base_digits: RefCell<Vec<(gtk::Button, u32)>>,
    // Conversion pane
    pub conversion: Rc<crate::convert::ConversionPane>,
    pub rates: crate::data::currency::SharedRates,
    // State
    pub mode: RefCell<Mode>,
    pub prev_mode: RefCell<Mode>,
    pub history_idx: RefCell<Option<usize>>,
    pub env: RefCell<radix_engine::Env>,
    pub fmt: RefCell<FormatPrefs>,
    pub history: RefCell<Vec<HistoryRow>>,
    pub primary_menu: gio::Menu,
    pub primary_menu_conv: gio::Menu,
}

impl RadixWindow {
    pub fn new(app: &adw::Application) -> Rc<RadixWindow> {
        load_css();

        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Radix")
            .default_width(NARROW_W)
            .default_height(DEFAULT_H)
            .icon_name("io.github.hakergeniusz.Radix")
            .build();
        // The original shows only the close button in the header (no
        // minimize/maximize, every mode) — pin the decoration layout.
        if let Some(settings) = gtk::Settings::default() {
            settings.set_gtk_decoration_layout(Some(":close"));
        }

        // ---- Header ----
        let header = adw::HeaderBar::builder()
            .show_start_title_buttons(true)
            .show_end_title_buttons(true)
            .build();
        // The observed header shows controls only — no title text (docs/02 §1);
        // the window title stays "Radix" for taskbars/WM. An EMPTY widget is
        // required: unset title-widget falls back to the window title.
        header.set_title_widget(Some(&gtk::Box::new(gtk::Orientation::Horizontal, 0)));

        let undo_btn = gtk::Button::from_icon_name("edit-undo-symbolic");
        let redo_btn = gtk::Button::from_icon_name("edit-redo-symbolic");
        redo_btn.set_sensitive(false);
        undo_btn.set_tooltip_text(Some("Undo [Ctrl+Z]"));
        redo_btn.set_tooltip_text(Some("Redo [Ctrl+Shift+Z]"));

        let mode_label = gtk::Label::new(Some(Mode::Basic.label()));
        let chevron = gtk::Image::from_icon_name("pan-down-symbolic");
        let mode_btn_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        mode_btn_box.append(&mode_label);
        mode_btn_box.append(&chevron);
        let mode_btn = gtk::MenuButton::builder().child(&mode_btn_box).build();
        mode_btn.add_css_class("flat");
        mode_btn.add_css_class("title-mode-btn");

        let hamburger = gtk::MenuButton::builder().icon_name("open-menu-symbolic").build();
        hamburger.add_css_class("flat");

        header.pack_start(&undo_btn);
        header.pack_start(&redo_btn);
        // Mode dropdown is the CENTERED title widget (owner evidence).
        header.set_title_widget(Some(&mode_btn));
        header.pack_end(&hamburger);

        // ---- History ----
        // Rows anchor to the bottom of the card, just above the entry;
        // spare space accumulates above (owner evidence E087/E089).
        let history_list = gtk::ListBox::builder()
            .css_classes(["history-list"])
            .valign(gtk::Align::End)
            .build();
        let history_scroll = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .child(&history_list)
            .vexpand(true)
            .height_request(150)
            .build();

        // ---- Error message line (docs/10 §1.2) ----
        let message = gtk::Label::builder()
            .xalign(0.0)
            .wrap(true)
            .css_classes(["error-message"])
            .visible(false)
            .margin_start(18)
            .margin_end(18)
            .build();

        // ---- Entry row (textview + backspace, docs/16 §3) ----
        let currency: crate::data::currency::SharedCurrencyPrefs =
            std::rc::Rc::new(crate::data::currency::CurrencyPrefState::default());
        let entry = EntryController::new(currency.clone());
        // ⌫-key glyph, matching the original entry row (E087 zoom).
        let backspace_btn = gtk::Button::from_icon_name("edit-clear-symbolic");
        backspace_btn.add_css_class("flat");
        backspace_btn.set_tooltip_text(Some("Backspace"));
        {
            let e = entry.clone();
            backspace_btn.connect_clicked(move |_| e.backspace());
        }
        backspace_btn.set_valign(gtk::Align::Start);
        backspace_btn.set_margin_top(4);
        backspace_btn.set_margin_end(12);
        // Strip = 40px text row + 28px radix readout row (the readout is
        // empty outside Programming but always reserves its height — the
        // original's Basic strip is the same 68px total).
        let radix_line = gtk::Label::builder()
            .height_request(28)
            .valign(gtk::Align::Center)
            .xalign(0.0)
            .margin_start(14)
            .margin_end(14)
            .css_classes(["radix-line"])
            .build();
        let entry_top = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .build();
        entry.view.set_height_request(40);
        entry_top.append(&entry.view);
        entry_top.append(&backspace_btn);
        let entry_row = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .css_classes(["entry-strip"])
            .build();
        entry_row.append(&entry_top);
        entry_row.append(&radix_line);

        // Display card: ONE rounded card per the original — history rows
        // bottom-anchored on the lighter card surface, the entry strip (and,
        // in Programming, the bit panel) pinned to the darker strip surface
        // at its bottom (owner evidence 2026-11-04 18:5x batch).
        let bit_separator = gtk::Separator::builder()
            .orientation(gtk::Orientation::Horizontal)
            .build();
        let (bit_panel, bit_cells) = build_bit_panel();
        bit_panel.add_css_class("bit-panel");
        let bit_area = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .css_classes(["entry-strip"])
            .visible(false)
            .build();
        bit_area.append(&bit_separator);
        bit_area.append(&bit_panel);

        let display_card = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .css_classes(["display-card"])
            .margin_top(12)
            .margin_start(12)
            .margin_end(12)
            .vexpand(true)
            .build();
        let display_card_clone = display_card.clone();
        display_card.set_overflow(gtk::Overflow::Hidden);
        display_card.append(&history_scroll);
        display_card.append(&message);
        display_card.append(&entry_row);
        display_card.append(&bit_area);

        // ---- Keypads ----
        let keypad_stack = gtk::Stack::builder()
            .transition_type(gtk::StackTransitionType::None)
            .vexpand(false)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();
        // Pages have very different natural widths (5-col Basic vs 12-col
        // Programming); homogeneous would inflate every mode's window to
        // the widest pad.
        keypad_stack.set_hhomogeneous(false);
        keypad_stack.set_vhomogeneous(false);

        let body = gtk::Box::new(gtk::Orientation::Vertical, 0);
        body.append(&display_card);
        body.append(&keypad_stack);

        // ---- Conversion pane + rates ----
        let rates = crate::data::currency::RatesCache::new();
        let conversion = crate::convert::build(rates.clone());

        // ---- Programming extras (base/word dropdowns live inside the pad
        // grid; the bit panel lives inside the display card above). ----
        let base_btn = gtk::MenuButton::builder().label("Decimal").build();
        let word_btn = gtk::MenuButton::builder().label("64-bit").build();

        let stack = gtk::Stack::builder()
            .transition_type(gtk::StackTransitionType::None)
            .vexpand(true)
            .build();
        // The Conversion page's natural width must not inflate the calc
        // modes: each page sizes independently (the window width is the
        // per-mode default otherwise — E085 410 / E087 750).
        stack.set_hhomogeneous(false);
        stack.set_vhomogeneous(false);
        stack.add_named(&body, Some("calc"));
        stack.add_named(&conversion.container, Some("conversion"));

        let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
        content.append(&header);
        content.append(&stack);
        window.set_content(Some(&content));

        // ---- Menus ----
        let mode_menu = gio::Menu::new();
        for m in Mode::all() {
            let item = gio::MenuItem::new(Some(m.label()), None);
            item.set_action_and_target_value(Some("win.mode"), Some(&m.key().to_variant()));
            mode_menu.append_item(&item);
        }

        // Sections draw the divider lines seen in the original menu
        // (evidence 2026-11-04 18:54:54): New Window | Clear History |
        // Result Format radios | Preferences/Shortcuts/Help | About.
        let mk = |items: &[(&str, &str)]| {
            let sec = gio::Menu::new();
            for (label, action) in items {
                sec.append(Some(label), Some(action));
            }
            sec
        };
        let fmt_section = gio::Menu::new();
        for (label, key) in [
            ("Automatic", "automatic"),
            ("Fixed", "fixed"),
            ("Scientific", "scientific"),
            ("Engineering", "engineering"),
        ] {
            let item = gio::MenuItem::new(Some(label), None);
            item.set_action_and_target_value(
                Some("win.result-format"),
                Some(&key.to_variant()),
            );
            fmt_section.append_item(&item);
        }
        // Section header via a section label attribute
        let fmt_wrapper = gio::Menu::new();
        fmt_wrapper.append_section(Some("Result Format"), &fmt_section);

        let primary_menu = gio::Menu::new();
        primary_menu.append_section(None, &mk(&[("New Window", "win.new-window")]));
        primary_menu.append_section(None, &mk(&[("Clear History", "win.clear-history")]));
        primary_menu.append_section(None, &fmt_wrapper);
        primary_menu.append_section(
            None,
            &mk(&[
                ("Preferences", "win.prefs"),
                ("Keyboard Shortcuts", "win.shortcuts"),
                ("Help", "win.help"),
            ]),
        );
        primary_menu.append_section(None, &mk(&[("About Calculator", "win.about")]));

        // Conversion has no history, so no Clear History item (2026-11-04
        // 20:17:57/20:18:07 menus: New Window | Result Format | … | About).
        let primary_menu_conv = gio::Menu::new();
        primary_menu_conv.append_section(None, &mk(&[("New Window", "win.new-window")]));
        primary_menu_conv.append_section(None, &fmt_wrapper);
        primary_menu_conv.append_section(
            None,
            &mk(&[
                ("Preferences", "win.prefs"),
                ("Keyboard Shortcuts", "win.shortcuts"),
                ("Help", "win.help"),
            ]),
        );
        primary_menu_conv.append_section(None, &mk(&[("About Calculator", "win.about")]));

        mode_btn.set_menu_model(Some(&mode_menu));
        hamburger.set_menu_model(Some(&primary_menu));

        // ---- Keypad construction per mode ----
        keypad_stack.add_named(&gtk::Box::new(gtk::Orientation::Vertical, 0), Some("keyboard"));

        let win = Rc::new(RadixWindow {
            window,
            undo_btn,
            redo_btn,
            mode_label,
            hamburger,
            stack,
            backspace_btn,
            history_scroll,
            history_list,
            message,
            entry,
            currency,
            keypad_stack,
            display_card: display_card_clone,
            radix_line,
            bit_area,
            base_btn,
            word_btn,
            bit_cells,
            conversion,
            rates,
            mode: RefCell::new(Mode::Basic),
            prev_mode: RefCell::new(Mode::Basic),
            history_idx: RefCell::new(None),
            prg_base_digits: RefCell::new(Vec::new()),
            env: RefCell::new(radix_engine::Env::new()),
            fmt: RefCell::new(FormatPrefs {
                format: ResultFormat::Automatic,
                decimals: 9,
                trailing_zeroes: false,
                group: false,
                base: Base::Dec,
            }),
            history: RefCell::new(Vec::new()),
            primary_menu,
            primary_menu_conv,
        });

        win.build_keypads();
        // Currency refresh: once shortly after start, then every 30 min the
        // interval preference is re-checked (docs/16 §6).
        {
            let w0 = win.clone();
            glib::timeout_add_seconds_local(1800, move || {
                w0.maybe_refresh_rates();
                glib::ControlFlow::Continue
            });
            win.maybe_refresh_rates();
        }
        // The Stack's visible child defaults to the first-added (empty
        // keyboard placeholder); select the actual initial-mode pad.
        win.keypad_stack.set_visible_child_name(win.mode.borrow().key());
        win.apply_mode_chrome(*win.mode.borrow());
        win.wire_actions(app);
        win.wire_entry_keys();
        win.wire_buttons();

        // Conversion pane's dialogs present over this window.
        win.conversion.parent.replace(Some(win.window.clone()));

        // Graceful close persists state; relaunch restores it (docs/09 §4).
        {
            let w = win.clone();
            win.window.connect_close_request(move |_| {
                crate::persist::save(&w);
                glib::Propagation::Proceed
            });
        }
        crate::persist::restore(&win);
        win
    }

    pub fn present(self: &Rc<Self>) {
        self.window.present();
        self.entry.grab_focus_view();
    }

    // ---- keypad construction ----
    fn build_keypads(self: &Rc<Self>) {
        let mk_pad = |kind: &'static str, widget: gtk::Widget| {
            self.keypad_stack.add_named(&widget, Some(kind));
        };
        // Basic
        let (grid, h) = keypads::basic();
        self.wire_pad(&h);
        mk_pad("basic", grid.upcast());
        // Advanced — single full grid (docs/16 §2)
        let (grid, h) = keypads::advanced();
        self.wire_pad(&h);
        self.wire_inverse(&h);
        mk_pad("advanced", grid.upcast());
        // Financial — formula dialogs + Currency Conversion mode button
        let (grid, h) = keypads::financial();
        self.wire_pad(&h);
        self.wire_inverse(&h);
        self.wire_financial_formulas(&h);
        if let Some(cc) = &h.currency_conv {
            let w = self.clone();
            cc.connect_clicked(move |_| {
                w.conversion
                    .set_category(crate::data::units::Category::Currency);
                w.set_mode(Mode::Conversion);
            });
        }
        mk_pad("financial", grid.upcast());
        // Programming — base/word dropdowns passed into the pad grid
        let (grid, h) = keypads::programming(&self.base_btn, &self.word_btn);
        self.wire_pad(&h);
        self.wire_inverse(&h);
        *self.prg_base_digits.borrow_mut() = h.base_digits.clone();
        mk_pad("programming", grid.upcast());
    }

    /// ⇧⁻¹ inverse-functions toggle (docs/16 §2, decision D16): swaps the
    /// label and token of the registered invertible buttons.
    fn wire_inverse(self: &Rc<Self>, h: &keypads::PadHandles) {
        let Some(t) = &h.inverse_toggle else { return };
        let _ = self;
        let invs = h.invertibles.clone();
        t.connect_toggled(move |tgl| {
            let active = tgl.is_active();
            for iv in &invs {
                let (label, token) = if active { iv.inverse } else { iv.direct };
                iv.button.set_label(label);
                iv.token_cell.set(token);
            }
        });
    }

    fn wire_pad(self: &Rc<Self>, h: &keypads::PadHandles) {
        for (btn, token, back) in &h.tokens {
            let entry = self.entry.clone();
            let tok = token.clone();
            let back_n = *back;
            btn.connect_clicked(move |_| {
                entry.insert_at_caret(tok.get(), back_n);
            });
        }
        // x and f(x) menu buttons get their popovers (docs/02 §10).
        if let Some(xv) = &h.variables {
            xv.set_popover(Some(&crate::popovers::build_variables(&self.entry)));
        }
        if let Some(fx) = &h.functions {
            fx.set_popover(Some(&crate::popovers::build_fx(&self.entry)));
        }
        // Insert Character Code (à button, docs/02 §10).
        if let Some(cc) = &h.char_code {
            let w = self.clone();
            cc.connect_clicked(move |_| crate::dialogs::open_char_code(&w));
        }
        if let Some(c) = &h.clear {
            let entry = self.entry.clone();
            c.connect_clicked(move |_| entry.clear());
        }
        if let Some(eq) = &h.equals {
            let w = self.clone();
            eq.connect_clicked(move |_| w.evaluate_from_entry());
        }
        if let Some(sup) = &h.super_mode {
            let entry = self.entry.clone();
            sup.connect_clicked(move |_| {
                let mut s = entry.super_mode.borrow_mut();
                *s = !*s;
                *entry.sub_mode.borrow_mut() = false;
            });
        }
        if let Some(sub) = &h.sub_mode {
            let entry = self.entry.clone();
            sub.connect_clicked(move |_| {
                let mut s = entry.sub_mode.borrow_mut();
                *s = !*s;
                *entry.super_mode.borrow_mut() = false;
            });
        }
        if let Some(deg) = &h.angle_cycle {
            let w = self.clone();
            let deg_clicked = deg.clone();
            let deg_btn = deg.clone();
            deg_clicked.connect_clicked(move |_| {
                let next = match w.env.borrow().angle_unit {
                    AngleUnit::Degrees => AngleUnit::Radians,
                    AngleUnit::Radians => AngleUnit::Gradians,
                    AngleUnit::Gradians => AngleUnit::Degrees,
                };
                let name = match next {
                    AngleUnit::Degrees => "Deg",
                    AngleUnit::Radians => "Rad",
                    AngleUnit::Gradians => "Grad",
                };
                deg_btn.set_label(name);
                deg_btn.set_tooltip_text(Some(&format!("Angle Unit: {}", match next {
                    AngleUnit::Degrees => "Degrees",
                    AngleUnit::Radians => "Radians",
                    AngleUnit::Gradians => "Gradians",
                })));
                w.env.borrow_mut().angle_unit = next;
            });
        }
        // x / f(x) menu buttons: popovers wired above.
    }

    fn wire_financial_formulas(self: &Rc<Self>, h: &keypads::PadHandles) {
        for (btn, formula_id) in &h.formulas {
            let w = self.clone();
            let id: &'static str = formula_id;
            btn.connect_clicked(move |_| {
                crate::dialogs::open_financial(&w, id);
            });
        }
    }

    // ---- actions & accelerators ----
    fn wire_actions(self: &Rc<Self>, app: &adw::Application) {
        let actions = gio::SimpleActionGroup::new();

        let mode_action = gio::SimpleAction::new_stateful(
            "mode",
            Some(glib::VariantTy::STRING),
            &Mode::Basic.key().to_variant(),
        );
        {
            let act = mode_action.clone();
            let w = self.clone();
            mode_action.connect_activate(move |_, param| {
                if let Some(s) = param.and_then(|p| p.str()) {
                    if let Some(m) = Mode::from_key(s) {
                        // Activation does not move the state by itself; the
                        // mode-menu radios follow the action state.
                        if let Some(p) = param {
                            act.set_state(p);
                        }
                        w.set_mode(m);
                    }
                }
            });
        }
        actions.add_action(&mode_action);

        let fmt_action = gio::SimpleAction::new_stateful(
            "result-format",
            Some(glib::VariantTy::STRING),
            &"automatic".to_variant(),
        );
        {
            let act = fmt_action.clone();
            let w = self.clone();
            fmt_action.connect_activate(move |_, param| {
                if let Some(s) = param.and_then(|p| p.str()) {
                    let f = match s {
                        "fixed" => ResultFormat::Fixed,
                        "scientific" => ResultFormat::Scientific,
                        "engineering" => ResultFormat::Engineering,
                        _ => ResultFormat::Automatic,
                    };
                    if let Some(p) = param {
                        act.set_state(p);
                    }
                    w.fmt.borrow_mut().format = f;
                    w.rerender_history();
                }
            });
        }
        actions.add_action(&fmt_action);

        for name in [
            "new-window",
            "clear-history",
            "prefs",
            "shortcuts",
            "help",
            "about",
        ] {
            let action = gio::SimpleAction::new(name, None);
            let w = self.clone();
            let name = name.to_string();
            action.connect_activate(move |_, _| match name.as_str() {
                "new-window" => {
                    let app = w.window.application().unwrap();
                    RadixWindow::new(app.downcast_ref::<adw::Application>().unwrap()).present();
                }
                "clear-history" => w.clear_history(),
                "prefs" => crate::dialogs::open_preferences(&w),
                "shortcuts" => crate::dialogs::open_shortcuts(&w),
                "help" => crate::dialogs::open_help(&w),
                "about" => crate::dialogs::open_about(&w),
                _ => {}
            });
            actions.add_action(&action);
        }

        // Programming base switching (docs/08 §1): re-render results.
        let base_action = gio::SimpleAction::new_stateful(
            "base",
            Some(glib::VariantTy::STRING),
            &"dec".to_variant(),
        );
        {
            let act = base_action.clone();
            let w = self.clone();
            base_action.connect_activate(move |_, param| {
                if let Some(s) = param.and_then(|p| p.str()) {
                    let (b, name) = match s {
                        "bin" => (Base::Bin, "Binary"),
                        "oct" => (Base::Oct, "Octal"),
                        "hex" => (Base::Hex, "Hexadecimal"),
                        _ => (Base::Dec, "Decimal"),
                    };
                    if let Some(p) = param {
                        act.set_state(p);
                    }
                    w.env.borrow_mut().base = b;
                    w.fmt.borrow_mut().base = b;
                    w.base_btn.set_label(name);
                    w.rerender_history();
                }
            });
        }
        actions.add_action(&base_action);

        // Word size (docs/02 §4/§8.4) — constrains the bitwise domain only.
        let word_action = gio::SimpleAction::new_stateful(
            "word-size",
            Some(glib::VariantTy::STRING),
            &"64".to_variant(),
        );
        {
            let act = word_action.clone();
            let w = self.clone();
            word_action.connect_activate(move |_, param| {
                if let Some(s) = param.and_then(|p| p.str()) {
                    let (ws, name) = match s {
                        "8" => (WordSize::W8, "8-bit"),
                        "16" => (WordSize::W16, "16-bit"),
                        "32" => (WordSize::W32, "32-bit"),
                        _ => (WordSize::W64, "64-bit"),
                    };
                    if let Some(p) = param {
                        act.set_state(p);
                    }
                    w.env.borrow_mut().word_size = ws;
                    w.word_btn.set_label(name);
                    w.update_programming();
                }
            });
        }
        actions.add_action(&word_action);

        // Menus for the programming dropdowns.
        let base_menu = gio::Menu::new();
        for (label, key) in [
            ("Binary", "bin"),
            ("Octal", "oct"),
            ("Decimal", "dec"),
            ("Hexadecimal", "hex"),
        ] {
            let item = gio::MenuItem::new(Some(label), None);
            item.set_action_and_target_value(Some("win.base"), Some(&key.to_variant()));
            base_menu.append_item(&item);
        }
        self.base_btn.set_menu_model(Some(&base_menu));
        let word_menu = gio::Menu::new();
        for (label, key) in [("8-bit", "8"), ("16-bit", "16"), ("32-bit", "32"), ("64-bit", "64")] {
            let item = gio::MenuItem::new(Some(label), None);
            item.set_action_and_target_value(Some("win.word-size"), Some(&key.to_variant()));
            word_menu.append_item(&item);
        }
        self.word_btn.set_menu_model(Some(&word_menu));

        self.window.insert_action_group("win", Some(&actions));

        // Accelerators (docs/08 §1)
        app.set_accels_for_action("win.mode::basic", &["<Control><Alt>b"]);
        app.set_accels_for_action("win.mode::advanced", &["<Control><Alt>a"]);
        app.set_accels_for_action("win.mode::financial", &["<Control><Alt>f"]);
        app.set_accels_for_action("win.mode::programming", &["<Control><Alt>p"]);
        app.set_accels_for_action("win.mode::keyboard", &["<Control><Alt>k"]);
        app.set_accels_for_action("win.mode::conversion", &["<Control><Alt>c"]);
        app.set_accels_for_action("win.new-window", &["<Control>n"]);
        app.set_accels_for_action("win.clear-history", &["<Control>Escape"]);
        app.set_accels_for_action("win.prefs", &["<Control>comma"]);
        app.set_accels_for_action("win.shortcuts", &["<Control>question"]);
        app.set_accels_for_action("win.help", &["F1"]);
        app.set_accels_for_action("win.base::bin", &["<Control>b"]);
        app.set_accels_for_action("win.base::oct", &["<Control>o"]);
        app.set_accels_for_action("win.base::dec", &["<Control>d"]);
        app.set_accels_for_action("win.base::hex", &["<Control>h"]);
        app.set_accels_for_action("window.close", &["<Control>w"]);
        app.set_accels_for_action("app.quit", &["<Control>q"]);
    }

    // ---- entry key handling ----
    fn wire_entry_keys(self: &Rc<Self>) {
        let controller = gtk::EventControllerKey::new();
        {
            let w = self.clone();
            self.entry.view.add_controller(controller.clone());
            controller.connect_key_pressed(move |_, keyval, _, mod_mask| {
                use gdk::Key;
                let w = &w;
                if std::env::var_os("RADIX_TRACE").is_some() {
                    eprintln!("DBG-KEY {keyval:?} mods={mod_mask:?}");
                }
                match keyval {
                    Key::Return | Key::KP_Enter => {
                        w.evaluate_from_entry();
                        return glib::Propagation::Stop;
                    }
                    Key::Tab if w.entry.completion_visible() => {
                        // Tab accepts the highlighted suggestion (docs/08 §6).
                        w.entry.accept_completion();
                        return glib::Propagation::Stop;
                    }
                    Key::equal if !mod_mask.contains(gdk::ModifierType::CONTROL_MASK) => {
                        w.evaluate_from_entry();
                        return glib::Propagation::Stop;
                    }
                    Key::Escape => {
                        // Escape dismisses the completion popover leaving the
                        // typed text (docs/08 §6); else selection-clear /
                        // entry-clear (docs/08 §2).
                        if w.entry.completion_visible() {
                            w.entry.dismiss_completion();
                            return glib::Propagation::Stop;
                        }
                        let buf = w.entry.view.buffer();
                        if buf.has_selection() {
                            let it = buf.iter_at_mark(&buf.get_insert());
                            buf.select_range(&it, &it);
                        } else {
                            w.entry.clear();
                        }
                        return glib::Propagation::Stop;
                    }
                    Key::Up
                        if mod_mask.is_empty()
                            && w.entry.text().is_empty()
                            && *w.mode.borrow() == Mode::Keyboard =>
                    {
                        // docs/08 §3: one-step recall of the last result.
                        if let Some(text) =
                            w.history.borrow().last().map(|r| r.render(&w.fmt.borrow()))
                        {
                            w.entry.show_input(&text);
                        }
                        return glib::Propagation::Stop;
                    }
                    Key::Left
                        if mod_mask.contains(gdk::ModifierType::ALT_MASK) =>
                    {
                        w.cycle_history(false);
                        return glib::Propagation::Stop;
                    }
                    Key::Right
                        if mod_mask.contains(gdk::ModifierType::ALT_MASK) =>
                    {
                        w.cycle_history(true);
                        return glib::Propagation::Stop;
                    }
                    Key::z if mod_mask.contains(gdk::ModifierType::CONTROL_MASK) => {
                        if mod_mask.contains(gdk::ModifierType::SHIFT_MASK) {
                            w.entry.redo();
                        } else {
                            w.entry.undo();
                        }
                        return glib::Propagation::Stop;
                    }
                    _ => {}
                }
                glib::Propagation::Proceed
            });
        }
        // Undo/redo header buttons + enablement
        {
            let w = self.clone();
            self.entry
                .on_stack_change
                .replace(Some(std::rc::Rc::new(move |u, r| {
                    w.undo_btn.set_sensitive(u);
                    w.redo_btn.set_sensitive(r);
                })));
        }
        // Programming bit panel follows the entry (docs/16 D24; E089 shows
        // zeros with an empty entry despite history rows).
        {
            let w = self.clone();
            self.entry
                .on_text_change
                .replace(Some(std::rc::Rc::new(move || {
                    if *w.mode.borrow() == Mode::Programming {
                        w.update_programming();
                    }
                })));
        }
        {
            let e = self.entry.clone();
            self.undo_btn.connect_clicked(move |_| e.undo());
        }
        {
            let e = self.entry.clone();
            self.redo_btn.connect_clicked(move |_| e.redo());
        }
    }

    fn wire_buttons(self: &Rc<Self>) {}

    // ---- mode switching ----
    /// Mode-dependent chrome: full-bleed display in Basic, keypad stack
    /// hidden in Keyboard, scroll reset, radix line. Also called once at
    /// startup (set_mode early-returns when the mode is unchanged).
    fn apply_mode_chrome(self: &Rc<Self>, mode: Mode) {
        // Basic renders the display full-bleed: rows and separators run
        // edge-to-edge, no card inset and no radius (2026-11-04 20:16 batch).
        let flush = mode == Mode::Basic;
        self.display_card.set_margin_top(if flush { 0 } else { 12 });
        self.display_card.set_margin_start(if flush { 0 } else { 12 });
        self.display_card.set_margin_end(if flush { 0 } else { 12 });
        self.display_card.remove_css_class("flush");
        if flush {
            self.display_card.add_css_class("flush");
        }
        // Keyboard mode has no keypad area: the history card extends to
        // 12px from the window bottom (2026-11-04 20:17:31).
        self.keypad_stack.set_visible(mode != Mode::Keyboard);
        // A mode switch resets the history scroll to its top (the original's
        // Programming pad shows the oldest rows with the newest clipped —
        // 2026-11-04 20:17:09); row appends still scroll to the bottom.
        self.history_scroll.vadjustment().set_value(0.0);
        if mode != Mode::Programming {
            self.radix_line.set_text("");
        }
    }

    pub fn set_mode(self: &Rc<Self>, mode: Mode) {
        let cur = *self.mode.borrow();
        if cur == mode {
            return;
        }
        if cur != Mode::Conversion {
            *self.prev_mode.borrow_mut() = cur;
        }
        *self.mode.borrow_mut() = mode;
        // Programmatic switches (Currency Conversion button) must move the
        // mode-menu radio along with the title.
        if let Some(act) = self.window.lookup_action("mode") {
            if let Some(sa) = act.downcast_ref::<gio::SimpleAction>() {
                sa.set_state(&mode.key().to_variant());
            }
        }
        self.mode_label.set_text(mode.label());
        self.stack
            .set_visible_child_name(if mode == Mode::Conversion { "conversion" } else { "calc" });
        self.bit_area.set_visible(mode == Mode::Programming);
        self.apply_mode_chrome(mode);
        if mode == Mode::Programming {
            // Initial digit dimming + bit panel state for the current base.
            self.update_programming();
        }
        // Entry placement: inside the display card for wide modes and
        // Keyboard; a separate strip below the card in Basic (docs/16 §3).
        self.place_entry(mode);
        if mode != Mode::Conversion {
            self.keypad_stack.set_visible_child_name(mode.key());
        }
        let conv = mode == Mode::Conversion;
        self.undo_btn.set_visible(!conv);
        self.redo_btn.set_visible(!conv);
        self.hamburger.set_menu_model(Some(if conv {
            &self.primary_menu_conv
        } else {
            &self.primary_menu
        }));
        self.window.set_default_size(mode.width(), DEFAULT_H);
    }

    /// Entry placement per the 2026-11-04 18:5x evidence batch: the entry
    /// strip lives INSIDE the display card in every calc mode (the old
    /// "Basic keeps a separate strip" reading of E085 was wrong). Keyboard
    /// hides ⌫.
    fn place_entry(self: &Rc<Self>, mode: Mode) {
        self.backspace_btn.set_visible(mode != Mode::Keyboard);
        if mode == Mode::Conversion {
            self.conversion.focus_source();
        } else {
            self.entry.grab_focus_view();
        }
    }

    // ---- evaluation ----
    pub fn evaluate_from_entry(self: &Rc<Self>) {
        let raw = self.entry.text();
        let expr = raw.trim_end();
        let expr = expr.strip_suffix('=').unwrap_or(expr);
        if expr.trim().is_empty() {
            return;
        }
        let expr_owned = expr.to_string();
        if std::env::var_os("RADIX_TRACE").is_some() {
            eprintln!("DBG-EVAL expr={expr_owned:?} state={:?}", self.entry.state());
        }

        // Conversion expressions (docs/06 §10): `<amount> <unit> in <unit>`
        // in the free entry. Currency codes are contract-verified; other
        // unit names resolve via the same tables (superset decision).
        if let Some(result) = self.try_conversion_expression(&expr_owned) {
            let (echo, value) = result;
            let formatted = crate::fmt::automatic(
                value,
                self.fmt.borrow().decimals,
                self.fmt.borrow().trailing_zeroes,
                self.fmt.borrow().group,
            );
            self.history.borrow_mut().push(HistoryRow {
                echo,
                value: RowValue::Conv(value),
            });
            *self.history_idx.borrow_mut() = None;
            self.append_history_row();
            self.message.set_visible(false);
            self.entry.finish_result(&formatted);
            return;
        }

        let result = evaluate(&expr_owned, &mut self.env.borrow_mut());
        if std::env::var_os("RADIX_TRACE").is_some() {
            match &result {
                Ok(v) => eprintln!("DBG-EVAL-OK {v:?}"),
                Err(e) => eprintln!("DBG-EVAL-ERR {e:?}"),
            }
        }
        match result {
            Ok(v) => {
                let formatted = format_value(&v, &self.fmt.borrow());
                let echo = normalize_echo(&expr_owned, self.fmt.borrow().group);
                self.history.borrow_mut().push(HistoryRow {
                    echo,
                    value: RowValue::Engine(v.clone()),
                });
                *self.history_idx.borrow_mut() = None;
                self.append_history_row();
                self.message.set_visible(false);
                self.entry.finish_result(&formatted);
                if *self.mode.borrow() == Mode::Programming {
                    self.update_programming();
                }
            }
            Err(e) => {
                let prose = prose_for(&e.kind);
                self.entry.show_error(&expr_owned, e.span);
                match prose {
                    Some(p) => {
                        self.message.set_text(p); // &String derefs to &str
                        self.message.set_visible(true);
                    }
                    None => self.message.set_visible(false),
                }
            }
        }
    }

    /// Detect and evaluate `<amount> <unit> in <unit>`; returns
    /// (echo, result) on success, None when the expression isn't a
    /// conversion (then it goes to the engine).
    fn try_conversion_expression(self: &Rc<Self>, expr: &str) -> Option<(String, f64)> {
        let (lhs, rhs) = expr.split_once(" in ")?;
        if rhs.contains(" in ") || lhs.contains(" in ") {
            return None;
        }
        let mut lhs_tokens = lhs.split_whitespace();
        let amount = lhs_tokens.next()?.replace('−', "-");
        let amount: f64 = amount.parse().ok()?;
        let from = lhs_tokens.next()?;
        if lhs_tokens.next().is_some() || rhs.split_whitespace().count() != 1 {
            return None;
        }
        let to = rhs.trim();
        let (from, to) = (from.to_string(), to.to_string());
        if from.eq_ignore_ascii_case(&to) && from.len() == 3 {
            return Some((expr.to_string(), amount));
        }
        // Currency first (docs/06 §10 contract case).
        let cf = crate::data::currency::by_code(&from.to_uppercase());
        let ct = crate::data::currency::by_code(&to.to_uppercase());
        if let (Some(_), Some(_)) = (cf, ct) {
            let rates = self.rates.rates.borrow();
            let v = if let Some(r) = rates.as_ref() {
                r.convert(&from.to_uppercase(), &to.to_uppercase(), amount)
            } else {
                crate::data::currency::fallback_rates()
                    .convert(&from.to_uppercase(), &to.to_uppercase(), amount)
            }?;
            return Some((expr.to_string(), v));
        }
        // General units across categories.
        for cat in crate::data::units::Category::ALL {
            if cat == crate::data::units::Category::Currency
                || cat == crate::data::units::Category::NumberBase
            {
                continue;
            }
            let find = |name: &str| {
                cat.units()
                    .iter()
                    .find(|u| u.name.eq_ignore_ascii_case(name))
                    .map(|u| u.factor.parse::<f64>().unwrap_or(1.0))
            };
            if let (Some(ff), Some(tf)) = (find(&from), find(&to)) {
                let conv = if cat == crate::data::units::Category::Temperature {
                    let f = cat
                        .units()
                        .iter()
                        .find(|u| u.name.eq_ignore_ascii_case(&from))
                        .map(|u| u.name)
                        .unwrap_or("");
                    let t = cat
                        .units()
                        .iter()
                        .find(|u| u.name.eq_ignore_ascii_case(&to))
                        .map(|u| u.name)
                        .unwrap_or("");
                    crate::data::units::temp_from_kelvin(
                        crate::data::units::temp_to_kelvin(amount, f),
                        t,
                    )
                } else {
                    amount * ff / tf
                };
                return Some((expr.to_string(), conv));
            }
        }
        None
    }

    /// Programming-mode readout + bit panel update (docs/02 §8.4).
    /// Refresh exchange rates when they are older than the interval
    /// preference (docs/02 §3 "Exchange Rate Refresh Interval", docs/16 §6).
    pub fn maybe_refresh_rates(self: &Rc<Self>) {
        let interval = self.currency.prefs.borrow().interval;
        if interval == crate::data::currency::RefreshInterval::Never {
            return;
        }
        let stale = match self.rates.fetched_at.get() {
            None => true, // never fetched this run
            Some(t) => t.elapsed().as_secs() > interval.hours() * 3600,
        };
        if stale {
            self.rates.refresh_async(|_| {});
        }
    }

    pub fn update_programming(self: &Rc<Self>) {
        let base = self.env.borrow().base;
        let word_bits = self.env.borrow().word_size.bits();
        let radix = match base {
            Base::Bin => 2u32,
            Base::Oct => 8,
            Base::Dec => 10,
            Base::Hex => 16,
        };
        // Bit panel follows the ENTRY (docs/16 D24; E089 shows zeros with an
        // empty entry despite history rows): parse the current text
        // tolerantly; empty or unparseable → zeros. Skipped while the env
        // is borrowed elsewhere (the next change re-renders).
        let text = self.entry.text();
        let trimmed = text.trim();
        let mut parsed: Option<radix_engine::Value> = None;
        if !trimmed.is_empty() {
            if let Ok(mut env) = self.env.try_borrow_mut() {
                if let Ok(v) = evaluate(trimmed, &mut env) {
                    parsed = Some(v);
                }
            }
        }
        // Low `word_bits` bits of the integer part, masked into the word
        // domain (docs/04 §6); cells beyond the word size blank.
        let bin = parsed
            .as_ref()
            .map(|v| radix_engine::render_in_base(v, Base::Bin, 0))
            .unwrap_or_default();
        let int_part = bin.split('.').next().unwrap_or("").replace('−', "-");
        let masked: u64 = if int_part.is_empty() {
            0
        } else if let Ok(mag) = u64::from_str_radix(&int_part, 2) {
            mag
        } else {
            // Negative or overflowing: mask via two's complement width.
            let mag = i128::from_str_radix(&int_part, 2).unwrap_or(0);
            let width = 1u128 << word_bits.min(64);
            ((mag.rem_euclid(width as i128)) as u64) & (u64::MAX >> (64 - word_bits.min(64)))
        };
        for (i, lbl) in self.bit_cells.iter().enumerate() {
            let bit_index = 63 - i as u32; // leftmost = MSB
            if bit_index >= word_bits {
                lbl.set_text("");
            } else {
                let bit = (masked >> bit_index) & 1;
                lbl.set_text(if bit == 1 { "1" } else { "0" });
            }
        }
        // Digit dimming: digit buttons >= the radix are insensitive
        // (docs/02 §8.4: hex letters dim when invalid for the base).
        for (btn, v) in self.prg_base_digits.borrow().iter() {
            btn.set_sensitive(*v < radix);
        }
        // Octal/hex readout under the entry ("52305307₈ A98AC7₁₆", evidence
        // 2026-11-04 20:17:09); zeros mirror the bit panel when empty.
        let octv = parsed
            .as_ref()
            .map(|v| radix_engine::render_in_base(v, Base::Oct, 0))
            .unwrap_or_else(|| "0".to_string());
        let hexv = parsed
            .as_ref()
            .map(|v| radix_engine::render_in_base(v, Base::Hex, 0))
            .unwrap_or_else(|| "0".to_string());
        self.radix_line.set_text(&format!("{octv}₈ {hexv}₁₆"));
    }

    /// Separator line under the last row (above the entry strip) only while
    /// rows exist — matches the original's boxed-row borders.
    fn sync_has_rows(self: &Rc<Self>) {
        if self.history.borrow().is_empty() {
            self.history_list.remove_css_class("has-rows");
        } else {
            self.history_list.add_css_class("has-rows");
        }
    }

    fn append_history_row(self: &Rc<Self>) {
        let (echo, result_text) = {
            let rows = self.history.borrow();
            let row = rows.last().unwrap();
            (row.echo.clone(), row.render(&self.fmt.borrow()))
        };
        let row_widget = history_row_widget(&echo, &result_text);
        self.history_list.append(&row_widget);
        self.sync_has_rows();
        // Auto-scroll to the newest row (docs/09 §2, inferred behavior).
        let scroll = self.history_scroll.clone();
        glib::idle_add_local_once(move || {
            let vadj = scroll.vadjustment();
            vadj.set_value(vadj.upper());
        });
    }

    pub fn rerender_history(self: &Rc<Self>) {
        // Rebuild all rows (results re-render live; echoes are stored text —
        // docs/07 §5). Cheap at realistic history sizes.
        while let Some(child) = self.history_list.first_child() {
            self.history_list.remove(&child);
        }
        let rows = self.history.borrow();
        for row in rows.iter() {
            let result_text = row.render(&self.fmt.borrow());
            let w = history_row_widget(&row.echo, &result_text);
            self.history_list.append(&w);
        }
        drop(rows);
        self.sync_has_rows();
        // Re-render a result shown IN the entry (Keyboard recall); the
        // post-evaluation entry is empty by design (docs/16 D22).
        if self.entry.state() == EntryState::ResultShown && !self.entry.text().trim().is_empty() {
            if let Some(text) = self.history.borrow().last().map(|r| r.render(&self.fmt.borrow())) {
                self.entry.show_result(&text);
            }
        }
    }

    pub fn clear_history(self: &Rc<Self>) {
        self.history.borrow_mut().clear();
        *self.history_idx.borrow_mut() = None;
        while let Some(child) = self.history_list.first_child() {
            self.history_list.remove(&child);
        }
        self.sync_has_rows();
    }

    fn cycle_history(self: &Rc<Self>, forward: bool) {
        let len = self.history.borrow().len();
        if len == 0 {
            return;
        }
        let cur = *self.history_idx.borrow();
        let next: Option<usize> = match (cur, forward) {
            (None, true) => Some(0),
            (None, false) => Some(len - 1),
            (Some(i), true) => {
                if i + 1 < len {
                    Some(i + 1)
                } else {
                    None
                }
            }
            (Some(0), false) => None,
            (Some(i), false) => Some(i - 1),
        };
        *self.history_idx.borrow_mut() = next;
        match next {
            Some(i) => {
                let echo = self.history.borrow()[i].echo.clone();
                self.entry.show_input(&echo);
            }
            None => self.entry.show_input(""),
        }
    }
}

pub fn prose_for(kind: &ErrorKind) -> Option<&'static str> {
    match kind {
        ErrorKind::DivisionByZero => Some("Division by zero is undefined"),
        ErrorKind::TangentUndefined => Some(
            "Tangent is undefined for angles that are multiples of π (180°) from π/2 (90°)",
        ),
        ErrorKind::ParseError => Some("Malformed expression"),
        _ => None,
    }
}

/// One history row widget: equation left, "=" center, bold result right
/// (docs/11 §2.2).
/// Display-only bit panel (docs/02 §8.4, layout per owner evidence):
/// two rows of eight nibble groups with bit-index labels 63/47/32 and
/// 31/15/0. Cells are returned MSB-first (row-major from bit 63).
fn build_bit_panel() -> (gtk::Box, Vec<gtk::Label>) {
    let panel = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(11)
        .build();
    let mut cells = Vec::new();
    for row in 0..2u32 {
        let h = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(14)
            .build();
        let mk_idx = |txt: &str| {
            gtk::Label::builder()
                .label(txt)
                .css_classes(["bit-index"])
                .valign(gtk::Align::Center)
                .build()
        };
        h.append(&mk_idx(if row == 0 { "63" } else { "31" }));
        for half in 0..2u32 {
            if half == 1 {
                h.append(&mk_idx(if row == 0 { "47" } else { "15" }));
            }
            for _g in 0..4u32 {
                let gb = gtk::Box::builder()
                    .orientation(gtk::Orientation::Horizontal)
                    .spacing(4)
                    .hexpand(true)
                    .halign(gtk::Align::Fill)
                    .build();
                for _b in 0..4u32 {
                    let idx = 63 - cells.len() as u32;
                    let cell = gtk::Label::builder()
                        .label("0")
                        .width_chars(1)
                        .css_classes(["bit-cell"])
                        .build();
                    cell.set_tooltip_text(Some(&format!("{idx}")));
                    cells.push(cell.clone());
                    gb.append(&cell);
                }
                h.append(&gb);
            }
        }
        h.append(&mk_idx(if row == 0 { "32" } else { "0" }));
        panel.append(&h);
    }
    (panel, cells)
}

fn history_row_widget(echo: &str, result_text: &str) -> gtk::ListBoxRow {
    let row_widget = gtk::ListBoxRow::builder().css_classes(["history-row"]).build();
    // Forty homogeneous columns (percentages measured off E085 at 410px and
    // the 750px batch): the echo spans the first 16 (40% — "3×111+(23÷60)×
    // 111" wraps after 16 glyphs at 410px, one line at 750px, while
    // "0.833333333×111" stays on one line), the dim "=" centers across
    // columns 24-26 (63.75% — measured 63.1–63.6%), and the result
    // right-aligns across the last 12, ellipsizing when narrow ("0.8333…"
    // at 410px, full at 750px). History text is ~19px at every width (digit
    // glyph height 14px in all evidence).
    let grid = gtk::Grid::builder()
        .column_homogeneous(true)
        // 20px side padding and a 37px row pitch (7px vertical margins)
        // measured on the 2026-11-04 20:16 batch at 360px.
        .margin_start(20)
        .margin_end(20)
        .margin_top(7)
        .margin_bottom(7)
        .build();
    let eq = gtk::Label::builder()
        .label(echo)
        .xalign(0.0)
        .wrap(true)
        // CHAR wrap: E085 splits the long echo mid-number ("…×11 / 1"),
        // and character wrapping also keeps the label's minimum at one
        // glyph so it can never inflate the homogeneous grid (GTK4 windows
        // grow to content natural — measured 410→474px without this).
        .wrap_mode(gtk::pango::WrapMode::Char)
        .width_request(1)
        // Caps the label's NATURAL width (width_request only sets the
        // minimum): GTK4 windows grow to content natural, so an uncapped
        // echo would push the 40-col grid past the 410px default. 750px
        // still renders every observed echo on one line.
        .max_width_chars(17)
        .valign(gtk::Align::Center)
        .css_classes(["echo"])
        .build();
    let sep = gtk::Label::builder()
        .label("=")
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .width_request(1)
        .css_classes(["dim"])
        .build();
    let res = gtk::Label::builder()
        .label(result_text)
        .halign(gtk::Align::End)
        .xalign(1.0)
        .valign(gtk::Align::Center)
        .ellipsize(gtk::pango::EllipsizeMode::End)
        .width_request(1)
        // Same natural-cap reasoning: ~9 chars keeps the 40-col grid under
        // the 410px default while the 750px result band (~210px) renders
        // every observed result unclipped.
        .max_width_chars(12)
        .css_classes(["result"])
        .build();
    grid.attach(&eq, 0, 0, 19, 1);
    grid.attach(&sep, 24, 0, 3, 1);
    grid.attach(&res, 27, 0, 10, 1);
    row_widget.set_child(Some(&grid));
    row_widget.set_tooltip_text(Some(&format!("{echo} = {result_text}")));
    row_widget
}

/// CSS provider with the docs/11 §3 palette for both themes; reloaded live on
/// color-scheme change (docs/11 §6.4).
pub fn load_css() {
    // Test hook: RADIX_DARK=1 forces the dark palette regardless of the
    // (sandboxed) system color scheme.
    if std::env::var_os("RADIX_DARK").is_some() {
        let display = gdk::Display::default().unwrap();
        adw::StyleManager::for_display(&display)
            .set_color_scheme(adw::ColorScheme::ForceDark);
    }
    fn css(dark: bool) -> String {
        // Surfaces sampled from the original (2026-11-04 18:5x evidence
        // batch): window/header stay STOCK libadwaita (the app must follow
        // the system palette like the original does); the card is the
        // elevated surface and the entry/bit strip a slightly darker inset.
        let (card, strip, btn, digit_btn, dim, err_fg, _err_hl, hover, press, sep, inv_bg, inv_fg, pane)
            = if dark {
            (
                "#3d3d41",
                "#333337",
                "#38383b",
                "#4e4e51",
                "#7e7e81",
                "#ff8787",
                "#572026",
                "#444448",
                "#4e4e52",
                "rgba(0,0,0,0.36)",
                "#3b2f38",
                "#f08aba",
                "#323236",
            )
        } else {
            (
                "#ffffff",
                "#f2f2f3",
                "#e6e6e7",
                "#d2d2d4",
                "#8d8d92",
                "#c30000",
                "#f6bbbe",
                "#c8c8ca",
                "#b4b4b7",
                "rgba(0,0,0,0.15)",
                "#f7e9f2",
                "#c2478a",
                "#e9e9eb",
            )
        };
        format!(
            r#"
            @define-color error_color {err_fg};
            /* No window/headerbar/accent overrides: stock libadwaita colors,
               like the original (accent follows the system scheme). */
            .title-mode-btn label {{ font-weight: 700; font-size: 15px; }}
            .display-card {{
                background: {card}; border-radius: 12px;
                /* drop shadow: the darkened ~12px bands around the card in
                   the original (2026-11-04 20:1x scans) */
                box-shadow: 0 2px 12px alpha(black, 0.45);
                padding: 0;
            }}
            /* Basic full-bleed: shadow falls below only (none over the
               headerbar edge in the evidence). */
            .display-card.flush {{
                border-radius: 0;
                box-shadow: 0 4px 10px alpha(black, 0.45);
            }}
            .conv-pane {{
                background: {pane};
                box-shadow: 0 4px 10px alpha(black, 0.45);
            }}
            .history-list {{ background: transparent; }}
            .history-list.has-rows {{ border-bottom: 1px solid {sep}; }}
            .history-row {{
                background: transparent; border-top: 1px solid {sep};
                padding: 0;
            }}
            .history-row:hover {{ background: alpha(currentColor, 0.05); }}
            .history-row .echo {{ font-size: 19px; }}
            .history-row .dim {{ color: {dim}; font-size: 14px; }}
            .history-row .result {{ font-weight: 700; font-size: 19px; }}
            .entry-strip {{ background: {strip}; }}
            .entry-strip textview {{
                background: transparent; color: inherit;
                padding: 8px 14px; font-size: 19px;
            }}
            .entry-strip textview text {{
                background: transparent; caret-color: currentColor;
            }}
            .bit-panel {{ padding: 12px 19px 13px 19px; }}
            .bit-cell {{ font-weight: 700; font-size: 11px; min-width: 11px; }}
            .bit-index {{ color: {dim}; font-size: 11px; }}
            .radix-line {{ color: {dim}; font-size: 13px; }}
            .radix-btn.bold-label {{ font-weight: 700; }}
            textview.conv-value, textview.conv-value text {{
                font-size: 26px; font-weight: 700; background: transparent;
            }}
            menubutton.conv-unit-btn {{ background: none; }}
            menubutton.conv-unit-btn button {{
                background: none; border: none; box-shadow: none;
                padding: 2px 4px; min-height: 0; color: @window_fg_color;
            }}
            /* unit + category labels: pin at label level too — the flat
               menubutton chain dims labels to ~62% otherwise */
            menubutton.conv-unit-btn button label,
            label.conv-unit {{
                font-size: 15px; font-weight: 700;
                color: @window_fg_color;
            }}
            .inverse-toggle {{
                background: {inv_bg}; color: {inv_fg};
                border-radius: 6px; font-size: 14px; padding: 0;
                min-width: 62px; min-height: 40px;
            }}
            .inverse-toggle:hover {{
                background-image: linear-gradient(alpha(white,0.06), alpha(white,0.06));
            }}
            .inverse-toggle:checked {{
                background: @accent_bg_color; color: @accent_fg_color;
            }}
            .error-message {{ color: {err_fg}; font-size: 12px; }}
            .radix-btn {{
                background: {btn}; border-radius: 7px; font-size: 15px; padding: 0;
                min-width: 40px; min-height: 40px;
            }}
            /* GtkMenuButton's internal toggle keeps the theme's ~11px side
             * padding, pushing its natural to 67px; with a homogeneous grid
             * that widens EVERY column (window grew to 385). Zero it. */
            menubutton.radix-btn > button {{
                padding: 0; min-width: 0; min-height: 0;
            }}
            .radix-btn:hover {{ background: {hover}; }}
            .radix-btn:active {{ background: {press}; }}
            .radix-btn.digit {{
                background: {digit_btn}; font-size: 17px; font-weight: 700;
            }}
            .radix-btn.digit:hover {{
                background-image: linear-gradient(alpha(white,0.06), alpha(white,0.06));
            }}
            .radix-btn.digit:active {{
                background-image: linear-gradient(alpha(black,0.13), alpha(black,0.13));
            }}
            .radix-btn.equals {{
                background: @accent_bg_color; color: @accent_fg_color;
                font-size: 17px; font-weight: 700;
            }}
            .radix-btn.equals:hover {{
                background-image: linear-gradient(alpha(black,0.10), alpha(black,0.10));
            }}
            .radix-btn.equals:active {{
                background-image: linear-gradient(alpha(black,0.20), alpha(black,0.20));
            }}
            "#,
        )
    }
    let display = gdk::Display::default().unwrap();
    let sm = adw::StyleManager::for_display(&display);
    PROVIDER.with(|p| {
        p.load_from_string(&css(sm.is_dark()));
        gtk::style_context_add_provider_for_display(
            &display,
            p,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    });
    // Live re-theme (docs/11 §6.4): swap palette when the scheme changes.
    sm.connect_notify(Some("dark"), move |sm, _| {
        PROVIDER.with(|p| {
            p.load_from_string(&css(sm.is_dark()));
        });
    });
}

thread_local! {
    static PROVIDER: gtk::CssProvider = gtk::CssProvider::new();
}
