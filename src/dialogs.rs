//! Dialogs: Preferences, Keyboard Shortcuts, About, Help, and the Financial
//! formula forms (docs/02 §4–§7, §9; docs/04 §7).

use std::rc::Rc;

use adw::prelude::*;

use crate::win::RadixWindow;
use radix_engine::{AngleUnit, WordSize};

pub fn open_about(w: &Rc<RadixWindow>) {
    let dialog = adw::AboutWindow::builder()
        .transient_for(&w.window)
        .application_name("Radix")
        .application_icon("io.github.hakergeniusz.Radix")
        .version(env!("CARGO_PKG_VERSION"))
        .developers(["hakergeniusz"])
        .license_type(gtk::License::MitX11)
        .website("https://github.com/hakergeniusz/radix")
        .issue_url("https://github.com/hakergeniusz/radix/issues")
        .copyright("© 2026 The Radix Authors")
        .build();
    dialog.present();
}

pub fn open_preferences(w: &Rc<RadixWindow>) {
    // Fixed narrow dialog, independent of the parent window (E093: 513-wide
    // screenshot shows a ~365px dialog over a 750px window; 2026-11-04
    // 18:56 evidence shows the same over a 410px window).
    let prefs = adw::PreferencesWindow::builder()
        .transient_for(&w.window)
        .title("Preferences")
        .search_enabled(false)
        .default_width(360)
        .default_height(590)
        .build();

    // ---- General ----
    let page = adw::PreferencesPage::new();
    let group = adw::PreferencesGroup::builder().title("General").build();

    let decimals = adw::SpinRow::with_range(0.0, 15.0, 1.0);
    decimals.set_title("Number of Decimals");
    decimals.set_value(w.fmt.borrow().decimals as f64);
    {
        let ww = w.clone();
        decimals.connect_changed(move |row| {
            ww.fmt.borrow_mut().decimals = row.value() as u32;
            ww.rerender_history();
        });
    }
    group.add(&decimals);

    let trailing = adw::SwitchRow::builder().title("Trailing Zeroes").build();
    trailing.set_active(w.fmt.borrow().trailing_zeroes);
    {
        let ww = w.clone();
        trailing.connect_active_notify(move |row| {
            ww.fmt.borrow_mut().trailing_zeroes = row.is_active();
            ww.rerender_history();
        });
    }
    group.add(&trailing);

    let grouping = adw::SwitchRow::builder().title("Group Digits").build();
    grouping.set_active(w.fmt.borrow().group);
    {
        let ww = w.clone();
        grouping.connect_active_notify(move |row| {
            ww.fmt.borrow_mut().group = row.is_active();
            ww.rerender_history();
        });
    }
    group.add(&grouping);

    let angle = adw::ComboRow::builder().title("Angle Units").build();
    let angle_model = gtk::StringList::new(&["Radians", "Degrees", "Gradians"]);
    angle.set_model(Some(&angle_model));
    angle.set_selected(match w.env.borrow().angle_unit {
        AngleUnit::Radians => 0,
        AngleUnit::Degrees => 1,
        AngleUnit::Gradians => 2,
    });
    {
        let ww = w.clone();
        angle.connect_selected_notify(move |row| {
            let unit = match row.selected() {
                0 => AngleUnit::Radians,
                1 => AngleUnit::Degrees,
                _ => AngleUnit::Gradians,
            };
            ww.env.borrow_mut().angle_unit = unit;
        });
    }
    group.add(&angle);

    let word = adw::ComboRow::builder().title("Word Size").build();
    let word_model = gtk::StringList::new(&["8-bit", "16-bit", "32-bit", "64-bit"]);
    word.set_model(Some(&word_model));
    word.set_selected(match w.env.borrow().word_size {
        WordSize::W8 => 0,
        WordSize::W16 => 1,
        WordSize::W32 => 2,
        WordSize::W64 => 3,
    });
    {
        let ww = w.clone();
        word.connect_selected_notify(move |row| {
            let ws = match row.selected() {
                0 => WordSize::W8,
                1 => WordSize::W16,
                2 => WordSize::W32,
                _ => WordSize::W64,
            };
            ww.env.borrow_mut().word_size = ws;
        });
    }
    group.add(&word);
    page.add(&group);

    // ---- Currency Conversion ----
    let cgroup = adw::PreferencesGroup::builder()
        .title("Currency Conversion")
        .build();
    let cur = &w.currency;
    let interval = adw::ComboRow::builder().title("Exchange Rate Refresh Interval").build();
    interval.set_model(Some(&gtk::StringList::new(&["Never", "Daily", "Weekly", "Monthly"])));
    {
        use crate::data::currency::RefreshInterval as I;
        interval.set_selected(match cur.prefs.borrow().interval {
            I::Never => 0,
            I::Daily => 1,
            I::Weekly => 2,
            I::Monthly => 3,
        });
    }
    {
        let ww = w.clone();
        interval.connect_selected_notify(move |row| {
            use crate::data::currency::RefreshInterval as I;
            let v = match row.selected() {
                0 => I::Never,
                1 => I::Daily,
                3 => I::Monthly,
                _ => I::Weekly,
            };
            ww.currency.prefs.borrow_mut().interval = v;
            // A shorter interval may make the cache immediately stale.
            ww.maybe_refresh_rates();
        });
    }
    cgroup.add(&interval);

    let display = adw::ComboRow::builder().title("Currency Display").build();
    display.set_model(Some(&gtk::StringList::new(&["Currency Name", "Currency Code"])));
    {
        use crate::data::currency::CurrencyDisplay as D;
        display.set_selected(match cur.prefs.borrow().display {
            D::Name => 0,
            D::Code => 1,
        });
    }
    {
        let ww = w.clone();
        display.connect_selected_notify(move |row| {
            use crate::data::currency::CurrencyDisplay as D;
            ww.currency.prefs.borrow_mut().display = match row.selected() {
                1 => D::Code,
                _ => D::Name,
            };
            ww.entry.update_completion();
        });
    }
    cgroup.add(&display);

    let completion = adw::ComboRow::builder()
        .title("Show Currencies in Auto-Completion List")
        // Wrap to two lines (the original wraps at its 360px width);
        // without this the unwrapped title inflates the window's natural
        // width past the 360px default (observed 405px).
        .title_lines(2)
        .build();
    completion.set_model(Some(&gtk::StringList::new(&["All", "Favorites"])));
    {
        use crate::data::currency::CurrencyShow as S;
        completion.set_selected(match cur.prefs.borrow().show {
            S::All => 0,
            S::Favorites => 1,
        });
    }
    {
        let ww = w.clone();
        completion.connect_selected_notify(move |row| {
            use crate::data::currency::CurrencyShow as S;
            ww.currency.prefs.borrow_mut().show = match row.selected() {
                1 => S::Favorites,
                _ => S::All,
            };
            ww.entry.update_completion();
        });
    }
    cgroup.add(&completion);

    let fav = adw::ActionRow::builder().title("Favorite Currencies").activatable(true).build();
    fav.add_suffix(&gtk::Image::from_icon_name("go-next-symbolic"));
    {
        let ww = w.clone();
        fav.connect_activated(move |_| open_favorite_currencies(&ww));
    }
    cgroup.add(&fav);
    page.add(&cgroup);
    prefs.add(&page);
    prefs.present();
}

