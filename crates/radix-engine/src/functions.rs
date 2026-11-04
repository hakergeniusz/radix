//! Built-in functions, constants, bitwise word operations and the
//! f(x)/x picker catalogs (doc 05).

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{Signed, Zero};

use crate::error::{ErrorKind, EvalResult};
use crate::eval::{AngleUnit, Env, WordSize};
use crate::num::{e as bf_e, pi as bf_pi, Value, WORK_P};
use crate::ops;

// ---------------------------------------------------------------------------
// constant tables
// ---------------------------------------------------------------------------

/// Non-ASCII constant glyphs → resolution key. Multi-char glyphs are
/// matched by the tokenizer before identifier scanning.
pub(crate) const GLYPH_CONSTS: &[(&str, &str)] = &[
    ("π", "pi"),
    ("τ", "tau"),
    ("φ", "phi"),
    ("α", "alpha"),
    ("σ", "sigma"),
    ("mₑ", "me"),
    ("mₚ", "mp"),
    ("mᵤ", "mu"),
    ("μ₀", "mu0"),
    ("ε₀", "eps0"),
    ("Z₀", "z0"),
];

/// Typed constant names (case-insensitive; matched lowercased).
pub(crate) const TYPED_CONSTS: &[(&str, &str)] = &[("e", "e"), ("pi", "pi"), ("tau", "tau")];

/// ASCII-letter constant glyphs. These resolve **case-sensitively** because
/// `G` (gravitational constant) and `g` (Earth acceleration) are distinct;
/// they exist so picker-inserted glyphs parse (typing them also works — a
/// harmless superset, no observed vector contradicts it).
pub(crate) const ASCII_CONSTS: &[(&str, &str)] = &[
    ("h", "h"),
    ("k", "k"),
    ("R", "R"),
    ("G", "G"),
    ("g", "g"),
    ("c", "c"),
    ("q", "q"),
    ("NA", "NA"),
];

pub(crate) fn glyph_consts() -> &'static [(&'static str, &'static str)] {
    GLYPH_CONSTS
}

pub(crate) fn typed_consts() -> &'static [(&'static str, &'static str)] {
    TYPED_CONSTS
}

pub(crate) fn ascii_consts() -> &'static [(&'static str, &'static str)] {
    ASCII_CONSTS
}

/// Resolve a constant key to its value. Irrational constants are
/// high-precision approximations; the physical constants are exact
/// decimals (as published).
pub(crate) fn constant_value(key: &str) -> EvalResult<Value> {
    Ok(match key {
        "e" => Value::approx(bf_e(WORK_P)?),
        "pi" => Value::approx(bf_pi(WORK_P)?),
        "tau" => {
            let two = Value::af_from_i64(2, WORK_P)?;
            Value::approx(bf_pi(WORK_P)?.mul(&two, WORK_P, crate::num::RM).map_err(crate::num::af_err)?)
        }
        "phi" => {
            // (1 + √5)/2
            let five = Value::exact_i64(5);
            let s = ops::sqrt_v(&five)?;
            ops::add(&s, &Value::exact_i64(1))?
                .div_value(&Value::exact_i64(2))?
        }
        "alpha" => Value::exact_from_decimal("7.2973525643e-3"),
        "sigma" => Value::exact_from_decimal("5.670374419e-8"),
        "me" => Value::exact_from_decimal("9.1093837139e-31"),
        "mp" => Value::exact_from_decimal("1.67262192595e-27"),
        "mu" => Value::exact_from_decimal("1.66053906892e-27"),
        "mu0" => Value::exact_from_decimal("1.25663706127e-6"),
        "eps0" => Value::exact_from_decimal("8.8541878128e-12"),
        "z0" => Value::exact_from_decimal("376.730313412"),
        "h" => Value::exact_from_decimal("6.62607015e-34"),
        "k" => Value::exact_from_decimal("1.380649e-23"),
        "R" => Value::exact_from_decimal("8.31446261815324"),
        "G" => Value::exact_from_decimal("6.67430e-11"),
        "g" => Value::exact_from_decimal("9.80665"),
        "c" => Value::exact_from_decimal("299792458"),
        "q" => Value::exact_from_decimal("1.602176634e-19"),
        "NA" => Value::exact_from_decimal("6.02214076e23"),
        _ => return Err(crate::error::EvalError::new(ErrorKind::OutOfDomain)),
    })
}

// ---------------------------------------------------------------------------
// function name table
// ---------------------------------------------------------------------------

pub(crate) const FUNCTION_NAMES: &[&str] = &[
    "sin", "cos", "tan", "asin", "acos", "atan", "sinh", "cosh", "tanh", //
    "conj", "arg", "re", "im", //
    "twos", "bswap", "mod", "modexp", //
    "round", "floor", "ceil", "int", "frac", //
    "sum", "sumsq", "average", "median", "min", "max", "stdev", "stdevp", "var", //
    "abs", "exp", "sqrt", "ln", "log", "log2",
];

