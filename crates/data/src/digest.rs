//! The digest of the verification streams, shared by the oracle's recorder and
//! the engine's verifier so both sides compute the same marks (docs/ORACLE.md, RELOCK §2).
//!
//! Every committed lock digest depends on these constants. They were moved
//! bit for bit from the old engine (git tag `archive/old-engine`).

use core::hash::Hasher;

/// The machine streams of a Tick in the format-1 lock, in stream order.
pub const MACHINE: [&str; 8] = [
    "wram", "sram", "vram", "cgram", "oam", "ppu", "video", "apu",
];

/// A word-at-a-time 64-bit digest of bytes or `Hash` values. It detects
/// change; it is not meant to resist anyone choosing the input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Digest(pub u64);

impl Default for Digest {
    fn default() -> Self {
        Self(0xcbf2_9ce4_8422_2325)
    }
}

impl Hasher for Digest {
    fn write(&mut self, bytes: &[u8]) {
        for chunk in bytes.chunks(8) {
            let mut word = [0; 8];
            word.iter_mut()
                .zip(chunk)
                .for_each(|(slot, byte)| *slot = *byte);
            self.0 = (self.0.rotate_left(23) ^ u64::from_le_bytes(word))
                .wrapping_mul(0x9e37_79b9_7f4a_7c15);
        }
    }
    fn finish(&self) -> u64 {
        self.0
    }
}

impl Digest {
    /// The digest of `bytes`.
    #[must_use]
    pub fn of(bytes: &[u8]) -> u64 {
        let mut digest = Self::default();
        digest.write(bytes);
        digest.finish()
    }
}

/// Fold one value's digest into a running digest.
#[must_use]
pub fn fold(running: u64, value: u64) -> u64 {
    let mut digest = Digest(running);
    digest.write_u64(value);
    digest.finish()
}

/// The fold of `values` from the empty digest: a span's mark from its
/// cuts' digests, and a row's digest from its marks (docs/ORACLE.md, Q1).
#[must_use]
pub fn fold_all(values: impl IntoIterator<Item = u64>) -> u64 {
    values.into_iter().fold(Digest::default().0, fold)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every committed lock digest depends on this exact function, so a
    /// change to it must fail here, not only on the ROM rows.
    #[test]
    fn the_digest_is_the_one_the_lock_recorded() {
        assert_eq!(Digest::of(b""), 0xcbf2_9ce4_8422_2325);
        let text = Digest::of(b"sm-decomp digest");
        assert_eq!(text, 0x619b_8bdf_b4a8_3537);
        assert_eq!(fold(text, 7), 0xf546_e3f9_213e_d8ea);
        assert_eq!(
            fold_all([text, 7]),
            fold(fold(Digest::default().0, text), 7)
        );
    }
}
