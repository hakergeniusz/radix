//! Acceptance battery: doc 12 vectors T001–T055 (engine-applicable), the
//! doc 04 edge battery, doc 10 error cases, formatting battery, exactness
//! and serialization round-trips.

use radix_engine::{
    evaluate, format_value, normalize_echo, parse_value_text, render_in_base, AngleUnit, Base,
    Env, ErrorKind, FormatPrefs, ResultFormat, Value, WordSize,
};

fn ev(expr: &str, env: &mut Env) -> String {
    let v = evaluate(expr, env).expect(expr);
    format_value(&v, &FormatPrefs::default())
}

fn evp(expr: &str, env: &mut Env, prefs: &FormatPrefs) -> String {
    let v = evaluate(expr, env).expect(expr);
    format_value(&v, prefs)
}

fn err_kind(expr: &str, env: &mut Env) -> ErrorKind {
    evaluate(expr, env).expect_err(expr).kind
}

// ---------------------------------------------------------------------------
// P0 — arithmetic & grammar core (T001–T031)
// ---------------------------------------------------------------------------

#[test]
fn t001_precedence() {
    let mut env = Env::new();
    assert_eq!(ev("2+3*4", &mut env), "14");
}

#[test]
fn t002_grouping() {
    let mut env = Env::new();
    assert_eq!(ev("(2+3)*4", &mut env), "20");
}

#[test]
fn t003_left_assoc_sub() {
    let mut env = Env::new();
    assert_eq!(ev("10-4-3", &mut env), "3");
}

#[test]
fn t004_left_assoc_div() {
    let mut env = Env::new();
    assert_eq!(ev("100/5/2", &mut env), "10");
}

#[test]
fn t005_right_assoc_power() {
    let mut env = Env::new();
    assert_eq!(ev("2^3^2", &mut env), "512");
}

#[test]
fn t006_unary_minus() {
    let mut env = Env::new();
    assert_eq!(ev("-5+3", &mut env), "−2");
}

#[test]
fn t007_t016_percent_battery() {
    let mut env = Env::new();
    assert_eq!(ev("50%", &mut env), "0.5", "T007");
    assert_eq!(ev("100+10%", &mut env), "110", "T008");
    assert_eq!(ev("100*10%", &mut env), "10", "T009");
    assert_eq!(ev("200-50%", &mut env), "100", "T010");
    assert_eq!(ev("10%%", &mut env), "0.001", "T011");
    assert_eq!(ev("1+100+10%", &mut env), "111.1", "T012");
    assert_eq!(ev("(100+10)%", &mut env), "1.1", "T013");
    assert_eq!(ev("10%^2", &mut env), "1", "T014");
    assert_eq!(ev("50%+50%", &mut env), "0.75", "T015");
    assert_eq!(ev("100+10%*2", &mut env), "100.2", "T016");
    assert_eq!(ev("10%+1", &mut env), "1.1", "doc 03 §4");
}

#[test]
fn t017_t021_factorial() {
    let mut env = Env::new();
    assert_eq!(ev("5!", &mut env), "120", "T017");
    assert_eq!(ev("0!", &mut env), "1", "T018");
    assert_eq!(ev("2.5!", &mut env), "3.32335097", "T019");
    assert_eq!(ev("-4!", &mut env), "−24", "T020");
    assert_eq!(ev("171!", &mut env), "1.24101807×10³⁰⁹", "T021");
}

#[test]
fn t022_exactness() {
    let mut env = Env::new();
    let v = evaluate("1÷3×3", &mut env).unwrap();
    assert_eq!(v, Value::from_i64(1), "1÷3×3 must be exactly 1");
    assert!(v.is_exact());
    assert_eq!(format_value(&v, &FormatPrefs::default()), "1");
    // rationals stay exact through chains
    let v2 = evaluate("10÷4+1÷2", &mut env).unwrap();
    assert!(v2.is_exact());
    assert_eq!(format_value(&v2, &FormatPrefs::default()), "3");
}

#[test]
fn t023_big_power() {
    let mut env = Env::new();
    assert_eq!(ev("2^1024", &mut env), "1.797693135×10³⁰⁸");
}

#[test]
fn t024_last_result() {
    let mut env = Env::new();
    assert_eq!(ev("7", &mut env), "7");
    assert_eq!(ev("_*2", &mut env), "14");
    assert_eq!(ev("_+1", &mut env), "15");
}

