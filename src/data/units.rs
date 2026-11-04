//! Unit-conversion data: the 14 categories of docs/06 §3 and their units.
//!
//! Length, Temperature, Number Base and Currency lists are contract items
//! (docs/06 §4, observed order for Length). The other ten categories are not
//! enumerated by the spec; standard SI-derived unit sets with exact decimal
//! factors are provided and documented as a decision (D6). Lists are ordered
//! largest→smallest by factor.

/// A physical unit: display name and conversion factor into the category's
/// canonical unit (exact decimal values).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Unit {
    pub name: &'static str,
    /// Factor into the canonical unit; for temperature categories this is
    /// unused (affine maps instead).
    pub factor: &'static str, // exact decimal, parsed at runtime
}

pub const fn u(name: &'static str, factor: &'static str) -> Unit {
    Unit { name, factor }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Angle,
    Area,
    Currency,
    DigitalStorage,
    Duration,
    Energy,
    Frequency,
    Length,
    Mass,
    NumberBase,
    Pressure,
    Speed,
    Temperature,
    Volume,
}

impl Category {
    pub const ALL: [Category; 14] = [
        Category::Angle,
        Category::Area,
        Category::Currency,
        Category::DigitalStorage,
        Category::Duration,
        Category::Energy,
        Category::Frequency,
        Category::Length,
        Category::Mass,
        Category::NumberBase,
        Category::Pressure,
        Category::Speed,
        Category::Temperature,
        Category::Volume,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Category::Angle => "Angle",
            Category::Area => "Area",
            Category::Currency => "Currency",
            Category::DigitalStorage => "Digital Storage",
            Category::Duration => "Duration",
            Category::Energy => "Energy",
            Category::Frequency => "Frequency",
            Category::Length => "Length",
            Category::Mass => "Mass",
            Category::NumberBase => "Number Base",
            Category::Pressure => "Pressure",
            Category::Speed => "Speed",
            Category::Temperature => "Temperature",
            Category::Volume => "Volume",
        }
    }

    /// Default unit per category (docs/06 §3: category switch resets units to
    /// defaults; e.g. Length defaults observed as Parsecs/Parsecs).
    pub fn default_unit(self) -> &'static str {
        match self {
            Category::Angle => "Turns",
            Category::Area => "Square Meters",
            Category::Currency => "Algerian Dinar",
            Category::DigitalStorage => "Bytes",
            Category::Duration => "Days",
            Category::Energy => "Joules",
            Category::Frequency => "Hertz",
            Category::Length => "Kilometers", // as-found default (E085)
            Category::Mass => "Grams",
            Category::NumberBase => "Decimal",
            Category::Pressure => "Pascals",
            Category::Speed => "Kilometers per Hour",
            Category::Temperature => "Celsius",
            Category::Volume => "Cubic Meters",
        }
    }

    /// Default source/target unit pair per category (docs/06 §3, corrected
    /// for Length by docs/16 D19: E091 shows Kilometers → Miles).
    pub fn default_pair(self) -> (&'static str, &'static str) {
        match self {
            Category::Length => ("Kilometers", "Miles"),
            _ => (self.default_unit(), self.default_unit()),
        }
    }

    pub fn units(self) -> &'static [Unit] {
        match self {
            Category::Angle => ANGLE,
            Category::Area => AREA,
            Category::Currency => CURRENCY_UNITS,
            Category::DigitalStorage => DIGITAL_STORAGE,
            Category::Duration => DURATION,
            Category::Energy => ENERGY,
            Category::Frequency => FREQUENCY,
            Category::Length => LENGTH,
            Category::Mass => MASS,
            Category::NumberBase => NUMBER_BASE,
            Category::Pressure => PRESSURE,
            Category::Speed => SPEED,
            Category::Temperature => TEMPERATURE,
            Category::Volume => VOLUME,
        }
    }

    /// Keypad contextual slot label (docs/02 §8.5): empty (most),
    /// "+/−" (Temperature), "ⓘ" (Currency).
    pub fn slot(self) -> &'static str {
        match self {
            Category::Temperature => "+/−",
            Category::Currency => "ⓘ",
            _ => "",
        }
    }
}

// Canonical units: meters (Length), square meters (Area), bytes (Digital
// Storage), seconds (Duration), joules (Energy), hertz (Frequency),
// kilograms? — no: kilograms vs grams… canonical = kilogram for Mass,
// pascals (Pressure), m/s (Speed), cubic meters (Volume), radians (Angle).

pub const LENGTH: &[Unit] = &[
    // Observed dropdown order (docs/06 §4) — kept exactly.
    u("Parsecs", "30857000000000000"),
    u("Light Years", "9460700000000000"),
    u("Astronomical Units", "149597870700"),
    u("Rack Units", "0.04445"),
    u("Nautical Miles", "1852"),
    u("Miles", "1609.344"),
    u("Kilometers", "1000"),
    u("Cables", "185.2"),
    u("Fathoms", "1.8288"),
    u("Meters", "1"),
    u("Yards", "0.9144"),
    u("Feet", "0.3048"),
    u("Inches", "0.0254"),
    u("Centimeters", "0.01"),
    u("Millimeters", "0.001"),
    u("Micrometers", "0.000001"),
    u("Nanometers", "0.000000001"),
    u("Desktop Publishing Points", "0.000352777777777777777"),
];

pub const TEMPERATURE: &[Unit] = &[
    u("Celsius", "0"),
    u("Fahrenheit", "0"),
    u("Kelvin", "0"),
    u("Rankine", "0"),
];