/// "Favorite Currencies" page (docs/02 §3; layout per the 2026-11-04
/// 18:56:15 evidence): back-arrow header, always-visible search entry,
/// two-line rows (currency name, code subtitle) sorted by NAME, a switch
/// per currency; starred codes drive the completion filter.
fn open_favorite_currencies(w: &Rc<RadixWindow>) {
    use adw::prelude::*;
    let dialog = adw::Window::builder()
        .title("Favorite Currencies")
        .transient_for(&w.window)
        .modal(true)
        .default_width(370)
        .default_height(620)
        .build();

    let header = adw::HeaderBar::new();
    let back = gtk::Button::from_icon_name("go-previous-symbolic");
    back.add_css_class("flat");
    {
        let d = dialog.clone();
        back.connect_clicked(move |_| d.close());
    }
    header.pack_start(&back);

    let search = gtk::SearchEntry::builder()
        .placeholder_text("Search currencies")
        .margin_top(12)
        .margin_bottom(6)
        .margin_start(12)
        .margin_end(12)
        .build();

    let list = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .css_classes(["boxed-list"])
        .margin_top(6)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .valign(gtk::Align::Start)
        .build();
    let mut sorted: Vec<_> = crate::data::currency::all().to_vec();
    sorted.sort_by_key(|c| c.name);
    let mut filter_rows: Vec<(adw::ActionRow, String)> = Vec::new();
    for c in &sorted {
        let row = adw::ActionRow::builder()
            .title(c.name)
            .subtitle(c.code)
            .build();
        let sw = gtk::Switch::builder().valign(gtk::Align::Center).build();
        sw.set_active(w.currency.favorites.borrow().iter().any(|f| f == c.code));
        row.add_suffix(&sw);
        {
            let ww = w.clone();
            let code = c.code.to_string();
            sw.connect_active_notify(move |s| {
                let mut favs = ww.currency.favorites.borrow_mut();
                if s.is_active() {
                    if !favs.contains(&code) {
                        favs.push(code.clone());
                    }
                } else {
                    favs.retain(|f| *f != code);
                }
                drop(favs);
                ww.entry.update_completion();
            });
        }
        filter_rows.push((row.clone(), format!("{} {}", c.name, c.code).to_lowercase()));
        list.append(&row);
    }
    {
        let rows = filter_rows.clone();
        search.connect_search_changed(move |e| {
            let q = e.text().trim().to_lowercase();
            for (row, hay) in &rows {
                row.set_visible(q.is_empty() || hay.contains(&q));
            }
        });
    }

    let scroll = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .child(&list)
        .build();
    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content.append(&header);
    content.append(&search);
    content.append(&scroll);
    dialog.set_content(Some(&content));
    dialog.present();
}

