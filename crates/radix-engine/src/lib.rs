//! Radix engine — expression parsing, arbitrary-precision evaluation and
//! display formatting for the Radix desktop calculator.
//!
//! Clean-room implementation from the behavioral specification in `docs/`
//! (00–15). The behavioural source of truth is the test catalog (doc 12,
//! T001–T055) plus the arithmetic (doc 04), grammar (doc 03), formatting
//! (doc 07) and error (doc 10) contracts.
//!
//! # Layout
//!
//! * [`num`] — the opaque [`Value`]: exact rational / high-precision /
//!   complex states, serialization.
//! * [`ops`] — arithmetic with exactness promotion, transcendental
//!   primitives, rounding family, complex primitives.
//! * [`parser`] — tokenizer + tiered parser with byte spans.
//! * [`eval`] — [`Env`], [`evaluate`], [`parse_value_text`].
//! * [`functions`] — built-ins, constants, bitwise word ops, catalogs.
//! * [`format`] — [`FormatPrefs`], the four result formats, base rendering.
//! * [`echo`] — history-row echo normalization.
//!
//! # Numeric model
//!
//! `+ − × ÷(non-zero) %`, integer-exponent `^` and integer `!` stay exact
//! rationals; transcendental/irrational steps promote to ~200-bit binary
//! floats (astro-float, pure Rust). Display rounding is exact (BigInt,
//! half away from zero) so 9-decimal vectors match exactly. `sqrt(-1) = i`
//! with an exactly-1 imaginary part.

pub mod echo;
pub mod error;
pub mod eval;
pub mod format;
pub mod functions;
pub mod num;
mod ops;
pub mod parser;

pub use echo::normalize_echo;
pub use error::{ErrorKind, EvalError};
pub use eval::{evaluate, parse_value_text, AngleUnit, Base, Env, WordSize};
pub use format::{format_value, render_in_base, FormatPrefs, ResultFormat};
pub use functions::{
    constant_catalog, function_catalog, ConstantCategory, ConstantInfo, FunctionCategory,
    FunctionInfo,
};
pub use num::Value;
