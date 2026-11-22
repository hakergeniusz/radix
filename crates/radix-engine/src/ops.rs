//! Arithmetic operations over [`Value`] with exactness promotion.
//!
//! Promotion rules (doc 04):
//! * exact ⊕ exact → exact for `+ − × ÷(non-zero) %` and integer-`^`;
//! * any approx operand → approx;
//! * any complex operand → complex (exact parts promoted per component);
//! * transcendental / irrational steps (sqrt of a non-perfect-square, log,
//!   trig, gamma) promote exact values to high-precision approximations.
//!
//! All high-precision work uses [`WORK_P`] bits with [`GUARD_P`] extra guard
//! bits for transcendentals; every astro-float error is surfaced as
//! [`ErrorKind::OutOfDomain`] (no panics, no silent overflow).

use astro_float::BigFloatNumber as Af;
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{One, Signed, Zero};

use crate::error::{ErrorKind, EvalError, EvalResult};
use crate::num::{
    af_err, consts, pi, with_guard, Inner, Value, GUARD_P, MAX_INT_OP, RM, WORK_P,
};

pub(crate) fn err(kind: ErrorKind) -> EvalError {
    EvalError::new(kind)
}

// ---------------------------------------------------------------------------
// BigInt helpers
// ---------------------------------------------------------------------------

/// Integer square root (floor) via Newton iteration.
pub(crate) fn isqrt(v: &BigInt) -> BigInt {
    if v <= &BigInt::one() {
        return v.clone();
    }
    // Initial guess ≥ √v.
    let mut x = BigInt::from(2).pow((v.bits() + 1).div_ceil(2) as u32);
    loop {
        let y = (&x + v / &x) >> 1u32;
        if y >= x {
            return x;
        }
        x = y;
    }
}

fn is_perfect_square(v: &BigInt) -> bool {
    if v.is_negative() {
        return false;
    }
    let s = isqrt(v);
    &s * &s == *v
}

// ---------------------------------------------------------------------------
// float helpers
// ---------------------------------------------------------------------------

/// High-precision float for a *real* value at the given precision.
pub(crate) fn af_at(v: &Value, p: usize) -> EvalResult<Af> {
    let r = v.to_ratio().ok_or_else(|| err(ErrorKind::OutOfDomain))?;
    Value::af_from_rational(&r, p)
}

/// Raise a float's storage precision to the guard level (no-op when the
/// number already has at least that much headroom; operands are treated as
/// exact either way).
fn promote_guard(f: &Af) -> EvalResult<Af> {
    let mut g = f.clone().map_err(af_err)?;
    let _ = g.set_precision(WORK_P + GUARD_P, RM);
    Ok(g)
}

fn approx(f: Af) -> Value {
    Value::approx(f)
}

fn exact_or_approx(r: BigRational, inexact: bool) -> Value {
    if inexact {
        match Value::af_from_rational(&r, WORK_P) {
            Ok(f) => Value::approx(f),
            Err(_) => Value::exact(r), // unreachable in practice
        }
    } else {
        Value::exact(r)
    }
}

pub(crate) fn complex_parts(v: &Value) -> Option<(&Value, &Value)> {
    v.inner_complex()
}

/// Float for a real argument at guard precision (exact values promoted).
fn promote_arg(v: &Value) -> EvalResult<Af> {
    match &v.inner {
        Inner::Approx(f) => promote_guard(f),
        Inner::Exact(_) => af_at(v, WORK_P + GUARD_P),
        Inner::Complex(_) => Err(err(ErrorKind::OutOfDomain)),
    }
}

// ---------------------------------------------------------------------------
// real transcendental primitives (radians in / radians out)
// ---------------------------------------------------------------------------

macro_rules! af_fun {
    ($name:ident, $m:ident) => {
        pub(crate) fn $name(x: &Af) -> EvalResult<Af> {
            let g = promote_guard(x)?;
            let c = &mut *consts();
            g.$m(WORK_P + GUARD_P, RM, c).map_err(af_err)
        }
    };
}