pub fn open_shortcuts(w: &Rc<RadixWindow>) {
    let win = gtk::ShortcutsWindow::builder()
        .transient_for(&w.window)
        .title("Keyboard Shortcuts")
        .modal(true)
        .build();
    let section = gtk::ShortcutsSection::builder().section_name("all").build();

    let rows: &[(&str, &[&str], &[&str])] = &[
        ("General", &["New Window", "Close Current Window", "Quit the Application", "Show Help", "Open Menu", "Show Preferences", "Keyboard Shortcuts", "Clear History"],
         &["Ctrl+N", "Ctrl+W", "Ctrl+Q", "F1", "F10", "Ctrl+,", "Ctrl+?", "Ctrl+Escape"]),
        ("Switching Modes", &["Basic", "Advanced", "Financial", "Programming", "Keyboard", "Conversion"],
         &["Ctrl+Alt+B", "Ctrl+Alt+A", "Ctrl+Alt+F", "Ctrl+Alt+P", "Ctrl+Alt+K", "Ctrl+Alt+C"]),
        ("Programming Mode", &["Binary", "Octal", "Decimal", "Hexadecimal"],
         &["Ctrl+B", "Ctrl+O", "Ctrl+D", "Ctrl+H"]),
        ("Keyboard Entry", &["Multiply / Divide", "Square ²", "Square Root √", "Inverse ⁻¹", "Pi π", "Tau τ", "Degree °", "Floor ⌊⌋", "Ceiling ⌈⌉", "Superscript digits", "Superscript minus"],
         &["* /", "Ctrl+2", "Ctrl+R", "Ctrl+I", "Ctrl+P", "Ctrl+T", "Ctrl+'", "Alt+[ then ]", "Ctrl+[ then ]", "Ctrl+0..9", "Ctrl+−"]),
        ("Equation", &["Copy", "Paste", "Undo", "Redo", "Previous / Next Equation"],
         &["Ctrl+C", "Ctrl+V", "Ctrl+Z", "Ctrl+Shift+Z", "Alt+←/→"]),
    ];
    for (title, actions, keys) in rows {
        let group = gtk::ShortcutsGroup::builder().title(*title).build();
        for (a, k) in actions.iter().zip(keys.iter()) {
            let s = gtk::ShortcutsShortcut::builder()
                .title(*a)
                .accelerator(*k)
                .build();
            group.append(&s);
        }
        section.append(&group);
    }
    win.set_child(Some(&section));
    win.present();
}

