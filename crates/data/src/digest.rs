//! The word-at-a-time digest the content fingerprint is computed with
//! (`assets::fingerprint`).
//!
//! Every stored fingerprint depends on these constants. They were moved
//! bit for bit from the old engine (commit `09486a2fd`).

use core::hash::Hasher;

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

#[cfg(test)]
mod tests {
    use super::*;

    /// Every stored fingerprint depends on this exact function, so a change to it must
    /// fail here.
    #[test]
    fn the_digest_is_the_one_the_fingerprints_were_taken_with() {
        let mut digest = Digest::default();
        assert_eq!(digest.finish(), 0xcbf2_9ce4_8422_2325);
        digest.write(b"sm-decomp digest");
        assert_eq!(digest.finish(), 0x619b_8bdf_b4a8_3537);
        digest.write_u64(7);
        assert_eq!(digest.finish(), 0xf546_e3f9_213e_d8ea);
    }
}
