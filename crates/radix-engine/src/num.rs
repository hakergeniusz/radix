//! Numeric core: the opaque [`Value`] type and its arithmetic.
//!
//! Three states (doc 04 §1):
//!
//! * **Exact** — an arbitrary-precision rational (`num-rational` over
//!   `num-bigint`). `+ − × ÷(non-zero) %`, integer-exponent `^` and integer
//!   `!` stay exact (`1÷3×3 = 1` exactly, T022).
//! * **Approx** — a high-precision binary float (`astro-float`
//!   `BigFloatNumber`, ~200 bits ≈ 60 decimal digits). Reached when a
//!   transcendental/irrational step occurs.
//! * **Complex** — a pair of [`Value`]s (re, im); `sqrt(-1) = i`. Complex
//!   parts may themselves be exact (`sqrt(-1)` has an exactly-1 imaginary
//!   part).
//!
//! Final display rounding is *not* done here — the formatter (doc 07) rounds
//! exactly (half-away-from-zero) on the exact rational / dyadic
//! representation, so 9-decimal vectors match exactly.

use std::cmp::Ordering;
use std::sync::{Mutex, OnceLock};

use astro_float::{BigFloatNumber as Af, Consts, Radix, RoundingMode, Sign as AfSign};
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{Signed, Zero};

use crate::error::{ErrorKind, EvalError, EvalResult};

/// Internal working precision in bits (~60 significant decimal digits).
/// The brief mandates ~200 bits; all observed display vectors use at most
/// 15 decimals, so this leaves an error margin of ~45 decimal digits.
pub(crate) const WORK_P: usize = 200;
/// Extra guard bits used for transcendental evaluation and constants;
/// results are rounded back down to [`WORK_P`].
pub(crate) const GUARD_P: usize = 32;
/// Rounding mode for every internal high-precision step. Display rounding
/// is exact (BigInt) and half-away-from-zero, performed by the formatter.
pub(crate) const RM: RoundingMode = RoundingMode::ToEven;
/// Maximum magnitude for an exact integer exponent of `^` / exact `!`.
pub(crate) const MAX_INT_OP: i64 = 1_000_000;
/// Maximum magnitude for an exact integer factorial (beyond it the gamma
/// extension would be used, but such factorials are out of display range).
pub(crate) const MAX_FACTORIAL: u32 = 50_000;

static CONSTS: OnceLock<Mutex<Consts>> = OnceLock::new();

pub(crate) fn consts() -> std::sync::MutexGuard<'static, Consts> {
    CONSTS
        .get_or_init(|| {
            Mutex::new(
                Consts::new().expect("constant cache initialization cannot fail in practice"),
            )
        })
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

/// Map an astro-float error to an engine error (no panics on user input).
pub(crate) fn af_err(e: astro_float::Error) -> EvalError {
    let _ = e;
    EvalError::new(ErrorKind::OutOfDomain)
}

/// π at the given precision.
pub(crate) fn pi(p: usize) -> EvalResult<Af> {
    consts().pi(p, RM).map_err(af_err)
}

/// e (Euler's number) at the given precision.
pub(crate) fn e(p: usize) -> EvalResult<Af> {
    consts().e(p, RM).map_err(af_err)
}

// ---------------------------------------------------------------------------
// Value
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub(crate) enum Inner {
    Exact(BigRational),
    Approx(Af),
    /// re / im pair. Invariant: `im` is never zero (complex values with a
    /// zero imaginary part are demoted to reals on construction).
    Complex(Box<(Value, Value)>),
}

impl Clone for Inner {
    fn clone(&self) -> Self {
        match self {
            Inner::Exact(r) => Inner::Exact(r.clone()),
            // astro-float's clone is fallible only on allocation failure,
            // which aborts the process like any other OOM.
            Inner::Approx(f) => Inner::Approx(
                f.clone()
                    .expect("float clone allocation failure is fatal"),
            ),
            Inner::Complex(p) => Inner::Complex(Box::new((p.0.clone(), p.1.clone()))),
        }
    }
}

/// The result of an evaluation. Opaque; inspect via the public helpers
/// (`is_exact`, `real_and_imaginary`, `to_compact`, the formatter, ...).
#[derive(Debug, Clone, PartialEq)]
pub struct Value {
    pub(crate) inner: Inner,
}

impl Value {
    // -- constructors -------------------------------------------------------

    pub(crate) fn exact(r: BigRational) -> Self {
        Value {
            inner: Inner::Exact(r),
        }
    }

    pub(crate) fn exact_int(i: BigInt) -> Self {
        Value::exact(BigRational::from_integer(i))
    }

    pub(crate) fn exact_i64(i: i64) -> Self {
        Value::exact_int(BigInt::from(i))
    }