#[test]
fn t025_variables() {
    let mut env = Env::new();
    assert_eq!(ev("x=5", &mut env), "5");
    assert_eq!(ev("x*2", &mut env), "10");
    // case-insensitive variable lookup
    assert_eq!(ev("X+1", &mut env), "6");
    // redefinition
    assert_eq!(ev("x=7", &mut env), "7");
    assert_eq!(ev("x*2", &mut env), "14");
}

#[test]
fn t026_e_notation_rejected() {
    let mut env = Env::new();
    assert_eq!(err_kind("1E5", &mut env), ErrorKind::ParseError);
    assert_eq!(err_kind("1e5", &mut env), ErrorKind::ParseError);
    assert_eq!(err_kind("1.5e3", &mut env), ErrorKind::ParseError);
    assert_eq!(err_kind("10e+5", &mut env), ErrorKind::ParseError);
}

#[test]
fn t027_sqrt() {
    let mut env = Env::new();
    assert_eq!(ev("sqrt(2)", &mut env), "1.414213562");
    // perfect squares stay exact
    assert_eq!(ev("sqrt(4)", &mut env), "2");
    assert_eq!(ev("√4", &mut env), "2");
}

#[test]
fn t028_logs() {
    let mut env = Env::new();
    assert_eq!(ev("log(8;2)", &mut env), "3");
    assert_eq!(ev("log(100)", &mut env), "2");
    assert_eq!(ev("log2(8)", &mut env), "3");
    assert_eq!(ev("ln(e)", &mut env), "1");
    assert_eq!(ev("e^2", &mut env), "7.389056099");
}

#[test]
fn t029_round_half_away() {
    let mut env = Env::new();
    assert_eq!(ev("round(2.5)", &mut env), "3");
    assert_eq!(ev("round(-2.5)", &mut env), "−3");
}

#[test]
fn t030_t031_bracket_round() {
    let mut env = Env::new();
    assert_eq!(ev("[2.5]", &mut env), "3");
    assert_eq!(ev("[2.3]", &mut env), "2");
    assert_eq!(ev("[2.7]", &mut env), "3");
    assert_eq!(ev("[-2.5]", &mut env), "−3");
}

// ---------------------------------------------------------------------------
// P0 — trigonometry (T032–T035)
// ---------------------------------------------------------------------------

#[test]
fn t032_t035_trig_degrees() {
    let mut env = Env::new();
    assert_eq!(ev("sin(30)", &mut env), "0.5", "T032");
    assert_eq!(ev("cos(60)", &mut env), "0.5");
    assert_eq!(ev("sin(90)", &mut env), "1");
    assert_eq!(ev("tan(45)", &mut env), "1", "T033");
    assert_eq!(ev("sin(pi/2)", &mut env), "0.027412134", "T034 (π numeric)");
    assert_eq!(ev("asin(0.5)", &mut env), "30", "T035");
    assert_eq!(ev("acos(0.5)", &mut env), "60");
    assert_eq!(ev("atan(1)", &mut env), "45");
}

#[test]
fn trig_radians_and_gradians() {
    let mut env = Env::new();
    env.angle_unit = AngleUnit::Radians;
    assert_eq!(ev("sin(pi/2)", &mut env), "1");
    assert_eq!(ev("tan(0)", &mut env), "0");
    env.angle_unit = AngleUnit::Gradians;
    assert_eq!(ev("sin(100)", &mut env), "1", "100 grad = 90°");
    let s200 = ev("sin(200)", &mut env);
    assert!(
        s200.contains("×10⁻") && !s200.starts_with('1'),
        "200 grad = 180°: sin ≈ 0 to working precision: {s200}"
    );
    assert_eq!(err_kind("tan(100)", &mut env), ErrorKind::TangentUndefined);
}

#[test]
fn hyperbolics_immune_to_angle_unit() {
    let mut env = Env::new();
    assert_eq!(ev("sinh(1)", &mut env), "1.175201194");
    env.angle_unit = AngleUnit::Gradians;
    assert_eq!(ev("sinh(1)", &mut env), "1.175201194");
}

// ---------------------------------------------------------------------------
// P0 — programming mode (T037–T047)
// ---------------------------------------------------------------------------

