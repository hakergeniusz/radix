//! Persistence: history, mode, conversion state, window size and color scheme
//! restored on graceful close (docs/09 §4), stored as a JSON state file under
//! XDG_STATE_HOME.
//!
//! Contract (docs/09 §4): graceful close restores mode, category + both unit
//! assignments, per-mode window width, color-scheme choice and complete
//! history; the pending entry text is NOT restored; a hard kill loses
//! history since the last orderly save.

use std::path::PathBuf;
use std::rc::Rc;

use serde_json::json;

use crate::convert::ConversionPane;
use crate::data::units::Category;
use crate::win::{Mode, RadixWindow};

pub fn state_path() -> Option<PathBuf> {
    let base = std::env::var("XDG_STATE_HOME")
        .ok()
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var("HOME").ok().map(|h| {
                PathBuf::from(h).join(".local").join("state")
            })
        })?;
    Some(base.join("io.github.hakergeniusz.Radix").join("state.json"))
}

pub fn save(w: &std::rc::Rc<RadixWindow>) {
    let Some(path) = state_path() else { return };
    let mode = *w.mode.borrow();
    let history: Vec<(String, String)> = w
        .history
        .borrow()
        .iter()
        .map(|r| {
            let payload = match &r.value {
                crate::win::RowValue::Engine(v) => v.to_compact(),
                crate::win::RowValue::Conv(f) => format!("conv:{f:?}"),
            };
            (r.echo.clone(), payload)
        })
        .collect();
    let conv = w.conversion.state_snapshot();
    let scheme = color_scheme_name();
    let (fmt, env) = (w.fmt.borrow().clone(), w.env.borrow().clone());
    let (cprefs, favorites) = {
        let p = w.currency.prefs.borrow();
        (*p, w.currency.favorites.borrow().clone())
    };
    let doc = json!({
        "mode": mode.key(),
        "history": history,
        "conversion": {
            "category": conv.0,
            "units": conv.1,
        },
        "color_scheme": scheme,
        "prefs": {
            "decimals": fmt.decimals,
            "trailing_zeroes": fmt.trailing_zeroes,
            "group_digits": fmt.group,
            "format": format_key(fmt.format),
            "angle_unit": angle_key(env.angle_unit),
            "word_size": env.word_size.bits(),
            "base": base_key(fmt.base),
        },
        "currency_prefs": {
            "interval": cprefs.interval.label(),
            "display": match cprefs.display {
                crate::data::currency::CurrencyDisplay::Name => "name",
                crate::data::currency::CurrencyDisplay::Code => "code",
            },
            "show": match cprefs.show {
                crate::data::currency::CurrencyShow::All => "all",
                crate::data::currency::CurrencyShow::Favorites => "favorites",
            },
            "favorites": favorites,
        },
    });
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Ok(s) = serde_json::to_string(&doc) {
        // Write via temp + rename for atomicity.
        let tmp = path.with_extension("json.tmp");
        if std::fs::write(&tmp, s).is_ok() {
            let _ = std::fs::rename(&tmp, &path);
        }
    }
}

