//! Currency data: static ISO 4217 catalog (code, name, symbol) and a live
//! rates provider with a bundled snapshot fallback (decision D5).
//!
//! The spec (docs/06 §6) observes the upstream's IMF-sourced rates; Radix
//! fetches from a free public endpoint and labels the source honestly.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use gtk::gio;
use gtk::glib;

/// One ISO 4217 currency: code, name, symbol (symbol may be empty).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Currency {
    pub code: &'static str,
    pub name: &'static str,
    pub symbol: &'static str,
}

pub const CURRENCIES: &[Currency] = &[
    Currency { code: "AED", name: "UAE Dirham", symbol: "د.إ" },
    Currency { code: "AFN", name: "Afghan Afghani", symbol: "؋" },
    Currency { code: "ALL", name: "Albanian Lek", symbol: "L" },
    Currency { code: "AMD", name: "Armenian Dram", symbol: "֏" },
    Currency { code: "ANG", name: "Netherlands Antillean Guilder", symbol: "ƒ" },
    Currency { code: "AOA", name: "Angolan Kwanza", symbol: "Kz" },
    Currency { code: "ARS", name: "Argentine Peso", symbol: "$" },
    Currency { code: "AUD", name: "Australian Dollar", symbol: "$" },
    Currency { code: "AWG", name: "Aruban Florin", symbol: "ƒ" },
    Currency { code: "AZN", name: "Azerbaijani Manat", symbol: "₼" },
    Currency { code: "BAM", name: "Bosnia-Herzegovina Convertible Mark", symbol: "KM" },
    Currency { code: "BBD", name: "Barbadian Dollar", symbol: "$" },
    Currency { code: "BDT", name: "Bangladeshi Taka", symbol: "৳" },
    Currency { code: "BGN", name: "Bulgarian Lev", symbol: "лв" },
    Currency { code: "BHD", name: "Bahraini Dinar", symbol: "ب.د" },
    Currency { code: "BIF", name: "Burundian Franc", symbol: "FBu" },
    Currency { code: "BMD", name: "Bermudian Dollar", symbol: "$" },
    Currency { code: "BND", name: "Brunei Dollar", symbol: "$" },
    Currency { code: "BOB", name: "Bolivian Boliviano", symbol: "Bs." },
    Currency { code: "BRL", name: "Brazilian Real", symbol: "R$" },
    Currency { code: "BSD", name: "Bahamian Dollar", symbol: "$" },
    Currency { code: "BTN", name: "Bhutanese Ngultrum", symbol: "Nu." },
    Currency { code: "BWP", name: "Botswana Pula", symbol: "P" },
    Currency { code: "BYN", name: "Belarusian Ruble", symbol: "Br" },
    Currency { code: "BZD", name: "Belize Dollar", symbol: "$" },
    Currency { code: "CAD", name: "Canadian Dollar", symbol: "$" },
    Currency { code: "CDF", name: "Congolese Franc", symbol: "FC" },
    Currency { code: "CHF", name: "Swiss Franc", symbol: "Fr" },
    Currency { code: "CLP", name: "Chilean Peso", symbol: "$" },
    Currency { code: "CNY", name: "Chinese Yuan", symbol: "¥" },
    Currency { code: "COP", name: "Colombian Peso", symbol: "$" },
    Currency { code: "CRC", name: "Costa Rican Colón", symbol: "₡" },
    Currency { code: "CUP", name: "Cuban Peso", symbol: "$" },
    Currency { code: "CVE", name: "Cape Verdean Escudo", symbol: "$" },
    Currency { code: "CZK", name: "Czech Koruna", symbol: "Kč" },
    Currency { code: "DJF", name: "Djiboutian Franc", symbol: "Fdj" },
    Currency { code: "DKK", name: "Danish Krone", symbol: "kr" },
    Currency { code: "DOP", name: "Dominican Peso", symbol: "$" },
    Currency { code: "DZD", name: "Algerian Dinar", symbol: "د.ج" },
    Currency { code: "EGP", name: "Egyptian Pound", symbol: "E£" },
    Currency { code: "ERN", name: "Eritrean Nakfa", symbol: "Nfk" },
    Currency { code: "ETB", name: "Ethiopian Birr", symbol: "Br" },
    Currency { code: "EUR", name: "Euro", symbol: "€" },
    Currency { code: "FJD", name: "Fijian Dollar", symbol: "$" },
    Currency { code: "FKP", name: "Falkland Islands Pound", symbol: "£" },
    Currency { code: "GBP", name: "British Pound Sterling", symbol: "£" },
    Currency { code: "GEL", name: "Georgian Lari", symbol: "₾" },
    Currency { code: "GHS", name: "Ghanaian Cedi", symbol: "₵" },
    Currency { code: "GIP", name: "Gibraltar Pound", symbol: "£" },
    Currency { code: "GMD", name: "Gambian Dalasi", symbol: "D" },
    Currency { code: "GNF", name: "Guinean Franc", symbol: "FG" },
    Currency { code: "GTQ", name: "Guatemalan Quetzal", symbol: "Q" },
    Currency { code: "GYD", name: "Guyanese Dollar", symbol: "$" },
    Currency { code: "HKD", name: "Hong Kong Dollar", symbol: "$" },
    Currency { code: "HNL", name: "Honduran Lempira", symbol: "L" },
    Currency { code: "HRK", name: "Croatian Kuna", symbol: "kn" },
    Currency { code: "HTG", name: "Haitian Gourde", symbol: "G" },
    Currency { code: "HUF", name: "Hungarian Forint", symbol: "Ft" },
    Currency { code: "IDR", name: "Indonesian Rupiah", symbol: "Rp" },
    Currency { code: "ILS", name: "Israeli New Shekel", symbol: "₪" },
    Currency { code: "INR", name: "Indian Rupee", symbol: "₹" },
    Currency { code: "IQD", name: "Iraqi Dinar", symbol: "ع.د" },
    Currency { code: "IRR", name: "Iranian Rial", symbol: "﷼" },
    Currency { code: "ISK", name: "Icelandic Króna", symbol: "kr" },
    Currency { code: "JMD", name: "Jamaican Dollar", symbol: "$" },
    Currency { code: "JOD", name: "Jordanian Dinar", symbol: "د.ا" },
    Currency { code: "JPY", name: "Japanese Yen", symbol: "¥" },
    Currency { code: "KES", name: "Kenyan Shilling", symbol: "KSh" },
    Currency { code: "KGS", name: "Kyrgystani Som", symbol: "с" },
    Currency { code: "KHR", name: "Cambodian Riel", symbol: "៛" },
    Currency { code: "KMF", name: "Comorian Franc", symbol: "CF" },
    Currency { code: "KPW", name: "North Korean Won", symbol: "₩" },
    Currency { code: "KRW", name: "South Korean Won", symbol: "₩" },
    Currency { code: "KWD", name: "Kuwaiti Dinar", symbol: "د.ك" },
    Currency { code: "KYD", name: "Cayman Islands Dollar", symbol: "$" },
    Currency { code: "KZT", name: "Kazakhstani Tenge", symbol: "₸" },
    Currency { code: "LAK", name: "Laotian Kip", symbol: "₭" },
    Currency { code: "LBP", name: "Lebanese Pound", symbol: "ل.ل" },
    Currency { code: "LKR", name: "Sri Lankan Rupee", symbol: "₨" },
    Currency { code: "LRD", name: "Liberian Dollar", symbol: "$" },
    Currency { code: "LSL", name: "Lesotho Loti", symbol: "L" },
    Currency { code: "LYD", name: "Libyan Dinar", symbol: "ل.د" },
    Currency { code: "MAD", name: "Moroccan Dirham", symbol: "د.م." },
    Currency { code: "MDL", name: "Moldovan Leu", symbol: "L" },
    Currency { code: "MGA", name: "Malagasy Ariary", symbol: "Ar" },
    Currency { code: "MKD", name: "Macedonian Denar", symbol: "ден" },
    Currency { code: "MMK", name: "Myanmar Kyat", symbol: "K" },
    Currency { code: "MNT", name: "Mongolian Tugrik", symbol: "₮" },
    Currency { code: "MOP", name: "Macanese Pataca", symbol: "$" },
    Currency { code: "MRU", name: "Mauritanian Ouguiya", symbol: "UM" },
    Currency { code: "MUR", name: "Mauritian Rupee", symbol: "₨" },
    Currency { code: "MVR", name: "Maldivian Rufiyaa", symbol: "Rf" },
    Currency { code: "MWK", name: "Malawian Kwacha", symbol: "MK" },
    Currency { code: "MXN", name: "Mexican Peso", symbol: "$" },
    Currency { code: "MYR", name: "Malaysian Ringgit", symbol: "RM" },
    Currency { code: "MZN", name: "Mozambican Metical", symbol: "MT" },
    Currency { code: "NAD", name: "Namibian Dollar", symbol: "$" },
    Currency { code: "NGN", name: "Nigerian Naira", symbol: "₦" },
    Currency { code: "NIO", name: "Nicaraguan Córdoba", symbol: "C$" },
    Currency { code: "NOK", name: "Norwegian Krone", symbol: "kr" },
    Currency { code: "NPR", name: "Nepalese Rupee", symbol: "₨" },
    Currency { code: "NZD", name: "New Zealand Dollar", symbol: "$" },
    Currency { code: "OMR", name: "Omani Rial", symbol: "ر.ع." },
    Currency { code: "PAB", name: "Panamanian Balboa", symbol: "B/." },
    Currency { code: "PEN", name: "Peruvian Sol", symbol: "S/" },
    Currency { code: "PGK", name: "Papua New Guinean Kina", symbol: "K" },
    Currency { code: "PHP", name: "Philippine Peso", symbol: "₱" },
    Currency { code: "PKR", name: "Pakistani Rupee", symbol: "₨" },
    Currency { code: "PLN", name: "Polish Zloty", symbol: "zł" },
    Currency { code: "PYG", name: "Paraguayan Guarani", symbol: "₲" },
    Currency { code: "QAR", name: "Qatari Riyal", symbol: "ر.ق" },
    Currency { code: "RON", name: "Romanian Leu", symbol: "lei" },
    Currency { code: "RSD", name: "Serbian Dinar", symbol: "дин" },
    Currency { code: "RUB", name: "Russian Ruble", symbol: "₽" },
    Currency { code: "RWF", name: "Rwandan Franc", symbol: "FRw" },
    Currency { code: "SAR", name: "Saudi Riyal", symbol: "ر.س" },
    Currency { code: "SBD", name: "Solomon Islands Dollar", symbol: "$" },
    Currency { code: "SCR", name: "Seychellois Rupee", symbol: "₨" },
    Currency { code: "SDG", name: "Sudanese Pound", symbol: "ج.س." },
    Currency { code: "SEK", name: "Swedish Krona", symbol: "kr" },
    Currency { code: "SGD", name: "Singapore Dollar", symbol: "$" },
    Currency { code: "SHP", name: "Saint Helena Pound", symbol: "£" },
    Currency { code: "SLE", name: "Sierra Leonean Leone", symbol: "Le" },
    Currency { code: "SOS", name: "Somali Shilling", symbol: "Sh" },
    Currency { code: "SRD", name: "Surinamese Dollar", symbol: "$" },
    Currency { code: "SSP", name: "South Sudanese Pound", symbol: "£" },
    Currency { code: "STN", name: "São Tomé & Príncipe Dobra", symbol: "Db" },
    Currency { code: "SVC", name: "Salvadoran Colón", symbol: "₡" },
    Currency { code: "SYP", name: "Syrian Pound", symbol: "£S" },
    Currency { code: "SZL", name: "Swazi Lilangeni", symbol: "L" },
    Currency { code: "THB", name: "Thai Baht", symbol: "฿" },
    Currency { code: "TJS", name: "Tajikistani Somoni", symbol: "SM" },
    Currency { code: "TMT", name: "Turkmenistani Manat", symbol: "m" },
    Currency { code: "TND", name: "Tunisian Dinar", symbol: "د.ت" },
    Currency { code: "TOP", name: "Tongan Paʻanga", symbol: "T$" },
    Currency { code: "TRY", name: "Turkish Lira", symbol: "₺" },
    Currency { code: "TTD", name: "Trinidad & Tobago Dollar", symbol: "$" },
    Currency { code: "TWD", name: "New Taiwan Dollar", symbol: "$" },
    Currency { code: "TZS", name: "Tanzanian Shilling", symbol: "TSh" },
    Currency { code: "UAH", name: "Ukrainian Hryvnia", symbol: "₴" },
    Currency { code: "UGX", name: "Ugandan Shilling", symbol: "USh" },
    Currency { code: "USD", name: "US Dollar", symbol: "$" },
    Currency { code: "UYU", name: "Uruguayan Peso", symbol: "$" },
    Currency { code: "UZS", name: "Uzbekistani Som", symbol: "сум" },
    Currency { code: "VES", name: "Venezuelan Bolívar", symbol: "Bs." },
    Currency { code: "VND", name: "Vietnamese Dong", symbol: "₫" },
    Currency { code: "VUV", name: "Vanuatu Vatu", symbol: "VT" },
    Currency { code: "WST", name: "Samoan Tala", symbol: "T" },
    Currency { code: "XAF", name: "Central African CFA Franc", symbol: "FCFA" },
    Currency { code: "XCD", name: "East Caribbean Dollar", symbol: "$" },
    Currency { code: "XOF", name: "West African CFA Franc", symbol: "CFA" },
    Currency { code: "XPF", name: "CFP Franc", symbol: "₣" },
    Currency { code: "YER", name: "Yemeni Rial", symbol: "﷼" },
    Currency { code: "ZAR", name: "South African Rand", symbol: "R" },
    Currency { code: "ZMW", name: "Zambian Kwacha", symbol: "ZK" },
    Currency { code: "ZWL", name: "Zimbabwean Dollar", symbol: "$" },
];