#[test]
fn t037_t039_bitwise_logic() {
    let mut env = Env::new();
    assert_eq!(ev("12∧10", &mut env), "8", "T037");
    assert_eq!(ev("12∨10", &mut env), "14", "T038");
    assert_eq!(ev("12⊻10", &mut env), "6", "T039");
}

#[test]
fn t040_bitwise_not_64() {
    let mut env = Env::new();
    assert_eq!(ev("¬5", &mut env), "1.844674407×10¹⁹", "T040");
    let v = evaluate("¬5", &mut env).unwrap();
    assert_eq!(
        render_in_base(&v, Base::Dec, 9),
        "18446744073709551610",
        "2^64 − 1 − 5"
    );
}

#[test]
fn t041_twos_complement() {
    let mut env = Env::new();
    let v = evaluate("twos 5", &mut env).unwrap();
    assert_eq!(
        render_in_base(&v, Base::Dec, 9),
        "18446744073709551611",
        "2^64 − 5"
    );
    assert_eq!(
        render_in_base(&v, Base::Hex, 9),
        "FFFFFFFFFFFFFFFB",
        "T041 hex form"
    );
    // differs from ¬ by 1
    assert_eq!(ev("¬5", &mut env), "1.844674407×10¹⁹");
}

#[test]
fn t042_shifts() {
    let mut env = Env::new();
    assert_eq!(ev("1<<10", &mut env), "1024");
    assert_eq!(ev("1024>>3", &mut env), "128");
    assert_eq!(
        ev("5<<63", &mut env),
        "9.223372037×10¹⁸",
        "masked into word (scientific ≥ 1e12)"
    );
    assert_eq!(ev("1<<64", &mut env), "0");
}

#[test]
fn t043_bswap() {
    let mut env = Env::new();
    assert_eq!(ev("bswap 255", &mut env), "1.837468648×10¹⁹");
    let v = evaluate("bswap 255", &mut env).unwrap();
    assert_eq!(
        render_in_base(&v, Base::Hex, 9),
        "FF00000000000000",
        "0xFF00000000000000"
    );
    // call form also accepted
    assert_eq!(ev("bswap(255)", &mut env), "1.837468648×10¹⁹");
}

#[test]
fn t044_8bit_arithmetic_unbounded() {
    let mut env = Env::new();
    env.word_size = WordSize::W8;
    assert_eq!(ev("255+1", &mut env), "256", "no arithmetic wraparound");
}

#[test]
fn t045_8bit_not() {
    let mut env = Env::new();
    env.word_size = WordSize::W8;
    assert_eq!(ev("¬0", &mut env), "255");
    assert_eq!(ev("¬5", &mut env), "250");
}

#[test]
fn t047_binary_base_parsing() {
    let mut env = Env::new();
    env.base = Base::Bin;
    assert_eq!(ev("1010.1", &mut env), "10.5");
    // decimal digits are invalid in binary
    assert_eq!(err_kind("1010.12", &mut env), ErrorKind::ParseError);
    // lenient conversion-row parse
    let v = parse_value_text("1010.1", Base::Bin).unwrap();
    assert_eq!(format_value(&v, &FormatPrefs::default()), "10.5");
    assert!(parse_value_text("1010.12", Base::Bin).is_none());
}

#[test]
fn base_rendering_negative_fraction() {
    let mut env = Env::new();
    let v = evaluate("-26.85", &mut env).unwrap();
    assert_eq!(render_in_base(&v, Base::Bin, 9), "−11010.110110011");
    assert_eq!(render_in_base(&v, Base::Hex, 9), "−1A.D9999999A");
}

#[test]
fn hex_digit_priority_over_identifiers() {
    let mut env = Env::new();
    env.base = Base::Hex;
    assert_eq!(ev("abc", &mut env), "2748");
    assert_eq!(ev("ff", &mut env), "255");
    assert_eq!(ev("1f+1", &mut env), "32");
    // non-hex letters remain identifiers/functions
    assert_eq!(ev("sqrt(4)", &mut env), "2");
    // decimal base still rejects hex letters as digits
    let mut dec = Env::new();
    assert_eq!(
        err_kind("ff", &mut dec),
        ErrorKind::UndefinedVariable("ff".into())
    );
}

// ---------------------------------------------------------------------------
// P1 — formatting (T048–T055)
// ---------------------------------------------------------------------------