pub(crate) fn function_exists(name: &str) -> bool {
    FUNCTION_NAMES.contains(&name)
}

// ---------------------------------------------------------------------------
// angle conversion
// ---------------------------------------------------------------------------

fn deg_factor() -> EvalResult<Value> {
    // π/180 — irrational, promotes to approx
    let pi = Value::approx(bf_pi(WORK_P)?);
    pi.div_value(&Value::exact_i64(180))
}

fn grad_factor() -> EvalResult<Value> {
    let pi = Value::approx(bf_pi(WORK_P)?);
    pi.div_value(&Value::exact_i64(200))
}

fn to_radians(v: &Value, unit: AngleUnit) -> EvalResult<Value> {
    match unit {
        AngleUnit::Radians => Ok(v.clone()),
        AngleUnit::Degrees => ops::mul(v, &deg_factor()?),
        AngleUnit::Gradians => ops::mul(v, &grad_factor()?),
    }
}

fn from_radians(v: &Value, unit: AngleUnit) -> EvalResult<Value> {
    match unit {
        AngleUnit::Radians => Ok(v.clone()),
        AngleUnit::Degrees => {
            let f = Value::exact_i64(180).div_value(&Value::approx(bf_pi(WORK_P)?))?;
            ops::mul(v, &f)
        }
        AngleUnit::Gradians => {
            let f = Value::exact_i64(200).div_value(&Value::approx(bf_pi(WORK_P)?))?;
            ops::mul(v, &f)
        }
    }
}

// ---------------------------------------------------------------------------
// word-size (programming) helpers
// ---------------------------------------------------------------------------

fn word_modulus(ws: WordSize) -> BigInt {
    BigInt::from(2).pow(ws.bits())
}

/// Operand of a bitwise operation: must be an exact integer (any sign);
/// it is masked into [0, 2^W) (two's-complement wrap).
fn to_word(v: &Value, ws: WordSize) -> EvalResult<BigInt> {
    let Some(n) = v.as_integral() else {
        return Err(crate::error::EvalError::new(ErrorKind::InvalidBitwiseOperand));
    };
    let m = word_modulus(ws);
    // floored mod → [0, 2^W)
    let r = &n % &m;
    Ok(if r.is_negative() { r + m } else { r })
}

fn shift_amount(v: &Value, ws: WordSize) -> EvalResult<Option<u64>> {
    let Some(n) = v.as_integral() else {
        return Err(crate::error::EvalError::new(ErrorKind::InvalidBitwiseOperand));
    };
    if n.is_negative() {
        return Err(crate::error::EvalError::new(ErrorKind::InvalidBitwiseOperand));
    }
    // Shifts ≥ word width yield 0; compare cheaply without building 2^W.
    if n.bits() > 64 {
        return Ok(None); // "too large" sentinel
    }
    use num_traits::ToPrimitive;
    let s = n.to_u64().expect("checked bits");
    Ok(Some(if s >= u64::from(ws.bits()) { u64::from(ws.bits()) } else { s }))
}

/// `¬x` — bitwise NOT within the word-size domain: 2^W − 1 − x.
pub(crate) fn bitwise_not(v: &Value, ws: WordSize) -> EvalResult<Value> {
    let x = to_word(v, ws)?;
    let m = word_modulus(ws);
    Ok(Value::exact_int(m - 1 - x))
}

/// `twos x` — two's complement: 2^W − x (differs from ¬ by 1, T041).
pub(crate) fn twos_complement(v: &Value, ws: WordSize) -> EvalResult<Value> {
    let x = to_word(v, ws)?;
    let m = word_modulus(ws);
    let r = (-x) % &m;
    Ok(Value::exact_int(if r.is_negative() { r + m } else { r }))
}

/// `bswap x` — byte swap within the word (T043).
pub(crate) fn byte_swap(v: &Value, ws: WordSize) -> EvalResult<Value> {
    let w = to_word(v, ws)?;
    let bytes = ws.bits() / 8;
    let byte_mask = BigInt::from(0xFF);
    let mut out = BigInt::from(0);
    let mut tmp = w;
    let mut chunks = Vec::with_capacity(bytes as usize);
    for _ in 0..bytes {
        chunks.push(&tmp & &byte_mask);
        tmp >>= 8;
    }
    for c in chunks {
        out = (out << 8) | c;
    }
    Ok(Value::exact_int(out))
}

/// Binary logic op selector.
pub(crate) enum LogicOp {
    And,
    Or,
    Xor,
}

/// `∧ ∨ ⊻` within the word-size domain.
pub(crate) fn bitwise_logic(l: &Value, r: &Value, ws: WordSize, op: LogicOp) -> EvalResult<Value> {
    let a = to_word(l, ws)?;
    let b = to_word(r, ws)?;
    let out = match op {
        LogicOp::And => a & b,
        LogicOp::Or => a | b,
        LogicOp::Xor => a ^ b,
    };
    Ok(Value::exact_int(out))
}

