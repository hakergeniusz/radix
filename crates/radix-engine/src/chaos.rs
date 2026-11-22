//! Opt-in chaos: flip a random bit in the expression before it is parsed.
//!
//! All randomness comes from the kernel's `/dev/urandom` — no PRNG seed is
//! stored in `Env`, and nothing is random while the setting is off. Each
//! evaluation draws one byte to decide *whether* to flip, then two more to
//! pick the byte and the bit within it. The flip frequency is user-configurable
//! through `Env::bit_flip_percent` (0–100%); the default of 12% keeps the
//! original ~1-in-8 behaviour. A flip that would produce invalid UTF-8 is
//! discarded and the original expression is evaluated.

use std::fs::File;
use std::io::Read;

/// Default flip frequency: closest whole percent to the legacy 32/256 (12.5%).
pub(crate) const DEFAULT_BIT_FLIP_PERCENT: u8 = 12;
/// Upper bound of the user-facing percentage.
pub(crate) const MAX_BIT_FLIP_PERCENT: u8 = 100;

/// Keep a user-supplied percentage inside the supported 0–100 range.
pub(crate) fn clamp_percent(percent: u8) -> u8 {
    percent.min(MAX_BIT_FLIP_PERCENT)
}

fn urandom_byte() -> Option<u8> {
    let mut f = File::open("/dev/urandom").ok()?;
    let mut b = [0u8; 1];
    f.read_exact(&mut b).ok()?;
    Some(b[0])
}

/// Map a percentage to a threshold in the 0..=256 range of a random byte.
///
/// `0` means "never flip", `256` means "always flip" (every `u8` draw is below
/// 256). Intermediate values are rounded to the nearest threshold.
fn flip_threshold(percent: u8) -> u16 {
    let p = u16::from(clamp_percent(percent));
    (p * 256 + 50) / 100
}

/// Whether a drawn `chance` byte triggers a flip at `percent` frequency.
pub(crate) fn should_flip(chance: u8, percent: u8) -> bool {
    u16::from(chance) < flip_threshold(percent)
}

/// Turn two random bytes into an in-range byte index and bit position.
pub(crate) fn positions(len: usize, byte_pick: u8, bit_pick: u8) -> Option<(usize, u8)> {
    if len == 0 {
        return None;
    }
    Some((byte_pick as usize % len, bit_pick % 8))
}

/// Flip bit `bit` of byte `idx`. `None` when the result is not valid UTF-8.
pub(crate) fn flip_at(bytes: &[u8], idx: usize, bit: u8) -> Option<String> {
    let original = *bytes.get(idx)?;
    let mut copy = bytes.to_vec();
    copy[idx] = original ^ (1u8 << bit);
    String::from_utf8(copy).ok()
}

/// The full flip decision with the randomness injected, so the path can be
/// tested without depending on `/dev/urandom`.
fn maybe_flip_with(
    expr: &str,
    percent: u8,
    chance: u8,
    byte_pick: u8,
    bit_pick: u8,
) -> Option<String> {
    if !should_flip(chance, percent) {
        return None;
    }
    let bytes = expr.as_bytes();
    let (idx, bit) = positions(bytes.len(), byte_pick, bit_pick)?;
    flip_at(bytes, idx, bit)
}