#[test]
fn t048_automatic_scientific() {
    let mut env = Env::new();
    let prefs = FormatPrefs::default();
    assert_eq!(evp("10^21", &mut env, &prefs), "1×10²¹");
    assert_eq!(ev("61917364224", &mut env), "61917364224", "stays plain");
    assert_eq!(ev("¬5", &mut env), "1.844674407×10¹⁹");
}

#[test]
fn t049_scientific_small_integers() {
    let mut env = Env::new();
    let prefs = FormatPrefs {
        format: ResultFormat::Scientific,
        ..FormatPrefs::default()
    };
    assert_eq!(evp("99", &mut env, &prefs), "9.9×10¹");
    assert_eq!(evp("9", &mut env, &prefs), "9");
    assert_eq!(evp("6", &mut env, &prefs), "6");
}

#[test]
fn t050_engineering() {
    let mut env = Env::new();
    let prefs = FormatPrefs {
        format: ResultFormat::Engineering,
        ..FormatPrefs::default()
    };
    assert_eq!(evp("1.23*10^-5", &mut env, &prefs), "12.3×10⁻⁶");
    assert_eq!(evp("333.333333333", &mut env, &prefs), "333.333333333", "stays plain, capped at 9 decimals");
    assert_eq!(evp("128", &mut env, &prefs), "128");
    assert_eq!(evp("bswap 255", &mut env, &prefs), "18.37468648×10¹⁸");
    assert_eq!(evp("10^21", &mut env, &prefs), "1×10²¹");
}

#[test]
fn t051_fixed_truncation() {
    let mut env = Env::new();
    let prefs = FormatPrefs {
        format: ResultFormat::Fixed,
        ..FormatPrefs::default()
    };
    assert_eq!(evp("¬5", &mut env, &prefs), "1844674407370…");
    assert_eq!(evp("2+3", &mut env, &prefs), "5");
    assert_eq!(evp("2.5", &mut env, &prefs), "2.5");
}

#[test]
fn t052_trailing_and_grouping() {
    let mut env = Env::new();
    let prefs = FormatPrefs {
        decimals: 6,
        trailing_zeroes: true,
        group: true,
        ..FormatPrefs::default()
    };
    assert_eq!(evp("2+3", &mut env, &prefs), "5.000000");
    assert_eq!(evp("1234567", &mut env, &prefs), "1,234,567.000000");
    let prefs6 = FormatPrefs {
        decimals: 6,
        trailing_zeroes: true,
        ..FormatPrefs::default()
    };
    assert_eq!(evp("2+3", &mut env, &prefs6), "5.000000");
}

#[test]
fn t053_decimals_cap() {
    let mut env = Env::new();
    let prefs = FormatPrefs {
        decimals: 6,
        ..FormatPrefs::default()
    };
    assert_eq!(evp("0.0000123", &mut env, &prefs), "0.000012");
    assert_eq!(evp("1000÷3", &mut env, &prefs), "333.333333");
    assert_eq!(evp("10^21", &mut env, &prefs), "1×10²¹", "mantissa capped");
}

#[test]
fn t055_echo_grouping() {
    assert_eq!(normalize_echo("1234+0", true), "1,234+0");
    assert_eq!(normalize_echo("1234+0", false), "1234+0");
    assert_eq!(normalize_echo("1000÷3", true), "1,000÷3");
}

#[test]
fn echo_normalization_glyphs() {
    assert_eq!(normalize_echo("2+3*4", false), "2+3×4", "T001 echo");
    assert_eq!(normalize_echo("10/4", false), "10÷4");
    assert_eq!(normalize_echo("10-4-3", false), "10−4−3", "true minus");
    assert_eq!(normalize_echo("2^3^2", false), "2^3^2", "caret literal");
    assert_eq!(normalize_echo("5³+4⁻¹", false), "5³+4⁻¹", "superscripts stay");
    assert_eq!(normalize_echo(" 10 - 4 ", false), " 10 − 4 ", "whitespace kept");
    // hex-letters and identifiers are not grouped
    assert_eq!(normalize_echo("FF12+x2", true), "FF12+x2");
}

// ---------------------------------------------------------------------------
// Edge battery (doc 04) & rounding family
// ---------------------------------------------------------------------------