/// `<<` (left) / `>>` (right) shifts; amounts ≥ word width yield 0.
pub(crate) fn shift(l: &Value, r: &Value, left: bool, ws: WordSize) -> EvalResult<Value> {
    let a = to_word(l, ws)?;
    match shift_amount(r, ws)? {
        None => Ok(Value::exact_int(BigInt::from(0))),
        Some(s) if s >= u64::from(ws.bits()) => Ok(Value::exact_int(BigInt::from(0))),
        Some(s) => {
            let out = if left {
                // results stay within the word: mask (5<<63 must wrap)
                (a << s) & (&word_modulus(ws) - 1u8)
            } else {
                a >> s
            };
            Ok(Value::exact_int(out))
        }
    }
}

// ---------------------------------------------------------------------------
// gamma (Lanczos) for non-integer factorial — public-domain mathematics
// ---------------------------------------------------------------------------

/// Lanczos approximation coefficients (g = 7, n = 9). Full-precision
/// decimal literals are deliberate.
#[allow(clippy::excessive_precision)]
const LANCZOS: &[f64] = &[
    0.999_999_999_999_809_93,
    676.520_368_121_885_1,
    -1_259.139_216_722_402_8,
    771.323_428_777_653_13,
    -176.615_029_162_140_59,
    12.507_343_278_686_905,
    -0.138_571_095_265_720_12,
    9.984_369_578_019_571_6e-6,
    1.505_632_735_149_311_6e-7,
];

fn af_from_f64(x: f64) -> EvalResult<astro_float::BigFloatNumber> {
    astro_float::BigFloatNumber::from_f64(WORK_P + crate::num::GUARD_P, x)
        .map_err(crate::num::af_err)
}

/// Γ(x) for real x via the Lanczos approximation (reflection for x < ½).
/// `exp(ln_gamma(x))` per the brief. Returns an approx value.
fn gamma_positive(v: &Value) -> EvalResult<Value> {
    let p = WORK_P + crate::num::GUARD_P;
    let xp = ops::af_at_guard(v)?;
    // A = c0 + Σ_{k=1..8} ck/(z−1+k)
    let mut a = af_from_f64(LANCZOS[0])?;
    for (k, c) in LANCZOS.iter().enumerate().skip(1) {
        let km1 = Value::af_from_i64(k as i64 - 1, p)?;
        let denom = ops::af_add(&xp, &km1)?; // z − 1 + k
        let term = af_from_f64(*c)?.div(&denom, p, crate::num::RM).map_err(crate::num::af_err)?;
        a = a.add(&term, p, crate::num::RM).map_err(crate::num::af_err)?;
    }
    let t = ops::af_add(&xp, &af_from_f64(6.5)?)?; // z + g − 0.5 with g = 7
    // Γ = √(2π) · t^(z−0.5) · e^(−t) · A
    let two = af_from_f64(2.0)?;
    let two_pi = bf_pi(p)?
        .mul(&two, p, crate::num::RM)
        .map_err(crate::num::af_err)?
        .sqrt(p, crate::num::RM)
        .map_err(crate::num::af_err)?;
    let z_minus_half = ops::af_sub(&xp, &af_from_f64(0.5)?)?;
    let tpow = t
        .pow(&z_minus_half, p, crate::num::RM, &mut crate::num::consts())
        .map_err(crate::num::af_err)?;
    let negt = ops::af_neg(&t)?;
    let exp_part = ops::af_exp_pub(&negt)?;
    let result = two_pi
        .mul(&tpow, p, crate::num::RM)
        .and_then(|x| x.mul(&exp_part, p, crate::num::RM))
        .and_then(|x| x.mul(&a, p, crate::num::RM))
        .map_err(crate::num::af_err)?;
    Ok(Value::approx(ops::with_guard_pub(result)))
}

fn gamma_real(v: &Value) -> EvalResult<Value> {
    // Γ(x): reflection for x < 0.5: Γ(x) = π / (sin(πx)·Γ(1−x))
    let r = v
        .to_ratio()
        .ok_or_else(|| crate::error::EvalError::new(ErrorKind::OutOfDomain))?;
    let half = BigRational::new(BigInt::from(1), BigInt::from(2));
    if r >= half {
        return gamma_positive(v);
    }
    let pi_v = Value::approx(bf_pi(WORK_P)?);
    let pi_x = ops::mul(&pi_v, v)?;
    let sin_part = ops::sin_rad(&pi_x)?;
    let one_minus = Value::exact_i64(1).sub_value(v)?;
    let g = gamma_positive(&one_minus)?;
    let denom = ops::mul(&sin_part, &g)?;
    ops::div(&pi_v, &denom)
}

