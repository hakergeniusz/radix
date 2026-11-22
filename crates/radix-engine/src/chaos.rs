//! Opt-in chaos: flip a random bit in the expression before it is parsed.
//!
//! All randomness comes from the kernel's `/dev/urandom` — no PRNG seed is
//! stored in `Env`, and nothing is random while the setting is off. Each
//! evaluation draws one byte to decide *whether* to flip (1 in 8), then two
//! more to pick the byte and the bit within it. A flip that would produce
//! invalid UTF-8 is discarded and the original expression is evaluated.

use std::fs::File;
use std::io::Read;

/// One in eight evaluations is flipped (`chance < 32` out of 256).
const FLIP_CHANCE: u8 = 32;

fn urandom_byte() -> Option<u8> {
    let mut f = File::open("/dev/urandom").ok()?;
    let mut b = [0u8; 1];
    f.read_exact(&mut b).ok()?;
    Some(b[0])
}

/// Whether a drawn `chance` byte triggers a flip.
pub(crate) fn should_flip(chance: u8) -> bool {
    chance < FLIP_CHANCE
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
fn maybe_flip_with(expr: &str, chance: u8, byte_pick: u8, bit_pick: u8) -> Option<String> {
    if !should_flip(chance) {
        return None;
    }
    let bytes = expr.as_bytes();
    let (idx, bit) = positions(bytes.len(), byte_pick, bit_pick)?;
    flip_at(bytes, idx, bit)
}

/// Return a bit-flipped copy of `expr` when the opt-in is on and the dice
/// say so; otherwise `None` (evaluate the untouched expression).
pub(crate) fn maybe_flip(expr: &str, enabled: bool) -> Option<String> {
    if !enabled {
        return None;
    }
    maybe_flip_with(
        expr,
        urandom_byte()?,
        urandom_byte()?,
        urandom_byte()?,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chance_threshold_is_one_in_eight() {
        assert!(should_flip(0));
        assert!(should_flip(31));
        assert!(!should_flip(32));
        assert!(!should_flip(255));
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
        // chance=0 says flip; byte_pick=0, bit_pick=0 flip the first bit of
        // the first byte: '2' → '3'.
        assert_eq!(maybe_flip_with("2+2", 0, 0, 0).as_deref(), Some("3+2"));
        // chance=200 says no flip, whatever the position bytes are.
        assert_eq!(maybe_flip_with("2+2", 200, 0, 0), None);
        // A draw that lands on a bit producing invalid UTF-8 is discarded.
        assert_eq!(maybe_flip_with("2+2", 0, 0, 7), None);
    }

    #[test]
    fn disabled_setting_never_draws_or_flips() {
        assert_eq!(maybe_flip("2+2", false), None);
        assert_eq!(maybe_flip("", false), None);
    }
}