pub const NUMBER_BASE: &[Unit] = &[
    u("Binary", "0"),
    u("Octal", "0"),
    u("Decimal", "0"),
    u("Hexadecimal", "0"),
];

pub const CURRENCY_UNITS: &[Unit] = &[u("__currency__", "0")]; // filled from data::currency

pub const ANGLE: &[Unit] = &[
    u("Turns", "6.283185307179586"),
    u("Radians", "1"),
    u("Degrees", "0.017453292519943295"),
    u("Gradians", "0.015707963267948967"),
    u("Milliradians", "0.001"),
    u("Arcminutes", "0.0002908882086657216"),
    u("Arcseconds", "0.00000484813681109536"),
];

pub const AREA: &[Unit] = &[
    u("Square Kilometers", "1000000"),
    u("Square Miles", "2589988.110336"),
    u("Hectares", "10000"),
    u("Acres", "4046.8564224"),
    u("Square Meters", "1"),
    u("Square Yards", "0.83612736"),
    u("Square Feet", "0.09290304"),
    u("Square Inches", "0.00064516"),
    u("Square Centimeters", "0.0001"),
    u("Square Millimeters", "0.000001"),
];

pub const DIGITAL_STORAGE: &[Unit] = &[
    u("Terabytes", "1000000000000"),
    u("Tebibytes", "1099511627776"),
    u("Gigabytes", "1000000000"),
    u("Gibibytes", "1073741824"),
    u("Megabytes", "1000000"),
    u("Mebibytes", "1048576"),
    u("Kilobytes", "1000"),
    u("Kibibytes", "1024"),
    u("Bytes", "1"),
    u("Bits", "0.125"),
];

pub const DURATION: &[Unit] = &[
    u("Years", "31556952"),
    u("Months", "2629746"),
    u("Weeks", "604800"),
    u("Days", "86400"),
    u("Hours", "3600"),
    u("Minutes", "60"),
    u("Seconds", "1"),
    u("Milliseconds", "0.001"),
    u("Microseconds", "0.000001"),
    u("Nanoseconds", "0.000000001"),
];

pub const ENERGY: &[Unit] = &[
    u("Kilowatt Hours", "3600000"),
    u("Megajoules", "1000000"),
    u("Kilocalories", "4184"),
    u("Kilojoules", "1000"),
    u("Calories", "4.184"),
    u("Joules", "1"),
    u("Millijoules", "0.001"),
    u("Electronvolts", "0.0000000000000000001602176634"),
];

pub const FREQUENCY: &[Unit] = &[
    u("Gigahertz", "1000000000"),
    u("Megahertz", "1000000"),
    u("Kilohertz", "1000"),
    u("Hertz", "1"),
    u("Millihertz", "0.001"),
];

pub const MASS: &[Unit] = &[
    u("Tonnes", "1000"),
    u("Stone", "6.35029318"),
    u("Kilograms", "1"),
    u("Pounds", "0.45359237"),
    u("Ounces", "0.028349523125"),
    u("Grams", "0.001"),
    u("Carats", "0.0002"),
    u("Milligrams", "0.000001"),
    u("Micrograms", "0.000000001"),
];

pub const PRESSURE: &[Unit] = &[
    u("Atmospheres", "101325"),
    u("Bars", "100000"),
    u("PSI", "6894.757293168"),
    u("Millimeters of Mercury", "133.322387415"),
    u("Torr", "133.32236842105263"),
    u("Kilopascals", "1000"),
    u("Millibars", "100"),
    u("Pascals", "1"),
];

pub const SPEED: &[Unit] = &[
    u("Kilometers per Second", "1000"),
    u("Meters per Second", "1"),
    u("Knots", "0.5144444444444445"),
    u("Miles per Hour", "0.44704"),
    u("Feet per Second", "0.3048"),
    u("Kilometers per Hour", "0.2777777777777778"),
];

pub const VOLUME: &[Unit] = &[
    u("Cubic Meters", "1"),
    u("Cubic Feet", "0.028316846592"),
    u("Imperial Gallons", "0.00454609"),
    u("Gallons", "0.003785411784"),
    u("Liters", "0.001"),
    u("Quarts", "0.000946352946"),
    u("Imperial Pints", "0.00056826125"),
    u("Pints", "0.000473176473"),
    u("Cups", "0.0002365882365"),
    u("Fluid Ounces", "0.0000295735295625"),
    u("Imperial Fluid Ounces", "0.0000284130625"),
    u("Cubic Inches", "0.000016387064"),
    u("Tablespoons", "0.00001478676478125"),
    u("Teaspoons", "0.00000492892159375"),
    u("Milliliters", "0.000001"),
];

/// Affine temperature conversion to Kelvin (docs/06 §5: exact affine maps;
/// below-absolute-zero accepted without clamping).
pub fn temp_to_kelvin(v: f64, from: &str) -> f64 {
    match from {
        "Celsius" => v + 273.15,
        "Fahrenheit" => (v + 459.67) * 5.0 / 9.0,
        "Kelvin" => v,
        "Rankine" => v * 5.0 / 9.0,
        _ => v,
    }
}

pub fn temp_from_kelvin(v: f64, to: &str) -> f64 {
    match to {
        "Celsius" => v - 273.15,
        "Fahrenheit" => v * 9.0 / 5.0 - 459.67,
        "Kelvin" => v,
        "Rankine" => v * 9.0 / 5.0,
        _ => v,
    }
}
