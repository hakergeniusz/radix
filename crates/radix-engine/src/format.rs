//! Number formatting (doc 07).
//!
//! All rounding for display is done **exactly** (BigInt arithmetic over the
//! exact rational / dyadic representation), half away from zero, at the
//! prefs' decimals count — so the 9-decimal observed vectors match exactly.
//!
//! Boundary decisions implemented (unobserved in the investigation, fixed
//! here per the implementation brief):
//! * Automatic: plain unless |x| ≥ 10¹² or 0 < |x| < 10⁻⁶ (then scientific).
//! * Scientific: integers with |x| < 10 render plain (9 and 6 plain; 99 →
//!   `9.9×10¹`).
//! * Engineering: magnitudes in [0.001, 1000) plain; exponent multiple of 3.
//! * Fixed: never scientific; integer parts longer than 13 digits truncate
//!   to 13 digits + `…` (matches `1844674407370…`).

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{Signed, Zero};

use crate::eval::Base;
use crate::num::{d10_of, round_scaled_radix, Value};
use crate::ops;

/// Result format radio (doc 07 §1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultFormat {
    /// Plain positional with scientific switch-overs (see module docs).
    Automatic,
    /// Never scientific; wide integers truncate with `…`.
    Fixed,
    /// Always mantissa×10ⁿ except small integers.
    Scientific,
    /// Exponent multiple of 3; [0.001, 1000) plain.
    Engineering,
}

/// Formatting preferences (doc 07 §2–4).
#[derive(Debug, Clone)]
pub struct FormatPrefs {
    /// Result format (default Automatic).
    pub format: ResultFormat,
    /// Fractional digit cap (default 9; app range 0..=15).
    pub decimals: u32,
    /// Pad to exactly `decimals` fractional digits (default off).
    pub trailing_zeroes: bool,
    /// Thousands separators on integer parts (default off).
    pub group: bool,
    /// Render base (default Decimal; non-Dec delegates to base rendering).
    pub base: Base,
}

impl Default for FormatPrefs {
    fn default() -> Self {
        Self {
            format: ResultFormat::Automatic,
            decimals: 9,
            trailing_zeroes: false,
            group: false,
            base: Base::Dec,
        }
    }
}

/// Format a value per the prefs (doc 07). Complex results render as
/// `a±bi` / `i` forms. Negative sign is U+2212.
pub fn format_value(v: &Value, prefs: &FormatPrefs) -> String {
    if let Some((re, im)) = v.inner_complex() {
        return complex_str(re, im, &|x| real_decimal_str(x, prefs));
    }
    if prefs.base != Base::Dec {
        return render_in_base(v, prefs.base, prefs.decimals);
    }
    real_decimal_str(v, prefs)
}

/// Render a value in `base` with up to `decimals` fractional digits
/// (radix point `.`, uppercase digits, no prefixes, trailing zeros
/// trimmed, U+2212 sign). Used by history re-render on base change.
pub fn render_in_base(v: &Value, base: Base, decimals: u32) -> String {
    if let Some((re, im)) = v.inner_complex() {
        return complex_str(re, im, &|x| real_base_str(x, base, decimals));
    }
    real_base_str(v, base, decimals)
}

// ---------------------------------------------------------------------------
// complex rendering
// ---------------------------------------------------------------------------

fn complex_str(re: &Value, im: &Value, part: &impl Fn(&Value) -> String) -> String {
    let im_abs = ops::abs_v(im).unwrap_or_else(|_| im.clone());
    let im_is_one = im_abs == Value::exact_i64(1);
    let coef = if im_is_one {
        String::new()
    } else {
        part(&im_abs)
    };
    let sign = if im.is_negative() { '−' } else { '+' };
    if re.is_zero() {
        if im_is_one {
            return if im.is_negative() { "−i".into() } else { "i".into() };
        }
        // part(&im_abs) already carries the sign of the magnitude? No — the
        // magnitude is non-negative; re-add the sign of im.
        return format!("{}{coef}i", if im.is_negative() { "−" } else { "" });
    }
    format!("{}{sign}{coef}i", part(re))
}

// ---------------------------------------------------------------------------
// decimal rendering
// ---------------------------------------------------------------------------

fn zero_str(decimals: u32, trailing: bool) -> String {
    if trailing && decimals > 0 {
        format!("0.{}", "0".repeat(decimals as usize))
    } else {
        "0".into()
    }
}

fn group_digits(int_part: &str) -> String {
    let bytes = int_part.as_bytes();
    let mut out = String::with_capacity(bytes.len() + bytes.len() / 3);
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(*b as char);
    }
    out
}