/// Factorial: exact BigInt product for non-negative integers, Lanczos
/// gamma extension otherwise (`2.5! = 3.32335097`, T019).
pub(crate) fn factorial(v: &Value) -> EvalResult<Value> {
    if let Some(n) = v.as_integral() {
        use num_traits::ToPrimitive;
        if n.is_negative() {
            return Err(crate::error::EvalError::new(ErrorKind::OutOfDomain));
        }
        let Some(n) = n.to_u32() else {
            return Err(crate::error::EvalError::new(ErrorKind::OutOfDomain));
        };
        if n > crate::num::MAX_FACTORIAL {
            return Err(crate::error::EvalError::new(ErrorKind::OutOfDomain));
        }
        let mut acc = BigInt::from(1);
        for i in 2..=n {
            acc *= i;
        }
        return Ok(Value::exact_int(acc));
    }
    // non-integer: Γ(x+1)
    let x1 = ops::add(v, &Value::exact_i64(1))?;
    gamma_real(&x1)
}

// ---------------------------------------------------------------------------
// statistics
// ---------------------------------------------------------------------------

fn stats_reals(args: &[Value]) -> EvalResult<Vec<BigRational>> {
    args.iter()
        .map(|v| {
            if v.inner_complex().is_some() {
                return Err(crate::error::EvalError::new(ErrorKind::OutOfDomain));
            }
            v.to_ratio()
                .ok_or_else(|| crate::error::EvalError::new(ErrorKind::OutOfDomain))
        })
        .collect()
}

fn mean_of(rs: &[BigRational]) -> BigRational {
    let n = BigInt::from(rs.len());
    let sum: BigRational = rs.iter().sum();
    sum / n
}

fn variance(rs: &[BigRational], sample: bool) -> BigRational {
    let m = mean_of(rs);
    let n = rs.len();
    let denom = if sample { n - 1 } else { n };
    let mut acc = BigRational::from_integer(BigInt::from(0));
    for r in rs {
        let d = r - &m;
        acc += &d * &d;
    }
    acc / BigInt::from(denom)
}

fn ratio_to_value(r: BigRational, inexact: bool) -> Value {
    if inexact {
        match Value::af_from_rational(&r, WORK_P) {
            Ok(f) => Value::approx(f),
            Err(_) => Value::exact(r),
        }
    } else {
        Value::exact(r)
    }
}

// ---------------------------------------------------------------------------
// dispatch
// ---------------------------------------------------------------------------

/// Wrong-arity helper.
fn arity(name: &str, expected: &'static str) -> crate::error::EvalError {
    crate::error::EvalError::new(ErrorKind::WrongArity {
        name: name.to_string(),
        expected,
    })
}

