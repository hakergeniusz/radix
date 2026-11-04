//! Entry controller: the free-form input line.
//!
//! Implements the interaction contract of docs/03 §1 (free-form text, caret
//! placement, fresh-start rule) and docs/08 §2/§5 (selection, character-level
//! undo/redo) and docs/10 §1 (inline error tags over the offending token).

use std::cell::RefCell;
use std::ops::Range;
use std::rc::Rc;

use gtk::gdk;
use gtk::prelude::*;
use gtk::TextBuffer;

/// What the entry is currently showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryState {
    /// User text being composed.
    Input,
    /// The formatted result string of the last evaluation.
    ResultShown,
    /// The erroneous text, with error tags applied.
    ErrorShown,
}

/// Owner callback: (undo-sensitive, redo-sensitive).
pub type StackChangeFn = std::rc::Rc<dyn Fn(bool, bool)>;
/// Fired on any entry text change (typing, clear, programmatic rewrite).
pub type TextChangeFn = std::rc::Rc<dyn Fn()>;

/// One character-level edit operation for undo/redo.
#[derive(Debug, Clone)]
enum EditOp {
    /// Text inserted at char offset.
    Insert { pos: usize, text: String },
    /// Text deleted at char offset.
    Delete { pos: usize, text: String },
}

pub struct EntryController {
    pub view: gtk::TextView,
    buffer: TextBuffer,
    // Shared-with-signal-handlers state MUST live behind Rc — `RefCell<T>`
    // where T: Clone would clone BY VALUE into each closure, giving every
    // handler a disconnected copy (the fresh-start bug, docs/16 §5).
    state: std::rc::Rc<RefCell<EntryState>>,
    undo: std::rc::Rc<RefCell<Vec<EditOp>>>,
    redo: std::rc::Rc<RefCell<Vec<EditOp>>>,
    /// True while the controller itself changes the buffer (no undo records,
    /// no fresh-start interpretation).
    programmatic: std::rc::Rc<std::cell::Cell<bool>>,
    /// Set while an undo/redo is being applied.
    applying: std::rc::Rc<std::cell::Cell<bool>>,
    /// Superscript input mode armed by the ↑n button: digits become
    /// superscripts until a non-digit is entered (docs/04 §5, docs/03 §2).
    pub super_mode: std::rc::Rc<RefCell<bool>>,
    /// Subscript input mode armed by the ↓n button.
    pub sub_mode: std::rc::Rc<RefCell<bool>>,
    /// Currency-code completion popover (docs/08 §6).
    completion: RefCell<Option<CompletionState>>,
    /// Currency-conversion preferences shared with the window (docs/16 §6).
    currency: crate::data::currency::SharedCurrencyPrefs,
    /// Suppress completion while the controller rewrites the buffer.
    suppress_completion: std::cell::Cell<bool>,
    /// Callbacks to notify the owner about enable/disable of undo/redo.
    pub on_stack_change: std::rc::Rc<RefCell<Option<StackChangeFn>>>,
    /// Callback fired on any buffer change; the Programming bit panel
    /// follows the entry (docs/16 D24, E089).
    pub on_text_change: std::rc::Rc<RefCell<Option<TextChangeFn>>>,
    /// Last evaluated result. The entry itself clears after `=` (E085/E087:
    /// caret-only entry); an operator typed afterwards continues from this
    /// value (docs/16 D22).
    last_result: std::rc::Rc<RefCell<Option<String>>>,
}

/// Active completion: the popover plus the token char range it would
/// replace and the highlighted code.
struct CompletionState {
    popover: gtk::Popover,
    list: gtk::ListBox,
    /// (start_char_offset, end_char_offset, selected_code)
    token: Rc<RefCell<Option<(i32, i32, String)>>>,
}

const SUPER_DIGITS: [char; 10] = ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];
const SUB_DIGITS: [char; 10] = ['₀', '₁', '₂', '₃', '₄', '₅', '₆', '₇', '₈', '₉'];