/// Return a bit-flipped copy of `expr` when the opt-in is on and the dice
/// say so; otherwise `None` (evaluate the untouched expression). A frequency
/// of 0 short-circuits before any `/dev/urandom` draw.
pub(crate) fn maybe_flip(expr: &str, enabled: bool, percent: u8) -> Option<String> {
    if !enabled || percent == 0 {
        return None;
    }
    maybe_flip_with(
        expr,
        percent,
        urandom_byte()?,
        urandom_byte()?,
        urandom_byte()?,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_percent_keeps_one_in_eight() {
        // 12% rounds to a threshold of 31/256 (~12.1%), the closest whole
        // percentage to the legacy hard-coded 32/256 = 12.5%.
        assert_eq!(flip_threshold(DEFAULT_BIT_FLIP_PERCENT), 31);
        assert!(should_flip(0, DEFAULT_BIT_FLIP_PERCENT));
        assert!(should_flip(30, DEFAULT_BIT_FLIP_PERCENT));
        assert!(!should_flip(31, DEFAULT_BIT_FLIP_PERCENT));
        assert!(!should_flip(255, DEFAULT_BIT_FLIP_PERCENT));
    }

    #[test]
    fn zero_percent_never_flips() {
        assert_eq!(flip_threshold(0), 0);
        for chance in u8::MIN..=u8::MAX {
            assert!(!should_flip(chance, 0), "chance {chance}");
        }
        assert_eq!(maybe_flip_with("2+2", 0, 0, 0, 0), None);
    }

    #[test]
    fn hundred_percent_always_flips() {
        assert_eq!(flip_threshold(100), 256);
        for chance in u8::MIN..=u8::MAX {
            assert!(should_flip(chance, 100), "chance {chance}");
        }
        // Even the highest draw flips; positions still decide what changes.
        assert_eq!(
            maybe_flip_with("2+2", 100, 255, 0, 0).as_deref(),
            Some("3+2")
        );
    }

    #[test]
    fn threshold_boundaries() {
        // Threshold = round(percent * 256 / 100).
        for (percent, threshold) in [(1, 3), (12, 31), (50, 128), (99, 253), (100, 256)] {
            assert_eq!(flip_threshold(percent), threshold, "percent {percent}");
            assert!(should_flip((threshold - 1) as u8, percent));
            if threshold < 256 {
                assert!(!should_flip(threshold as u8, percent));
            }
        }
    }

    #[test]
    fn out_of_range_percent_is_clamped() {
        assert_eq!(clamp_percent(200), MAX_BIT_FLIP_PERCENT);
        assert_eq!(flip_threshold(200), 256);
        assert!(should_flip(255, 200));
    }

    #[test]
    fn positions_are_always_in_range() {
        assert_eq!(positions(0, 9, 9), None);
        assert_eq!(positions(5, 7, 9), Some((2, 1)));
        assert_eq!(positions(256, 255, 16), Some((255, 0)));
    }

    #[test]
    fn flip_changes_exactly_one_bit() {
        // '2' is 0x32; flipping bit 0 gives 0x33 = '3'.
        let out = flip_at("2+2".as_bytes(), 0, 0).expect("valid UTF-8");
        assert_eq!(out, "3+2");
        assert_eq!(out.len(), 3);
        assert_eq!(out.as_bytes()[0] ^ b'2', 1);
    }

    #[test]
    fn invalid_utf8_flips_are_discarded() {
        // 0x32 ^ 0x80 = 0xB2, a stray lead byte before '+' → invalid UTF-8.
        assert_eq!(flip_at("2+2".as_bytes(), 0, 7), None);
    }

    #[test]
    fn injected_random_bytes_drive_the_whole_flip_path() {
        // chance=0 says flip at the default frequency; byte_pick=0, bit_pick=0
        // flip the first bit of the first byte: '2' → '3'.
        assert_eq!(
            maybe_flip_with("2+2", DEFAULT_BIT_FLIP_PERCENT, 0, 0, 0).as_deref(),
            Some("3+2")
        );
        // chance=200 says no flip at the default frequency.
        assert_eq!(
            maybe_flip_with("2+2", DEFAULT_BIT_FLIP_PERCENT, 200, 0, 0),
            None
        );
        // A draw that lands on a bit producing invalid UTF-8 is discarded.
        assert_eq!(
            maybe_flip_with("2+2", DEFAULT_BIT_FLIP_PERCENT, 0, 0, 7),
            None
        );
    }

    #[test]
    fn disabled_setting_never_draws_or_flips() {
        assert_eq!(maybe_flip("2+2", false, DEFAULT_BIT_FLIP_PERCENT), None);
        assert_eq!(maybe_flip("", false, DEFAULT_BIT_FLIP_PERCENT), None);
        // 0% short-circuits even when the opt-in is enabled.
        assert_eq!(maybe_flip("2+2", true, 0), None);
    }
}