/// Call a built-in function with already-evaluated arguments.
/// `spans[i]` is the source span of `args[i]` (used for the tangent-pole
/// highlight).
#[allow(clippy::too_many_lines)]
pub(crate) fn call(
    name: &str,
    args: &[Value],
    spans: &[std::ops::Range<usize>],
    env: &Env,
) -> EvalResult<Value> {
    if !function_exists(name) {
        return Err(crate::error::EvalError::new(ErrorKind::UndefinedFunction(
            name.to_string(),
        )));
    }
    let one = |n: usize| -> EvalResult<Value> {
        if args.len() != 1 {
            return Err(arity(name, "1"));
        }
        Ok(args[n].clone())
    };
    match name {
        // -- trigonometry (angle unit applies; doc 04 §4) ----------------
        "sin" => {
            let a = to_radians(&one(0)?, env.angle_unit)?;
            match a.inner_complex() {
                Some((re, im)) => ops::c_sin(re, im),
                None => ops::sin_rad(&a),
            }
        }
        "cos" => {
            let a = to_radians(&one(0)?, env.angle_unit)?;
            match a.inner_complex() {
                Some((re, im)) => ops::c_cos(re, im),
                None => ops::cos_rad(&a),
            }
        }
        "tan" => {
            let arg = one(0)?;
            let span = spans.first().cloned().unwrap_or(0..0);
            let is_complex = arg.inner_complex().is_some();
            if !is_complex {
                match &arg.inner {
                    // Exact rational angle: exact pole test only (doc:
                    // tan(90) in Degrees; grad equivalent). No numeric
                    // check, so near-pole exact angles keep huge values.
                    crate::num::Inner::Exact(_) => {
                        if !matches!(env.angle_unit, AngleUnit::Radians) {
                            let r = arg.to_ratio().expect("exact has ratio");
                            let d = match env.angle_unit {
                                AngleUnit::Degrees => 90,
                                _ => 100,
                            };
                            let q = r / BigRational::from_integer(BigInt::from(d));
                            if q.is_integer() && (q.numer() % 2u8) == 1u8.into() {
                                return Err(crate::error::EvalError::with_span(
                                    ErrorKind::TangentUndefined,
                                    span,
                                ));
                            }
                        }
                    }
                    // Approx angle (incl. computed pi/2): numeric pole check.
                    _ => {
                        let rad = to_radians(&arg, env.angle_unit)?;
                        let c = ops::cos_rad(&rad)?;
                        if let Some(cr) = c.to_ratio() {
                            let tiny =
                                BigRational::new(BigInt::from(1), BigInt::from(10).pow(18));
                            if cr.abs() < tiny {
                                return Err(crate::error::EvalError::with_span(
                                    ErrorKind::TangentUndefined,
                                    span,
                                ));
                            }
                        }
                    }
                }
            }
            match to_radians(&arg, env.angle_unit)?.inner_complex() {
                Some((re, im)) => ops::c_tan(re, im),
                None => ops::tan_rad(&to_radians(&arg, env.angle_unit)?),
            }
        }
        "asin" => from_radians(&ops::asin_rad(&one(0)?)?, env.angle_unit),
        "acos" => from_radians(&ops::acos_rad(&one(0)?)?, env.angle_unit),
        "atan" => from_radians(&ops::atan_rad(&one(0)?)?, env.angle_unit),
        "sinh" => match one(0)?.inner_complex() {
            Some((re, im)) => ops::c_sinh(re, im),
            None => ops::sinh_v(&args[0]),
        },
        "cosh" => match one(0)?.inner_complex() {
            Some((re, im)) => ops::c_cosh(re, im),
            None => ops::cosh_v(&args[0]),
        },
        "tanh" => match one(0)?.inner_complex() {
            Some((re, im)) => ops::c_tanh(re, im),
            None => ops::tanh_v(&args[0]),
        },
        // -- complex ------------------------------------------------------
        "conj" => {
            let v = one(0)?;
            if let Some((re, im)) = v.inner_complex() {
                Ok(Value::complex(re.clone(), im.neg_value()?))
            } else {
                Ok(v)
            }
        }
        "arg" => {
            let v = one(0)?;
            match v.inner_complex() {
                Some((re, im)) => {
                    let re_af = ops::af_at_guard(re)?;
                    let im_af = ops::af_at_guard(im)?;
                    if re.is_zero() && im.is_zero() {
                        return Ok(Value::exact_i64(0));
                    }
                    let t = ops::af_atan2_guarded(&im_af, &re_af)?;
                    Ok(Value::approx(t))
                }
                None => {
                    if v.is_negative() {
                        Ok(Value::approx(bf_pi(WORK_P)?))
                    } else {
                        Ok(Value::exact_i64(0))
                    }
                }
            }
        }
        "re" => {
            let v = one(0)?;
            Ok(v.real_and_imaginary().0)
        }
        "im" => {
            let v = one(0)?;
            Ok(v.real_and_imaginary().1)
        }
        // -- programming ---------------------------------------------------
        "twos" => twos_complement(&one(0)?, env.word_size),
        "bswap" => byte_swap(&one(0)?, env.word_size),
        "mod" => {
            if args.len() != 2 {
                return Err(arity(name, "2"));
            }
            ops::floor_mod(&args[0], &args[1])
        }
        "modexp" => {
            if args.len() != 3 {
                return Err(arity(name, "3"));
            }
            let x = args[0]
                .as_integral()
                .ok_or_else(|| crate::error::EvalError::new(ErrorKind::InvalidBitwiseOperand))?;
            let y = args[1]
                .as_integral()
                .ok_or_else(|| crate::error::EvalError::new(ErrorKind::InvalidBitwiseOperand))?;
            let p = args[2]
                .as_integral()
                .ok_or_else(|| crate::error::EvalError::new(ErrorKind::InvalidBitwiseOperand))?;
            if y.is_negative() || p.is_negative() || p.is_zero() {
                return Err(crate::error::EvalError::new(ErrorKind::InvalidBitwiseOperand));
            }
            Ok(Value::exact_int(x.modpow(&y, &p)))
        }
        // -- rounding (doc 04 §3) ------------------------------------------
        "round" => ops::round_v(&one(0)?),
        "floor" => ops::floor_v(&one(0)?),
        "ceil" => ops::ceil_v(&one(0)?),
        "int" => ops::trunc_v(&one(0)?),
        "frac" => ops::fract_v(&one(0)?),
        // -- statistics ------------------------------------------------------
        "sum" => {
            if args.is_empty() {
                return Err(arity(name, "at least 1"));
            }
            let rs = stats_reals(args)?;
            let inexact = args.iter().any(|v| !v.is_exact());
            let sum: BigRational = rs.iter().sum();
            Ok(ratio_to_value(sum, inexact))
        }
        "sumsq" => {
            if args.is_empty() {
                return Err(arity(name, "at least 1"));
            }
            let rs = stats_reals(args)?;
            let inexact = args.iter().any(|v| !v.is_exact());
            let mut acc = BigRational::from_integer(BigInt::from(0));
            for r in &rs {
                acc += r * r;
            }
            Ok(ratio_to_value(acc, inexact))
        }
        "average" => {
            if args.is_empty() {
                return Err(arity(name, "at least 1"));
            }
            let rs = stats_reals(args)?;
            let inexact = args.iter().any(|v| !v.is_exact());
            Ok(ratio_to_value(mean_of(&rs), inexact))
        }
        "median" => {
            if args.is_empty() {
                return Err(arity(name, "at least 1"));
            }
            let mut rs = stats_reals(args)?;
            let inexact = args.iter().any(|v| !v.is_exact());
            rs.sort();
            let n = rs.len();
            let med = if n % 2 == 1 {
                rs[n / 2].clone()
            } else {
                (&rs[n / 2 - 1] + &rs[n / 2]) / BigInt::from(2)
            };
            Ok(ratio_to_value(med, inexact))
        }
        "min" | "max" => {
            if args.is_empty() {
                return Err(arity(name, "at least 1"));
            }
            let rs = stats_reals(args)?;
            let best = if name == "min" {
                rs.iter().min().ok_or_else(|| arity(name, "at least 1"))?
            } else {
                rs.iter().max().ok_or_else(|| arity(name, "at least 1"))?
            };
            Ok(ratio_to_value(best.clone(), !args.iter().all(|v| v.is_exact())))
        }
        "stdev" | "stdevp" | "var" => {
            let sample = name != "stdevp";
            if args.len() < 2 {
                return Err(arity(name, "at least 2"));
            }
            let rs = stats_reals(args)?;
            let inexact = args.iter().any(|v| !v.is_exact());
            let v = variance(&rs, sample);
            if name == "var" {
                Ok(ratio_to_value(v, inexact))
            } else {
                ops::sqrt_v(&ratio_to_value(v, inexact))
            }
        }
        // -- sundry ----------------------------------------------------------
        "abs" => ops::abs_v(&one(0)?),
        "exp" => {
            let v = one(0)?;
            match v.inner_complex() {
                Some((re, im)) => ops::c_exp(re, im),
                None => ops::exp_v(&v),
            }
        }
        "sqrt" => ops::sqrt_v(&one(0)?),
        "ln" => {
            let v = one(0)?;
            match v.inner_complex() {
                Some((re, im)) => ops::c_ln(re, im),
                None => ops::ln_v(&v),
            }
        }
        "log" => {
            if args.is_empty() || args.len() > 2 {
                return Err(arity(name, "1 or 2"));
            }
            let x = &args[0];
            if args.len() == 1 {
                // common log
                let ten = Value::exact_i64(10);
                let lb = ops::ln_v(&ten)?;
                let lx = match x.inner_complex() {
                    Some((re, im)) => ops::c_ln(re, im)?,
                    None => ops::ln_v(x)?,
                };
                return ops::div(&lx, &lb);
            }
            let base = &args[1];
            let lb = match base.inner_complex() {
                Some((re, im)) => ops::c_ln(re, im)?,
                None => ops::ln_v(base)?,
            };
            if lb.is_zero() {
                return Err(crate::error::EvalError::new(ErrorKind::DivisionByZero));
            }
            let lx = match x.inner_complex() {
                Some((re, im)) => ops::c_ln(re, im)?,
                None => ops::ln_v(x)?,
            };
            ops::div(&lx, &lb)
        }
        "log2" => {
            let v = one(0)?;
            let two = Value::exact_i64(2);
            let lb = ops::ln_v(&two)?;
            let lx = match v.inner_complex() {
                Some((re, im)) => ops::c_ln(re, im)?,
                None => ops::ln_v(&v)?,
            };
            ops::div(&lx, &lb)
        }
        _ => Err(crate::error::EvalError::new(ErrorKind::UndefinedFunction(
            name.to_string(),
        ))),
    }
}

