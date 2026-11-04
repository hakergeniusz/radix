//! Tokenizer + parser for the Radix expression grammar (doc 03).
//!
//! Precedence (tightest → loosest), derived from the vectors:
//!
//! 1. postfix `!` and superscript powers (`5³`, `4⁻¹`)
//! 2. power `^` (right-assoc; binds tighter than `%`: `10%^2 = 1`, T014)
//! 3. postfix `%`
//! 4. prefix `−` / `¬` / `√` / `twos` / `bswap`
//! 5. `× ÷ mod` (left-assoc)
//! 6. `+ −` (left-assoc; the relative-percent rule applies here, doc 03 §4)
//! 7. `<< >>` shifts
//! 8. `∧ ∨ ⊻` bitwise (loosest; placement unobserved — see NOTES.md)
//!
//! The percent-relative rule is decided at eval time from the AST shape: an
//! `Add`/`Sub` whose right child is exactly a percent-postfix node computes
//! `a ± a·(b/100)`.

use crate::error::{ErrorKind, EvalError, EvalResult};
use crate::eval::Base;
use crate::functions::{ascii_consts, function_exists, glyph_consts, typed_consts};

// ---------------------------------------------------------------------------
// tokens
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TokKind {
    /// Digit run with at most one radix point, valid for the active base.
    Num(String),
    /// ASCII identifier ([A-Za-z_][A-Za-z0-9_]*), original case preserved
    /// (`G` and `g` are distinct constant glyphs).
    Ident(String),
    /// Constant glyph — resolution key.
    Glyph(&'static str),
    Plus,
    Minus,
    Mul,
    Div,
    Pow,
    Fac,
    Percent,
    LParen,
    RParen,
    LBrack,
    RBrack,
    VBar,
    Semi,
    Eq,
    And,
    Or,
    Xor,
    Not,
    Shl,
    Shr,
    Sqrt,
    /// Superscript run value (`³` → 3, `⁻¹` → −1).
    Sup(i32),
    /// `_` — last result.
    Last,
}

#[derive(Debug, Clone)]
pub(crate) struct Tok {
    pub kind: TokKind,
    pub span: std::ops::Range<usize>,
}

fn superscript_value(c: char) -> Option<u32> {
    match c {
        '⁰' => Some(0),
        '¹' => Some(1),
        '²' => Some(2),
        '³' => Some(3),
        '⁴' => Some(4),
        '⁵' => Some(5),
        '⁶' => Some(6),
        '⁷' => Some(7),
        '⁸' => Some(8),
        '⁹' => Some(9),
        _ => None,
    }
}

fn is_subscript(c: char) -> bool {
    ('₀'..='₉').contains(&c)
}

fn is_base_digit(c: char, base: Base) -> bool {
    match base {
        Base::Bin => ('0'..='1').contains(&c),
        Base::Oct => ('0'..='7').contains(&c),
        Base::Dec => c.is_ascii_digit(),
        Base::Hex => c.is_ascii_hexdigit(),
    }
}

/// Multi-character constant glyphs (longest-prefix match happens before
/// identifier scanning; `Z₀`'s subscript must not be read as an error run).
const MULTI_GLYPHS: &[(&str, &str)] = &[
    ("mₑ", "me"),
    ("mₚ", "mp"),
    ("mᵤ", "mu"),
    ("μ₀", "mu0"),
    ("ε₀", "eps0"),
    ("Z₀", "z0"),
];

fn tokenize(src: &str, base: Base) -> EvalResult<Vec<Tok>> {
    let chars: Vec<(usize, char)> = src.char_indices().collect();
    let n = chars.len();
    let mut i = 0usize;
    let mut out = Vec::new();

    while i < n {
        let (pos, c) = chars[i];

        if c.is_whitespace() {
            i += 1;
            continue;
        }

        // multi-char constant glyphs
        let rest = &src[pos..];
        if let Some((glyph, key)) = MULTI_GLYPHS.iter().find(|(g, _)| rest.starts_with(g)) {
            out.push(Tok {
                kind: TokKind::Glyph(key),
                span: pos..pos + glyph.len(),
            });
            i += glyph.chars().count();
            continue;
        }

        // single-char constant glyphs
        let single: Option<&'static str> = match c {
            'π' => Some("pi"),
            'τ' => Some("tau"),
            'φ' => Some("phi"),
            'α' => Some("alpha"),
            'σ' => Some("sigma"),
            _ => None,
        };
        if let Some(key) = single {
            out.push(Tok {
                kind: TokKind::Glyph(key),
                span: pos..pos + c.len_utf8(),
            });
            i += 1;
            continue;
        }

        // operators & punctuation
        let simple: Option<(TokKind, usize)> = match c {
            '+' => Some((TokKind::Plus, 1)),
            '-' | '−' => Some((TokKind::Minus, 1)),
            '*' | '×' => Some((TokKind::Mul, 1)),
            '/' | '÷' => Some((TokKind::Div, 1)),
            '^' => Some((TokKind::Pow, 1)),
            '!' => Some((TokKind::Fac, 1)),
            '%' => Some((TokKind::Percent, 1)),
            '(' => Some((TokKind::LParen, 1)),
            ')' => Some((TokKind::RParen, 1)),
            '[' => Some((TokKind::LBrack, 1)),
            ']' => Some((TokKind::RBrack, 1)),
            '|' => Some((TokKind::VBar, 1)),
            ';' => Some((TokKind::Semi, 1)),
            '=' => Some((TokKind::Eq, 1)),
            '∧' => Some((TokKind::And, 1)),
            '∨' => Some((TokKind::Or, 1)),
            '⊻' => Some((TokKind::Xor, 1)),
            '¬' => Some((TokKind::Not, 1)),
            '√' => Some((TokKind::Sqrt, 1)),
            '_' => Some((TokKind::Last, 1)),
            '<' if i + 1 < n && chars[i + 1].1 == '<' => Some((TokKind::Shl, 2)),
            '>' if i + 1 < n && chars[i + 1].1 == '>' => Some((TokKind::Shr, 2)),
            _ => None,
        };
        if let Some((kind, len)) = simple {
            out.push(Tok {
                kind,
                span: pos..pos + len,
            });
            i += len;
            continue;
        }

        out.push(scan_word(&chars, &mut i, base)?);
    }
    Ok(out)
}