#[test]
fn doc04_edge_battery() {
    let mut env = Env::new();
    assert_eq!(ev("0!", &mut env), "1");
    assert_eq!(ev("floor(-2.5)", &mut env), "−3");
    assert_eq!(ev("ceil(-2.5)", &mut env), "−2");
    assert_eq!(ev("int(-2.5)", &mut env), "−2");
    assert_eq!(ev("frac(-2.5)", &mut env), "−0.5");
    assert_eq!(ev("floor(pi)", &mut env), "3");
    assert_eq!(ev("10-4-3", &mut env), "3");
    assert_eq!(ev("100/5/2", &mut env), "10");
    assert_eq!(ev("-5+3", &mut env), "−2");
    assert_eq!(ev("floor(2.5)", &mut env), "2");
    assert_eq!(ev("ceil(2.5)", &mut env), "3");
    assert_eq!(ev("int(2.5)", &mut env), "2");
    assert_eq!(ev("frac(2.5)", &mut env), "0.5");
}

#[test]
fn superscript_powers() {
    let mut env = Env::new();
    assert_eq!(ev("5³", &mut env), "125");
    assert_eq!(ev("4⁻¹", &mut env), "0.25");
    assert_eq!(ev("2²*2", &mut env), "8");
    assert_eq!(ev("(2+3)²", &mut env), "25");
}

#[test]
fn modulo_and_modexp() {
    let mut env = Env::new();
    assert_eq!(ev("12 mod 10", &mut env), "2");
    assert_eq!(ev("mod(12;10)", &mut env), "2");
    assert_eq!(ev("-7 mod 3", &mut env), "2", "floor-mod");
    assert_eq!(ev("5.5 mod 2", &mut env), "1.5");
    assert_eq!(ev("modexp(5;3;13)", &mut env), "8");
    assert_eq!(ev("modexp(2;10;1000)", &mut env), "24");
}

#[test]
fn statistics_battery() {
    let mut env = Env::new();
    assert_eq!(ev("sum(1;2;3)", &mut env), "6");
    assert_eq!(ev("sumsq(1;2;3)", &mut env), "14");
    assert_eq!(ev("average(2;4;6)", &mut env), "4");
    assert_eq!(ev("median(1;2;3;4)", &mut env), "2.5");
    assert_eq!(ev("median(3;1;2)", &mut env), "2");
    assert_eq!(ev("min(3;1;2)", &mut env), "1");
    assert_eq!(ev("max(3;1;2)", &mut env), "3");
    assert_eq!(ev("stdev(2;4;4;4;5;5;7;9)", &mut env), "2.138089935");
    assert_eq!(ev("var(2;4;4;4;5;5;7;9)", &mut env), "4.571428571");
    assert_eq!(
        ev("stdevp(2;4;4;4;5;5;7;9)", &mut env),
        "2"
    );
}

#[test]
fn stdev_arity() {
    let mut env = Env::new();
    assert_eq!(
        err_kind("stdev(5)", &mut env),
        ErrorKind::WrongArity {
            name: "stdev".into(),
            expected: "at least 2"
        }
    );
}

// ---------------------------------------------------------------------------
// Errors (doc 10 §2)
// ---------------------------------------------------------------------------

#[test]
fn division_by_zero_spans() {
    let mut env = Env::new();
    let e = evaluate("1/0", &mut env).unwrap_err();
    assert_eq!(e.kind, ErrorKind::DivisionByZero);
    assert_eq!(e.span, Some(2..3), "divisor highlighted");
    let e = evaluate("0/0", &mut env).unwrap_err();
    assert_eq!(e.kind, ErrorKind::DivisionByZero);
    let e = evaluate("5 mod 0", &mut env).unwrap_err();
    assert_eq!(e.kind, ErrorKind::DivisionByZero);
}

#[test]
fn tangent_pole_span() {
    let mut env = Env::new();
    let e = evaluate("tan(90)", &mut env).unwrap_err();
    assert_eq!(e.kind, ErrorKind::TangentUndefined);
    assert_eq!(e.span, Some(4..6), "the angle highlighted");
    // radians pi/2 also poles
    env.angle_unit = AngleUnit::Radians;
    assert_eq!(err_kind("tan(pi/2)", &mut env), ErrorKind::TangentUndefined);
    // near-pole exact angles stay defined
    env.angle_unit = AngleUnit::Degrees;
    let v = ev("tan(89.999999999)", &mut env);
    assert!(v.starts_with("572957795"), "huge but defined: {v}");
}