// ---------------------------------------------------------------------------
// catalogs (doc 05 — drive the UI pickers)
// ---------------------------------------------------------------------------

/// f(x) picker categories.
pub struct FunctionCategory {
    /// Category heading.
    pub name: &'static str,
    /// Functions in display order.
    pub functions: &'static [FunctionInfo],
}

/// One f(x) picker row.
pub struct FunctionInfo {
    /// Typed name (what resolves in the entry).
    pub name: &'static str,
    /// Template inserted on click, display form (`sin⁻¹(`).
    pub template: &'static str,
    /// Human description; names the typed form for display-form templates.
    pub description: &'static str,
}

/// x picker categories.
pub struct ConstantCategory {
    /// Category heading.
    pub name: &'static str,
    /// Constants in display order.
    pub constants: &'static [ConstantInfo],
}

/// One x picker row.
pub struct ConstantInfo {
    /// Display name.
    pub name: &'static str,
    /// Glyph inserted on click.
    pub glyph: &'static str,
    /// Typed name, when one resolves (`phi` does not — T086).
    pub typed_name: Option<&'static str>,
    /// Human description.
    pub description: &'static str,
}

const TRIG: &[FunctionInfo] = &[
    FunctionInfo { name: "sin", template: "sin(", description: "sin(x) — sine of x" },
    FunctionInfo { name: "cos", template: "cos(", description: "cos(x) — cosine of x" },
    FunctionInfo { name: "tan", template: "tan(", description: "tan(x) — tangent of x" },
    FunctionInfo { name: "asin", template: "sin⁻¹(", description: "asin(x) — inverse sine (typed as asin)" },
    FunctionInfo { name: "acos", template: "cos⁻¹(", description: "acos(x) — inverse cosine (typed as acos)" },
    FunctionInfo { name: "atan", template: "tan⁻¹(", description: "atan(x) — inverse tangent (typed as atan)" },
    FunctionInfo { name: "sinh", template: "sinh(", description: "sinh(x) — hyperbolic sine" },
    FunctionInfo { name: "cosh", template: "cosh(", description: "cosh(x) — hyperbolic cosine" },
    FunctionInfo { name: "tanh", template: "tanh(", description: "tanh(x) — hyperbolic tangent" },
];

