//! Conversion mode: category dropdown, two role-based value/unit rows, and
//! the digit keypad (docs/06; geometry docs/11 §2.5).

use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use gtk::glib;
use gtk::gio;

use crate::data::currency::{self, SharedRates};
use crate::data::units::{self, Category};
use crate::keypads;

use radix_engine::{parse_value_text, render_in_base, Base};

/// The conversion pane's state and widgets. Positions are fixed (top/bottom);
/// roles (source/target) move per docs/06 §2.
pub struct ConversionPane {
    pub container: gtk::Box,
    category_btn: gtk::MenuButton,
    pub(crate) category_label: gtk::Label,
    rows: [ValueRow; 2],
    keypad_slot: gtk::Box,
    pub(crate) state: RefCell<ConvState>,
    rates: SharedRates,
    /// Parent window for dialogs, set by the main window during assembly.
    pub parent: RefCell<Option<adw::ApplicationWindow>>,
    /// Coalescing flag: a recompute is scheduled on the main loop.
    recompute_pending: std::cell::Cell<bool>,
    /// The pane's stateful category action (radios follow its state).
    pub(crate) cat_action: RefCell<Option<gio::SimpleAction>>,
}

struct ValueRow {
    text: gtk::TextView,
    unit_btn: gtk::MenuButton,
    unit_label: gtk::Label,
}

#[derive(Debug, Clone)]
pub(crate) struct ConvState {
    pub(crate) category: Category,
    /// Raw text of each row.
    pub(crate) texts: [String; 2],
    /// Unit name per row.
    pub(crate) units: [String; 2],
    /// Index of the source row (0 = top, 1 = bottom).
    pub(crate) source: usize,
    /// Index of the row that owns the caret.
    pub(crate) active: usize,
}

pub fn build(rates: SharedRates) -> Rc<ConversionPane> {
    let container = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(0)
        .build();
    // The tinted pane surface (#323236 measured, 2026-11-04 20:18 batch)
    // covers category + value rows and the spare space above the keypad;
    // the keypad itself sits on the plain window background below it.
    let pane_bg = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(0)
        .css_classes(["conv-pane"])
        .build();

    // Category dropdown (docs/06 §1)
    let category_label = gtk::Label::new(Some(Category::Angle.label()));
    let category_btn = gtk::MenuButton::builder()
        .child(&{
            let b = gtk::Box::new(gtk::Orientation::Horizontal, 4);
            b.append(&category_label);
            b.append(&gtk::Image::from_icon_name("pan-down-symbolic"));
            b
        })
        .margin_start(20)
        .margin_top(8)
        .build();
    category_btn.add_css_class("flat");
    category_btn.add_css_class("conv-unit-btn");

    // Value rows
    let mk_row = || -> ValueRow {
        let text = gtk::TextView::builder()
            .wrap_mode(gtk::WrapMode::None)
            .accepts_tab(false)
            .height_request(40) // fixed-height value row (E091); a page that
            // is visible from first map can otherwise allocate this 0
            .css_classes(["conv-value"])
            .build();
        let unit_label = gtk::Label::new(None);
        unit_label.add_css_class("conv-unit");
        let unit_btn = gtk::MenuButton::builder()
            .child(&{
                let b = gtk::Box::new(gtk::Orientation::Horizontal, 4);
                b.append(&unit_label);
                b.append(&gtk::Image::from_icon_name("pan-down-symbolic"));
                b
            })
            .build();
        unit_btn.add_css_class("flat");
        unit_btn.add_css_class("conv-unit-btn");
        ValueRow { text, unit_btn, unit_label }
    };
    let rows = [mk_row(), mk_row()];

    let top_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(8)
        .margin_start(20)
        .margin_end(20)
        .margin_top(36)
        .build();
    top_box.append(&rows[0].text);
    top_box.append(&rows[0].unit_btn);
    let bottom_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(8)
        .margin_start(20)
        .margin_end(20)
        .margin_top(42)
        .build();
    bottom_box.append(&rows[1].text);
    bottom_box.append(&rows[1].unit_btn);

    let pane_fill = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .vexpand(true)
        .build();

    pane_bg.append(&category_btn);
    pane_bg.append(&top_box);
    pane_bg.append(&bottom_box);
    pane_bg.append(&pane_fill);

    let keypad_slot = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();

    container.append(&pane_bg);
    container.append(&keypad_slot);

    let pane = Rc::new(ConversionPane {
        container,
        category_btn,
        category_label,
        rows,
        keypad_slot,
        state: RefCell::new(ConvState {
            // Factory default is Angle (docs/16 §14: upstream 51.1 opens on
            // Angle / Degrees (Decimal) ↔ Radians; the old Length reading
            // was the owner's persisted state).
            category: Category::Angle,
            texts: ["0".into(), "0".into()],
            units: [
                Category::Angle.default_pair().0.into(),
                Category::Angle.default_pair().1.into(),
            ],
            source: 0,
            active: 0,
        }),
        rates,
        parent: RefCell::new(None),
        recompute_pending: std::cell::Cell::new(false),
        cat_action: RefCell::new(None),
    });

    pane.wire();
    pane.rebuild_keypad();
    pane.refresh_unit_labels();
    pane.sync_buffers();
    pane.recompute();
    pane
}

