//! The f(x) function picker and x variables/constants pickers (docs/02 §10,
//! docs/05). Templates insert `name(` at the entry caret; constants insert
//! their glyph.
//!
//! Layout evidence 2026-11-05 17:11–17:12 (owner side-by-side): category
//! rows carry end chevrons, the search field sits BELOW the list, the
//! submenu header reads "‹ Category" and function rows show the template
//! form ("sin(x)"). Rows activate via the LISTBOX's row-activated signal —
//! the row's own "activate" action signal is keyboard-only, so mouse clicks
//! used to merely select (same trap as E-conv-pop, D38).

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use adw::prelude::*;

use crate::win::RadixWindow;
use radix_engine::{constant_catalog, function_catalog};

/// One padded menu row: label (+ optional end chevron / suffix widget).
fn menu_row(label: &str, chevron: bool) -> gtk::ListBoxRow {
    let row_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(14)
        .margin_end(14)
        .build();
    let lbl = gtk::Label::builder().label(label).xalign(0.0).hexpand(true).build();
    row_box.append(&lbl);
    if chevron {
        row_box.append(&gtk::Image::from_icon_name("pan-end-symbolic"));
    }
    gtk::ListBoxRow::builder().activatable(true).child(&row_box).build()
}

/// A list styled like the original pickers: flat rows on the popover
/// background (no boxed-list card).
fn menu_list() -> gtk::ListBox {
    gtk::ListBox::builder().css_classes(["fx-menu"]).build()
}

fn scrolled(list: &gtk::ListBox) -> gtk::ScrolledWindow {
    gtk::ScrolledWindow::builder()
        .child(list)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .build()
}