    /// Exact integer value (public helper for embedders/tests).
    pub fn from_i64(i: i64) -> Self {
        Value::exact_i64(i)
    }

    pub(crate) fn approx(f: Af) -> Self {
        Value {
            inner: Inner::Approx(f),
        }
    }

    /// Exact rational from a decimal literal such as `"6.62607015e-34"`.
    /// (Used for the physical constants; keeps them exact.)
    pub(crate) fn exact_from_decimal(s: &str) -> Self {
        let (mantissa, exp) = match s.split_once(['e', 'E']) {
            Some((m, e)) => (m, e.parse::<i32>().unwrap_or(0)),
            None => (s, 0),
        };
        let neg = mantissa.starts_with('-');
        let m = mantissa.trim_start_matches(['-', '+']);
        let (int_part, frac_part) = m.split_once('.').unwrap_or((m, ""));
        let digits = format!("{int_part}{frac_part}");
        let mut num = BigInt::parse_bytes(digits.as_bytes(), 10).unwrap_or_else(BigInt::zero);
        if neg {
            num = -num;
        }
        // value = num × 10^(exp − number of fractional digits)
        let shift = (exp - frac_part.len() as i32).clamp(-10_000, 10_000);
        let r = if shift >= 0 {
            BigRational::from_integer(num * BigInt::from(10).pow(shift as u32))
        } else {
            BigRational::new(num, BigInt::from(10).pow((-shift) as u32))
        };
        Value::exact(r)
    }

    /// High-precision float from an integer.
    pub(crate) fn af_from_i64(i: i64, p: usize) -> EvalResult<Af> {
        Af::from_i64(i, p).map_err(af_err)
    }

    /// High-precision float from an arbitrary-precision integer.
    pub(crate) fn af_from_bigint(i: &BigInt, p: usize) -> EvalResult<Af> {
        if i.is_zero() {
            return Af::new(p).map_err(af_err);
        }
        // convert_from_radix convention: value = digits·radix^(e − len).
        // With e = len and decimal digits this is exactly the integer.
        let neg = i.is_negative();
        let mag = i.abs().to_str_radix(10);
        let digits: Vec<u8> = mag.bytes().map(|b| b - b'0').collect();
        let s = if neg { AfSign::Neg } else { AfSign::Pos };
        #[allow(clippy::cast_possible_truncation)]
        let e = digits.len() as astro_float::Exponent;
        Af::convert_from_radix(s, &digits, e, Radix::Dec, p, RM).map_err(af_err)
    }

    /// High-precision float from an exact rational (numerator over
    /// denominator, correctly rounded).
    pub(crate) fn af_from_rational(r: &BigRational, p: usize) -> EvalResult<Af> {
        let num = Value::af_from_bigint(r.numer(), p)?;
        let den = Value::af_from_bigint(r.denom(), p)?;
        num.div(&den, p, RM).map_err(af_err)
    }

    /// Complex value; demotes to a real when the imaginary part is zero.
    pub(crate) fn complex(re: Value, im: Value) -> Self {
        if im.is_zero() {
            re
        } else {
            Value {
                inner: Inner::Complex(Box::new((re, im))),
            }
        }
    }

    /// Fresh pseudo-random value in [0, 1) (doc 04 §9), kept as an *exact*
    /// rational with a 64-bit denominator (≥ 19 significant digits, which
    /// satisfies the ≥ 9 significant digit requirement).
    pub fn random_unit() -> Self {
        use rand::Rng;
        let mut rng = rand::rng();
        // Ensure ≥ 9 significant decimal digits (rejects with p ≈ 2^-30).
        let word = loop {
            let w: u64 = rng.random();
            if w >= 1u64 << 30 {
                break w;
            }
        };
        Value::exact(BigRational::new(
            BigInt::from(word),
            BigInt::from(2).pow(64u32),
        ))
    }

    // -- predicates ---------------------------------------------------------

    /// True when the value is an exact rational (no float rounding).
    pub fn is_exact(&self) -> bool {
        matches!(self.inner, Inner::Exact(_))
    }

    pub(crate) fn is_zero(&self) -> bool {
        match &self.inner {
            Inner::Exact(r) => r.is_zero(),
            Inner::Approx(f) => f.is_zero(),
            Inner::Complex(_) => false, // im ≠ 0 by invariant
        }
    }

    pub(crate) fn is_negative(&self) -> bool {
        match &self.inner {
            Inner::Exact(r) => r.is_negative(),
            Inner::Approx(f) => f.is_negative(),
            Inner::Complex(_) => false,
        }
    }