impl ConversionPane {
    fn wire(self: &Rc<Self>) {
        // Category menu via a pane-owned action group.
        let actions = gio::SimpleActionGroup::new();
        let cat_action = gio::SimpleAction::new_stateful(
            "category",
            Some(glib::VariantTy::INT32),
            &(Category::Angle as i32).to_variant(),
        );
        {
            let pane = self.clone();
            cat_action.connect_activate(move |_, param| {
                if let Some(v) = param.and_then(|p| p.get::<i32>()) {
                    if let Some(cat) = Category::ALL.iter().find(|c| **c as i32 == v) {
                        pane.set_category(*cat);
                    }
                }
            });
        }
        actions.add_action(&cat_action);
        *self.cat_action.borrow_mut() = Some(cat_action);
        self.container.insert_action_group("conv", Some(&actions));

        let cat_menu = gio::Menu::new();
        for c in Category::ALL {
            let item = gio::MenuItem::new(Some(c.label()), None);
            item.set_action_and_target_value(Some("conv.category"), Some(&(c as i32).to_variant()));
            cat_menu.append_item(&item);
        }
        self.category_btn.set_menu_model(Some(&cat_menu));

        // Row carets: track the active row; map typed '-' to '−' (docs/06 §7:
        // free-form text, minus renders as a true minus).
        for (i, row) in self.rows.iter().enumerate() {
            let pane = self.clone();
            row.text.buffer().connect_insert_text(move |buf, iter, text| {
                if std::env::var_os("RADIX_TRACE").is_some() {
                    eprintln!("DBG-CONV-INSERT row={i} text={text:?}");
                }
                pane.state.borrow_mut().active = i;
                // A digit replaces a lone "0" (docs/16 §6; E091: typing 69
                // into "0" yields "69").
                let (s0, e0) = (buf.start_iter(), buf.end_iter());
                let cur = buf.text(&s0, &e0, true).to_string();
                let caret_at_end = iter.offset() == e0.offset();
                if cur == "0" && caret_at_end && iter.offset() > 0
                    && text.chars().next().is_some_and(|c| c.is_ascii_digit())
                {
                    let mut ms = buf.start_iter();
                    let mut me = buf.end_iter();
                    buf.delete(&mut ms, &mut me);
                    // The caller's iterator is invalidated by the delete;
                    // redirect it to the start so the insert lands cleanly.
                    *iter = buf.start_iter();
                }
                if text.contains('-') {
                    let mapped = text.replace('-', "−");
                    buf.stop_signal_emission_by_name("insert-text");
                    buf.insert(iter, &mapped);
                }
                // Deferred: recompute must run AFTER the signal emission
                // settles, or set_text inside recompute re-enters these
                // handlers and recurses forever.
                pane.schedule_recompute();
            });
            let pane = self.clone();
            row.text.buffer().connect_delete_range(move |_, _, _| {
                pane.state.borrow_mut().active = i;
                pane.schedule_recompute();
            });
            let pane = self.clone();
            row.text.buffer().connect_changed(move |_| {
                let b = pane.rows[i].text.buffer();
                let (s, e) = (b.start_iter(), b.end_iter());
                let t = b.text(&s, &e, true).to_string();
                pane.state.borrow_mut().texts[i] = t;
                pane.schedule_recompute();
            });
        }

        // Unit dropdown buttons get custom popovers (docs/06 §4).
        for i in 0..2 {
            // The popover must be PRE-SET: GtkMenuButton's click handling
            // lives on its internal button, and the outer "activate" action
            // signal never fires for a popover-less MenuButton (E-conv-pop,
            // 2026-11-05 rig probe: connect_activate on a bare MenuButton is
            // dead code). Popovers are built once here and refilled on map.
            self.build_unit_popover(i);
        }
    }