/// f(x) popover: category list (chevrons) → template list with a "‹ Cat"
/// header row, a search field BELOW the list filtering all functions, and
/// the custom-function builder row (docs/02 §10). The builder inserts a
/// `name(params) = ` template into the entry; pressing Enter then defines
/// the function in the engine. Defined functions appear under "Defined".
pub fn build_fx(w: &Rc<RadixWindow>) -> gtk::Popover {
    let entry = &w.entry;
    let popover = gtk::Popover::new();
    let vbox = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(6)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(8)
        .margin_end(8)
        .width_request(260)
        .height_request(360)
        .build();

    let stack = gtk::Stack::builder().vexpand(true).build();
    let cat_list = menu_list();
    let fn_list = menu_list();

    // (display text, entry template) currently rendered in fn_list;
    // row-activated hands us an index, not a payload.
    let fn_payloads: Rc<RefCell<Vec<(String, String)>>> = Rc::new(RefCell::new(Vec::new()));
    let in_results = Rc::new(std::cell::Cell::new(false));

    // Level 1: categories with end chevrons (evidence 17:11:45), then a
    // "Defined" row for the user's own functions.
    for cat in function_catalog() {
        let row = menu_row(cat.name, true);
        cat_list.append(&row);
    }
    let defined_idx = function_catalog().len() as i32;
    cat_list.append(&menu_row("Defined", true));
    {
        let stack = stack.clone();
        let fn_list = fn_list.clone();
        let payloads = fn_payloads.clone();
        let in_results = in_results.clone();
        let w = w.clone();
        cat_list.connect_row_activated(move |_, row| {
            let idx = row.index();
            if idx < 0 {
                return;
            }
            // Level 2: header row "‹ Cat" + template rows ("sin(x)").
            while let Some(c) = fn_list.first_child() {
                fn_list.remove(&c);
            }
            payloads.borrow_mut().clear();
            if idx == defined_idx {
                fn_list.append(&back_row("Defined"));
                let env = w.env.borrow();
                let mut defs: Vec<_> = env.functions.values().collect();
                defs.sort_by(|a, b| a.name.cmp(&b.name));
                if defs.is_empty() {
                    let row = menu_row("No functions defined", false);
                    row.set_sensitive(false);
                    fn_list.append(&row);
                }
                for f in defs {
                    let display = format!("{}({})", f.name, f.params.join("; "));
                    let template = format!("{}(", f.name);
                    payloads.borrow_mut().push((display.clone(), template));
                    fn_list.append(&menu_row(&display, false));
                }
            } else if let Some(cat) = function_catalog().get(idx as usize) {
                fn_list.append(&back_row(cat.name));
                for f in cat.functions {
                    let display = if f.template.ends_with('(') {
                        format!("{}x)", f.template)
                    } else {
                        f.template.to_string()
                    };
                    payloads
                        .borrow_mut()
                        .push((display.clone(), f.template.to_string()));
                    fn_list.append(&menu_row(&display, false));
                }
            } else {
                return;
            }
            in_results.set(false);
            stack.set_visible_child_name("fns");
        });
    }

    // fn_list activation: index 0 on the submenu = back; otherwise insert.
    {
        let entry2 = entry.clone();
        let popover2 = popover.clone();
        let stack = stack.clone();
        let payloads = fn_payloads.clone();
        let in_results = in_results.clone();
        fn_list.connect_row_activated(move |_, row| {
            let idx = row.index();
            if idx < 0 {
                return;
            }
            if !in_results.get() && idx == 0 {
                stack.set_visible_child_name("cats");
                return;
            }
            let base = if in_results.get() { 0 } else { 1 };
            if let Some((_, template)) = payloads.borrow().get((idx as usize) - base) {
                entry2.insert_at_caret(template, 0);
                popover2.popdown();
            }
        });
    }

    let scroll1 = scrolled(&cat_list);
    let scroll2 = scrolled(&fn_list);
    stack.add_named(&scroll1, Some("cats"));
    stack.add_named(&scroll2, Some("fns"));
    vbox.append(&stack);

    // Search field BELOW the list (evidence 17:11:45); results replace the
    // submenu with plain typed names (evidence 17:12:45).
    let search = gtk::SearchEntry::builder().placeholder_text("Function name").build();
    vbox.append(&search);
    {
        let stack = stack.clone();
        let fn_list = fn_list.clone();
        let payloads = fn_payloads.clone();
        let in_results = in_results.clone();
        let entry2 = entry.clone();
        let popover2 = popover.clone();
        search.connect_search_changed(move |s| {
            let q = s.text().to_lowercase();
            while let Some(c) = fn_list.first_child() {
                fn_list.remove(&c);
            }
            payloads.borrow_mut().clear();
            if q.is_empty() {
                in_results.set(false);
                stack.set_visible_child_name("cats");
                return;
            }
            for cat in function_catalog() {
                for f in cat.functions {
                    if f.name.to_lowercase().contains(&q) {
                        payloads
                            .borrow_mut()
                            .push((f.name.to_string(), f.template.to_string()));
                        fn_list.append(&menu_row(f.name, false));
                    }
                }
            }
            in_results.set(true);
            stack.set_visible_child_name("fns");
            let _ = (&entry2, &popover2);
        });
    }

    // Custom-function builder (docs/02 §10): a name field, an argument
    // stepper and ⊕, which inserts "name(p;q) = " into the entry. Typing the
    // body and pressing Enter defines the function (GNOME help, "Functions").
    let builder_row = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(6)
        .halign(gtk::Align::End)
        .build();
    let name_field = gtk::Entry::builder()
        .placeholder_text("name")
        .text("f")
        .width_request(72)
        .build();
    name_field.set_tooltip_text(Some("Function name"));
    builder_row.append(&name_field);

    let arg_count = gtk::Label::new(Some("1"));
    arg_count.set_margin_start(8);
    arg_count.set_margin_end(8);
    let count = Rc::new(Cell::new(1usize));
    let pager = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(0)
        .css_classes(["linked"])
        .build();
    let minus = gtk::Button::with_label("−");
    let plus = gtk::Button::with_label("+");
    {
        let (count, label) = (count.clone(), arg_count.clone());
        minus.connect_clicked(move |_| {
            let n = count.get().saturating_sub(1).max(1);
            count.set(n);
            label.set_text(&n.to_string());
        });
    }
    {
        let (count, label) = (count.clone(), arg_count.clone());
        plus.connect_clicked(move |_| {
            let n = (count.get() + 1).min(12);
            count.set(n);
            label.set_text(&n.to_string());
        });
    }
    for b in [&minus, &plus] {
        b.add_css_class("flat");
        b.set_tooltip_text(Some("Number of arguments"));
        pager.append(b);
    }
    let add = gtk::Button::from_icon_name("list-add-symbolic");
    add.add_css_class("flat");
    add.set_tooltip_text(Some("Insert function definition template"));
    {
        let (entry, popover, count, name_field) =
            (entry.clone(), popover.clone(), count.clone(), name_field.clone());
        add.connect_clicked(move |_| {
            let name = name_field.text().trim().to_lowercase();
            if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphabetic() || c == '_') {
                return;
            }
            let params: Vec<String> = (0..count.get()).map(param_name).collect();
            let template = format!("{}({}) = ", name, params.join("; "));
            entry.insert_at_caret(&template, 0);
            popover.popdown();
        });
    }
    builder_row.append(&arg_count);
    builder_row.append(&pager);
    builder_row.append(&add);
    vbox.append(&builder_row);

    popover.set_child(Some(&vbox));
    popover
}

