//! Error types for the Radix engine (doc 10).
//!
//! [`EvalError`] carries a byte span into the original input string so the
//! UI can highlight the offending token (doc 10 §1).

use std::ops::Range;

/// Classification of an evaluation failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    /// Malformed expression (tokenizer/parser level).
    ParseError,
    /// Division (or modulo / zero power) by zero.
    DivisionByZero,
    /// Tangent at one of its poles (`tan(90)` in Degrees, `tan(pi/2)`, ...).
    TangentUndefined,
    /// Reference to a name that is neither a constant, a function nor a
    /// defined variable. Carries the name as written (lowercased).
    UndefinedVariable(String),
    /// Call of an unknown function name.
    UndefinedFunction(String),
    /// Known function called with the wrong number of arguments.
    /// `expected` is a human-readable description such as `"1"`, `"2"` or
    /// `"at least 2"`.
    WrongArity {
        /// Function name (lowercased).
        name: String,
        /// Human-readable arity description.
        expected: &'static str,
    },
    /// Argument outside the function's domain (negative factorial, log of
    /// zero, non-integer bitwise operand, ...).
    OutOfDomain,
    /// Fractional, complex or otherwise unusable operand for a bitwise
    /// operation.
    InvalidBitwiseOperand,
}

/// An evaluation failure with an optional byte span of the offending token
/// in the original input (doc 10 §2: divisor of `1/0`, the `90` of
/// `tan(90)`, the undefined variable name, the offending parse token).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvalError {
    /// Error classification.
    pub kind: ErrorKind,
    /// Byte range of the offending token in the original input, when known.
    pub span: Option<Range<usize>>,
}

impl EvalError {
    /// Convenience constructor without a span.
    pub fn new(kind: ErrorKind) -> Self {
        Self { kind, span: None }
    }

    /// Convenience constructor with a span.
    pub fn with_span(kind: ErrorKind, span: Range<usize>) -> Self {
        Self {
            kind,
            span: Some(span),
        }
    }
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ErrorKind::ParseError => write!(f, "Malformed expression"),
            ErrorKind::DivisionByZero => write!(f, "Division by zero is undefined"),
            ErrorKind::TangentUndefined => write!(
                f,
                "Tangent is undefined for angles that are multiples of π (180°) from π/2 (90°)"
            ),
            ErrorKind::UndefinedVariable(name) => write!(f, "Undefined variable: {name}"),
            ErrorKind::UndefinedFunction(name) => write!(f, "Undefined function: {name}"),
            ErrorKind::WrongArity { name, expected } => {
                write!(f, "Wrong number of arguments for {name}: expected {expected}")
            }
            ErrorKind::OutOfDomain => write!(f, "Argument outside function domain"),
            ErrorKind::InvalidBitwiseOperand => {
                write!(f, "Operand is not usable for a bitwise operation")
            }
        }
    }
}

impl std::error::Error for EvalError {}

pub(crate) type EvalResult<T> = Result<T, EvalError>;