    /// Build one row's unit popover (search + list) and attach it to the
    /// row's MenuButton. The list refills on every map so the popover always
    /// reflects the current category and unit.
    fn build_unit_popover(self: &Rc<Self>, row_idx: usize) {
        let popover = gtk::Popover::new();
        let search = gtk::SearchEntry::builder()
            .placeholder_text("Search...")
            .build();
        let list = gtk::ListBox::builder().css_classes(["boxed-list"]).build();
        let scroll = gtk::ScrolledWindow::builder()
            .child(&list)
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .height_request(280)
            .width_request(290)
            .build();
        let vbox = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(6)
            .margin_top(8)
            .margin_bottom(8)
            .margin_start(8)
            .margin_end(8)
            .build();
        vbox.append(&search);
        vbox.append(&scroll);
        popover.set_child(Some(&vbox));

        let pane = self.clone();
        let pane_map = self.clone();
        let pane_act = self.clone();
        let popover_for_fill = popover.clone();
        // Names of the rows currently in the list, in order — row-activated
        // hands us an index, not a name.
        let shown: Rc<std::cell::RefCell<Vec<String>>> = Rc::new(std::cell::RefCell::new(Vec::new()));
        let shown_act = shown.clone();
        let list_act = list.clone();
        let fill = Rc::new(move |filter: &str| {
            while let Some(c) = list.first_child() {
                list.remove(&c);
            }
            // Names are recomputed per fill: the popover persists across
            // category switches.
            let (names, current) = {
                let st = pane.state.borrow();
                let names: Vec<String> = match st.category {
                    Category::Currency => {
                        // Name-alphabetical display order (evidence 17:15:06);
                        // the catalog itself is code-ordered.
                        let mut v: Vec<String> =
                            currency::CURRENCIES.iter().map(|c| c.name.to_string()).collect();
                        v.sort_by_key(|n| n.to_lowercase());
                        v
                    }
                    _ => st.category.units().iter().map(|u| u.name.to_string()).collect(),
                };
                (names, st.units[row_idx].clone())
            };
            shown.borrow_mut().clear();
            for n in &names {
                if !filter.is_empty() && !n.to_lowercase().contains(&filter.to_lowercase()) {
                    continue;
                }
                shown.borrow_mut().push(n.clone());
                let row_box = gtk::Box::builder()
                    .orientation(gtk::Orientation::Horizontal)
                    .spacing(8)
                    .margin_top(4)
                    .margin_bottom(4)
                    .margin_start(10)
                    .margin_end(10)
                    .build();
                // Checkmark trails the label on the current unit (evidence
                // 2026-11-05 16:21: "Kilometers ✓" / "Miles ✓"); hidden while
                // filtering (docs/06 §4).
                let check = gtk::Image::from_icon_name("object-select-symbolic");
                check.set_visible(filter.is_empty() && *n == current);
                let lbl = gtk::Label::builder().label(n).xalign(0.0).build();
                row_box.append(&lbl);
                row_box.append(&check);
                let lr = gtk::ListBoxRow::builder().child(&row_box).activatable(true).build();
                list.append(&lr);
            }
        });
        {
            let fill = fill.clone();
            search.connect_search_changed(move |s| fill(s.text().as_str()));
        }
        // Activation: clicks emit the LISTBOX's row-activated (the row's own
        // "activate" action signal only fires via keyboard widget-activation,
        // so connecting the row left mouse clicks dead — E-conv-pop).
        let pop_act = popover_for_fill.clone();
        list_act.connect_row_activated(move |_, row| {
            let idx = row.index();
            if idx < 0 {
                return;
            }
            let name = shown_act.borrow().get(idx as usize).cloned();
            if let Some(n) = name {
                pane_act.set_unit(row_idx, &n);
                pop_act.popdown();
            }
        });
        let fill2 = fill.clone();
        popover.connect_map(move |_| {
            // Search field only on long lists (docs/06 §4: observed threshold
            // between 4 and ~160 items; decision D13 pins it at > 10). The
            // category can change between opens, so re-decide here.
            let long = {
                let st = pane_map.state.borrow();
                match st.category {
                    Category::Currency => currency::CURRENCIES.len() > 10,
                    _ => st.category.units().len() > 10,
                }
            };
            search.set_visible(long);
            fill2("");
            search.set_text("");
            if long {
                search.grab_focus();
            }
        });
        self.rows[row_idx].unit_btn.set_popover(Some(&popover));
    }