/// Scan a maximal ASCII alphanumeric word starting at `chars[i]` and
/// classify it as a number (all chars valid digits in `base`), an
/// identifier, or an error (E-notation, stray subscripts, garbage).
/// Scan a number/identifier starting at `chars[i]`.
///
/// Classification rules:
/// * a maximal digit run (with optional `.` fraction) valid for `base` is a
///   number; digits immediately followed by letters split (`65bswap` →
///   number + identifier, which then fails as an adjacent token);
/// * decimal E-notation (`1E5`, `1.5e3`) is rejected (doc 03 §2);
/// * a leading `.` starts a fractional literal (`.5` = 0.5);
/// * subscript runs outside glyphs error (doc 03 §2);
/// * superscript runs become postfix power tokens (`5³`, `4⁻¹`);
/// * everything else must start with a letter/underscore → identifier
///   (original case preserved; `G` vs `g` are distinct constants).
fn scan_word(chars: &[(usize, char)], i: &mut usize, base: Base) -> EvalResult<Tok> {
    let n = chars.len();
    let (start, first) = chars[*i];
    let malformed = |span| EvalError::with_span(ErrorKind::ParseError, span);

    // Subscript run outside a recognized glyph → parse error (doc 03 §2).
    if is_subscript(first) {
        return Err(malformed(start..start + first.len_utf8()));
    }

    // Superscript run → postfix power token.
    if superscript_value(first).is_some() || first == '⁻' {
        let mut digits = String::new();
        let mut j = *i;
        if first == '⁻' {
            digits.push('-');
            j += 1;
        }
        let mut any = false;
        while j < n {
            match superscript_value(chars[j].1) {
                Some(d) => {
                    digits.push(char::from_u32(b'0' as u32 + d).unwrap_or('0'));
                    any = true;
                    j += 1;
                }
                None => break,
            }
        }
        let sup_end = if j > *i {
            chars[j - 1].0 + chars[j - 1].1.len_utf8()
        } else {
            start + first.len_utf8()
        };
        *i = j;
        if !any {
            return Err(malformed(start..sup_end)); // bare '⁻'
        }
        let value: i64 = digits.parse().unwrap_or(i64::from(i32::MAX));
        if value > 1_000_000 {
            return Err(malformed(start..sup_end));
        }
        #[allow(clippy::cast_possible_truncation)]
        return Ok(Tok {
            kind: TokKind::Sup(value as i32),
            span: start..sup_end,
        });
    }

    // Leading '.' → fractional literal (".5" = 0.5, harmless superset).
    if first == '.' {
        let mut k = *i + 1;
        let mut frac = String::new();
        while k < n && is_base_digit(chars[k].1, base) {
            frac.push(chars[k].1);
            k += 1;
        }
        let end = if k > *i + 1 {
            chars[k - 1].0 + chars[k - 1].1.len_utf8()
        } else {
            start + 1
        };
        *i = k;
        if frac.is_empty() {
            return Err(malformed(start..end));
        }
        return Ok(Tok {
            kind: TokKind::Num(format!("0.{frac}")),
            span: start..end,
        });
    }

    // Number: leading base digit (in Hex, hex letters a-f start numbers —
    // hex digits take priority over identifier reading).
    if is_base_digit(first, base) {
        let mut j = *i;
        let mut text = String::new();
        while j < n && is_base_digit(chars[j].1, base) {
            text.push(chars[j].1);
            j += 1;
        }
        // optional fractional part
        if j < n && chars[j].1 == '.' {
            let mut k = j + 1;
            let mut frac = String::new();
            while k < n && is_base_digit(chars[k].1, base) {
                frac.push(chars[k].1);
                k += 1;
            }
            if !frac.is_empty() {
                text.push('.');
                text.push_str(&frac);
                j = k;
            } else {
                // bare trailing '.' — accept as part of the number ("5." = 5)
                text.push('.');
                j += 1;
            }
        }
        let end = chars[j - 1].0 + chars[j - 1].1.len_utf8();
        // E-notation rejection (Dec): digits [eE][+-]?digits (doc 03 §2).
        if base == Base::Dec && j < n && matches!(chars[j].1, 'e' | 'E') {
            let mut k = j + 1;
            if k < n && matches!(chars[k].1, '+' | '-') {
                k += 1;
            }
            if k < n && chars[k].1.is_ascii_digit() {
                let mut exp_end = k;
                while exp_end < n && chars[exp_end].1.is_ascii_digit() {
                    exp_end += 1;
                }
                return Err(malformed(
                    start..chars[exp_end - 1].0 + chars[exp_end - 1].1.len_utf8(),
                ));
            }
        }
        *i = j;
        return Ok(Tok {
            kind: TokKind::Num(text),
            span: start..end,
        });
    }

    // Identifier: maximal ASCII alphanumeric run starting with a letter/`_`.
    let mut j = *i;
    let mut word = String::new();
    while j < n && chars[j].1.is_ascii_alphanumeric() {
        word.push(chars[j].1);
        j += 1;
    }
    let end = if j > *i {
        chars[j - 1].0 + chars[j - 1].1.len_utf8()
    } else {
        start + first.len_utf8()
    };
    *i = j;
    if word.is_empty() {
        return Err(malformed(start..end));
    }
    Ok(Tok {
        kind: TokKind::Ident(word),
        span: start..end,
    })
}