fn apply_int_part(int_str: &str, group: bool) -> String {
    if group && int_str.len() > 3 {
        group_digits(int_str)
    } else {
        int_str.to_string()
    }
}

/// Split a rounded integer `n` (= |value|·10^decimals rounded) into
/// (sign-applied) integer and fractional digit strings.
fn split_rounded(n: &BigInt, decimals: u32, radix: u32) -> (String, String) {
    let base = BigInt::from(radix).pow(decimals);
    let (int_part, frac_part) = if n.is_zero() {
        (BigInt::from(0), BigInt::from(0))
    } else {
        (n / &base, n % &base)
    };
    let int_str = if radix == 10 {
        int_part.to_string()
    } else {
        int_part.to_str_radix(radix).to_uppercase()
    };
    if decimals == 0 {
        return (int_str, String::new());
    }
    let mut frac_str = if radix == 10 {
        frac_part.to_string()
    } else {
        frac_part.to_str_radix(radix).to_uppercase()
    };
    while frac_str.len() < decimals as usize {
        frac_str.insert(0, '0');
    }
    (int_str, frac_str)
}

fn finish_frac(frac_str: &str, trailing: bool) -> String {
    let mut f = frac_str.to_string();
    if trailing {
        return f;
    }
    while f.ends_with('0') {
        f.pop();
    }
    f
}

/// Plain positional rendering at `decimals` fractional digits.
fn plain_from(n: &BigInt, neg: bool, decimals: u32, trailing: bool, group: bool) -> String {
    let (int_str, frac_full) = split_rounded(n, decimals, 10);
    let frac = finish_frac(&frac_full, trailing);
    let sign = if n.is_zero() || !neg { "" } else { "−" };
    let ip = apply_int_part(&int_str, group);
    if frac.is_empty() {
        format!("{sign}{ip}")
    } else {
        format!("{sign}{ip}.{frac}")
    }
}

/// Scientific rendering: `d.ddd×10ⁿ` with the mantissa rounded to
/// `decimals` fractional digits.
fn sci_from(r: &BigRational, neg: bool, decimals: u32, trailing: bool) -> String {
    if r.is_zero() {
        return zero_str(decimals, trailing);
    }
    let d10 = d10_of(r);
    let n_sig = decimals + 1;
    let n = crate::num::round_scaled(r, n_sig as i32 - 1 - d10);
    if n.is_zero() {
        return zero_str(decimals, trailing);
    }
    let digits = n.to_string();
    let d10 = if digits.len() > n_sig as usize {
        // rounding carried into a new decade (e.g. 9.99 → 10.0)
        d10 + 1
    } else {
        d10
    };
    // mantissa = first digit + exactly `decimals` following digits
    // (a carry produces one extra digit beyond the required precision)
    let mut frac: String = digits.chars().skip(1).take(decimals as usize).collect();
    if trailing {
        while frac.len() < decimals as usize {
            frac.push('0');
        }
    } else {
        while frac.ends_with('0') {
            frac.pop();
        }
    }
    let mantissa = if frac.is_empty() {
        digits[..1].to_string()
    } else {
        format!("{}.{}", &digits[..1], frac)
    };
    format!(
        "{}{}×10{}",
        if neg { "−" } else { "" },
        mantissa,
        sup_exp(d10)
    )
}

/// Engineering rendering: exponent multiple of 3, mantissa in [1, 1000).
fn eng_from(r: &BigRational, neg: bool, decimals: u32, trailing: bool) -> String {
    if r.is_zero() {
        return zero_str(decimals, trailing);
    }
    let mut d10 = d10_of(r);
    // magnitudes in [0.001, 1000) stay plain
    if (-3..=2).contains(&d10) {
        let n = crate::num::round_scaled(r, decimals as i32);
        return plain_from(&n, neg, decimals, trailing, false);
    }
    let mut e3 = 3 * d10.div_euclid(3);
    let mut int_digits = d10 - e3 + 1;
    let mut n = crate::num::round_scaled(
        r,
        (int_digits + decimals as i32) - 1 - d10,
    );
    let mut digits = n.to_string();
    let mut want = (int_digits + decimals as i32) as usize;
    // carry across an engineering decade (e.g. 999.9997 → 1000.00 → 1.00×10³)
    let mut guard = 0;
    while digits.len() > want && guard < 4 {
        d10 += 1;
        e3 = 3 * d10.div_euclid(3);
        int_digits = d10 - e3 + 1;
        n = crate::num::round_scaled(r, (int_digits + decimals as i32) - 1 - d10);
        digits = n.to_string();
        want = (int_digits + decimals as i32) as usize;
        guard += 1;
    }
    if n.is_zero() {
        return zero_str(decimals, trailing);
    }
    let mut frac: String = digits
        .chars()
        .skip(int_digits as usize)
        .take(decimals as usize)
        .collect();
    if trailing {
        while frac.len() < decimals as usize {
            frac.push('0');
        }
    } else {
        while frac.ends_with('0') {
            frac.pop();
        }
    }
    let int_part = apply_int_part(&digits[..int_digits as usize], false);
    let mantissa = if frac.is_empty() {
        int_part
    } else {
        format!("{int_part}.{frac}")
    };
    format!(
        "{}{}×10{}",
        if neg { "−" } else { "" },
        mantissa,
        sup_exp(e3)
    )
}

