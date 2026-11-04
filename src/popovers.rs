//! The f(x) function picker and x variables/constants pickers (docs/02 §10,
//! docs/05). Templates insert `name(` at the entry caret; constants insert
//! their glyph.

use std::rc::Rc;

use adw::prelude::*;

use crate::entry::EntryController;
use radix_engine::{constant_catalog, function_catalog};

/// f(x) popover: search field, category list → template list, and the
/// custom-function builder affordance row (docs/02 §10; its flow was not
/// exercised upstream — docs/14 #38 — so the row is present but inert).
pub fn build_fx(entry: &Rc<EntryController>) -> gtk::Popover {
    let popover = gtk::Popover::new();
    let vbox = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(6)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(8)
        .margin_end(8)
        .width_request(260)
        .height_request(320)
        .build();

    let search = gtk::SearchEntry::builder().placeholder_text("Function name").build();
    vbox.append(&search);

    let stack = gtk::Stack::builder().vexpand(true).build();
    let cat_list = gtk::ListBox::builder().css_classes(["boxed-list"]).build();
    let fn_list = gtk::ListBox::builder().css_classes(["boxed-list"]).build();

    // Level 1: categories.
    for cat in function_catalog() {
        let row = gtk::ListBoxRow::builder().activatable(true).build();
        let lbl = gtk::Label::builder()
            .label(cat.name)
            .xalign(0.0)
            .margin_top(6)
            .margin_bottom(6)
            .margin_start(10)
            .margin_end(10)
            .build();
        row.set_child(Some(&lbl));
        let stack = stack.clone();
        let fn_list = fn_list.clone();
        let cat_fns: Vec<(String, String)> = cat
            .functions
            .iter()
            .map(|f| (f.name.to_string(), f.template.to_string()))
            .collect();
        let popover2 = popover.clone();
        let entry2 = entry.clone();
        row.connect_activate(move |_| {
            // Level 2: function templates of this category.
            while let Some(c) = fn_list.first_child() {
                fn_list.remove(&c);
            }
            for (name, template) in &cat_fns {
                let frow = gtk::ListBoxRow::builder().activatable(true).build();
                let flbl = gtk::Label::builder()
                    .label(name)
                    .xalign(0.0)
                    .margin_top(6)
                    .margin_bottom(6)
                    .margin_start(10)
                    .margin_end(10)
                    .build();
                frow.set_child(Some(&flbl));
                let entry3 = entry2.clone();
                let t = template.clone();
                let pop = popover2.clone();
                frow.connect_activate(move |_| {
                    entry3.insert_at_caret(&t, 0);
                    pop.popdown();
                });
                fn_list.append(&frow);
            }
            stack.set_visible_child_name("fns");
        });
        cat_list.append(&row);
    }

    // Level 2 back row.
    let back = gtk::ListBoxRow::builder().activatable(true).build();
    let back_lbl = gtk::Label::builder()
        .label("‹ Categories")
        .xalign(0.0)
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(10)
        .margin_end(10)
        .build();
    back.set_child(Some(&back_lbl));
    {
        let stack = stack.clone();
        back.connect_activate(move |_| stack.set_visible_child_name("cats"));
    }
    fn_list.prepend(&back);

    let scroll1 = scrolled(&cat_list);
    let scroll2 = scrolled(&fn_list);
    stack.add_named(&scroll1, Some("cats"));
    stack.add_named(&scroll2, Some("fns"));
    vbox.append(&stack);

    // Search filters across all categories; a match jumps to the flat list.
    {
        let stack = stack.clone();
        let fn_list = fn_list.clone();
        let entry2 = entry.clone();
        let popover2 = popover.clone();
        search.connect_search_changed(move |s| {
            let q = s.text().to_lowercase();
            while let Some(c) = fn_list.first_child() {
                fn_list.remove(&c);
            }
            if q.is_empty() {
                stack.set_visible_child_name("cats");
                return;
            }
            for cat in function_catalog() {
                for f in cat.functions {
                    if f.name.to_lowercase().contains(&q) {
                        let frow = gtk::ListBoxRow::builder().activatable(true).build();
                        let flbl = gtk::Label::builder()
                            .label(format!("{}  ·  {}", f.name, cat.name))
                            .xalign(0.0)
                            .margin_top(6)
                            .margin_bottom(6)
                            .margin_start(10)
                            .margin_end(10)
                            .build();
                        frow.set_child(Some(&flbl));
                        let entry3 = entry2.clone();
                        let t = f.template.to_string();
                        let pop = popover2.clone();
                        frow.connect_activate(move |_| {
                            entry3.insert_at_caret(&t, 0);
                            pop.popdown();
                        });
                        fn_list.append(&frow);
                    }
                }
            }
            stack.set_visible_child_name("fns");
        });
    }

    // Custom function builder affordance (docs/02 §10): "1 [−] [+] [⊕]".
    let builder_row = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(6)
        .halign(gtk::Align::End)
        .build();
    let arg_count = gtk::Label::new(Some("1"));
    let minus = gtk::Button::with_label("−");
    let plus = gtk::Button::with_label("+");
    let add = gtk::Button::with_label("⊕");
    for b in [&minus, &plus, &add] {
        b.add_css_class("flat");
        b.set_sensitive(false);
        b.set_tooltip_text(Some("Custom function builder (not available)"));
    }
    builder_row.append(&arg_count);
    builder_row.append(&minus);
    builder_row.append(&plus);
    builder_row.append(&add);
    vbox.append(&builder_row);

    popover.set_child(Some(&vbox));
    popover
}