impl EntryController {
    pub fn new(currency: crate::data::currency::SharedCurrencyPrefs) -> Rc<Self> {
        let view = gtk::TextView::builder()
            .wrap_mode(gtk::WrapMode::None)
            .accepts_tab(false)
            .hexpand(true)
            .css_classes(["radix-entry"])
            .build();
        let buffer = view.buffer();
        let tags = buffer.tag_table();
        let bold = gtk::TextTag::builder().weight(700).name("result-bold").build();
        tags.add(&bold);
        let err = gtk::TextTag::builder()
            .foreground("#c30000")
            .name("error-fg")
            .build();
        tags.add(&err);
        let err_bg = gtk::TextTag::builder().background("#f6bbbe").name("error-hl").build();
        tags.add(&err_bg);

        let this = Self {
            view,
            buffer,
            state: std::rc::Rc::new(RefCell::new(EntryState::Input)),
            undo: std::rc::Rc::new(RefCell::new(Vec::new())),
            redo: std::rc::Rc::new(RefCell::new(Vec::new())),
            programmatic: std::rc::Rc::new(std::cell::Cell::new(false)),
            applying: std::rc::Rc::new(std::cell::Cell::new(false)),
            super_mode: std::rc::Rc::new(RefCell::new(false)),
            sub_mode: std::rc::Rc::new(RefCell::new(false)),
            completion: RefCell::new(None),
            currency,
            suppress_completion: std::cell::Cell::new(false),
            on_stack_change: std::rc::Rc::new(RefCell::new(None)),
            on_text_change: std::rc::Rc::new(RefCell::new(None)),
            last_result: std::rc::Rc::new(RefCell::new(None)),
        };
        this.connect_signals();
        let rc = Rc::new(this);
        // Completion re-evaluation on every buffer change (docs/08 §6).
        let rc2 = rc.clone();
        rc.buffer.connect_changed(move |_| rc2.update_completion());
        rc
    }

    // ---- currency completion (docs/08 §6, T078/T079) ----