const COMPLEX_FNS: &[FunctionInfo] = &[
    FunctionInfo { name: "conj", template: "conj(", description: "conj(x) — complex conjugate" },
    FunctionInfo { name: "arg", template: "arg(", description: "arg(x) — argument (phase angle)" },
    FunctionInfo { name: "re", template: "re(", description: "re(x) — real part" },
    FunctionInfo { name: "im", template: "im(", description: "im(x) — imaginary part" },
];

const PROGRAMMING_FNS: &[FunctionInfo] = &[
    FunctionInfo { name: "twos", template: "twos(", description: "twos(x) — two's complement (also prefix: twos 5)" },
    FunctionInfo { name: "bswap", template: "bswap(", description: "bswap(x) — byte swap (also prefix: bswap 255)" },
    FunctionInfo { name: "mod", template: "mod(", description: "mod(x;y) — modulo (infix: x mod y)" },
    FunctionInfo { name: "modexp", template: "modexp(", description: "modexp(x;y;p) — modular exponentiation" },
];

const ROUNDING_FNS: &[FunctionInfo] = &[
    FunctionInfo { name: "round", template: "round(", description: "round(x) — nearest integer, halves away from zero" },
    FunctionInfo { name: "floor", template: "floor(", description: "floor(x) — largest integer ≤ x" },
    FunctionInfo { name: "ceil", template: "ceil(", description: "ceil(x) — smallest integer ≥ x" },
    FunctionInfo { name: "int", template: "int(", description: "int(x) — truncate toward zero" },
    FunctionInfo { name: "frac", template: "frac(", description: "frac(x) — fractional part, sign of x" },
];

const STAT_FNS: &[FunctionInfo] = &[
    FunctionInfo { name: "sum", template: "sum(", description: "sum(x₁;x₂;…) — sum of arguments" },
    FunctionInfo { name: "sumsq", template: "sumsq(", description: "sumsq(x₁;x₂;…) — sum of squares" },
    FunctionInfo { name: "average", template: "average(", description: "average(x₁;x₂;…) — arithmetic mean" },
    FunctionInfo { name: "median", template: "median(", description: "median(x₁;x₂;…) — middle value" },
    FunctionInfo { name: "min", template: "min(", description: "min(x₁;x₂;…) — smallest argument" },
    FunctionInfo { name: "max", template: "max(", description: "max(x₁;x₂;…) — largest argument" },
    FunctionInfo { name: "stdev", template: "stdev(", description: "stdev(x₁;x₂;…) — sample standard deviation (n−1)" },
    FunctionInfo { name: "stdevp", template: "stdevp(", description: "stdevp(x₁;x₂;…) — population standard deviation (n)" },
    FunctionInfo { name: "var", template: "var(", description: "var(x₁;x₂;…) — sample variance (n−1)" },
];

const SUNDRY_FNS: &[FunctionInfo] = &[
    FunctionInfo { name: "abs", template: "abs(", description: "abs(x) — absolute value (|x|)" },
    FunctionInfo { name: "exp", template: "exp(", description: "exp(x) — e to the power of x" },
    FunctionInfo { name: "sqrt", template: "sqrt(", description: "sqrt(x) — square root (√ prefix also accepted)" },
];