// ---------------------------------------------------------------------------
// AST
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub(crate) struct Node {
    pub kind: AstKind,
    pub span: std::ops::Range<usize>,
}

#[derive(Debug, Clone)]
pub(crate) enum AstKind {
    /// Numeric literal (exact rational, parsed in the active base).
    Num(crate::num::Value),
    /// Constant glyph or typed name — resolution key.
    Const(&'static str),
    /// Identifier to resolve at eval time (variable).
    Var(String),
    /// `_` — last result.
    Last,
    /// `rand`.
    Rand,
    /// The imaginary unit `i`.
    Imag,
    Neg(Box<Node>),
    Add(Box<Node>, Box<Node>),
    Sub(Box<Node>, Box<Node>),
    Mul(Box<Node>, Box<Node>),
    Div(Box<Node>, Box<Node>),
    Mod(Box<Node>, Box<Node>),
    Pow(Box<Node>, Box<Node>),
    /// Postfix `!`.
    Fact(Box<Node>),
    /// Postfix `%`.
    Percent(Box<Node>),
    /// Superscript power (`5³`, `4⁻¹`).
    Sup(Box<Node>, i32),
    /// `name(args…)`; the node span covers the name.
    Call {
        name: String,
        args: Vec<Node>,
    },
    /// `|x|`.
    Abs(Box<Node>),
    /// `[x]` — round half away from zero (doc 03 §2).
    Round(Box<Node>),
    /// `¬x`.
    Not(Box<Node>),
    /// Prefix `twos x`.
    Twos(Box<Node>),
    /// Prefix `bswap x`.
    Bswap(Box<Node>),
    /// Prefix `√x`.
    SqrtP(Box<Node>),
    And(Box<Node>, Box<Node>),
    Or(Box<Node>, Box<Node>),
    Xor(Box<Node>, Box<Node>),
    Shl(Box<Node>, Box<Node>),
    Shr(Box<Node>, Box<Node>),
    /// `name = value` at top level.
    Assign {
        name: String,
        value: Box<Node>,
    },
}

// ---------------------------------------------------------------------------
// parser
// ---------------------------------------------------------------------------

pub(crate) struct Parser {
    toks: Vec<Tok>,
    pos: usize,
    base: Base,
    /// Nesting depth of brackets/calls — guards against stack overflow on
    /// adversarial input (`((((…`).
    depth: u32,
}

/// Maximum bracket/call nesting depth.
const MAX_DEPTH: u32 = 64;

impl Parser {
    pub(crate) fn new(src: &str, base: Base) -> EvalResult<Self> {
        Ok(Self {
            toks: tokenize(src, base)?,
            pos: 0,
            base,
            depth: 0,
        })
    }

    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos)
    }

    fn peek_kind(&self) -> Option<&TokKind> {
        self.toks.get(self.pos).map(|t| &t.kind)
    }

    fn peek2_kind(&self) -> Option<&TokKind> {
        self.toks.get(self.pos + 1).map(|t| &t.kind)
    }

    fn next(&mut self) -> Option<Tok> {
        let t = self.toks.get(self.pos).cloned();
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    fn err_here(&self) -> EvalError {
        match self.peek() {
            Some(t) => EvalError::with_span(ErrorKind::ParseError, t.span.clone()),
            None => EvalError::new(ErrorKind::ParseError),
        }
    }

    fn expect(&mut self, k: &TokKind) -> EvalResult<Tok> {
        match self.peek_kind() {
            Some(kk) if kk == k => Ok(self.next().expect("peeked")),
            _ => Err(self.err_here()),
        }
    }

    /// Parse a full entry; rejects trailing garbage; detects `name = value`.
    pub(crate) fn parse_entry(&mut self) -> EvalResult<Node> {
        if let (Some(TokKind::Ident(name)), Some(TokKind::Eq)) =
            (self.peek_kind().cloned(), self.peek2_kind().cloned())
        {
            if !name_is_reserved(&name) {
                let name_span = self.toks[self.pos].span.clone();
                self.next(); // name
                self.next(); // '='
                let value = self.parse_expr()?;
                if let Some(t) = self.peek() {
                    return Err(EvalError::with_span(ErrorKind::ParseError, t.span.clone()));
                }
                return Ok(Node {
                    kind: AstKind::Assign {
                        name: name.to_lowercase(),
                        value: Box::new(value),
                    },
                    span: name_span,
                });
            }
        }
        let node = self.parse_expr()?;
        if let Some(t) = self.peek() {
            return Err(EvalError::with_span(ErrorKind::ParseError, t.span.clone()));
        }
        Ok(node)
    }

    // tier 8: ∧ ∨ ⊻ (loosest; placement unobserved)
    fn parse_expr(&mut self) -> EvalResult<Node> {
        let mut lhs = self.parse_shift()?;
        loop {
            let op = match self.peek_kind() {
                Some(TokKind::And) => 0,
                Some(TokKind::Or) => 1,
                Some(TokKind::Xor) => 2,
                _ => break,
            };
            self.next();
            let rhs = self.parse_shift()?;
            let span = lhs.span.start..rhs.span.end;
            lhs = Node {
                kind: match op {
                    0 => AstKind::And(Box::new(lhs), Box::new(rhs)),
                    1 => AstKind::Or(Box::new(lhs), Box::new(rhs)),
                    _ => AstKind::Xor(Box::new(lhs), Box::new(rhs)),
                },
                span,
            };
        }
        Ok(lhs)
    }

    // tier 7: << >>
    fn parse_shift(&mut self) -> EvalResult<Node> {
        let mut lhs = self.parse_add()?;
        loop {
            let op = match self.peek_kind() {
                Some(TokKind::Shl) => 0,
                Some(TokKind::Shr) => 1,
                _ => break,
            };
            self.next();
            let rhs = self.parse_add()?;
            let span = lhs.span.start..rhs.span.end;
            lhs = Node {
                kind: if op == 0 {
                    AstKind::Shl(Box::new(lhs), Box::new(rhs))
                } else {
                    AstKind::Shr(Box::new(lhs), Box::new(rhs))
                },
                span,
            };
        }
        Ok(lhs)
    }

    // tier 6: + −
    fn parse_add(&mut self) -> EvalResult<Node> {
        let mut lhs = self.parse_mul()?;
        loop {
            let op = match self.peek_kind() {
                Some(TokKind::Plus) => 0,
                Some(TokKind::Minus) => 1,
                _ => break,
            };
            self.next();
            let rhs = self.parse_mul()?;
            let span = lhs.span.start..rhs.span.end;
            lhs = Node {
                kind: if op == 0 {
                    AstKind::Add(Box::new(lhs), Box::new(rhs))
                } else {
                    AstKind::Sub(Box::new(lhs), Box::new(rhs))
                },
                span,
            };
        }
        Ok(lhs)
    }

    // tier 5: × ÷ mod
    fn parse_mul(&mut self) -> EvalResult<Node> {
        let mut lhs = self.parse_unary()?;
        loop {
            let op = match self.peek_kind() {
                Some(TokKind::Mul) => 0,
                Some(TokKind::Div) => 1,
                Some(TokKind::Ident(n)) if n.eq_ignore_ascii_case("mod") => 2,
                _ => break,
            };
            self.next();
            let rhs = self.parse_unary()?;
            let span = lhs.span.start..rhs.span.end;
            lhs = Node {
                kind: match op {
                    0 => AstKind::Mul(Box::new(lhs), Box::new(rhs)),
                    1 => AstKind::Div(Box::new(lhs), Box::new(rhs)),
                    _ => AstKind::Mod(Box::new(lhs), Box::new(rhs)),
                },
                span,
            };
        }
        Ok(lhs)
    }

    // tier 4: prefix − ¬ √ twos bswap
    fn parse_unary(&mut self) -> EvalResult<Node> {
        match self.peek_kind().cloned() {
            Some(TokKind::Minus) => {
                self.next();
                let operand = self.parse_unary()?;
                let span = operand.span.clone();
                Ok(Node {
                    kind: AstKind::Neg(Box::new(operand)),
                    span,
                })
            }
            Some(TokKind::Not) => {
                self.next();
                let operand = self.parse_unary()?;
                let span = operand.span.clone();
                Ok(Node {
                    kind: AstKind::Not(Box::new(operand)),
                    span,
                })
            }
            Some(TokKind::Sqrt) => {
                self.next();
                let operand = self.parse_postfix()?;
                let span = operand.span.clone();
                Ok(Node {
                    kind: AstKind::SqrtP(Box::new(operand)),
                    span,
                })
            }
            Some(TokKind::Ident(name))
                if (name.eq_ignore_ascii_case("twos") || name.eq_ignore_ascii_case("bswap"))
                    && !matches!(self.peek2_kind(), Some(TokKind::LParen)) =>
            {
                // prefix form binding to the following primary (`bswap 255`);
                // the call form `bswap(255)` is handled in parse_primary
                self.next();
                let operand = self.parse_postfix()?;
                let span = operand.span.clone();
                Ok(Node {
                    kind: if name.eq_ignore_ascii_case("twos") {
                        AstKind::Twos(Box::new(operand))
                    } else {
                        AstKind::Bswap(Box::new(operand))
                    },
                    span,
                })
            }
            _ => self.parse_percent(),
        }
    }

    // tier 3: postfix % (chains: `10%%`)
    //
    // `^` binds tighter than `%` even though it sits to the RIGHT in the
    // token stream: `10%^2` = (10²)% = 1 (T014). After consuming a `%`, a
    // following `^ exponent` therefore attaches INSIDE the percent operand.
    fn parse_percent(&mut self) -> EvalResult<Node> {
        let mut lhs = self.parse_power()?;
        while matches!(self.peek_kind(), Some(TokKind::Percent)) {
            self.next();
            if matches!(self.peek_kind(), Some(TokKind::Pow)) {
                self.next();
                let exp = self.parse_unary()?;
                let span = lhs.span.start..exp.span.end;
                lhs = Node {
                    kind: AstKind::Pow(Box::new(lhs), Box::new(exp)),
                    span,
                };
            }
            let span = lhs.span.clone();
            lhs = Node {
                kind: AstKind::Percent(Box::new(lhs)),
                span,
            };
        }
        Ok(lhs)
    }

    // tier 2: ^ right-assoc (rhs allows unary minus: `2^-3`)
    fn parse_power(&mut self) -> EvalResult<Node> {
        let base = self.parse_postfix()?;
        if matches!(self.peek_kind(), Some(TokKind::Pow)) {
            self.next();
            let exp = self.parse_unary()?;
            let span = base.span.start..exp.span.end;
            return Ok(Node {
                kind: AstKind::Pow(Box::new(base), Box::new(exp)),
                span,
            });
        }
        Ok(base)
    }

    // tier 1: postfix ! and superscript powers
    fn parse_postfix(&mut self) -> EvalResult<Node> {
        let mut lhs = self.parse_primary()?;
        loop {
            match self.peek_kind().cloned() {
                Some(TokKind::Fac) => {
                    self.next();
                    let span = lhs.span.clone();
                    lhs = Node {
                        kind: AstKind::Fact(Box::new(lhs)),
                        span,
                    };
                }
                Some(TokKind::Sup(n)) => {
                    self.next();
                    let span = lhs.span.clone();
                    lhs = Node {
                        kind: AstKind::Sup(Box::new(lhs), n),
                        span,
                    };
                }
                _ => break,
            }
        }
        Ok(lhs)
    }

    // tier 0: primaries
    fn parse_primary(&mut self) -> EvalResult<Node> {
        let Some(tok) = self.next() else {
            return Err(EvalError::new(ErrorKind::ParseError));
        };
        let span = tok.span.clone();
        let node = match tok.kind {
            TokKind::Num(text) => {
                let v = crate::parser::parse_number_in_base(&text, self.base)
                    .ok_or_else(|| EvalError::with_span(ErrorKind::ParseError, span.clone()))?;
                Node {
                    kind: AstKind::Num(v),
                    span,
                }
            }
            TokKind::Glyph(key) => Node {
                kind: AstKind::Const(key),
                span,
            },
            TokKind::Last => Node {
                kind: AstKind::Last,
                span,
            },
            TokKind::LParen => {
                let inner = self.enter_nested(|p| p.parse_expr())?;
                let closing = self.expect(&TokKind::RParen)?;
                Node {
                    kind: inner.kind,
                    span: span.start..closing.span.end,
                }
            }
            TokKind::LBrack => {
                let inner = self.enter_nested(|p| p.parse_expr())?;
                self.expect(&TokKind::RBrack)?;
                Node {
                    kind: AstKind::Round(Box::new(inner)),
                    span,
                }
            }
            TokKind::VBar => {
                let inner = self.enter_nested(|p| p.parse_expr())?;
                self.expect(&TokKind::VBar)?;
                Node {
                    kind: AstKind::Abs(Box::new(inner)),
                    span,
                }
            }
            TokKind::Ident(name) => self.parse_ident(name, span)?,
            _ => return Err(EvalError::with_span(ErrorKind::ParseError, span)),
        };
        Ok(node)
    }

    /// Enter a nested bracket/call context with a depth guard.
    fn enter_nested<T>(&mut self, f: impl FnOnce(&mut Self) -> EvalResult<T>) -> EvalResult<T> {
        if self.depth >= MAX_DEPTH {
            return Err(EvalError::new(ErrorKind::ParseError));
        }
        self.depth += 1;
        let r = f(self);
        self.depth -= 1;
        r
    }

    fn parse_ident(&mut self, name: String, span: std::ops::Range<usize>) -> EvalResult<Node> {
        let lower = name.to_lowercase();
        // function call?
        if matches!(self.peek_kind(), Some(TokKind::LParen)) {
            self.next(); // consume '('
            let mut args = Vec::new();
            if !matches!(self.peek_kind(), Some(TokKind::RParen)) {
                loop {
                    args.push(self.enter_nested(|p| p.parse_expr())?);
                    match self.peek_kind().cloned() {
                        Some(TokKind::Semi) => {
                            self.next();
                        }
                        Some(TokKind::RParen) => break,
                        _ => return Err(self.err_here()),
                    }
                }
            }
            self.expect(&TokKind::RParen)?;
            return Ok(Node {
                kind: AstKind::Call { name: lower, args },
                span,
            });
        }
        if lower == "rand" {
            return Ok(Node {
                kind: AstKind::Rand,
                span,
            });
        }
        if lower == "i" {
            return Ok(Node {
                kind: AstKind::Imag,
                span,
            });
        }
        if let Some((_, key)) = typed_consts().iter().find(|(n, _)| *n == lower) {
            return Ok(Node {
                kind: AstKind::Const(key),
                span,
            });
        }
        // ASCII single-letter physics glyphs are case-sensitive (G vs g).
        if let Some((_, key)) = ascii_consts().iter().find(|(n, _)| *n == name) {
            return Ok(Node {
                kind: AstKind::Const(key),
                span,
            });
        }
        Ok(Node {
            kind: AstKind::Var(lower),
            span,
        })
    }
}