    fn set_unit(self: &Rc<Self>, row_idx: usize, unit: &str) {
        {
            let mut st = self.state.borrow_mut();
            st.units[row_idx] = unit.to_string();
            // Changing a row's unit makes that row the TARGET (docs/06 §2).
            st.source = 1 - row_idx;
            // Reset rule: unit change while source unparseable resets both
            // values to 0 (docs/06 §7).
            if !self.source_parseable(&st) {
                st.texts = ["0".into(), "0".into()];
            }
        }
        self.refresh_unit_labels();
        self.sync_buffers();
        self.recompute();
    }

    /// Focus the active source field (called when the mode is entered).
    pub fn focus_source(self: &Rc<Self>) {
        let idx = self.state.borrow().source;
        let t = &self.rows[idx].text;
        let buf = t.buffer();
        let end = buf.end_iter();
        buf.place_cursor(&end);
        t.grab_focus();
    }

    pub fn set_category(self: &Rc<Self>, cat: Category) {
        {
            let mut st = self.state.borrow_mut();
            if st.category == cat {
                return;
            }
            st.category = cat;
            // Category switch: the TARGET's number is carried into both rows
            // (the source row's number is lost); units reset to defaults
            // (docs/06 §3).
            let target_idx = 1 - st.source;
            let carried = st.texts[target_idx].clone();
            st.texts = [carried.clone(), carried];
            let (du, dt) = cat.default_pair();
            st.units = [du.into(), dt.into()];
        }
        self.category_label.set_text(cat.label());
        self.rebuild_keypad();
        self.refresh_unit_labels();
        self.sync_buffers();
        self.recompute();
    }

    pub(crate) fn refresh_unit_labels(&self) {
        let st = self.state.borrow();
        for (i, row) in self.rows.iter().enumerate() {
            row.unit_label.set_text(&st.units[i]);
        }
    }

    pub(crate) fn sync_buffers(&self) {
        // Snapshot first: set_text fires insert-text, whose handler borrows
        // state mutably (docs/06 §7 live-recalculate loop).
        let texts = self.state.borrow().texts.clone();
        for (i, row) in self.rows.iter().enumerate() {
            let b = row.text.buffer();
            let (s, e) = (b.start_iter(), b.end_iter());
            let cur = b.text(&s, &e, true).to_string();
            if cur != texts[i] {
                b.set_text(&texts[i]);
            }
        }
    }

    fn source_parseable(&self, st: &ConvState) -> bool {
        let text = st.texts[st.source].replace('−', "-");
        match st.category {
            Category::Temperature | Category::Currency => text.parse::<f64>().is_ok(),
            Category::NumberBase => parse_value_text(&text, base_of(&st.units[st.source])).is_some(),
            Category::Angle => {
                if has_dms_glyphs(&text) {
                    parse_angle_degrees(&text).is_some()
                } else {
                    parse_value_text(&text, Base::Dec).is_some()
                }
            }
            _ => parse_value_text(&text, Base::Dec).is_some(),
        }
    }