    /// Re-evaluate whether the completion popover should be shown for the
    /// current trailing token; called after every user edit.
    pub fn update_completion(&self) {
        if self.programmatic.get()
            || self.applying.get()
            || self.suppress_completion.get()
        {
            return;
        }
        let text = self.text();
        let chars: Vec<char> = text.chars().collect();
        // Trailing token: run of ASCII letters at the end of the text.
        let mut n = 0usize;
        for c in chars.iter().rev() {
            if c.is_ascii_alphabetic() {
                n += 1;
            } else {
                break;
            }
        }
        let show = n >= 1 && chars.len() == n || n >= 1;
        let token: String = chars[chars.len() - n..].iter().collect();
        if !show || token.is_empty() {
            self.dismiss_completion();
            return;
        }
        let matches: Vec<_> = {
            let prefs = self.currency.prefs.borrow();
            let favs = self.currency.favorites.borrow();
            crate::data::currency::matching(&token)
                .into_iter()
                .filter(|c| {
                    prefs.show == crate::data::currency::CurrencyShow::All
                        || favs.iter().any(|f| f == c.code)
                })
                .collect()
        };
        if matches.is_empty() {
            self.dismiss_completion();
            return;
        }

        let mut slot = self.completion.borrow_mut();
        if slot.is_none() {
            let popover = gtk::Popover::new();
            popover.set_parent(&self.view);
            popover.set_autohide(false);
            let list = gtk::ListBox::builder().css_classes(["boxed-list"]).build();
            let scroll = gtk::ScrolledWindow::builder()
                .child(&list)
                .hscrollbar_policy(gtk::PolicyType::Never)
                .vscrollbar_policy(gtk::PolicyType::Never)
                .max_content_height(220)
                .propagate_natural_height(true)
                .build();
            popover.set_child(Some(&scroll));
            *slot = Some(CompletionState {
                popover,
                list,
                token: Rc::new(RefCell::new(None)),
            });
        }
        let comp = slot.as_ref().unwrap();
        // (Re)populate.
        while let Some(c) = comp.list.first_child() {
            comp.list.remove(&c);
        }
        let mut first_code = String::new();
        for (i, cur) in matches.iter().enumerate() {
            if i == 0 {
                first_code = cur.code.to_string();
            }
            let hbox = gtk::Box::builder()
                .orientation(gtk::Orientation::Horizontal)
                .spacing(10)
                .margin_top(4)
                .margin_bottom(4)
                .margin_start(10)
                .margin_end(10)
                .build();
            let code = gtk::Label::builder()
                .label(cur.code)
                .css_classes(["result"])
                .build();
            let detail_text = match self.currency.prefs.borrow().display {
                crate::data::currency::CurrencyDisplay::Name => {
                    format!("{} - {}", cur.name, cur.symbol)
                }
                crate::data::currency::CurrencyDisplay::Code => {
                    cur.symbol.to_string()
                }
            };
            let detail = gtk::Label::builder()
                .label(detail_text)
                .xalign(0.0)
                .hexpand(true)
                .css_classes(["dim-label"])
                .build();
            hbox.append(&code);
            hbox.append(&detail);
            let row = gtk::ListBoxRow::builder().child(&hbox).activatable(true).build();
            let code_str = cur.code.to_string();
            let view = self.view.clone();
            let pop = comp.popover.clone();
            let token_cell = comp.token.clone();
            row.connect_activate(move |_| {
                // Replace the token with the selected code (T079 Tab path).
                let b = view.buffer();
                if let Some((s, e, _)) = *token_cell.borrow() {
                    let mut si = b.iter_at_offset(s);
                    let mut ei = b.iter_at_offset(e);
                    b.delete(&mut si, &mut ei);
                    b.insert(&mut si, &code_str);
                }
                pop.popdown();
            });
            comp.list.append(&row);
        }
        // Position above the caret.
        let it = self.buffer.iter_at_mark(&self.buffer.get_insert());
        let loc = self.view.iter_location(&it);
        let (wx, wy) = self.view.buffer_to_window_coords(
            gtk::TextWindowType::Widget,
            loc.x(),
            loc.y(),
        );
        let rect = gdk::Rectangle::new(wx - 4, wy - 4, 1, 1);
        comp.popover.set_pointing_to(Some(&rect));
        *comp.token.borrow_mut() = Some((
            (chars.len() - n) as i32,
            chars.len() as i32,
            first_code,
        ));
        comp.popover.popup();
    }

    pub fn completion_visible(&self) -> bool {
        self.completion
            .borrow()
            .as_ref()
            .map(|c| c.popover.is_visible())
            .unwrap_or(false)
    }

    pub fn dismiss_completion(&self) {
        if let Some(c) = self.completion.borrow().as_ref() {
            c.popover.popdown();
        }
    }

    /// Tab acceptance: replace the trailing token with the highlighted code.
    pub fn accept_completion(&self) {
        let c = self
            .completion
            .borrow()
            .as_ref()
            .map(|c| (c.popover.clone(), c.token.clone()));
        let Some((popover, token)) = c else { return };
        popover.popdown();
        let Some((s, e, code)) = token.borrow().clone() else { return };
        self.suppress_completion.set(true);
        let mut si = self.buffer.iter_at_offset(s);
        let mut ei = self.buffer.iter_at_offset(e);
        self.buffer.delete(&mut si, &mut ei);
        self.buffer.insert(&mut si, &code);
        self.suppress_completion.set(false);
    }

    fn notify_stack(&self) {
        if let Some(cb) = self.on_stack_change.borrow().as_ref() {
            cb(!self.undo.borrow().is_empty(), !self.redo.borrow().is_empty());
        }
    }