// ---- Currency-conversion preferences (docs/02 §3, docs/16 §6) ----

/// How often live exchange rates are refreshed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefreshInterval {
    Never,
    Daily,
    Weekly,
    Monthly,
}

impl RefreshInterval {
    pub fn label(self) -> &'static str {
        match self {
            RefreshInterval::Never => "Never",
            RefreshInterval::Daily => "Daily",
            RefreshInterval::Weekly => "Weekly",
            RefreshInterval::Monthly => "Monthly",
        }
    }
    pub fn from_label(s: &str) -> Self {
        match s {
            "Never" => RefreshInterval::Never,
            "Daily" => RefreshInterval::Daily,
            "Monthly" => RefreshInterval::Monthly,
            _ => RefreshInterval::Weekly,
        }
    }
    /// Staleness threshold in hours (Never → u64::MAX).
    pub fn hours(self) -> u64 {
        match self {
            RefreshInterval::Never => u64::MAX,
            RefreshInterval::Daily => 24,
            RefreshInterval::Weekly => 24 * 7,
            RefreshInterval::Monthly => 24 * 30,
        }
    }
}

/// How a currency is named in lists (docs/02 §3: "Currency Display").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrencyDisplay {
    Name,
    Code,
}

/// Which currencies appear in the auto-completion list (docs/02 §3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrencyShow {
    All,
    Favorites,
}

