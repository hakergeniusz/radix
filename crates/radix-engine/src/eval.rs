//! Evaluation environment and the AST evaluator.

use std::collections::HashMap;

use crate::error::{ErrorKind, EvalError, EvalResult};
use crate::functions;
use crate::num::Value;
use crate::ops;
use crate::parser::{AstKind, Node, Parser};

/// Angle unit for trig/inverse-trig (doc 04 §4). Hyperbolics are immune.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AngleUnit {
    Degrees,
    Radians,
    Gradians,
}

/// Word size for the bitwise domain (doc 04 §6): bitwise operands are
/// masked into [0, 2^W) and results stay there; plain arithmetic is
/// unbounded (`255+1 = 256` at 8 bits, T044).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordSize {
    W8,
    W16,
    W32,
    W64,
}

impl WordSize {
    /// Width in bits.
    pub fn bits(self) -> u32 {
        match self {
            WordSize::W8 => 8,
            WordSize::W16 => 16,
            WordSize::W32 => 32,
            WordSize::W64 => 64,
        }
    }
}

/// Number base for entry parsing and (via the formatter) base re-rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Base {
    Bin,
    Oct,
    Dec,
    Hex,
}

/// Evaluation environment: mode settings, variables, last result.
#[derive(Debug, Clone)]
pub struct Env {
    /// Trig angle unit (default Degrees).
    pub angle_unit: AngleUnit,
    /// Bitwise word size (default 64-bit).
    pub word_size: WordSize,
    /// Entry number base (default Decimal).
    pub base: Base,
    /// User variables (`x=5`), stored lowercase (names are case-insensitive).
    pub variables: HashMap<String, Value>,
    /// `_` — last successful result.
    pub last_result: Option<Value>,
}

impl Env {
    /// Defaults: Degrees, 64-bit words, Decimal base, no variables.
    pub fn new() -> Self {
        Self {
            angle_unit: AngleUnit::Degrees,
            word_size: WordSize::W64,
            base: Base::Dec,
            variables: HashMap::new(),
            last_result: None,
        }
    }
}

impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}

/// Evaluate an expression. On success the result is also stored as
/// `env.last_result` (the `_` variable). A single trailing `=` is treated
/// as an evaluation marker and stripped (`9=` evaluates `9`).
pub fn evaluate(expr: &str, env: &mut Env) -> Result<Value, EvalError> {
    let mut text = expr.trim();
    if let Some(stripped) = text.strip_suffix('=') {
        text = stripped.trim();
    }
    if text.is_empty() {
        return Err(EvalError::new(ErrorKind::ParseError));
    }
    let mut parser = Parser::new(text, env.base)?;
    let ast = parser.parse_entry()?;
    let value = eval_node(&ast, env)?;
    env.last_result = Some(value.clone());
    Ok(value)
}

/// Lenient single-value parse for Conversion-mode rows (doc 06 §7):
/// accepts optional sign, whitespace, and one number literal in `base`.
/// Returns `None` for anything unparseable (`-e.5` → None, T061 family).
pub fn parse_value_text(s: &str, base: Base) -> Option<Value> {
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    let (neg, rest) = match t.strip_prefix('-').or_else(|| t.strip_prefix('−')) {
        Some(r) => (true, r.trim()),
        None => (false, t),
    };
    let (int_part, frac_part) = match rest.split_once('.') {
        Some((a, b)) => (a, b),
        None => (rest, ""),
    };
    if int_part.is_empty() && frac_part.is_empty() {
        return None;
    }
    let valid = |c: char| match base {
        Base::Bin => ('0'..='1').contains(&c),
        Base::Oct => ('0'..='7').contains(&c),
        Base::Dec => c.is_ascii_digit(),
        Base::Hex => c.is_ascii_hexdigit(),
    };
    if !int_part.chars().all(valid) || !frac_part.chars().all(valid) {
        return None;
    }
    let radix = match base {
        Base::Bin => 2,
        Base::Oct => 8,
        Base::Dec => 10,
        Base::Hex => 16,
    };
    use num_bigint::BigInt;
    use num_rational::BigRational;
    let mut num = if int_part.is_empty() {
        BigInt::from(0)
    } else {
        BigInt::parse_bytes(int_part.as_bytes(), radix)?
    };
    if neg {
        num = -num;
    }
    if frac_part.is_empty() {
        return Some(Value::exact_int(num));
    }
    let frac = BigInt::parse_bytes(frac_part.as_bytes(), radix)?;
    let den = BigInt::from(radix).pow(frac_part.len() as u32);
    let signed_frac = if neg { -frac } else { frac };
    Some(Value::exact(BigRational::new(num * &den + signed_frac, den)))
}

// ---------------------------------------------------------------------------
// AST evaluation
// ---------------------------------------------------------------------------