af_fun!(af_sin, sin);
af_fun!(af_cos, cos);
af_fun!(af_tan, tan);
af_fun!(af_asin, asin);
af_fun!(af_acos, acos);
af_fun!(af_exp, exp);
af_fun!(af_ln, ln);
af_fun!(af_sinh, sinh);
af_fun!(af_cosh, cosh);
af_fun!(af_tanh, tanh);

pub(crate) fn af_atan(x: &Af) -> EvalResult<Af> {
    let g = promote_guard(x)?;
    let c = &mut *consts();
    g.atan(WORK_P + GUARD_P, RM, c).map_err(af_err)
}

/// atan2(im, re) at guard precision (callers round down before storing).
pub(crate) fn af_atan2(im: &Af, re: &Af) -> EvalResult<Af> {
    let p = WORK_P + GUARD_P;
    if re.is_zero() {
        // ±π/2
        let two = Af::from_i32(2, p).map_err(af_err)?;
        let half_pi = with_guard(pi(p)?.div(&two, p, RM).map_err(af_err)?);
        return Ok(if im.is_negative() {
            half_pi.neg().map_err(af_err)?
        } else {
            half_pi
        });
    }
    let ratio = im.div(re, p, RM).map_err(af_err)?;
    let a = af_atan(&ratio)?;
    if re.is_negative() {
        // atan + π·sign(im)
        let pv = pi(p)?;
        Ok(if im.is_negative() {
            a.sub(&pv, p, RM).map_err(af_err)?
        } else {
            a.add(&pv, p, RM).map_err(af_err)?
        })
    } else {
        Ok(a)
    }
}

pub(crate) fn sin_rad(v: &Value) -> EvalResult<Value> {
    Ok(approx(with_guard(af_sin(&promote_arg(v)?)?)))
}

pub(crate) fn cos_rad(v: &Value) -> EvalResult<Value> {
    Ok(approx(with_guard(af_cos(&promote_arg(v)?)?)))
}

pub(crate) fn tan_rad(v: &Value) -> EvalResult<Value> {
    Ok(approx(with_guard(af_tan(&promote_arg(v)?)?)))
}

pub(crate) fn atan_rad(v: &Value) -> EvalResult<Value> {
    Ok(approx(with_guard(af_atan(&promote_arg(v)?)?)))
}

pub(crate) fn asin_rad(v: &Value) -> EvalResult<Value> {
    if let Some(r) = v.to_ratio() {
        let one = BigRational::from_integer(BigInt::from(1));
        if r.abs() > one {
            return Err(err(ErrorKind::OutOfDomain));
        }
    }
    Ok(approx(with_guard(af_asin(&promote_arg(v)?)?)))
}

pub(crate) fn acos_rad(v: &Value) -> EvalResult<Value> {
    if let Some(r) = v.to_ratio() {
        let one = BigRational::from_integer(BigInt::from(1));
        if r.abs() > one {
            return Err(err(ErrorKind::OutOfDomain));
        }
    }
    Ok(approx(with_guard(af_acos(&promote_arg(v)?)?)))
}

pub(crate) fn exp_v(v: &Value) -> EvalResult<Value> {
    Ok(approx(with_guard(af_exp(&promote_arg(v)?)?)))
}

pub(crate) fn sinh_v(v: &Value) -> EvalResult<Value> {
    Ok(approx(with_guard(af_sinh(&promote_arg(v)?)?)))
}

pub(crate) fn cosh_v(v: &Value) -> EvalResult<Value> {
    Ok(approx(with_guard(af_cosh(&promote_arg(v)?)?)))
}

pub(crate) fn tanh_v(v: &Value) -> EvalResult<Value> {
    Ok(approx(with_guard(af_tanh(&promote_arg(v)?)?)))
}

/// Natural log of a real value; negative input returns the complex
/// principal branch `ln|x| + iπ`.
pub(crate) fn ln_v(v: &Value) -> EvalResult<Value> {
    if v.is_zero() {
        return Err(err(ErrorKind::OutOfDomain));
    }
    if v.is_negative() {
        let m = promote_arg(v)?.neg().map_err(af_err)?;
        let l = real_ln_positive(&Value::approx(m))?;
        let pi_v = Value::approx(pi(WORK_P)?);
        return Ok(Value::complex(l, pi_v));
    }
    real_ln_positive(v)
}