#[test]
fn parse_errors() {
    let mut env = Env::new();
    assert_eq!(err_kind("1E5", &mut env), ErrorKind::ParseError);
    assert_eq!(err_kind("5₃", &mut env), ErrorKind::ParseError);
    assert_eq!(err_kind("5twos", &mut env), ErrorKind::ParseError);
    assert_eq!(err_kind("65bswap", &mut env), ErrorKind::ParseError);
    let e = evaluate("65bswap", &mut env).unwrap_err();
    assert_eq!(e.span, Some(2..7), "span at bswap");
    assert_eq!(err_kind("2pi", &mut env), ErrorKind::ParseError, "no implicit mult");
    assert_eq!(err_kind("2(3)", &mut env), ErrorKind::ParseError);
    assert_eq!(err_kind("⌊2.5⌋", &mut env), ErrorKind::ParseError, "glyph brackets");
    assert_eq!(err_kind("", &mut env), ErrorKind::ParseError);
    assert_eq!(err_kind("1+", &mut env), ErrorKind::ParseError);
}

#[test]
fn undefined_names() {
    let mut env = Env::new();
    assert_eq!(err_kind("y", &mut env), ErrorKind::UndefinedVariable("y".into()));
    let e = evaluate("y", &mut env).unwrap_err();
    assert_eq!(e.span, Some(0..1));
    // phi has no typed name (T086) — must error
    assert!(matches!(
        err_kind("phi", &mut env),
        ErrorKind::UndefinedVariable(_) | ErrorKind::ParseError
    ));
    // but e, pi, tau resolve
    assert_eq!(ev("tau÷2", &mut env), "3.141592654");
    assert_eq!(ev("pi", &mut env), "3.141592654");
    assert_eq!(ev("e", &mut env), "2.718281828");
}

#[test]
fn wrong_arity() {
    let mut env = Env::new();
    assert_eq!(
        err_kind("round(2.567;2)", &mut env),
        ErrorKind::WrongArity {
            name: "round".into(),
            expected: "1"
        }
    );
    assert_eq!(err_kind("sqrt(1;2)", &mut env), ErrorKind::WrongArity {
        name: "sqrt".into(),
        expected: "1"
    });
    assert_eq!(err_kind("nosuchfn(1)", &mut env), ErrorKind::UndefinedFunction("nosuchfn".into()));
}

#[test]
fn trailing_equals_marker() {
    let mut env = Env::new();
    assert_eq!(ev("9=", &mut env), "9");
    assert_eq!(ev("2+3=", &mut env), "5");
}

// ---------------------------------------------------------------------------
// Complex
// ---------------------------------------------------------------------------

#[test]
fn complex_sqrt_and_i() {
    let mut env = Env::new();
    let v = evaluate("sqrt(-1)", &mut env).unwrap();
    let (re, im) = v.real_and_imaginary();
    assert_eq!(re, Value::from_i64(0));
    assert_eq!(im, Value::from_i64(1), "im part exactly 1");
    assert_eq!(format_value(&v, &FormatPrefs::default()), "i");
    assert_eq!(ev("i^2", &mut env), "−1");
    assert_eq!(ev("sqrt(-4)", &mut env), "2i");
    assert_eq!(ev("2*i", &mut env), "2i");
    assert_eq!(ev("-i", &mut env), "−i");
}

#[test]
fn complex_arithmetic() {
    let mut env = Env::new();
    assert_eq!(ev("(1+i)*(1-i)", &mut env), "2");
    assert_eq!(ev("(1+i)+(1-i)", &mut env), "2");
    assert_eq!(ev("(1+2*i)*(2-i)", &mut env), "4+3i");
    assert_eq!(ev("(4+2*i)/(1+i)", &mut env), "3−i");
    assert_eq!(ev("conj(3+2*i)", &mut env), "3−2i");
    assert_eq!(ev("re(3+2*i)", &mut env), "3");
    assert_eq!(ev("im(3+2*i)", &mut env), "2");
    assert_eq!(ev("abs(3+4*i)", &mut env), "5");
    assert_eq!(ev("arg(i)", &mut env), "1.570796327");
    // exp(iπ) = −1 (a ~1e-60 imaginary residue remains at 200 bits)
    let v = evaluate("exp(i*pi)", &mut env).unwrap();
    assert!(format_value(&v, &FormatPrefs::default()).starts_with("−1"));
}

