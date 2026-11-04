//! Shared display formatting for f64-based results (financial dialogs,
//! conversion values). The engine formats `Value`s; these helpers follow the
//! same docs/07 display rules for plain-float producers.

const SUPER_DIGITS: [char; 10] = ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];

pub fn superscript(n: i32) -> String {
    let neg = n < 0;
    let digits: String = n
        .abs()
        .to_string()
        .chars()
        .map(|c| SUPER_DIGITS[c.to_digit(10).unwrap() as usize])
        .collect();
    if neg {
        format!("⁻{digits}")
    } else {
        digits
    }
}

fn group_int(int_part: &str) -> String {
    let negative = int_part.starts_with('-');
    let digits = int_part.trim_start_matches('-');
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    format!("{}{}", if negative { "-" } else { "" }, out)
}

/// Plain decimal formatting: round half away from zero at `decimals`,
/// trim trailing zeros (and a trailing ".") unless `trailing`, optional
/// thousands grouping, U+2212 minus.
pub fn plain(v: f64, decimals: u32, trailing: bool, group: bool) -> String {
    if v == 0.0 {
        return "0".into();
    }
    let neg = v < 0.0;
    let mag = v.abs();
    let factor = 10f64.powi(decimals as i32);
    let rounded = (mag * factor + 0.5).floor() / factor;
    let s = format!("{:.*}", decimals as usize, rounded);
    let s = if trailing {
        s
    } else {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    };
    let s = if group {
        match s.split_once('.') {
            Some((i, f)) => format!("{}.{}", group_int(i), f),
            None => group_int(&s),
        }
    } else {
        s
    };
    if neg && !s.starts_with('-') {
        return format!("−{s}");
    }
    s.replace('-', "−")
}

/// Automatic format for conversion targets (docs/06 §9 / docs/07 §1):
/// plain until |x| ≥ 10¹² or 0 < |x| < 10⁻⁶, then scientific with
/// superscript exponent and `decimals` mantissa digits.
pub fn automatic(v: f64, decimals: u32, trailing: bool, group: bool) -> String {
    if v == 0.0 {
        return "0".into();
    }
    let mag = v.abs();
    if (1e-6..1e12).contains(&mag) {
        return plain(v, decimals, trailing, group);
    }
    sci(v, decimals, trailing)
}

/// Scientific: mantissa (1 digit + up to `decimals` fractional) ×10ⁿ with
/// superscript exponent.
pub fn sci(v: f64, decimals: u32, trailing: bool) -> String {
    if v == 0.0 {
        return "0".into();
    }
    let neg = v < 0.0;
    let mag = v.abs();
    let mut exp = mag.log10().floor() as i32;
    let mut mant = mag / 10f64.powi(exp);
    // Correct boundary rounding (e.g. 9.99999… → 1.0×10ⁿ⁺¹).
    if mant >= 10.0 - 5e-10 {
        mant /= 10.0;
        exp += 1;
    }
    let factor = 10f64.powi(decimals as i32);
    let m = ((mant * factor + 0.5).floor()) / factor;
    let m = if m >= 10.0 {
        exp += 1;
        m / 10.0
    } else {
        m
    };
    let ms = format!("{:.*}", decimals as usize, m);
    let ms = if trailing {
        ms
    } else {
        ms.trim_end_matches('0').trim_end_matches('.').to_string()
    };
    format!(
        "{}{}×10{}",
        if neg { "−" } else { "" },
        ms,
        superscript(exp)
    )
}