    /// The exact rational behind a *real* value. Approx floats convert to
    /// their exact dyadic rational (m·2^e), so this never fails for reals.
    pub(crate) fn to_ratio(&self) -> Option<BigRational> {
        match &self.inner {
            Inner::Exact(r) => Some(r.clone()),
            Inner::Approx(f) => Some(bf_to_rational(f)),
            Inner::Complex(_) => None,
        }
    }

    /// Borrow the (re, im) parts when this is a complex value.
    pub(crate) fn inner_complex(&self) -> Option<(&Value, &Value)> {
        match &self.inner {
            Inner::Complex(p) => Some((&p.0, &p.1)),
            _ => None,
        }
    }

    /// Real/imaginary decomposition. Real values report an exact-0
    /// imaginary part. `sqrt(-1)` yields im exactly 1.
    pub fn real_and_imaginary(&self) -> (Value, Value) {
        match &self.inner {
            Inner::Complex(p) => (p.0.clone(), p.1.clone()),
            other => {
                let re = Value {
                    inner: other.clone(),
                };
                (re, Value::exact_i64(0))
            }
        }
    }

    /// The value as an integer when it is exactly integral.
    pub(crate) fn as_integral(&self) -> Option<BigInt> {
        match &self.inner {
            Inner::Exact(r) => {
                if r.is_integer() {
                    Some(r.to_integer())
                } else {
                    None
                }
            }
            Inner::Approx(f) => {
                let (m, e) = bf_parts(f);
                if m.is_zero() {
                    return Some(BigInt::from(0));
                }
                if e >= 0 {
                    Some(m << e)
                } else {
                    let k = BigInt::from(2).pow((-e) as u32);
                    let q = &m / &k;
                    let rem = &m - &q * &k;
                    if rem.is_zero() {
                        Some(q)
                    } else {
                        None
                    }
                }
            }
            Inner::Complex(_) => None,
        }
    }

    // -- serialization ------------------------------------------------------

    /// Lossless, persistable serialization.
    pub fn to_compact(&self) -> String {
        match &self.inner {
            Inner::Exact(r) => format!("E:{}/{}", r.numer(), r.denom()),
            Inner::Approx(f) => {
                if f.is_zero() {
                    return "A:0".into();
                }
                let (words, n, sign, e) = f.to_raw_parts();
                let mut m = BigInt::from(0u8);
                // astro mantissa words are little-endian (word 0 least
                // significant).
                for (i, w) in words.iter().enumerate() {
                    #[allow(clippy::cast_possible_truncation)]
                    let shift = (i * 64) as u32;
                    m += BigInt::from(*w) << shift;
                }
                format!(
                    "A:{};{n};{};{}",
                    m.to_str_radix(16),
                    if sign == AfSign::Pos { "+" } else { "-" },
                    e
                )
            }
            Inner::Complex(p) => format!("C:{};{}", p.0.to_compact(), p.1.to_compact()),
        }
    }

    /// Inverse of [`Value::to_compact`]; returns `None` for malformed input.
    pub fn from_compact(s: &str) -> Option<Self> {
        if let Some(rest) = s.strip_prefix("E:") {
            let (n, d) = rest.split_once('/')?;
            let num = BigInt::parse_bytes(n.as_bytes(), 10)?;
            let den = BigInt::parse_bytes(d.as_bytes(), 10)?;
            if den.is_zero() {
                return None;
            }
            return Some(Value::exact(BigRational::new(num, den)));
        }
        if let Some(rest) = s.strip_prefix("A:") {
            if rest == "0" {
                return Some(Value::approx(Af::new(WORK_P).ok()?));
            }
            let mut it = rest.split(';');
            let mhex = it.next()?;
            let n: usize = it.next()?.parse().ok()?;
            let sign = match it.next()? {
                "+" => AfSign::Pos,
                "-" => AfSign::Neg,
                _ => return None,
            };
            let e: i32 = it.next()?.parse().ok()?;
            let m = BigInt::parse_bytes(mhex.as_bytes(), 16)?;
            if m.is_zero() {
                return None;
            }
            // Split the mantissa back into little-endian 64-bit words.
            let nbytes = n.div_ceil(64);
            let bytes = m.to_bytes_be().1; // magnitude
            let mut padded = vec![0u8; nbytes.saturating_sub(bytes.len())];
            padded.extend_from_slice(&bytes);
            let mut words = Vec::with_capacity(nbytes);
            for chunk in padded.rchunks(8) {
                let mut w: u64 = 0;
                for b in chunk {
                    w = (w << 8) | u64::from(*b);
                }
                words.push(w);
            }
            let f = Af::from_raw_parts(&words, n, sign, e).ok()?;
            return Some(Value::approx(f));
        }
        if let Some(rest) = s.strip_prefix("C:") {
            let (re, im) = rest.split_once(';')?;
            return Some(Value::complex(
                Value::from_compact(re)?,
                Value::from_compact(im)?,
            ));
        }
        None
    }
}