#[test]
fn negative_base_fractional_exponent_is_complex() {
    let mut env = Env::new();
    let v = evaluate("(-8)^(1÷3)", &mut env).unwrap();
    let (re, im) = v.real_and_imaginary();
    // principal value: 1 + i·√3
    assert_eq!(format_value(&re, &FormatPrefs::default()), "1");
    assert_eq!(format_value(&im, &FormatPrefs::default()), "1.732050808");
}

// ---------------------------------------------------------------------------
// Constants, rand, misc
// ---------------------------------------------------------------------------

#[test]
fn physical_constants() {
    let mut env = Env::new();
    assert_eq!(ev("c", &mut env), "299792458");
    assert_eq!(ev("g", &mut env), "9.80665");
    // tiny constants go scientific under Automatic (|x| < 1e-6)
    assert_eq!(ev("h", &mut env), "6.62607015×10⁻³⁴");
    assert_eq!(ev("NA", &mut env), "6.02214076×10²³");
    assert_eq!(ev("G", &mut env), "6.6743×10⁻¹¹");
    assert_eq!(ev("σ", &mut env), "5.670374419×10⁻⁸");
}

#[test]
fn rand_in_range_and_fresh() {
    let mut env = Env::new();
    let a = evaluate("rand", &mut env).unwrap();
    let b = evaluate("rand", &mut env).unwrap();
    for v in [&a, &b] {
        let s = format_value(v, &FormatPrefs::default());
        assert!(s.starts_with("0."), "rand ∈ [0,1): {s}");
        assert!(s != "0.000000000" && s != "1", "has significant digits: {s}");
    }
    assert!(a != b, "fresh per evaluation");
}

#[test]
fn case_insensitive_names() {
    let mut env = Env::new();
    assert_eq!(ev("SQRT(4)", &mut env), "2");
    assert_eq!(ev("Pi*2", &mut env), "6.283185307");
    assert_eq!(ev("TAU/2", &mut env), "3.141592654");
}

#[test]
fn leading_dot_and_spacing() {
    let mut env = Env::new();
    assert_eq!(ev(".5", &mut env), "0.5");
    assert_eq!(ev("10 - 4 - 3", &mut env), "3");
    assert_eq!(ev("  2+3  ", &mut env), "5");
    assert_eq!(ev("5.", &mut env), "5");
}

#[test]
fn display_negative_sign_and_leading_zero() {
    let mut env = Env::new();
    assert_eq!(ev("-2.5", &mut env), "−2.5", "U+2212");
    assert_eq!(ev("0.007379646", &mut env), "0.007379646", "leading zero");
}

// ---------------------------------------------------------------------------
// Serialization
// ---------------------------------------------------------------------------

#[test]
fn serialization_round_trip() {
    let mut env = Env::new();
    let cases = [
        evaluate("1÷3", &mut env).unwrap(),      // exact
        evaluate("-7", &mut env).unwrap(),        // exact negative int
        evaluate("sqrt(2)", &mut env).unwrap(),   // approx
        evaluate("sqrt(-1)", &mut env).unwrap(),  // complex
        evaluate("2^1024", &mut env).unwrap(),    // huge exact
        evaluate("171!", &mut env).unwrap(),      // factorial
    ];
    for v in &cases {
        let s = v.to_compact();
        let back = Value::from_compact(&s).expect("parses back");
        assert_eq!(&back, v, "round trip through {s}");
    }
}

#[test]
fn serialize_all_value_shapes() {
    let mut env = Env::new();
    for expr in ["0", "0.5", "-1/3", "rand", "e^10", "tan(1)", "i", "1+2*i"] {
        let v = evaluate(expr, &mut env).unwrap();
        let back = Value::from_compact(&v.to_compact());
        assert_eq!(back.as_ref(), Some(v.clone()).as_ref(), "{expr}");
    }
}

// ---------------------------------------------------------------------------
// parse_value_text
// ---------------------------------------------------------------------------