/// x popover: last-result variable row, rand, constant categories, and the
/// variable assignment row (docs/02 §10, docs/05).
pub fn build_variables(entry: &Rc<EntryController>) -> gtk::Popover {
    let popover = gtk::Popover::new();
    let vbox = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(6)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(8)
        .margin_end(8)
        .width_request(240)
        .height_request(320)
        .build();

    let stack = gtk::Stack::builder().vexpand(true).build();
    let main_list = gtk::ListBox::builder().css_classes(["boxed-list"]).build();
    let const_list = gtk::ListBox::builder().css_classes(["boxed-list"]).build();

    // "_" row: last-result variable (live value in tooltip; the app updates
    // it via update_last_result_label on each evaluation).
    let underscore_lbl = gtk::Label::builder()
        .label("_")
        .xalign(0.0)
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(10)
        .margin_end(10)
        .hexpand(true)
        .build();
    let underscore_row = gtk::ListBoxRow::builder().activatable(true).build();
    underscore_row.set_child(Some(&underscore_lbl));
    {
        let entry = entry.clone();
        let pop = popover.clone();
        underscore_row.connect_activate(move |_| {
            entry.insert_at_caret("_", 0);
            pop.popdown();
        });
    }
    main_list.append(&underscore_row);

    // rand row.
    let rand_row = gtk::ListBoxRow::builder().activatable(true).build();
    let rand_lbl = gtk::Label::builder()
        .label("rand")
        .xalign(0.0)
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(10)
        .margin_end(10)
        .build();
    rand_row.set_child(Some(&rand_lbl));
    {
        let entry = entry.clone();
        let pop = popover.clone();
        rand_row.connect_activate(move |_| {
            entry.insert_at_caret("rand", 0);
            pop.popdown();
        });
    }
    main_list.append(&rand_row);

    // Constants categories → glyph list.
    for cat in constant_catalog() {
        let row = gtk::ListBoxRow::builder().activatable(true).build();
        let lbl = gtk::Label::builder()
            .label(cat.name)
            .xalign(0.0)
            .margin_top(6)
            .margin_bottom(6)
            .margin_start(10)
            .margin_end(10)
            .build();
        row.set_child(Some(&lbl));
        let stack2 = stack.clone();
        let const_list2 = const_list.clone();
        let items: Vec<(String, String)> = cat
            .constants
            .iter()
            .map(|c| (format!("{} ({})", c.name, c.glyph), c.glyph.to_string()))
            .collect();
        let pop = popover.clone();
        let entry2 = entry.clone();
        row.connect_activate(move |_| {
            while let Some(c) = const_list2.first_child() {
                const_list2.remove(&c);
            }
            let back = gtk::ListBoxRow::builder().activatable(true).build();
            let blbl = gtk::Label::builder()
                .label("‹ Variables")
                .xalign(0.0)
                .margin_top(6)
                .margin_bottom(6)
                .margin_start(10)
                .margin_end(10)
                .build();
            back.set_child(Some(&blbl));
            {
                let stack3 = stack2.clone();
                back.connect_activate(move |_| stack3.set_visible_child_name("main"));
            }
            const_list2.append(&back);
            for (label, glyph) in &items {
                let crow = gtk::ListBoxRow::builder().activatable(true).build();
                let clbl = gtk::Label::builder()
                    .label(label)
                    .xalign(0.0)
                    .margin_top(6)
                    .margin_bottom(6)
                    .margin_start(10)
                    .margin_end(10)
                    .build();
                crow.set_child(Some(&clbl));
                let entry3 = entry2.clone();
                let g = glyph.clone();
                let pop2 = pop.clone();
                crow.connect_activate(move |_| {
                    // Clicking a constant inserts its glyph (docs/05 §3).
                    entry3.insert_at_caret(&g, 0);
                    pop2.popdown();
                });
                const_list2.append(&crow);
            }
            stack2.set_visible_child_name("consts");
        });
        main_list.append(&row);
    }

    let scroll1 = scrolled(&main_list);
    let scroll2 = scrolled(&const_list);
    stack.add_named(&scroll1, Some("main"));
    stack.add_named(&scroll2, Some("consts"));
    vbox.append(&stack);

    // Variable assignment row: "Variable name" field + assign button
    // (docs/05 §4). Writes `name=` into the entry for the user to complete.
    let assign_row = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(6)
        .build();
    let name_field = gtk::Entry::builder().placeholder_text("Variable name").build();
    let assign_btn = gtk::Button::with_label("↓");
    assign_btn.add_css_class("flat");
    {
        let entry = entry.clone();
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

fn scrolled(list: &gtk::ListBox) -> gtk::ScrolledWindow {
    gtk::ScrolledWindow::builder()
        .child(list)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .build()
}