impl PartialEq for Inner {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Inner::Exact(a), Inner::Exact(b)) => a == b,
            (Inner::Approx(a), Inner::Approx(b)) => a.cmp(b) == 0,
            (Inner::Complex(a), Inner::Complex(b)) => a.0 == b.0 && a.1 == b.1,
            // Cross-kind: compare exact numeric values (floats are exact
            // dyadic rationals, so this is a precise comparison).
            (Inner::Exact(a), Inner::Approx(b)) | (Inner::Approx(b), Inner::Exact(a)) => {
                *a == bf_to_rational(b)
            }
            _ => false,
        }
    }
}

// ---------------------------------------------------------------------------
// astro-float helpers
// ---------------------------------------------------------------------------

/// Decompose a finite float into (mantissa, base-2 exponent) with
/// |value| = m·2^e, m ≥ 0. Returns (0, 0) for zero.
///
/// astro-float stores the mantissa as a *fraction* in [0.5, 1) — the binary
/// point sits before the top bit of the word array — so the effective
/// exponent is `e − 64·words.len()`.
pub(crate) fn bf_parts(f: &Af) -> (BigInt, i32) {
    if f.is_zero() {
        return (BigInt::from(0u8), 0);
    }
    let (words, _n, sign, e) = f.to_raw_parts();
    let mut m = BigInt::from(0u8);
    for (i, w) in words.iter().enumerate() {
        #[allow(clippy::cast_possible_truncation)]
        let shift = (i * 64) as u32;
        m += BigInt::from(*w) << shift;
    }
    if sign == AfSign::Neg {
        m = -m;
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let e_eff = e.saturating_sub((words.len() * 64) as i32);
    (m, e_eff)
}

/// Exact dyadic rational behind a finite float.
pub(crate) fn bf_to_rational(f: &Af) -> BigRational {
    let (m, e) = bf_parts(f);
    if e >= 0 {
        BigRational::from_integer(m << e)
    } else {
        BigRational::new(m, BigInt::from(2).pow((-e) as u32))
    }
}

/// Round a guard-precision float back down to [`WORK_P`].
pub(crate) fn with_guard(f: Af) -> Af {
    let mut g = f;
    let _ = g.set_precision(WORK_P, RM);
    g
}

// ---------------------------------------------------------------------------
// BigInt helpers
// ---------------------------------------------------------------------------

/// Round the exact rational `num/den` (den > 0) half away from zero.
pub(crate) fn half_away_div(num: &BigInt, den: &BigInt) -> BigInt {
    debug_assert!(!den.is_zero());
    if num.is_zero() {
        return BigInt::from(0u8);
    }
    let neg = num.is_negative();
    let n = num.abs();
    // round(n/d) half-up for positive operands == half away from zero.
    let two_n = &n << 1u32;
    let q = (&two_n + den) / (den << 1u32);
    if neg {
        -q
    } else {
        q
    }
}

/// `round(|v| · 10^k)` half away from zero, for the exact rational |v|.
pub(crate) fn round_scaled(r: &BigRational, k: i32) -> BigInt {
    round_scaled_radix(r, k, 10)
}

/// `round(|v| · radix^k)` half away from zero (base rendering).
pub(crate) fn round_scaled_radix(r: &BigRational, k: i32, radix: u32) -> BigInt {
    let num = r.numer().clone();
    let den = r.denom().clone();
    let rb = BigInt::from(radix);
    if k >= 0 {
        half_away_div(&(num * rb.pow(k as u32)), &den)
    } else {
        half_away_div(&num, &(den * rb.pow((-k) as u32)))
    }
}

/// Compare exact rational `r` with `10^d` (d may be negative).
pub(crate) fn cmp_pow10(r: &BigRational, d: i32) -> Ordering {
    let num = r.numer();
    let den = r.denom();
    if d >= 0 {
        let p10 = BigInt::from(10).pow(d as u32);
        num.cmp(&(den * &p10))
    } else {
        let p10 = BigInt::from(10).pow((-d) as u32);
        (num * &p10).cmp(den)
    }
}

/// floor(log10(|r|)) for a positive exact rational.
pub(crate) fn d10_of(r: &BigRational) -> i32 {
    // Estimate from digit counts, then correct by exact comparison.
    let digits_num = r.numer().to_string().len() as i32;
    let digits_den = r.denom().to_string().len() as i32;
    let mut d = digits_num - digits_den;
    while cmp_pow10(r, d) == Ordering::Less {
        d -= 1;
    }
    while cmp_pow10(r, d + 1) != Ordering::Less {
        d += 1;
    }
    d
}