const MATH_CONSTS: &[ConstantInfo] = &[
    ConstantInfo {
        name: "Euler's Number",
        glyph: "e",
        typed_name: Some("e"),
        description: "e = 2.718281828…",
    },
    ConstantInfo {
        name: "Pi",
        glyph: "π",
        typed_name: Some("pi"),
        description: "π = 3.141592654…",
    },
    ConstantInfo {
        name: "Tau",
        glyph: "τ",
        typed_name: Some("tau"),
        description: "τ = 2π = 6.283185307…",
    },
    ConstantInfo {
        name: "Golden Ratio",
        glyph: "φ",
        typed_name: None,
        description: "φ = (1+√5)/2 = 1.618033989… (no typed name)",
    },
];

const ATOMIC_CONSTS: &[ConstantInfo] = &[
    ConstantInfo {
        name: "Planck's Constant",
        glyph: "h",
        typed_name: None,
        description: "h = 6.62607015×10⁻³⁴ J·Hz⁻¹",
    },
    ConstantInfo {
        name: "Fine-Structure Constant",
        glyph: "α",
        typed_name: None,
        description: "α = 7.2973525643×10⁻³",
    },
    ConstantInfo {
        name: "Electron Mass",
        glyph: "mₑ",
        typed_name: None,
        description: "mₑ = 9.1093837139×10⁻³¹ kg",
    },
    ConstantInfo {
        name: "Proton Mass",
        glyph: "mₚ",
        typed_name: None,
        description: "mₚ = 1.67262192595×10⁻²⁷ kg",
    },
];

const THERMO_CONSTS: &[ConstantInfo] = &[
    ConstantInfo {
        name: "Boltzmann Constant",
        glyph: "k",
        typed_name: None,
        description: "k = 1.380649×10⁻²³ J/K",
    },
    ConstantInfo {
        name: "Atomic Mass Unit",
        glyph: "mᵤ",
        typed_name: None,
        description: "mᵤ = 1.66053906892×10⁻²⁷ kg",
    },
    ConstantInfo {
        name: "Molar Gas Constant",
        glyph: "R",
        typed_name: None,
        description: "R = 8.31446261815324 J/(mol·K)",
    },
    ConstantInfo {
        name: "Stefan-Boltzmann Constant",
        glyph: "σ",
        typed_name: None,
        description: "σ = 5.670374419×10⁻⁸ W/(m²·K⁴)",
    },
    ConstantInfo {
        name: "Avogadro's Number",
        glyph: "NA",
        typed_name: None,
        description: "NA = 6.02214076×10²³ mol⁻¹",
    },
];

const GRAV_CONSTS: &[ConstantInfo] = &[
    ConstantInfo {
        name: "Gravitational Constant",
        glyph: "G",
        typed_name: None,
        description: "G = 6.67430×10⁻¹¹ m³/(kg·s²)",
    },
    ConstantInfo {
        name: "Earth Acceleration",
        glyph: "g",
        typed_name: None,
        description: "g = 9.80665 m/s²",
    },
];

const EM_CONSTS: &[ConstantInfo] = &[
    ConstantInfo {
        name: "Speed of Light",
        glyph: "c",
        typed_name: None,
        description: "c = 299792458 m/s",
    },
    ConstantInfo {
        name: "Elementary Charge",
        glyph: "q",
        typed_name: None,
        description: "q = 1.602176634×10⁻¹⁹ C",
    },
    ConstantInfo {
        name: "Magnetic Constant",
        glyph: "μ₀",
        typed_name: None,
        description: "μ₀ = 1.25663706127×10⁻⁶ N/A²",
    },
    ConstantInfo {
        name: "Electric Constant",
        glyph: "ε₀",
        typed_name: None,
        description: "ε₀ = 8.8541878128×10⁻¹² F/m",
    },
    ConstantInfo {
        name: "Vacuum Impedance",
        glyph: "Z₀",
        typed_name: None,
        description: "Z₀ = 376.730313412 Ω",
    },
];

/// Function picker catalog (f(x) popover), doc 05 §1 order.
pub fn function_catalog() -> &'static [FunctionCategory] {
    &[
        FunctionCategory { name: "Trigonometry", functions: TRIG },
        FunctionCategory { name: "Complex", functions: COMPLEX_FNS },
        FunctionCategory { name: "Programming", functions: PROGRAMMING_FNS },
        FunctionCategory { name: "Rounding", functions: ROUNDING_FNS },
        FunctionCategory { name: "Statistics", functions: STAT_FNS },
        FunctionCategory { name: "Sundry", functions: SUNDRY_FNS },
    ]
}

/// Constant picker catalog (x popover), doc 05 §3 order.
pub fn constant_catalog() -> &'static [ConstantCategory] {
    &[
        ConstantCategory { name: "Mathematics", constants: MATH_CONSTS },
        ConstantCategory { name: "Atomic & Nuclear", constants: ATOMIC_CONSTS },
        ConstantCategory { name: "Thermodynamics", constants: THERMO_CONSTS },
        ConstantCategory { name: "Gravitation", constants: GRAV_CONSTS },
        ConstantCategory { name: "Electromagnetism", constants: EM_CONSTS },
    ]
}