pub fn restore(w: &std::rc::Rc<RadixWindow>) {
    let Some(path) = state_path() else { return };
    let Ok(text) = std::fs::read_to_string(path) else { return };
    let Ok(doc) = serde_json::from_str::<serde_json::Value>(&text) else { return };

    // History (echo + serialized value) — rows re-render live via the values.
    if let Some(rows) = doc.get("history").and_then(|v| v.as_array()) {
        for row in rows {
            let Some(echo) = row.get(0).and_then(|v| v.as_str()) else { continue };
            let Some(compact) = row.get(1).and_then(|v| v.as_str()) else { continue };
            let value = if let Some(f) = compact.strip_prefix("conv:") {
                f.replace('−', "-").parse::<f64>().ok().map(crate::win::RowValue::Conv)
            } else {
                radix_engine::Value::from_compact(compact).map(crate::win::RowValue::Engine)
            };
            let Some(value) = value else { continue };
            w.history.borrow_mut().push(crate::win::HistoryRow {
                echo: echo.to_string(),
                value,
            });
        }
        w.rerender_history();
    }

    // Conversion category + units.
    if let Some(conv) = doc.get("conversion") {
        let cat = conv.get("category").and_then(|v| v.as_i64()).unwrap_or(7) as i32;
        let units: Vec<String> = conv
            .get("units")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
            .unwrap_or_default();
        w.conversion.restore_state(cat, &units);
    }

    // Color scheme (docs/09 §4: scheme choice persists).
    if let Some(scheme) = doc.get("color_scheme").and_then(|v| v.as_str()) {
        apply_color_scheme(scheme);
    }

    // Preferences (docs/09 §3).
    if let Some(p) = doc.get("prefs") {
        if let Some(d) = p.get("decimals").and_then(|v| v.as_u64()) {
            w.fmt.borrow_mut().decimals = d.min(15) as u32;
        }
        if let Some(t) = p.get("trailing_zeroes").and_then(|v| v.as_bool()) {
            w.fmt.borrow_mut().trailing_zeroes = t;
        }
        if let Some(g) = p.get("group_digits").and_then(|v| v.as_bool()) {
            w.fmt.borrow_mut().group = g;
        }
        if let Some(f) = p.get("format").and_then(|v| v.as_str()) {
            w.fmt.borrow_mut().format = match f {
                "fixed" => radix_engine::ResultFormat::Fixed,
                "scientific" => radix_engine::ResultFormat::Scientific,
                "engineering" => radix_engine::ResultFormat::Engineering,
                _ => radix_engine::ResultFormat::Automatic,
            };
        }
        if let Some(a) = p.get("angle_unit").and_then(|v| v.as_str()) {
            w.env.borrow_mut().angle_unit = match a {
                "radians" => radix_engine::AngleUnit::Radians,
                "gradians" => radix_engine::AngleUnit::Gradians,
                _ => radix_engine::AngleUnit::Degrees,
            };
        }
        if let Some(ws) = p.get("word_size").and_then(|v| v.as_u64()) {
            w.env.borrow_mut().word_size = match ws {
                8 => radix_engine::WordSize::W8,
                16 => radix_engine::WordSize::W16,
                32 => radix_engine::WordSize::W32,
                _ => radix_engine::WordSize::W64,
            };
            w.word_btn.set_label(match ws {
                8 => "8-bit",
                16 => "16-bit",
                32 => "32-bit",
                _ => "64-bit",
            });
        }
        if let Some(c) = doc.get("currency_prefs") {
            use crate::data::currency::{CurrencyDisplay, CurrencyShow, RefreshInterval};
            if let Some(i) = c.get("interval").and_then(|v| v.as_str()) {
                w.currency.prefs.borrow_mut().interval = RefreshInterval::from_label(i);
            }
            if let Some(d) = c.get("display").and_then(|v| v.as_str()) {
                w.currency.prefs.borrow_mut().display = match d {
                    "code" => CurrencyDisplay::Code,
                    _ => CurrencyDisplay::Name,
                };
            }
            if let Some(sh) = c.get("show").and_then(|v| v.as_str()) {
                w.currency.prefs.borrow_mut().show = match sh {
                    "favorites" => CurrencyShow::Favorites,
                    _ => CurrencyShow::All,
                };
            }
            if let Some(f) = c.get("favorites").and_then(|v| v.as_array()) {
                *w.currency.favorites.borrow_mut() = f
                    .iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect();
            }
        }
        if let Some(b) = p.get("base").and_then(|v| v.as_str()) {
            let base = match b {
                "bin" => radix_engine::Base::Bin,
                "oct" => radix_engine::Base::Oct,
                "hex" => radix_engine::Base::Hex,
                _ => radix_engine::Base::Dec,
            };
            w.env.borrow_mut().base = base;
            w.fmt.borrow_mut().base = base;
            w.base_btn.set_label(match b {
                "bin" => "Binary",
                "oct" => "Octal",
                "hex" => "Hexadecimal",
                _ => "Decimal",
            });
        }
        w.rerender_history();
    }

    // Mode last (drives window width).
    if let Some(mode_key) = doc.get("mode").and_then(|v| v.as_str()) {
        if let Some(m) = Mode::from_key(mode_key) {
            w.set_mode(m);
        }
    }
}

pub fn color_scheme_name() -> String {
    let display = gtk::gdk::Display::default();
    let Some(d) = display else { return "default".into() };
    let sm = adw::StyleManager::for_display(&d);
    match sm.color_scheme() {
        adw::ColorScheme::PreferLight => "light".into(),
        adw::ColorScheme::PreferDark => "dark".into(),
        _ => "default".into(),
    }
}

pub fn apply_color_scheme(name: &str) {
    let display = gtk::gdk::Display::default();
    let Some(d) = display else { return };
    let sm = adw::StyleManager::for_display(&d);
    sm.set_color_scheme(match name {
        "light" => adw::ColorScheme::PreferLight,
        "dark" => adw::ColorScheme::PreferDark,
        _ => adw::ColorScheme::Default,
    });
}

fn format_key(f: radix_engine::ResultFormat) -> &'static str {
    match f {
        radix_engine::ResultFormat::Automatic => "automatic",
        radix_engine::ResultFormat::Fixed => "fixed",
        radix_engine::ResultFormat::Scientific => "scientific",
        radix_engine::ResultFormat::Engineering => "engineering",
    }
}

fn angle_key(a: radix_engine::AngleUnit) -> &'static str {
    match a {
        radix_engine::AngleUnit::Radians => "radians",
        radix_engine::AngleUnit::Gradians => "gradians",
        radix_engine::AngleUnit::Degrees => "degrees",
    }
}

fn base_key(b: radix_engine::Base) -> &'static str {
    match b {
        radix_engine::Base::Bin => "bin",
        radix_engine::Base::Oct => "oct",
        radix_engine::Base::Hex => "hex",
        radix_engine::Base::Dec => "dec",
    }
}

impl ConversionPane {
    /// Snapshot (category, units) for persistence.
    pub fn state_snapshot(&self) -> (i32, [String; 2]) {
        let st = self.state.borrow();
        (st.category as i32, st.units.clone())
    }

    /// Restore category + unit assignments (values reset per category
    /// switch semantics; docs/09 §4 restores units, not values).
    pub fn restore_state(self: &Rc<Self>, category: i32, units: &[String]) {
        let Some(cat) = Category::ALL.iter().find(|c| **c as i32 == category) else {
            return;
        };
        {
            let mut st = self.state.borrow_mut();
            st.category = *cat;
            st.texts = ["0".into(), "0".into()];
            st.source = 0;
            st.active = 0;
            if units.len() == 2 {
                st.units = [units[0].clone(), units[1].clone()];
            } else {
                st.units = [cat.default_unit().into(), cat.default_unit().into()];
            }
        }
        self.category_label.set_text(cat.label());
        self.rebuild_keypad();
        self.refresh_unit_labels();
        self.sync_buffers();
        self.recompute();    }
}