fn real_ln_positive(v: &Value) -> EvalResult<Value> {
    Ok(approx(with_guard(af_ln(&promote_arg(v)?)?)))
}

// ---------------------------------------------------------------------------
// complex primitives (operate on (re, im) Values)
// ---------------------------------------------------------------------------

pub(crate) fn c_exp(re: &Value, im: &Value) -> EvalResult<Value> {
    if im.is_zero() {
        return exp_v(re);
    }
    let r = exp_v(re)?;
    let c = cos_rad(im)?;
    let s = sin_rad(im)?;
    let a = mul(&r, &c)?;
    let b = mul(&r, &s)?;
    Ok(Value::complex(a, b))
}

/// Principal-branch complex logarithm.
pub(crate) fn c_ln(re: &Value, im: &Value) -> EvalResult<Value> {
    if re.is_zero() && im.is_zero() {
        return Err(err(ErrorKind::OutOfDomain));
    }
    if im.is_zero() && !re.is_negative() {
        return ln_v(re);
    }
    // ln|z| + i·atan2(im, re)
    let re2 = mul(re, re)?;
    let im2 = mul(im, im)?;
    let r = sqrt_v(&add(&re2, &im2)?)?;
    let theta = with_guard(af_atan2(&promote_arg(im)?, &promote_arg(re)?)?);
    Ok(Value::complex(real_ln_positive(&r)?, Value::approx(theta)))
}

pub(crate) fn c_sin(re: &Value, im: &Value) -> EvalResult<Value> {
    // sin x cosh y + i cos x sinh y
    let a = mul(&sin_rad(re)?, &cosh_v(im)?)?;
    let b = mul(&cos_rad(re)?, &sinh_v(im)?)?;
    Ok(Value::complex(a, b))
}

pub(crate) fn c_cos(re: &Value, im: &Value) -> EvalResult<Value> {
    // cos x cosh y − i sin x sinh y
    let a = mul(&cos_rad(re)?, &cosh_v(im)?)?;
    let b = mul(&sin_rad(re)?, &sinh_v(im)?)?;
    Ok(Value::complex(a, b.neg_value()?))
}

/// Complex tangent: tan z = sin z / cos z.
pub(crate) fn c_tan(re: &Value, im: &Value) -> EvalResult<Value> {
    let n = c_sin(re, im)?;
    let d = c_cos(re, im)?;
    div(&n, &d)
}

pub(crate) fn c_sinh(re: &Value, im: &Value) -> EvalResult<Value> {
    // sinh x cos y + i cosh x sin y
    let a = mul(&sinh_v(re)?, &cos_rad(im)?)?;
    let b = mul(&cosh_v(re)?, &sin_rad(im)?)?;
    Ok(Value::complex(a, b))
}

pub(crate) fn c_cosh(re: &Value, im: &Value) -> EvalResult<Value> {
    // cosh x cos y + i sinh x sin y
    let a = mul(&cosh_v(re)?, &cos_rad(im)?)?;
    let b = mul(&sinh_v(re)?, &sin_rad(im)?)?;
    Ok(Value::complex(a, b))
}

pub(crate) fn c_tanh(re: &Value, im: &Value) -> EvalResult<Value> {
    let n = c_sinh(re, im)?;
    let d = c_cosh(re, im)?;
    div(&n, &d)
}

/// Principal square root of a complex number (im ≠ 0 expected).
pub(crate) fn c_sqrt(re: &Value, im: &Value) -> EvalResult<Value> {
    // u = sqrt((|z| + a)/2), v = sign(b)·sqrt((|z| − a)/2)
    let a2 = mul(re, re)?;
    let b2 = mul(im, im)?;
    let r = sqrt_v(&add(&a2, &b2)?)?;
    let two = Value::exact_i64(2);
    let u = sqrt_v(&add(&r, re)?.div_value(&two)?)?;
    let mut v = sqrt_v(&sub(&r, re)?.div_value(&two)?)?;
    if im.is_negative() && !v.is_zero() {
        v = v.neg_value()?;
    }
    Ok(Value::complex(u, v))
}