/// Defaults per the owner evidence: Weekly / Currency Name / All.
#[derive(Debug, Clone, Copy)]
pub struct CurrencyPrefs {
    pub interval: RefreshInterval,
    pub display: CurrencyDisplay,
    pub show: CurrencyShow,
}

impl Default for CurrencyPrefs {
    fn default() -> Self {
        Self {
            interval: RefreshInterval::Weekly,
            display: CurrencyDisplay::Name,
            show: CurrencyShow::All,
        }
    }
}

/// Shared between the window (preferences UI, refresh scheduling) and the
/// entry controller (completion filtering). Starred codes persist via
/// docs/09 §4 state.
#[derive(Default)]
pub struct CurrencyPrefState {
    pub prefs: RefCell<CurrencyPrefs>,
    pub favorites: RefCell<Vec<String>>,
}

pub type SharedCurrencyPrefs = std::rc::Rc<CurrencyPrefState>;

/// The full currency catalog (for the favorites dialog).
pub fn all() -> &'static [Currency] {
    CURRENCIES
}

pub fn by_code(code: &str) -> Option<Currency> {
    CURRENCIES.iter().find(|c| c.code == code).copied()
}

/// Codes whose name starts with the given prefix (for the completion
/// popover, docs/08 §6: matches on the currency NAME, listing codes with a
/// detail line "Name - symbol").
pub fn matching(prefix: &str) -> Vec<Currency> {
    let p = prefix.to_lowercase();
    CURRENCIES
        .iter()
        .filter(|c| c.name.to_lowercase().starts_with(&p) || c.code.to_lowercase().starts_with(&p))
        .copied()
        .collect()
}

