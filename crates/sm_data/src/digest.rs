//! The digest of the verification streams.
use core::hash::Hasher;

/// The machine streams of a Tick, in stream order.
pub const MACHINE: [&str; 8] = [
    "wram", "sram", "vram", "cgram", "oam", "ppu", "video", "apu",
];

/// A word-at-a-time 64-bit digest of bytes or `Hash` values. It detects
/// change; it is not meant to resist anyone choosing the input.
#[derive(Clone, Copy, Debug)]
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
            word[..chunk.len()].copy_from_slice(chunk);
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
    pub fn of(bytes: &[u8]) -> u64 {
        let mut digest = Self::default();
        digest.write(bytes);
        digest.finish()
    }
}

/// Fold one Tick's digest into a running digest.
pub fn fold(running: u64, value: u64) -> u64 {
    let mut digest = Digest(running);
    digest.write_u64(value);
    digest.finish()
}