// ---------------------------------------------------------------------------
// binary arithmetic
// ---------------------------------------------------------------------------

pub(crate) fn add(a: &Value, b: &Value) -> EvalResult<Value> {
    match (complex_parts(a), complex_parts(b)) {
        (Some((a1, a2)), Some((b1, b2))) => Ok(Value::complex(add(a1, b1)?, add(a2, b2)?)),
        (Some((a1, a2)), None) => Ok(Value::complex(add(a1, b)?, a2.clone())),
        (None, Some((b1, b2))) => Ok(Value::complex(add(a, b1)?, b2.clone())),
        (None, None) => arith(a, b, ArithOp::Add),
    }
}

pub(crate) fn sub(a: &Value, b: &Value) -> EvalResult<Value> {
    match (complex_parts(a), complex_parts(b)) {
        (Some((a1, a2)), Some((b1, b2))) => Ok(Value::complex(sub(a1, b1)?, sub(a2, b2)?)),
        (Some((a1, a2)), None) => Ok(Value::complex(sub(a1, b)?, a2.clone())),
        (None, Some((b1, b2))) => Ok(Value::complex(sub(a, b1)?, b2.neg_value()?)),
        (None, None) => arith(a, b, ArithOp::Sub),
    }
}

pub(crate) fn mul(a: &Value, b: &Value) -> EvalResult<Value> {
    match (complex_parts(a), complex_parts(b)) {
        (Some((a1, a2)), Some((b1, b2))) => {
            // (ac − bd) + (ad + bc)i
            let ac = mul(a1, b1)?;
            let bd = mul(a2, b2)?;
            let ad = mul(a1, b2)?;
            let bc = mul(a2, b1)?;
            Ok(Value::complex(sub(&ac, &bd)?, add(&ad, &bc)?))
        }
        (Some((a1, a2)), None) => Ok(Value::complex(mul(a1, b)?, mul(a2, b)?)),
        (None, Some((b1, b2))) => Ok(Value::complex(mul(a, b1)?, mul(a, b2)?)),
        (None, None) => arith(a, b, ArithOp::Mul),
    }
}

pub(crate) fn div(a: &Value, b: &Value) -> EvalResult<Value> {
    if b.is_zero() {
        return Err(err(ErrorKind::DivisionByZero));
    }
    match (complex_parts(a), complex_parts(b)) {
        (Some((a1, a2)), Some((b1, b2))) => {
            // ((ac + bd) + (bc − ad)i) / (c² + d²)
            let ac = mul(a1, b1)?;
            let bd = mul(a2, b2)?;
            let bc = mul(a2, b1)?;
            let ad = mul(a1, b2)?;
            let den = add(&mul(b1, b1)?, &mul(b2, b2)?)?;
            Ok(Value::complex(
                add(&ac, &bd)?.div_value(&den)?,
                sub(&bc, &ad)?.div_value(&den)?,
            ))
        }
        (Some((a1, a2)), None) => Ok(Value::complex(div(a1, b)?, div(a2, b)?)),
        (None, Some((b1, b2))) => {
            let den = add(&mul(b1, b1)?, &mul(b2, b2)?)?;
            let re = mul(a, b1)?.div_value(&den)?;
            let im = mul(a, b2)?.neg_value()?.div_value(&den)?;
            Ok(Value::complex(re, im))
        }
        (None, None) => arith(a, b, ArithOp::Div),
    }
}

#[derive(Clone, Copy)]
enum ArithOp {
    Add,
    Sub,
    Mul,
    Div,
}