// ---------------------------------------------------------------------------
// Rates provider
// ---------------------------------------------------------------------------

pub const PROVIDER_LABEL: &str = "ExchangeRate-API (open.er-api.com)";
const RATES_URL: &str = "https://open.er-api.com/v6/latest/USD";

/// One rate snapshot: USD-based rates + update timestamp (unix seconds).
#[derive(Debug, Clone, Default)]
pub struct Rates {
    pub usd_based: BTreeMap<String, f64>,
    pub updated_unix: i64,
}

impl Rates {
    /// Rate for `from` → `to` (via USD base), None if either code unknown.
    pub fn convert(&self, from: &str, to: &str, amount: f64) -> Option<f64> {
        let rf = if from == "USD" { 1.0 } else { *self.usd_based.get(from)? };
        let rt = if to == "USD" { 1.0 } else { *self.usd_based.get(to)? };
        if rf == 0.0 {
            return None;
        }
        Some(amount / rf * rt)
    }
}

pub fn fetch_blocking() -> Result<Rates, String> {
    let resp = ureq::get(RATES_URL)
        .timeout(std::time::Duration::from_secs(10))
        .call()
        .map_err(|e| format!("network: {e}"))?;
    let json: serde_json::Value = resp.into_json().map_err(|e| format!("json: {e}"))?;
    let updated_unix = json
        .get("time_last_update_unix")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let mut rates = Rates { updated_unix, ..Rates::default() };
    if let Some(obj) = json.get("rates").and_then(|v| v.as_object()) {
        for (code, val) in obj {
            if let Some(v) = val.as_f64() {
                rates.usd_based.insert(code.clone(), v);
            }
        }
    }
    if rates.usd_based.is_empty() {
        return Err("no rates in response".into());
    }
    Ok(rates)
}