    fn connect_signals(&self) {
        // Clones for the insert-text closure.
        let programmatic = self.programmatic.clone();
        let applying = self.applying.clone();
        let undo_stack = self.undo.clone();
        let redo_stack = self.redo.clone();
        let state = self.state.clone();
        let super_mode = self.super_mode.clone();
        let sub_mode = self.sub_mode.clone();
        let notify = self.on_stack_change.clone();
        let last_result = self.last_result.clone();
        let text_change = self.on_text_change.clone();
        let this_view = self.view.clone();

        self.buffer.connect_changed(move |_| {
            if let Some(cb) = text_change.borrow().as_ref() {
                cb();
            }
        });

        self.buffer.connect_insert_text(move |buf, iter, text| {
            if std::env::var_os("RADIX_TRACE").is_some() {
                eprintln!(
                    "DBG-INSERT-RAW text={text:?} prog={} applying={}",
                    programmatic.get(),
                    applying.get()
                );
            }
            if programmatic.get() || applying.get() {
                return;
            }
            let fresh_start = {
                let st = *state.borrow();
                let fs = st == EntryState::ResultShown
                    && text.chars().next().is_some_and(|c| c.is_ascii_digit() || c == '.');
                if std::env::var_os("RADIX_TRACE").is_some() {
                    eprintln!("DBG-INSERT text={text:?} state={st:?} fresh={fs}");
                }
                fs
            };
            if fresh_start {
                // docs/03 §1: typing a digit while a result is shown starts a
                // new expression.
                programmatic.set(true);
                let mut start = buf.start_iter();
                let mut end = buf.end_iter();
                buf.delete(&mut start, &mut end);
                *iter = buf.start_iter();
                programmatic.set(false);
                *state.borrow_mut() = EntryState::Input;
                undo_stack.borrow_mut().clear();
                redo_stack.borrow_mut().clear();
                if let Some(cb) = notify.borrow().as_ref() {
                    cb(false, false);
                }
                return;
            }
            if *state.borrow() == EntryState::ResultShown && buf.char_count() == 0 {
                // ResultShown with an empty buffer (no visible echo). A digit
                // already started fresh above; an operator continues from the
                // last result (docs/03 §1 semantics). Emission is
                // stopped and the result+operator inserted as one unit — the
                // incoming iter cannot be redirected from inside this handler.
                let cont = text
                    .chars()
                    .next()
                    .is_some_and(|c| "+-−*×/÷^%".contains(c));
                if cont {
                    if let Some(res) = last_result.borrow().clone() {
                        let joined = format!("{res}{text}");
                        buf.stop_signal_emission_by_name("insert-text");
                        programmatic.set(true);
                        let mut at = buf.start_iter();
                        buf.insert(&mut at, &joined);
                        programmatic.set(false);
                        let end = buf.end_iter();
                        buf.place_cursor(&end);
                        *state.borrow_mut() = EntryState::Input;
                        undo_stack.borrow_mut().push(EditOp::Insert {
                            pos: 0,
                            text: joined,
                        });
                        redo_stack.borrow_mut().clear();
                        if let Some(cb) = notify.borrow().as_ref() {
                            cb(true, false);
                        }
                        return;
                    }
                }
            }
            if *state.borrow() != EntryState::Input {
                // Typing after an error appends; after a result with a
                // non-digit operator it continues from the result text.
                *state.borrow_mut() = EntryState::Input;
            }
            let pos = iter.offset() as usize;
            let mapped: String = {
                let sup = *super_mode.borrow();
                let sub = *sub_mode.borrow();
                if sup || sub {
                    text.chars()
                        .map(|c| {
                            if let Some(d) = c.to_digit(10) {
                                if sup {
                                    SUPER_DIGITS[d as usize]
                                } else {
                                    SUB_DIGITS[d as usize]
                                }
                            } else {
                                c
                            }
                        })
                        .collect()
                } else {
                    text.to_string()
                }
            };
            if mapped != text {
                // Re-route: cancel this insert, insert the mapped text and
                // record it as one op.
                buf.stop_signal_emission_by_name("insert-text");
                let _ = &this_view; // (view kept alive by owner)
                buf.insert(iter, &mapped);
                undo_stack.borrow_mut().push(EditOp::Insert {
                    pos,
                    text: mapped,
                });
                redo_stack.borrow_mut().clear();
                if let Some(cb) = notify.borrow().as_ref() {
                    cb(true, false);
                }
                return;
            }
            undo_stack.borrow_mut().push(EditOp::Insert {
                pos,
                text: text.to_string(),
            });
            redo_stack.borrow_mut().clear();
            if let Some(cb) = notify.borrow().as_ref() {
                cb(true, false);
            }
        });

        // Clones for the delete-range closure.
        let programmatic = self.programmatic.clone();
        let applying = self.applying.clone();
        let undo_stack = self.undo.clone();
        let redo_stack = self.redo.clone();
        let notify = self.on_stack_change.clone();

        self.buffer.connect_delete_range(move |buf, start, end| {
            if programmatic.get() || applying.get() {
                return;
            }
            let pos = start.offset() as usize;
            let text = buf.text(start, end, true).to_string();
            undo_stack.borrow_mut().push(EditOp::Delete { pos, text });
            redo_stack.borrow_mut().clear();
            if let Some(cb) = notify.borrow().as_ref() {
                cb(true, false);
            }
        });
    }