fn arith(a: &Value, b: &Value, op: ArithOp) -> EvalResult<Value> {
    match (&a.inner, &b.inner) {
        (Inner::Exact(x), Inner::Exact(y)) => {
            let r = match op {
                ArithOp::Add => x + y,
                ArithOp::Sub => x - y,
                ArithOp::Mul => x * y,
                ArithOp::Div => {
                    if y.is_zero() {
                        return Err(err(ErrorKind::DivisionByZero));
                    }
                    x / y
                }
            };
            Ok(Value::exact(r))
        }
        _ => {
            let x = af_at(a, WORK_P)?;
            let y = af_at(b, WORK_P)?;
            let f = match op {
                ArithOp::Add => x.add(&y, WORK_P, RM),
                ArithOp::Sub => x.sub(&y, WORK_P, RM),
                ArithOp::Mul => x.mul(&y, WORK_P, RM),
                ArithOp::Div => x.div(&y, WORK_P, RM),
            }
            .map_err(af_err)?;
            Ok(approx(f))
        }
    }
}

/// Floor-modulo (`mod`): result has the sign of the divisor.
pub(crate) fn floor_mod(a: &Value, b: &Value) -> EvalResult<Value> {
    if b.is_zero() {
        return Err(err(ErrorKind::DivisionByZero));
    }
    if complex_parts(a).is_some() || complex_parts(b).is_some() {
        return Err(err(ErrorKind::OutOfDomain));
    }
    let x = a.to_ratio().ok_or_else(|| err(ErrorKind::OutOfDomain))?;
    let y = b.to_ratio().ok_or_else(|| err(ErrorKind::OutOfDomain))?;
    // a − b·floor(a/b)
    let q = (&x / &y).floor();
    let r = &x - &y * q;
    let inexact = !matches!((&a.inner, &b.inner), (Inner::Exact(_), Inner::Exact(_)));
    Ok(exact_or_approx(r, inexact))
}

// ---------------------------------------------------------------------------
// power
// ---------------------------------------------------------------------------

/// `base ^ exp` with full promotion (exact for exact base + integer exp).
pub(crate) fn pow(base: &Value, exp: &Value) -> EvalResult<Value> {
    // x^0 = 1 (including 0^0 — calculator convention)
    if exp.is_zero() {
        return Ok(Value::exact_i64(1));
    }
    // 0^x
    if base.is_zero() {
        if let Some(r) = exp.to_ratio() {
            if r.is_negative() {
                return Err(err(ErrorKind::DivisionByZero));
            }
            return Ok(Value::exact_i64(0)); // 0^positive → 0
        }
        return Err(err(ErrorKind::OutOfDomain)); // 0^complex
    }

    // Integer exponent fast paths.
    if let Some(n_big) = exp.as_integral() {
        return pow_integral(base, &n_big);
    }

    // General exponent: exp(y·ln z) (complex when z is negative/complex).
    match complex_parts(base) {
        Some(_) => {
            let l = match complex_parts(base) {
                Some((bre, bim)) => c_ln(bre, bim)?,
                None => unreachable!("matched complex above"),
            }; // complex Value
            let e = match complex_parts(exp) {
                Some(_) => exp.clone(),
                None => Value::complex(exp.clone(), Value::exact_i64(0)),
            };
            let p = mul(&e, &l)?;
            let (pre, pim) = p.real_and_imaginary();
            c_exp(&pre, &pim)
        }
        None => {
            let r = base.to_ratio().ok_or_else(|| err(ErrorKind::OutOfDomain))?;
            if r.is_negative() {
                // (−x)^y = exp(y·(ln x + iπ))
                let neg = promote_arg(base)?.neg().map_err(af_err)?;
                let l = real_ln_positive(&Value::approx(neg))?;
                let pi_v = Value::approx(pi(WORK_P)?);
                let e = Value::complex(exp.clone(), Value::exact_i64(0));
                let l = Value::complex(l, pi_v);
                let p = mul(&e, &l)?;
                let (pre, pim) = p.real_and_imaginary();
                c_exp(&pre, &pim)
            } else {
                // exp(y·ln x), real
                let l = real_ln_positive(base)?;
                exp_v(&mul(&exp.clone(), &l)?)
            }
        }
    }
}