pub fn open_help(w: &Rc<RadixWindow>) {
    let win = adw::Window::builder()
        .transient_for(&w.window)
        .title("Radix Help")
        .default_width(700)
        .default_height(560)
        .build();
    let search = gtk::SearchEntry::new();
    let list = gtk::ListBox::builder().css_classes(["boxed-list"]).build();
    let items: &[(&str, Vec<&str>)] = &[
        ("User Interface", vec!["Using the History View", "Using the Mouse", "Using the Keyboard", "Superscript and Subscript"]),
        ("Equations", vec!["Basic Equations", "Absolute Values", "Bitwise Operations", "Factorials", "Factorization", "Percentages", "Powers and Roots", "Rounding", "Logarithms", "Modulus Division", "Trigonometry", "Variables and Constants"]),
    ];
    for (section, entries) in items {
        let header = gtk::ListBoxRow::new();
        let l = gtk::Label::builder().label(*section).xalign(0.0).css_classes(["heading", "dim-label"]).build();
        header.set_child(Some(&l));
        header.set_selectable(false);
        list.append(&header);
        for e in entries {
            let row = gtk::ListBoxRow::new();
            let l = gtk::Label::builder().label(*e).xalign(0.0).margin_top(4).margin_bottom(4).margin_start(10).margin_end(10).build();
            row.set_child(Some(&l));
            row.set_selectable(false);
            list.append(&row);
        }
    }
    let scroll = gtk::ScrolledWindow::builder()
        .child(&list)
        .vexpand(true)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .build();
    let box_ = gtk::Box::new(gtk::Orientation::Vertical, 8);
    box_.set_margin_top(12);
    box_.set_margin_bottom(12);
    box_.set_margin_start(12);
    box_.set_margin_end(12);
    box_.append(&search);
    box_.append(&scroll);
    win.set_content(Some(&box_));
    win.present();
}

// ---------------------------------------------------------------------------
// Financial formula dialogs (docs/02 §9; formulas per D9)
// ---------------------------------------------------------------------------

struct FinSpec {
    id: &'static str,
    title: &'static str,
    description: &'static str,
    fields: &'static [&'static str],
}

const FIN_SPECS: &[FinSpec] = &[
    FinSpec {
        id: "Ctrm",
        title: "Compounding Term",
        description: "Number of periods for an investment to grow from present value to future value at a periodic interest rate.",
        fields: &["Periodic Interest Rate", "Future Value", "Present Value"],
    },
    FinSpec {
        id: "Ddb",
        title: "Double-Declining Depreciation",
        description: "Depreciation for a period under the double-declining balance method.",
        fields: &["Cost", "Life", "Period"],
    },
    FinSpec {
        id: "Fv",
        title: "Future Value",
        description: "Future value of a series of equal periodic payments at a periodic interest rate.",
        fields: &["Periodic Payment", "Periodic Interest Rate", "Number of Periods"],
    },
    FinSpec {
        id: "Gpm",
        title: "Gross Profit Margin",
        description: "Selling price for a given cost and gross profit margin.",
        fields: &["Cost", "Margin"],
    },
    FinSpec {
        id: "Pmt",
        title: "Periodic Payment",
        description: "Payment for a loan of a given principal, periodic interest rate and term.",
        fields: &["Principal", "Periodic Interest Rate", "Term"],
    },
    FinSpec {
        id: "Pv",
        title: "Present Value",
        description: "Present value of a series of equal periodic payments at a periodic interest rate.",
        fields: &["Periodic Payment", "Periodic Interest Rate", "Number of Periods"],
    },
    FinSpec {
        id: "Rate",
        title: "Periodic Interest Rate",
        description: "Periodic interest rate required for a present value to grow to a future value over a term.",
        fields: &["Future Value", "Present Value", "Term"],
    },
    FinSpec {
        id: "Sln",
        title: "Straight-Line Depreciation",
        description: "Depreciation per period under the straight-line method.",
        fields: &["Cost", "Salvage", "Life"],
    },
    FinSpec {
        id: "Syd",
        title: "Sum-of-the-Years'-Digits Depreciation",
        description: "Depreciation for a period under the sum-of-the-years'-digits method.",
        fields: &["Cost", "Salvage", "Life", "Period"],
    },
    FinSpec {
        id: "Term",
        title: "Payment Period",
        description: "Number of periodic payments needed to reach a future value.",
        fields: &["Periodic Payment", "Future Value", "Periodic Interest Rate"],
    },
];