#[test]
fn parse_value_text_lenient() {
    assert!(parse_value_text("123", Base::Dec).is_some());
    assert!(parse_value_text(" -12.5 ", Base::Dec).is_some());
    assert!(parse_value_text("FF", Base::Hex).is_some());
    assert!(parse_value_text("ff", Base::Hex).is_some());
    assert!(parse_value_text("1010.1", Base::Bin).is_some());
    // free-form junk fails (T061: "-e.5" is an unparseable source)
    assert!(parse_value_text("-e.5", Base::Dec).is_none());
    assert!(parse_value_text("", Base::Dec).is_none());
    assert!(parse_value_text("abc", Base::Dec).is_none());
    assert!(parse_value_text("1.2.3", Base::Dec).is_none());
    let v = parse_value_text("-12.5", Base::Dec).unwrap();
    assert_eq!(format_value(&v, &FormatPrefs::default()), "−12.5");
}

// ---------------------------------------------------------------------------
// Catalogs
// ---------------------------------------------------------------------------

#[test]
fn catalogs_present() {
    let fns = radix_engine::function_catalog();
    let names: Vec<&str> = fns
        .iter()
        .flat_map(|c| c.functions.iter().map(|f| f.name))
        .collect();
    for expected in [
        "sin", "cos", "tan", "asin", "acos", "atan", "sinh", "cosh", "tanh", "conj", "arg", "re",
        "im", "twos", "bswap", "mod", "modexp", "round", "floor", "ceil", "int", "frac", "sum",
        "sumsq", "average", "median", "min", "max", "stdev", "stdevp", "var", "abs", "exp",
        "sqrt",
    ] {
        assert!(names.contains(&expected), "missing {expected}");
    }
    let cats: Vec<&str> = fns.iter().map(|c| c.name).collect();
    assert_eq!(cats, ["Trigonometry", "Complex", "Programming", "Rounding", "Statistics", "Sundry"]);

    let consts = radix_engine::constant_catalog();
    let glyphs: Vec<&str> = consts
        .iter()
        .flat_map(|c| c.constants.iter().map(|k| k.glyph))
        .collect();
    for g in ["e", "π", "τ", "φ", "h", "α", "mₑ", "mₚ", "k", "mᵤ", "R", "σ", "NA", "G", "g", "c", "q", "μ₀", "ε₀", "Z₀"] {
        assert!(glyphs.contains(&g), "missing constant glyph {g}");
    }
    // φ has no typed name
    let phi = consts
        .iter()
        .flat_map(|c| c.constants.iter())
        .find(|k| k.glyph == "φ")
        .unwrap();
    assert!(phi.typed_name.is_none());
    assert_eq!(
        consts
            .iter()
            .flat_map(|c| c.constants.iter())
            .find(|k| k.glyph == "π")
            .unwrap()
            .typed_name,
        Some("pi")
    );
}

// ---------------------------------------------------------------------------
// helper shims used above
// ---------------------------------------------------------------------------

#[test]
fn evaluate_rejects_garbage_types() {
    let mut env = Env::new();
    // a fractional bitwise operand
    assert_eq!(
        err_kind("12∧2.5", &mut env),
        ErrorKind::InvalidBitwiseOperand
    );
    assert_eq!(
        err_kind("sqrt(−1)∧1", &mut env),
        ErrorKind::InvalidBitwiseOperand
    );
    // negative non-integer factorial is defined via gamma: Γ(−1.5) = 4√π/3
    assert_eq!(ev("(-2.5)!", &mut env), "2.363271801");
    // negative integer factorials are gamma poles
    assert_eq!(err_kind("(-2)!", &mut env), ErrorKind::OutOfDomain);
    assert_eq!(err_kind("(0-3)!", &mut env), ErrorKind::OutOfDomain);
    assert_eq!(err_kind("ln(0)", &mut env), ErrorKind::OutOfDomain);
    assert_eq!(err_kind("asin(2)", &mut env), ErrorKind::OutOfDomain);
    assert_eq!(ev("0^0", &mut env), "1", "calculator convention");
}

#[test]
fn deep_nesting_is_an_error_not_a_crash() {
    let mut env = Env::new();
    let deep = format!("{}5{}", "(".repeat(10_000), ")".repeat(10_000));
    assert_eq!(evaluate(&deep, &mut env).unwrap_err().kind, ErrorKind::ParseError);
}