    /// Record a programmatic full replacement of the buffer contents.
    fn set_text_programmatic(&self, text: &str) {
        self.programmatic.set(true);
        let mut start = self.buffer.start_iter();
        let mut end = self.buffer.end_iter();
        self.buffer.delete(&mut start, &mut end);
        let mut at = self.buffer.start_iter();
        self.buffer.insert(&mut at, text);
        self.programmatic.set(false);
    }

    pub fn state(&self) -> EntryState {
        *self.state.borrow()
    }

    pub fn set_state(&self, st: EntryState) {
        if std::env::var_os("RADIX_TRACE").is_some() {
            eprintln!("DBG-STATE {:?} -> {st:?}", *self.state.borrow());
        }
        *self.state.borrow_mut() = st;
    }

    pub fn text(&self) -> String {
        let (s, e) = (self.buffer.start_iter(), self.buffer.end_iter());
        self.buffer.text(&s, &e, true).to_string()
    }

    /// Show user text (Input state, plain tags).
    pub fn show_input(&self, text: &str) {
        self.clear_error_tags();
        self.set_text_programmatic(text);
        let (s, e) = (self.buffer.start_iter(), self.buffer.end_iter());
        self.buffer.remove_tag_by_name("result-bold", &s, &e);
        self.buffer.remove_tag_by_name("error-fg", &s, &e);
        self.set_state(EntryState::Input);
        self.undo.borrow_mut().clear();
        self.redo.borrow_mut().clear();
        self.notify_stack();
    }

    /// Post-evaluation display: the result goes to history AND stays in the
    /// entry as a bold echo with the caret at the end (owner screenshot
    /// 2026-11-05: the original shows "4" in the entry after 2+2 =; this
    /// supersedes the E085/E087 caret-only reading, docs/16 D22). The value
    /// is kept for operator continuation; digits still start fresh.
    pub fn finish_result(&self, text: &str) {
        *self.last_result.borrow_mut() = Some(text.to_string());
        self.dismiss_completion();
        self.show_result(text);
    }

    /// Show the formatted result (ResultShown state, bold echo).
    pub fn show_result(&self, text: &str) {
        self.clear_error_tags();
        self.set_text_programmatic(text);
        let (s, e) = (self.buffer.start_iter(), self.buffer.end_iter());
        let bold = self.buffer.tag_table().lookup("result-bold").unwrap();
        self.buffer.apply_tag(&bold, &s, &e);
        self.set_state(EntryState::ResultShown);
        self.undo.borrow_mut().clear();
        self.redo.borrow_mut().clear();
        self.notify_stack();
        let end = self.buffer.end_iter();
        self.buffer.place_cursor(&end);
        self.view.grab_focus();
    }

    /// Apply error tags: `fg` colors the whole text, `hl` highlights the
    /// offending token span (byte range into the original expression).
    pub fn show_error(&self, expr: &str, span: Option<Range<usize>>) {
        self.dismiss_completion();
        self.clear_error_tags();
        let (s, e) = (self.buffer.start_iter(), self.buffer.end_iter());
        let err = self.buffer.tag_table().lookup("error-fg").unwrap();
        self.buffer.apply_tag(&err, &s, &e);
        if let Some(sp) = span {
            if sp.start <= expr.len() && sp.end <= expr.len() {
                let start_ch = expr[..sp.start].chars().count() as i32;
                let end_ch = expr[..sp.end].chars().count() as i32;
                let si = self.buffer.iter_at_offset(start_ch);
                let ei = self.buffer.iter_at_offset(end_ch);
                let hl = self.buffer.tag_table().lookup("error-hl").unwrap();
                self.buffer.apply_tag(&hl, &si, &ei);
            }
        }
        self.set_state(EntryState::ErrorShown);
    }