fn pow_integral(base: &Value, n: &BigInt) -> EvalResult<Value> {
    use num_traits::ToPrimitive;
    let neg = n.is_negative();
    let m = n.abs();
    // Exponent sanity caps (2^1024 must work; 2^(2^40) must not).
    let Some(mi) = m.to_i64() else {
        return Err(err(ErrorKind::OutOfDomain));
    };
    if mi > MAX_INT_OP {
        return Err(err(ErrorKind::OutOfDomain));
    }
    match &base.inner {
        Inner::Exact(r) => {
            // Estimate result size: base_bits × exponent ≤ 8 Mbit.
            let bits_cap: u64 = 8_000_000;
            if r.numer().bits().saturating_mul(m.bits()) > bits_cap
                || r.denom().bits().saturating_mul(m.bits()) > bits_cap
            {
                return Err(err(ErrorKind::OutOfDomain));
            }
            let Some(signed) = n.to_i32() else {
                return Err(err(ErrorKind::OutOfDomain));
            };
            // num-rational handles negative exponents via reciprocal
            Ok(Value::exact(r.pow(signed)))
        }
        Inner::Approx(f) => {
            let p = if neg {
                f.reciprocal(WORK_P, RM).map_err(af_err)?
            } else {
                f.clone().map_err(af_err)?
            };
            let out = p.powi(mi as u32 as usize, WORK_P, RM).map_err(af_err)?;
            Ok(approx(out))
        }
        Inner::Complex(b) => {
            if mi <= 65_536 {
                // Repeated multiplication keeps exact parts exact: (1+i)² = 2i.
                let mut acc = Value::complex(Value::exact_i64(1), Value::exact_i64(0));
                for _ in 0..mi {
                    acc = mul(&acc, base)?;
                }
                let out = if neg {
                    div(&Value::exact_i64(1), &acc)?
                } else {
                    acc
                };
                Ok(out)
            } else {
                // exp(n·ln z)
                let l = c_ln(&b.0, &b.1)?;
                let nval = Value::complex(
                    Value::exact_int(if neg { -m } else { m }),
                    Value::exact_i64(0),
                );
                let p = mul(&nval, &l)?;
                let (pre, pim) = p.real_and_imaginary();
                c_exp(&pre, &pim)
            }
        }
    }
}

/// Square root: exact for exact perfect squares of non-negative rationals;
/// complex principal branch for negatives.
pub(crate) fn sqrt_v(v: &Value) -> EvalResult<Value> {
    match complex_parts(v) {
        Some((re, im)) => c_sqrt(re, im),
        None => {
            let r = v.to_ratio().ok_or_else(|| err(ErrorKind::OutOfDomain))?;
            if r.is_negative() {
                // sqrt(−x) = sqrt(x)·i
                let neg = promote_arg(v)?.neg().map_err(af_err)?;
                let s = real_sqrt_positive(&Value::approx(neg))?;
                let i = Value::complex(Value::exact_i64(0), Value::exact_i64(1));
                return mul(&s, &i);
            }
            if r.is_integer() {
                let n = r.to_integer();
                if is_perfect_square(&n) {
                    return Ok(Value::exact_int(isqrt(&n)));
                }
            } else {
                let (num, den) = (r.numer(), r.denom());
                if is_perfect_square(num) && is_perfect_square(den) {
                    return Ok(Value::exact(BigRational::new(isqrt(num), isqrt(den))));
                }
            }
            real_sqrt_positive(v)
        }
    }
}

fn real_sqrt_positive(v: &Value) -> EvalResult<Value> {
    let s = promote_arg(v)?.sqrt(WORK_P + GUARD_P, RM).map_err(af_err)?;
    Ok(approx(with_guard(s)))
}

// ---------------------------------------------------------------------------
// rounding family (real values only; doc 04 §3)
// ---------------------------------------------------------------------------

pub(crate) fn floor_v(v: &Value) -> EvalResult<Value> {
    rounding(v, |r| r.floor())
}

pub(crate) fn ceil_v(v: &Value) -> EvalResult<Value> {
    rounding(v, |r| r.ceil())
}