pub fn open_financial(w: &Rc<RadixWindow>, id: &str) {
    let Some(spec) = FIN_SPECS.iter().find(|s| s.id == id) else {
        return;
    };
    let dialog = adw::Dialog::builder()
        .title(spec.title)
        .content_width(340)
        .build();

    let box_ = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .margin_top(18)
        .margin_bottom(18)
        .margin_start(18)
        .margin_end(18)
        .build();

    let desc = gtk::Label::builder()
        .label(spec.description)
        .wrap(true)
        .xalign(0.0)
        .css_classes(["dim-label"])
        .build();
    box_.append(&desc);

    let mut entries: Vec<gtk::Entry> = Vec::new();
    for (i, field) in spec.fields.iter().enumerate() {
        let row = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(8)
            .build();
        let lbl = gtk::Label::builder().label(*field).xalign(0.0).hexpand(true).build();
        let entry = gtk::Entry::builder().text("0").width_chars(12).build();
        if i == 0 {
            entry.grab_focus();
            entry.select_region(0, 1);
        }
        row.append(&lbl);
        row.append(&entry);
        box_.append(&row);
        entries.push(entry);
    }

    let err_label = gtk::Label::builder()
        .wrap(true)
        .xalign(0.0)
        .css_classes(["error-message"])
        .visible(false)
        .build();
    box_.append(&err_label);

    let btns = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .halign(gtk::Align::End)
        .build();
    let cancel = gtk::Button::with_label("Cancel");
    let calc = gtk::Button::with_label("Calculate");
    calc.add_css_class("suggested-action");
    btns.append(&cancel);
    btns.append(&calc);
    box_.append(&btns);

    dialog.set_child(Some(&box_));
    let dlg = dialog.clone();
    cancel.connect_clicked(move |_| {
        dlg.close();
    });

    let dlg2 = dialog.clone();
    let w2 = w.clone();
    let spec_id = spec.id;
    calc.connect_clicked(move |_| {
        let vals: Vec<f64> = entries
            .iter()
            .map(|e| e.text().trim().parse::<f64>().unwrap_or(f64::NAN))
            .collect();
        if vals.iter().any(|v| v.is_nan()) {
            err_label.set_text("All fields must be numeric.");
            err_label.set_visible(true);
            return;
        }
        match compute_financial(spec_id, &vals) {
            Ok(v) => {
                let fmt = w2.fmt.borrow();
                let text = crate::dialogs::fmt_f64(v, fmt.decimals, fmt.trailing_zeroes, fmt.group);
                // Dialog results fill the entry; NO history row (docs/09 §1).
                w2.entry.show_result(&text);
                w2.message.set_visible(false);
                dlg2.close();
            }
            Err(msg) => {
                err_label.set_text(&msg);
                err_label.set_visible(true);
            }
        }
    });

    dialog.present(Some(&w.window));
}

/// Insert Character Code dialog (docs/02 §10, T084): a "Character" field +
/// "Convert to: Unicode Code Point" dropdown + Cancel/Insert. Inserting 'A'
/// enters the code point (65) rendered in the current base.
pub fn open_char_code(w: &Rc<RadixWindow>) {
    let dialog = adw::Dialog::builder()
        .title("Insert Character Code")
        .content_width(300)
        .build();
    let box_ = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .margin_top(16)
        .margin_bottom(16)
        .margin_start(16)
        .margin_end(16)
        .build();

    let row = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .build();
    row.append(&gtk::Label::builder().label("Character").xalign(0.0).hexpand(true).build());
    let field = gtk::Entry::builder().max_width_chars(4).width_chars(4).build();
    field.grab_focus();
    row.append(&field);
    box_.append(&row);

    let combo_row = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .build();
    combo_row.append(&gtk::Label::builder().label("Convert to").xalign(0.0).hexpand(true).build());
    let combo = gtk::DropDown::from_strings(&["Unicode Code Point"]);
    combo_row.append(&combo);
    box_.append(&combo_row);

    let btns = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .halign(gtk::Align::End)
        .build();
    let cancel = gtk::Button::with_label("Cancel");
    let insert = gtk::Button::with_label("Insert");
    insert.add_css_class("suggested-action");
    btns.append(&cancel);
    btns.append(&insert);
    box_.append(&btns);

    dialog.set_child(Some(&box_));
    let dlg = dialog.clone();
    cancel.connect_clicked(move |_| {
        dlg.close();
    });

    let dlg2 = dialog.clone();
    let w2 = w.clone();
    insert.connect_clicked(move |_| {
        let text = field.text();
        if let Some(ch) = text.chars().next() {
            let cp = ch as u32;
            let base = w2.env.borrow().base;
            let rendered = to_base(cp, base);
            w2.entry.insert_at_caret(&rendered, 0);
            dlg2.close();
        }
    });

    dialog.present(Some(&w.window));
}

fn to_base(mut v: u32, base: radix_engine::Base) -> String {
    let digits = "0123456789ABCDEF";
    let radix = match base {
        radix_engine::Base::Bin => 2,
        radix_engine::Base::Oct => 8,
        radix_engine::Base::Hex => 16,
        radix_engine::Base::Dec => 10,
    };
    if v == 0 {
        return "0".into();
    }
    let mut out = Vec::new();
    while v > 0 {
        out.push(digits.as_bytes()[(v % radix) as usize] as char);
        v /= radix;
    }
    out.into_iter().rev().collect()
}