/// Superscript exponent string (digits + superscript minus, no ×10).
fn sup_exp(e: i32) -> String {
    let mut out = String::new();
    for c in e.to_string().chars() {
        out.push(match c {
            '-' => '⁻',
            '0' => '⁰',
            '1' => '¹',
            '2' => '²',
            '3' => '³',
            '4' => '⁴',
            '5' => '⁵',
            '6' => '⁶',
            '7' => '⁷',
            '8' => '⁸',
            '9' => '⁹',
            _ => c,
        });
    }
    out
}

fn real_decimal_str(v: &Value, prefs: &FormatPrefs) -> String {
    let Some(r) = v.to_ratio() else {
        return "0".into();
    };
    let neg = r.is_negative();
    let ar = r.abs();
    let d = prefs.decimals;
    let tz = prefs.trailing_zeroes;
    match prefs.format {
        ResultFormat::Automatic => {
            // ≥ 1e12 or 0 < |x| < 1e-6 → scientific (boundary decisions)
            let big = cmp_vs_pow10(&ar, 12) != std::cmp::Ordering::Less;
            let small = !ar.is_zero() && cmp_vs_pow10(&ar, -6) == std::cmp::Ordering::Less;
            if big || small {
                sci_from(&ar, neg, d, tz)
            } else {
                // Plain decimals cap — NO display-correction swallow:
                // upstream 51.1 (live Xvfb comparison, docs/16 §14) renders
                // 0.833333333×111 as 92.499999963 at its default 9 decimals
                // and as 92.5 only when decimals = 2. D23's swallow
                // reconciled an unfalsifiable E087 reading; dropped (D45).
                let n = crate::num::round_scaled(&ar, d as i32);
                plain_from(&n, neg, d, tz, prefs.group)
            }
        }
        ResultFormat::Scientific => {
            // small integers exempt (9 and 6 plain; 99 → 9.9×10¹)
            let ten = BigRational::from_integer(BigInt::from(10));
            if ar.is_integer() && ar < ten {
                let n = crate::num::round_scaled(&ar, d as i32);
                plain_from(&n, neg, d, tz, prefs.group)
            } else {
                sci_from(&ar, neg, d, tz)
            }
        }
        ResultFormat::Engineering => eng_from(&ar, neg, d, tz),
        ResultFormat::Fixed => {
            let n = crate::num::round_scaled(&ar, d as i32);
            let (int_str, _) = split_rounded(&n, d, 10);
            if int_str.len() > 13 {
                // truncate to 13 digits + ellipsis (never scientific)
                let mut head: String = int_str.chars().take(13).collect();
                if prefs.group {
                    head = group_digits(&head);
                }
                format!("{}{head}…", if neg && !n.is_zero() { "−" } else { "" })
            } else {
                plain_from(&n, neg, d, tz, prefs.group)
            }
        }
    }
}

/// Compare |r| with 10^p exactly (p may be negative).
fn cmp_vs_pow10(r: &BigRational, p: i32) -> std::cmp::Ordering {
    crate::num::cmp_pow10(r, p)
}

// ---------------------------------------------------------------------------
// base rendering
// ---------------------------------------------------------------------------

fn real_base_str(v: &Value, base: Base, decimals: u32) -> String {
    let radix = match base {
        Base::Bin => 2,
        Base::Oct => 8,
        Base::Dec => 10,
        Base::Hex => 16,
    };
    let Some(r) = v.to_ratio() else {
        return "0".into();
    };
    let neg = r.is_negative();
    let ar = r.abs();
    let n = round_scaled_radix(&ar, decimals as i32, radix);
    if n.is_zero() {
        return "0".into();
    }
    let (int_str, frac_full) = split_rounded(&n, decimals, radix);
    let frac = finish_frac(&frac_full, false);
    let sign = if neg { "−" } else { "" };
    if frac.is_empty() {
        format!("{sign}{int_str}")
    } else {
        format!("{sign}{int_str}.{frac}")
    }
}