/// Parse digits (validated for `base`) into an exact rational.
pub(crate) fn parse_number_in_base(text: &str, base: Base) -> Option<crate::num::Value> {
    use num_bigint::BigInt;
    use num_rational::BigRational;
    let radix = match base {
        Base::Bin => 2,
        Base::Oct => 8,
        Base::Dec => 10,
        Base::Hex => 16,
    };
    let (int_part, frac_part) = match text.strip_suffix('.') {
        Some(t) => (t, ""),
        None => match text.split_once('.') {
            Some((a, b)) => (a, b),
            None => (text, ""),
        },
    };
    let int_val = if int_part.is_empty() {
        BigInt::from(0)
    } else {
        BigInt::parse_bytes(int_part.as_bytes(), radix)?
    };
    if frac_part.is_empty() {
        return Some(crate::num::Value::exact_int(int_val));
    }
    let frac_val = BigInt::parse_bytes(frac_part.as_bytes(), radix)?;
    let den = BigInt::from(radix).pow(frac_part.len() as u32);
    Some(crate::num::Value::exact(BigRational::new(
        int_val * &den + frac_val,
        den,
    )))
}

/// Is `name` reserved (unassignable)? Functions, constants, keywords.
pub(crate) fn name_is_reserved(name: &str) -> bool {
    let lower = name.to_lowercase();
    function_exists(&lower)
        || typed_consts().iter().any(|(n, _)| *n == lower)
        || ascii_consts().iter().any(|(n, _)| *n == name)
        || glyph_consts()
            .iter()
            .any(|(g, _)| g.is_ascii() && g.eq_ignore_ascii_case(&lower))
        || matches!(lower.as_str(), "rand" | "mod" | "in" | "i" | "_")
}