/// Standard financial formula implementations (D9). Only Fv is
/// contract-verified (docs/12 T083); the others follow standard forms.
pub fn compute_financial(id: &str, v: &[f64]) -> Result<f64, String> {
    let r_zero = |r: f64| r.abs() < 1e-15;
    match id {
        // Ctrm: rate, future value, present value
        "Ctrm" => {
            let (r, fv, pv) = (v[0], v[1], v[2]);
            if r_zero(r) || r == -1.0 {
                return Err("Periodic interest rate must be non-zero.".into());
            }
            if pv == 0.0 || (fv / pv) <= 0.0 {
                return Err("Values must have the same sign and present value must be non-zero.".into());
            }
            Ok((fv / pv).ln() / (1.0 + r).ln())
        }
        // Ddb: cost, life, period
        "Ddb" => {
            let (cost, life, period) = (v[0], v[1], v[2]);
            if life <= 0.0 || period <= 0.0 {
                return Err("Life and period must be positive.".into());
            }
            let mut book = cost;
            let mut dep = 0.0;
            for _ in 1..=(period as i64).max(1) {
                dep = (2.0 / life) * book;
                if dep > book {
                    dep = book;
                }
                book -= dep;
            }
            Ok(dep)
        }
        // Fv: payment, rate, periods — contract vector: (−100, 0.05, 10) → −1257.789253555
        "Fv" => {
            let (pmt, r, n) = (v[0], v[1], v[2]);
            if r_zero(r) {
                return Ok(pmt * n);
            }
            Ok(pmt * ((1.0 + r).powf(n) - 1.0) / r)
        }
        // Gpm: cost, margin
        "Gpm" => {
            let (cost, margin) = (v[0], v[1]);
            if margin >= 1.0 {
                return Err("Margin must be below 1 (100%).".into());
            }
            Ok(cost / (1.0 - margin))
        }
        // Pmt: principal, rate, term
        "Pmt" => {
            let (p, r, n) = (v[0], v[1], v[2]);
            if r_zero(r) {
                return Ok(p / n);
            }
            Ok(p * r / (1.0 - (1.0 + r).powf(-n)))
        }
        // Pv: payment, rate, periods
        "Pv" => {
            let (pmt, r, n) = (v[0], v[1], v[2]);
            if r_zero(r) {
                return Ok(pmt * n);
            }
            Ok(pmt * (1.0 - (1.0 + r).powf(-n)) / r)
        }
        // Rate: future value, present value, term
        "Rate" => {
            let (fv, pv, n) = (v[0], v[1], v[2]);
            if pv == 0.0 || n == 0.0 || (fv / pv) <= 0.0 {
                return Err("Values must have the same sign; present value and term must be non-zero.".into());
            }
            Ok((fv / pv).powf(1.0 / n) - 1.0)
        }
        // Sln: cost, salvage, life
        "Sln" => {
            let (cost, salvage, life) = (v[0], v[1], v[2]);
            if life == 0.0 {
                return Err("Life must be non-zero.".into());
            }
            Ok((cost - salvage) / life)
        }
        // Syd: cost, salvage, life, period
        "Syd" => {
            let (cost, salvage, life, period) = (v[0], v[1], v[2], v[3]);
            if life <= 0.0 || period <= 0.0 || period > life {
                return Err("Life and period must be positive; period must not exceed life.".into());
            }
            Ok((cost - salvage) * (life - period + 1.0) * 2.0 / (life * (life + 1.0)))
        }
        // Term: payment, future value, rate
        "Term" => {
            let (pmt, fv, r) = (v[0], v[1], v[2]);
            if r_zero(r) || pmt == 0.0 {
                return Err("Payment and periodic interest rate must be non-zero.".into());
            }
            let base = 1.0 + fv * r / pmt;
            if base <= 0.0 {
                return Err("No solution for these values.".into());
            }
            Ok(base.ln() / (1.0 + r).ln())
        }
        _ => Err("Unknown formula.".into()),
    }
}

/// Format a financial result with the docs/07 display rules (delegates to
/// the shared plain formatter).
pub fn fmt_f64(v: f64, decimals: u32, trailing: bool, group: bool) -> String {
    if !v.is_finite() {
        return if v.is_nan() { "NaN".into() } else { "−".into() };
    }
    crate::fmt::plain(v, decimals, trailing, group)
}