/// Parameter names offered by the f(x) builder. Avoids names that are
/// constants (`q`, `c`, ...) so the generated definition always parses.
fn param_name(i: usize) -> String {
    const NAMES: &[&str] = &["x", "y", "z", "a", "b", "d", "u", "v", "w", "s", "t", "n"];
    NAMES
        .get(i)
        .map(|s| (*s).to_string())
        .unwrap_or_else(|| format!("p{}", i + 1))
}

/// The submenu's first row: "‹ Category" (evidence 17:12:00).
fn back_row(title: &str) -> gtk::ListBoxRow {
    let row_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(10)
        .margin_end(14)
        .build();
    row_box.append(&gtk::Image::from_icon_name("pan-start-symbolic"));
    let lbl = gtk::Label::builder().label(title).hexpand(true).build();
    row_box.append(&lbl);
    gtk::ListBoxRow::builder().activatable(true).child(&row_box).build()
}

/// x popover: last-result row ("_ = …"), rand, user variables with delete,
/// constant categories (chevrons) → glyph list, and the variable assignment
/// row (docs/05 §4; variables layout per evidence 17:13:49).
pub fn build_variables(w: &Rc<RadixWindow>) -> gtk::Popover {
    let w = w.clone(); // closures must own the window, not borrow it
    let popover = gtk::Popover::new();
    let vbox = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(6)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(8)
        .margin_end(8)
        .width_request(240)
        .height_request(360)
        .build();

    let stack = gtk::Stack::builder().vexpand(true).build();
    let main_list = menu_list();
    let const_list = menu_list();

    // "_" row: last result, re-filled on every open.
    let underscore_lbl = gtk::Label::builder().label("_").xalign(0.0).build();
    let underscore_row = gtk::ListBoxRow::builder()
        .activatable(true)
        .child(&{
            let b = gtk::Box::builder()
                .orientation(gtk::Orientation::Horizontal)
                .spacing(8)
                .margin_top(12)
                .margin_bottom(12)
                .margin_start(14)
                .margin_end(14)
                .build();
            b.append(&underscore_lbl);
            b
        })
        .build();
    main_list.append(&underscore_row);

    // rand row.
    main_list.append(&menu_row("rand", false));

    // User variables section (refilled on map): "name = value" rows with a
    // delete button (evidence 17:13:49 "x = 5 🗑").
    let var_list = gtk::ListBox::builder().css_classes(["fx-menu"]).build();
    let var_section = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(0)
        .build();
    var_section.append(&var_list);
    let var_section_map = var_section.clone();

    // Constants section header + categories with chevrons.
    main_list.append(&section_header("Constants"));
    for cat in constant_catalog() {
        main_list.append(&menu_row(cat.name, true));
    }
    {
        let stack2 = stack.clone();
        let const_list2 = const_list.clone();
        main_list.connect_row_activated(move |_, row| {
            let idx = row.index();
            if idx < 0 {
                return;
            }
            // Rows: 0 = "_", 1 = rand, 2 = Constants header (inert), 3..
            // = categories. (Variables live in their own list.)
            let cat_idx = idx - 3;
            if cat_idx < 0 {
                return;
            }
            let Some(cat) = constant_catalog().get(cat_idx as usize) else {
                return;
            };
            while let Some(c) = const_list2.first_child() {
                const_list2.remove(&c);
            }
            const_list2.append(&back_row("Variables"));
            for c in cat.constants {
                let label = match c.typed_name {
                    Some(t) => format!("{} ({})", c.name, t),
                    None => c.name.to_string(),
                };
                const_list2.append(&menu_row(&label, false));
            }
            stack2.set_visible_child_name("consts");
        });
    }

    // main_list activation for "_" and "rand".
    {
        let entry = w.entry.clone();
        let pop = popover.clone();
        let var_box = var_section.clone();
        main_list.connect_row_activated(move |_, row| {
            match row.index() {
                0 => {
                    entry.insert_at_caret("_", 0);
                    pop.popdown();
                }
                1 => {
                    entry.insert_at_caret("rand", 0);
                    pop.popdown();
                }
                _ => {
                    let _ = &var_box;
                }
            }
        });
    }

    // const_list activation: index 0 = back; otherwise insert the glyph.
    {
        let stack3 = stack.clone();
        let entry2 = w.entry.clone();
        let pop2 = popover.clone();
        const_list.connect_row_activated(move |_, row| {
            let idx = row.index();
            if idx < 0 {
                return;
            }
            if idx == 0 {
                stack3.set_visible_child_name("main");
                return;
            }
            // Flat scan of all constants matches the appended order.
            let mut glyphs: Vec<&str> = Vec::new();
            for cat in constant_catalog() {
                for c in cat.constants {
                    glyphs.push(c.glyph);
                }
            }
            if let Some(g) = glyphs.get(idx as usize - 1) {
                entry2.insert_at_caret(g, 0);
                pop2.popdown();
            }
        });
    }

    // Variables list activation + refill on map. Payloads tracked by index.
    let var_payloads: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    {
        let payloads = var_payloads.clone();
        let entry3 = w.entry.clone();
        let pop3 = popover.clone();
        var_list.connect_row_activated(move |_, row| {
            let idx = row.index();
            if idx < 0 {
                return;
            }
            if let Some(name) = payloads.borrow().get(idx as usize) {
                entry3.insert_at_caret(name, 0);
                pop3.popdown();
            }
        });
    }
    let w_map = w.clone();
    popover.connect_map(move |_| {
        let w = &w_map;
        // "_ = value" (evidence 17:13:49). Values render through the
        // engine's own formatter — to_compact is a serialization form
        // ("E:4/1"), not display text.
        let shown = {
            let env = w.env.borrow();
            env.last_result
                .as_ref()
                .map(|v| radix_engine::format_value(v, &w.fmt.borrow()))
        };
        underscore_lbl.set_text(&match shown {
            Some(s) => format!("_ = {s}"),
            None => "_".to_string(),
        });
        // User variables with delete buttons.
        while let Some(c) = var_list.first_child() {
            var_list.remove(&c);
        }
        var_payloads.borrow_mut().clear();
        let vars: Vec<(String, String)> = {
            let env = w.env.borrow();
            let fmt = w.fmt.borrow();
            let mut v: Vec<(String, String)> = env
                .variables
                .iter()
                .map(|(k, val)| (k.clone(), radix_engine::format_value(val, &fmt)))
                .collect();
            v.sort_by(|a, b| a.0.cmp(&b.0));
            v
        };
        for (name, disp) in vars {
            var_payloads.borrow_mut().push(name.clone());
            let row_box = gtk::Box::builder()
                .orientation(gtk::Orientation::Horizontal)
                .spacing(8)
                .margin_top(12)
                .margin_bottom(12)
                .margin_start(14)
                .margin_end(14)
                .build();
            row_box.append(
                &gtk::Label::builder()
                    .label(format!("{name} = {disp}"))
                    .xalign(0.0)
                    .hexpand(true)
                    .build(),
            );
            let del = gtk::Button::from_icon_name("user-trash-symbolic");
            del.add_css_class("flat");
            {
                let w2 = w.clone();
                let name2 = name.clone();
                del.connect_clicked(move |_| {
                    w2.env.borrow_mut().variables.remove(&name2);
                });
            }
            row_box.append(&del);
            var_list.append(
                &gtk::ListBoxRow::builder()
                    .activatable(true)
                    .child(&row_box)
                    .build(),
            );
        }
        var_section_map.set_visible(!var_payloads.borrow().is_empty());
    });

    let scroll1 = scrolled(&main_list);
    let scroll2 = scrolled(&const_list);
    stack.add_named(&scroll1, Some("main"));
    stack.add_named(&scroll2, Some("consts"));
    vbox.append(&stack);
    vbox.append(&var_section);

    // Variable assignment row: "Variable name" field + assign button
    // (docs/05 §4). Writes `name=` into the entry for the user to complete.
    let assign_row = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(6)
        .build();
    let name_field = gtk::Entry::builder().placeholder_text("Variable name").build();
    let assign_btn = gtk::Button::from_icon_name("go-down-symbolic");
    assign_btn.add_css_class("flat");
    {
        let entry = w.entry.clone();
        let field = name_field.clone();
        let pop = popover.clone();
        assign_btn.connect_clicked(move |_| {
            let name = field.text().trim().to_string();
            if !name.is_empty() {
                entry.insert_at_caret(&format!("{name}="), 0);
                pop.popdown();
            }
        });
    }
    assign_row.append(&name_field);
    assign_row.append(&assign_btn);
    vbox.append(&assign_row);

    popover.set_child(Some(&vbox));
    popover
}

/// Non-activatable section header row ("Constants", evidence 17:13:49).
fn section_header(title: &str) -> gtk::ListBoxRow {
    let lbl = gtk::Label::builder()
        .label(title)
        .xalign(0.0)
        .margin_top(14)
        .margin_bottom(6)
        .margin_start(14)
        .margin_end(14)
        .build();
    lbl.add_css_class("heading");
    gtk::ListBoxRow::builder().child(&lbl).activatable(false).selectable(false).build()
}