    /// Schedule one coalesced recompute on the main loop (never reentrant).
    fn schedule_recompute(self: &Rc<Self>) {
        if self.recompute_pending.get() {
            return;
        }
        self.recompute_pending.set(true);
        let pane = self.clone();
        glib::idle_add_local_once(move || {
            pane.recompute_pending.set(false);
            pane.recompute();
        });
    }

    /// Recompute the target row from the source row (live, docs/06 §1).
    pub fn recompute(self: &Rc<Self>) {
        let st = self.state.borrow();
        let src = st.source;
        let tgt = 1 - src;
        let from = st.units[src].clone();
        let to = st.units[tgt].clone();
        let src_text = st.texts[src].replace('−', "-");

        let out: String = match st.category {
            Category::Temperature => match src_text.parse::<f64>() {
                Ok(v) => crate::fmt::automatic(
                    units::temp_from_kelvin(units::temp_to_kelvin(v, &from), &to),
                    9,
                    false,
                    false,
                ),
                Err(_) => "Error".into(),
            },
            Category::NumberBase => match parse_value_text(&src_text, base_of(&from)) {
                Some(v) => render_in_base(&v, base_of(&to), 9),
                None => "Error".into(),
            },
            Category::Currency => match src_text.parse::<f64>() {
                Ok(v) => {
                    let rates = self.rates.rates.borrow();
                    let converted = if let Some(r) = rates.as_ref() {
                        r.convert(&from, &to, v)
                    } else {
                        currency::fallback_rates().convert(&from, &to, v)
                    };
                    match converted {
                        Some(x) => crate::fmt::automatic(x, 9, false, false),
                        None => "Error".into(),
                    }
                }
                Err(_) => "Error".into(),
            },
            _ => {
                // Angle: a value carrying ° ′ ″ glyphs is a DMS literal —
                // an absolute angle in degrees, independent of the selected
                // unit (upstream evidence docs/16 §14: "0°'12.5" → 12.5″).
                let deg = if st.category == Category::Angle && has_dms_glyphs(&src_text) {
                    parse_angle_degrees(&src_text)
                } else {
                    parse_value_text(&src_text, Base::Dec).and_then(|v| {
                        render_in_base(&v, Base::Dec, 15)
                            .replace('−', "-")
                            .trim_end()
                            .parse::<f64>()
                            .ok()
                    })
                };
                match deg {
                    Some(f) => {
                        let ff: f64 = if st.category == Category::Angle && has_dms_glyphs(&src_text) {
                            factor_of("Degrees (Decimal)").parse().unwrap_or(1.0)
                        } else {
                            factor_of(&from).parse().unwrap_or(1.0)
                        };
                        let tf: f64 = factor_of(&to).parse().unwrap_or(1.0);
                        let out = f * ff / tf;
                        if st.category == Category::Angle && to == "Degrees (D°M'S\")" {
                            render_angle_dms(out)
                        } else {
                            crate::fmt::automatic(out, 9, false, false)
                        }
                    }
                    None => "Error".into(),
                }
            }
        };
        drop(st);

        let b = self.rows[tgt].text.buffer();
        let (s, e) = (b.start_iter(), b.end_iter());
        let cur = b.text(&s, &e, true).to_string();
        if cur != out {
            b.set_text(&out);
        }
        self.state.borrow_mut().texts[tgt] = out;
    }