    fn clear_error_tags(&self) {
        let (s, e) = (self.buffer.start_iter(), self.buffer.end_iter());
        self.buffer.remove_tag_by_name("error-fg", &s, &e);
        self.buffer.remove_tag_by_name("error-hl", &s, &e);
        self.buffer.remove_tag_by_name("result-bold", &s, &e);
    }

    /// Insert a token at the caret. `caret_back` moves the caret back N
    /// characters after inserting (for pair buttons like `( )`).
    pub fn insert_at_caret(&self, token: &str, caret_back: usize) {
        // Disarm script modes when a non-digit token is inserted.
        if !token.chars().all(|c| c.is_ascii_digit()) {
            *self.super_mode.borrow_mut() = false;
            *self.sub_mode.borrow_mut() = false;
        }
        self.view.grab_focus();
        let buf = self.buffer.clone();
        let (_, e) = (buf.start_iter(), buf.end_iter());
        if !buf.has_selection() {
            buf.place_cursor(&e);
        }
        if let Some((mut sel_s, mut sel_e)) = buf.selection_bounds() {
            buf.delete(&mut sel_s, &mut sel_e);
        }
        let mut ins = buf.iter_at_mark(&buf.get_insert());
        buf.insert(&mut ins, token);
        if caret_back > 0 {
            let back = buf.iter_at_offset(ins.offset() - caret_back as i32);
            buf.place_cursor(&back);
        }
    }

    /// Clear the entry (C button / Escape fallback).
    /// Delete the character before the caret (entry-row ⌫ button). Treated
    /// as a user edit: the buffer handlers record it for undo.
    pub fn backspace(&self) {
        let mut ins = self.buffer.iter_at_mark(&self.buffer.get_insert());
        if ins.offset() == 0 {
            return;
        }
        let mut prev = self.buffer.iter_at_offset(ins.offset() - 1);
        self.buffer.delete(&mut prev, &mut ins);
        self.view.grab_focus();
    }

    pub fn clear(&self) {
        self.show_input("");
        self.view.grab_focus();
    }

    pub fn undo(&self) {
        let op = match self.undo.borrow_mut().pop() {
            Some(o) => o,
            None => return,
        };
        self.applying.set(true);
        match &op {
            EditOp::Insert { pos, text } => {
                let mut s = self.buffer.iter_at_offset(*pos as i32);
                let mut e = self.buffer.iter_at_offset((*pos + text.chars().count()) as i32);
                self.buffer.delete(&mut s, &mut e);
            }
            EditOp::Delete { pos, text } => {
                let mut at = self.buffer.iter_at_offset(*pos as i32);
                self.buffer.insert(&mut at, text);
            }
        }
        self.applying.set(false);
        self.redo.borrow_mut().push(op);
        self.notify_stack();
        self.view.grab_focus();
    }

    pub fn redo(&self) {
        let op = match self.redo.borrow_mut().pop() {
            Some(o) => o,
            None => return,
        };
        self.applying.set(true);
        match &op {
            EditOp::Insert { pos, text } => {
                let mut at = self.buffer.iter_at_offset(*pos as i32);
                self.buffer.insert(&mut at, text);
            }
            EditOp::Delete { pos, text } => {
                let mut s = self.buffer.iter_at_offset(*pos as i32);
                let mut e = self.buffer.iter_at_offset((*pos + text.chars().count()) as i32);
                self.buffer.delete(&mut s, &mut e);
            }
        }
        self.applying.set(false);
        self.undo.borrow_mut().push(op);
        self.notify_stack();
        self.view.grab_focus();
    }

    pub fn grab_focus_view(&self) {
        self.view.grab_focus();
    }
}