fn eval_node(node: &Node, env: &mut Env) -> EvalResult<Value> {
    match &node.kind {
        AstKind::Num(v) => Ok(v.clone()),
        AstKind::Const(key) => functions::constant_value(key),
        AstKind::Var(name) => env
            .variables
            .get(name)
            .cloned()
            .ok_or_else(|| EvalError::with_span(ErrorKind::UndefinedVariable(name.clone()), node.span.clone())),
        AstKind::Last => env
            .last_result
            .clone()
            .ok_or_else(|| EvalError::with_span(ErrorKind::UndefinedVariable("_".into()), node.span.clone())),
        AstKind::Rand => Ok(Value::random_unit()),
        AstKind::Imag => Ok(Value::complex(Value::exact_i64(0), Value::exact_i64(1))),
        AstKind::Neg(x) => eval_node(x, env)?.neg_value(),
        AstKind::Add(l, r) => {
            let lv = eval_node(l, env)?;
            if let AstKind::Percent(inner) = &r.kind {
                // relative: a + a·(b/100) (doc 03 §4)
                let pv = eval_node(inner, env)?;
                let rel = lv.mul_value(&pv)?.div_value(&Value::exact_i64(100))?;
                ops::add(&lv, &rel)
            } else {
                let rv = eval_node(r, env)?;
                ops::add(&lv, &rv)
            }
        }
        AstKind::Sub(l, r) => {
            let lv = eval_node(l, env)?;
            if let AstKind::Percent(inner) = &r.kind {
                // relative: a − a·(b/100)
                let pv = eval_node(inner, env)?;
                let rel = lv.mul_value(&pv)?.div_value(&Value::exact_i64(100))?;
                ops::sub(&lv, &rel)
            } else {
                let rv = eval_node(r, env)?;
                ops::sub(&lv, &rv)
            }
        }
        AstKind::Mul(l, r) => {
            let (lv, rv) = (eval_node(l, env)?, eval_node(r, env)?);
            ops::mul(&lv, &rv)
        }
        AstKind::Div(l, r) => {
            let (lv, rv) = (eval_node(l, env)?, eval_node(r, env)?);
            ops::div(&lv, &rv).map_err(|mut e| {
                // highlight the divisor token (doc 10 §2)
                if e.kind == ErrorKind::DivisionByZero {
                    e.span = Some(r.span.clone());
                }
                e
            })
        }
        AstKind::Mod(l, r) => {
            let (lv, rv) = (eval_node(l, env)?, eval_node(r, env)?);
            ops::floor_mod(&lv, &rv)
        }
        AstKind::Pow(l, r) => {
            let (lv, rv) = (eval_node(l, env)?, eval_node(r, env)?);
            ops::pow(&lv, &rv)
        }
        AstKind::Fact(x) => {
            let v = eval_node(x, env)?;
            functions::factorial(&v)
        }
        AstKind::Percent(inner) => {
            let v = eval_node(inner, env)?;
            v.div_value(&Value::exact_i64(100))
        }
        AstKind::Sup(x, n) => {
            let v = eval_node(x, env)?;
            ops::pow(&v, &Value::exact_i64(i64::from(*n)))
        }
        AstKind::Call { name, args } => {
            let mut values = Vec::with_capacity(args.len());
            let mut spans = Vec::with_capacity(args.len());
            for a in args {
                values.push(eval_node(a, env)?);
                spans.push(a.span.clone());
            }
            functions::call(name, &values, &spans, env).map_err(|mut e| {
                // point arity/unknown-function errors at the function name
                if e.span.is_none()
                    && matches!(
                        e.kind,
                        ErrorKind::WrongArity { .. } | ErrorKind::UndefinedFunction(_)
                    )
                {
                    e.span = Some(node.span.clone());
                }
                e
            })
        }
        AstKind::Abs(x) => {
            let v = eval_node(x, env)?;
            ops::abs_v(&v)
        }
        AstKind::Round(x) => {
            let v = eval_node(x, env)?;
            ops::round_v(&v)
        }
        AstKind::Not(x) => {
            let v = eval_node(x, env)?;
            functions::bitwise_not(&v, env.word_size)
        }
        AstKind::Twos(x) => {
            let v = eval_node(x, env)?;
            functions::twos_complement(&v, env.word_size)
        }
        AstKind::Bswap(x) => {
            let v = eval_node(x, env)?;
            functions::byte_swap(&v, env.word_size)
        }
        AstKind::SqrtP(x) => {
            let v = eval_node(x, env)?;
            ops::sqrt_v(&v)
        }
        AstKind::And(l, r) | AstKind::Or(l, r) | AstKind::Xor(l, r) => {
            let (lv, rv) = (eval_node(l, env)?, eval_node(r, env)?);
            let op = match &node.kind {
                AstKind::And(_, _) => functions::LogicOp::And,
                AstKind::Or(_, _) => functions::LogicOp::Or,
                _ => functions::LogicOp::Xor,
            };
            functions::bitwise_logic(&lv, &rv, env.word_size, op)
        }
        AstKind::Shl(l, r) => {
            let (lv, rv) = (eval_node(l, env)?, eval_node(r, env)?);
            functions::shift(&lv, &rv, true, env.word_size)
        }
        AstKind::Shr(l, r) => {
            let (lv, rv) = (eval_node(l, env)?, eval_node(r, env)?);
            functions::shift(&lv, &rv, false, env.word_size)
        }
        AstKind::Assign { name, value } => {
            let v = eval_node(value, env)?;
            env.variables.insert(name.clone(), v.clone());
            Ok(v)
        }
    }
}