    pub(crate) fn rebuild_keypad(self: &Rc<Self>) {
        let cat = self.state.borrow().category;
        let hex = cat == Category::NumberBase;
        let (grid, h) = keypads::conversion(hex, cat.slot());
        for (btn, token, _) in &h.tokens {
            let pane = self.clone();
            let tok = token.clone();
            btn.connect_clicked(move |_| pane.insert_into_active(tok.get()));
        }
        if let Some(c) = &h.clear {
            let pane = self.clone();
            c.connect_clicked(move |_| pane.clear_all());
        }
        if let Some(sw) = &h.swap {
            let pane = self.clone();
            sw.connect_clicked(move |_| pane.swap_units());
        }
        if let Some(bs) = &h.backspace {
            let pane = self.clone();
            bs.connect_clicked(move |_| pane.backspace());
        }
        if let Some(slot) = &h.context_slot {
            let pane = self.clone();
            if cat == Category::Temperature {
                slot.connect_clicked(move |_| pane.toggle_sign());
            } else if cat == Category::Currency {
                slot.set_tooltip_text(Some("Exchange Rate Details"));
                slot.connect_clicked(move |_| pane.open_rates_dialog());
            } else if cat == Category::Angle {
                slot.set_tooltip_text(Some("Degrees, minutes, seconds"));
                slot.connect_clicked(move |_| pane.insert_dms_glyph());
            }
        }
        // Digit dimming follows the TOP row's base in Number Base; visual
        // affordance only (docs/06 §8).
        if hex {
            let top_base = base_of(&self.state.borrow().units[0]);
            let max = match top_base {
                Base::Bin => 1,
                Base::Oct => 7,
                Base::Dec => 9,
                Base::Hex => 15,
            };
            let mut child = grid.first_child();
            while let Some(w) = child {
                child = w.next_sibling();
                if let Some(b) = w.downcast_ref::<gtk::Button>() {
                    if let Some(l) = b.label() {
                        if let Some(c) = l.chars().next() {
                            if let Some(v) = c.to_digit(16) {
                                if v > max {
                                    b.add_css_class("dim");
                                    b.set_sensitive(false);
                                }
                            }
                        }
                    }
                }
            }
        }
        while let Some(c) = self.keypad_slot.first_child() {
            self.keypad_slot.remove(&c);
        }
        self.keypad_slot.append(&grid);
    }

    fn active_row(&self) -> usize {
        self.state.borrow().active
    }