/// Shared, GUI-thread rates cache with a bundled fallback snapshot.
pub struct RatesCache {
    pub rates: RefCell<Option<Rates>>,
    pub error: RefCell<Option<String>>,
    /// When the live rates last landed (for the refresh-interval pref).
    pub fetched_at: std::cell::Cell<Option<std::time::Instant>>,
}

pub type SharedRates = Rc<RatesCache>;

impl RatesCache {
    pub fn new() -> SharedRates {
        Rc::new(RatesCache {
            rates: RefCell::new(None),
            error: RefCell::new(None),
            fetched_at: std::cell::Cell::new(None),
        })
    }

    /// Fetch on gio's thread pool, then apply to the cache on the main
    /// context. `done` runs on the main thread after the fetch settles.
    pub fn refresh_async(self: &SharedRates, done: impl FnOnce(bool) + 'static) {
        let shared = self.clone();
        glib::spawn_future_local(async move {
            let result = match gio::spawn_blocking(fetch_blocking).await {
                Ok(r) => r,
                Err(_) => Err("background rate fetch failed".to_string()),
            };
            match result {
                Ok(r) => {
                    *shared.rates.borrow_mut() = Some(r);
                    *shared.error.borrow_mut() = None;
                    shared
                        .fetched_at
                        .set(Some(std::time::Instant::now()));
                    done(true);
                }
                Err(e) => {
                    *shared.error.borrow_mut() = Some(e);
                    done(false);
                }
            }
        });
    }
}

/// Bundled snapshot used until the first successful refresh (approximate
/// mid-2026 USD rates; replaced by live data as soon as a refresh succeeds).
pub fn fallback_rates() -> Rates {
    let mut r = Rates { updated_unix: 0, ..Rates::default() };
    for (code, rate) in [
        ("USD", 1.0), ("EUR", 0.92), ("GBP", 0.78), ("JPY", 155.0), ("CNY", 7.1),
        ("CHF", 0.88), ("CAD", 1.36), ("AUD", 1.5), ("NZD", 1.63), ("SEK", 10.5),
        ("NOK", 10.6), ("DKK", 6.85), ("PLN", 3.95), ("CZK", 22.8), ("HUF", 360.0),
        ("RON", 4.58), ("BGN", 1.8), ("TRY", 34.0), ("RUB", 92.0), ("UAH", 41.0),
        ("INR", 83.5), ("IDR", 15900.0), ("MYR", 4.45), ("PHP", 57.5), ("SGD", 1.33),
        ("THB", 35.5), ("VND", 25300.0), ("KRW", 1360.0), ("HKD", 7.8), ("TWD", 32.0),
        ("BRL", 5.75), ("MXN", 19.6), ("ARS", 1000.0), ("CLP", 950.0), ("COP", 4050.0),
        ("PEN", 3.78), ("ZAR", 18.2), ("NGN", 1600.0), ("EGP", 48.5), ("MAD", 9.9),
        ("DZD", 135.5), ("TND", 3.12), ("AED", 3.6725), ("SAR", 3.75), ("QAR", 3.64),
        ("KWD", 0.307), ("BHD", 0.376), ("OMR", 0.3845), ("JOD", 0.709), ("ILS", 3.7),
        ("PKR", 278.0), ("BDT", 119.0), ("LKR", 300.0), ("NPR", 133.6), ("IRR", 42000.0),
        ("IQD", 1310.0), ("LYD", 4.85), ("ETB", 120.0), ("KES", 129.0), ("TZS", 2650.0),
        ("UGX", 3700.0), ("GHS", 15.5), ("XOF", 604.0), ("XAF", 604.0),
    ] {
        r.usd_based.insert(code.to_string(), rate);
    }
    r
}