pub(crate) fn trunc_v(v: &Value) -> EvalResult<Value> {
    rounding(v, |r| r.trunc())
}

pub(crate) fn fract_v(v: &Value) -> EvalResult<Value> {
    rounding(v, |r| r.fract())
}

fn rounding(v: &Value, f: impl Fn(&BigRational) -> BigRational) -> EvalResult<Value> {
    if complex_parts(v).is_some() {
        return Err(err(ErrorKind::OutOfDomain));
    }
    let r = v.to_ratio().ok_or_else(|| err(ErrorKind::OutOfDomain))?;
    let inexact = !matches!(v.inner, Inner::Exact(_));
    Ok(exact_or_approx(f(&r), inexact))
}

/// round(x): nearest integer, halves away from zero (doc 04 §3).
pub(crate) fn round_v(v: &Value) -> EvalResult<Value> {
    if complex_parts(v).is_some() {
        return Err(err(ErrorKind::OutOfDomain));
    }
    let r = v.to_ratio().ok_or_else(|| err(ErrorKind::OutOfDomain))?;
    let n = crate::num::round_scaled(&r.abs(), 0);
    let n = if r.is_negative() { -n } else { n };
    let inexact = !matches!(v.inner, Inner::Exact(_));
    Ok(exact_or_approx(BigRational::from_integer(n), inexact))
}

// ---------------------------------------------------------------------------
// misc
// ---------------------------------------------------------------------------

/// Absolute value; complex → magnitude √(re² + im²).
pub(crate) fn abs_v(v: &Value) -> EvalResult<Value> {
    match complex_parts(v) {
        Some((re, im)) => {
            let re2 = mul(re, re)?;
            let im2 = mul(im, im)?;
            sqrt_v(&add(&re2, &im2)?)
        }
        None => {
            let r = v.to_ratio().ok_or_else(|| err(ErrorKind::OutOfDomain))?;
            if matches!(v.inner, Inner::Exact(_)) {
                Ok(Value::exact(r.abs()))
            } else {
                let f = Value::af_from_rational(&r.abs(), WORK_P)?;
                Ok(Value::approx(f))
            }
        }
    }
}

impl Value {
    pub(crate) fn neg_value(&self) -> EvalResult<Value> {
        match &self.inner {
            Inner::Exact(r) => Ok(Value::exact(-r.clone())),
            Inner::Approx(f) => {
                let n = f.neg().map_err(af_err)?;
                Ok(Value::approx(n))
            }
            Inner::Complex(p) => Ok(Value::complex(p.0.neg_value()?, p.1.neg_value()?)),
        }
    }

    pub(crate) fn div_value(&self, other: &Value) -> EvalResult<Value> {
        div(self, other)
    }

    pub(crate) fn mul_value(&self, other: &Value) -> EvalResult<Value> {
        mul(self, other)
    }

    pub(crate) fn sub_value(&self, other: &Value) -> EvalResult<Value> {
        sub(self, other)
    }

}

// ---------------------------------------------------------------------------
// float shims shared with functions.rs
// ---------------------------------------------------------------------------

pub(crate) fn af_at_guard(v: &Value) -> EvalResult<Af> {
    promote_arg(v)
}

pub(crate) fn af_add(a: &Af, b: &Af) -> EvalResult<Af> {
    a.add(b, WORK_P + GUARD_P, RM).map_err(af_err)
}

pub(crate) fn af_sub(a: &Af, b: &Af) -> EvalResult<Af> {
    a.sub(b, WORK_P + GUARD_P, RM).map_err(af_err)
}

pub(crate) fn af_neg(a: &Af) -> EvalResult<Af> {
    a.neg().map_err(af_err)
}

pub(crate) fn af_exp_pub(a: &Af) -> EvalResult<Af> {
    af_exp(a)
}

pub(crate) fn with_guard_pub(a: Af) -> Af {
    with_guard(a)
}

pub(crate) fn af_atan2_guarded(im: &Af, re: &Af) -> EvalResult<Af> {
    Ok(with_guard(af_atan2(im, re)?))
}