    pub fn insert_into_active(self: &Rc<Self>, token: &str) {
        let idx = self.active_row();
        let b = self.rows[idx].text.buffer();
        self.rows[idx].text.grab_focus();
        if !b.has_selection() {
            let end = b.end_iter();
            b.place_cursor(&end);
        }
        if let Some((mut s, mut e)) = b.selection_bounds() {
            b.delete(&mut s, &mut e);
        }
        // A digit replaces a lone "0" (docs/16 §6; typing 69 into "0"
        // yields "69", matching E091).
        let (start, end) = (b.start_iter(), b.end_iter());
        let text = b.text(&start, &end, true);
        let caret_at_end = {
            let ins = b.iter_at_mark(&b.get_insert());
            ins.offset() == end.offset()
        };
        if text.as_str() == "0" && caret_at_end && token.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            let mut s0 = start;
            let mut e0 = end;
            b.delete(&mut s0, &mut e0);
        }
        let mut at = b.iter_at_mark(&b.get_insert());
        b.insert(&mut at, token);
        // recompute happens via the insert-text signal
    }

    /// °′″ button (Angle): append the next DMS separator — °, then ′, then
    /// ″ (upstream evidence docs/16 §14: sequential insertion).
    fn insert_dms_glyph(self: &Rc<Self>) {
        let idx = self.active_row();
        let b = self.rows[idx].text.buffer();
        let (s, e) = (b.start_iter(), b.end_iter());
        let t = b.text(&s, &e, true).to_string();
        let glyph = if !t.contains('°') {
            '°'
        } else if !t.contains('′') {
            '′'
        } else if !t.contains('″') {
            '″'
        } else {
            return;
        };
        self.rows[idx].text.grab_focus();
        let end = b.end_iter();
        b.place_cursor(&end);
        let mut at = b.iter_at_mark(&b.get_insert());
        b.insert(&mut at, &glyph.to_string());
        self.state.borrow_mut().texts[idx] = {
            let (s2, e2) = (b.start_iter(), b.end_iter());
            b.text(&s2, &e2, true).to_string()
        };
        self.recompute();
    }

    pub fn clear_all(self: &Rc<Self>) {
        {
            let mut st = self.state.borrow_mut();
            st.texts = ["0".into(), "0".into()];
            st.source = 1; // caret goes to the bottom field (docs/06 §2)
            st.active = 1;
        }
        self.sync_buffers();
        self.recompute();
        self.rows[1].text.grab_focus();
    }

    /// ⇅ swap (docs/06 §2): exchange unit assignments, preserve the TOP
    /// field's number reinterpreted in its new unit, recompute the bottom.
    pub fn swap_units(self: &Rc<Self>) {
        {
            let mut st = self.state.borrow_mut();
            st.units.swap(0, 1);
            st.source = 0;
            st.active = 0;
        }
        self.refresh_unit_labels();
        self.recompute();
        self.rows[0].text.grab_focus();
    }


    pub fn backspace(self: &Rc<Self>) {
        let idx = self.active_row();
        let b = self.rows[idx].text.buffer();
        self.rows[idx].text.grab_focus();
        if let Some((mut s, mut e)) = b.selection_bounds() {
            b.delete(&mut s, &mut e);
        } else {
            let mut ins = b.iter_at_mark(&b.get_insert());
            let off = ins.offset();
            if off > 0 {
                let mut start = b.iter_at_offset(off - 1);
                b.delete(&mut start, &mut ins);
            }
        }
        self.recompute();
    }

    /// Temperature +/−: toggle the source's sign in place, caret retained
    /// (docs/06 §5).
    pub fn toggle_sign(self: &Rc<Self>) {
        let idx = self.active_row();
        let b = self.rows[idx].text.buffer();
        let (s, e) = (b.start_iter(), b.end_iter());
        let t = b.text(&s, &e, true).to_string();
        let new = if let Some(rest) = t.strip_prefix('−') {
            rest.to_string()
        } else if let Some(rest) = t.strip_prefix('-') {
            rest.to_string()
        } else {
            format!("−{t}")
        };
        b.set_text(&new);
        let end = b.end_iter();
        b.place_cursor(&end);
        self.state.borrow_mut().texts[idx] = new;
        self.recompute();
        self.rows[idx].text.grab_focus();
    }

    /// ⓘ Exchange Rate Details dialog (docs/06 §6): one card per row unit,
    /// Refresh Rates button closes the dialog.
    pub fn open_rates_dialog(self: &Rc<Self>) {
        let dialog = adw::Dialog::builder()
            .title("Exchange Rate Details")
            .content_width(330)
            .build();
        let vbox = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(12)
            .margin_top(16)
            .margin_bottom(16)
            .margin_start(16)
            .margin_end(16)
            .build();
        for i in 0..2 {
            let name = self.state.borrow().units[i].clone();
            let card = gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .spacing(4)
                .css_classes(["card"])
                .margin_top(8)
                .margin_bottom(8)
                .margin_start(8)
                .margin_end(8)
                .build();
            let heading = gtk::Label::builder()
                .label(&name)
                .xalign(0.0)
                .css_classes(["heading"])
                .build();
            let updated = {
                let rates = self.rates.rates.borrow();
                rates.as_ref().map(|r| r.updated_unix).unwrap_or(0)
            };
            let date = if updated == 0 {
                "Updated: bundled snapshot".to_string()
            } else {
                format!("Updated {}", unix_to_date(updated))
            };
            let updated_lbl = gtk::Label::builder().label(&date).xalign(0.0).build();
            let source_lbl = gtk::Label::builder()
                .label(format!("Source {}", currency::PROVIDER_LABEL))
                .xalign(0.0)
                .css_classes(["dim-label"])
                .wrap(true)
                .build();
            card.append(&heading);
            card.append(&updated_lbl);
            card.append(&source_lbl);
            vbox.append(&card);
        }
        let refresh = gtk::Button::with_label("Refresh Rates");
        refresh.add_css_class("suggested-action");
        vbox.append(&refresh);
        dialog.set_child(Some(&vbox));
        let dlg = dialog.clone();
        let pane = self.clone();
        refresh.connect_clicked(move |_| {
            let pane2 = pane.clone();
            pane.rates.refresh_async(move |_ok| {
                pane2.recompute();
            });
            dlg.close();
        });
        let parent = self.parent.borrow().clone();
        match parent {
            Some(w) => dialog.present(Some(&w)),
            None => dialog.present(Some(&self.container)),
        }
    }
}

/// Does the text carry any DMS separator glyph?
fn has_dms_glyphs(text: &str) -> bool {
    text.contains('°') || text.contains('′') || text.contains('″')
}

/// Parse a DMS literal into DEGREES (evidence-pinned grammar, docs/16 §14):
/// the glyphs are separators — the number before ° is degrees, the number
/// between ° and ′ is minutes, the number between ′ and ″ is seconds; a
/// number after ″ is invalid ("0°'″12.5" → Error upstream). Examples:
/// "12.5°" = 12.5°, "0°12.5" = 12.5′, "0°'12.5" = 12.5″, "12°30'15″" =
/// 12.504167°.
fn parse_angle_degrees(text: &str) -> Option<f64> {
    let neg = text.trim_start().starts_with('−') || text.trim_start().starts_with('-');
    let t = text.trim_start().trim_start_matches(['−', '-']);
    let mut parts = [0.0f64; 3]; // deg, min, sec
    let mut filled = [false; 3];
    let mut num = String::new();
    let mut slot = 0usize;
    let mut after_sec_marker = false;
    for ch in t.chars() {
        match ch {
            '0'..='9' | '.' => num.push(ch),
            '°' | '′' | '″' => {
                let idx = match ch {
                    '°' => 0usize,
                    '′' => 1,
                    _ => 2,
                };
                let v = if num.is_empty() {
                    0.0
                } else {
                    num.parse::<f64>().ok()?
                };
                if ch == '″' {
                    // seconds sit BEFORE the ″ marker
                    if !filled[2] && !num.is_empty() {
                        parts[2] = v;
                        filled[2] = true;
                    }
                } else if idx < 2 {
                    // the number before this marker is the unit BEFORE it:
                    // number before ° = degrees, before ′ = minutes
                    let unit = if idx == 0 { 0 } else { 1 };
                    if !filled[unit] {
                        parts[unit] = v;
                        filled[unit] = true;
                    } else if !num.is_empty() {
                        return None;
                    }
                }
                slot = idx + 1;
                if ch == '″' {
                    after_sec_marker = true;
                }
                num.clear();
            }
            c if c.is_whitespace() => {}
            _ => return None,
        }
    }
    if after_sec_marker && !num.is_empty() {
        // digits after the closing ″ are invalid
        return None;
    }
    if !num.is_empty() {
        // trailing number with no following marker: unit = slot
        // slot 0 = degrees (bare number), 1 = minutes (after °), 2 = seconds (after ′)
        if slot > 2 || filled[slot] {
            return None;
        }
        parts[slot] = num.parse::<f64>().ok()?;
    }
    let deg = parts[0] + parts[1] / 60.0 + parts[2] / 3600.0;
    Some(if neg { -deg } else { deg })
}

/// Render a degree value as D°M'S″ (integer seconds, upstream evidence:
/// 12.5° → "12°30'0″").
fn render_angle_dms(deg: f64) -> String {
    let neg = deg < 0.0;
    let total = ((deg.abs() * 3600.0) + 0.5).floor() as i64;
    let d = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    format!("{}{d}°{m}′{s}″", if neg { "−" } else { "" })
}

fn unix_to_date(secs: i64) -> String {
    let days = secs / 86400;
    let (mut y, mut d) = (1970i64, days);
    loop {
        let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
        let len = if leap { 366 } else { 365 };
        if d < len {
            break;
        }
        d -= len;
        y += 1;
    }
    // Rough month mapping is acceptable for a "Updated" display.
    const MONTH_LEN: [i64; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let mut m = 0;
    while m < 12 {
        let mut len = MONTH_LEN[m];
        if m == 1 && leap {
            len += 1;
        }
        if d < len {
            break;
        }
        d -= len;
        m += 1;
    }
    format!("{y:04}-{m:02}-{d:02}")
}

fn base_of(name: &str) -> Base {
    match name {
        "Binary" => Base::Bin,
        "Octal" => Base::Oct,
        "Hexadecimal" => Base::Hex,
        _ => Base::Dec,
    }
}

fn factor_of(name: &str) -> String {
    for cat in Category::ALL {
        for u in cat.units() {
            if u.name == name {
                return u.factor.to_string();
            }
        }
    }
    "1".into()
}
