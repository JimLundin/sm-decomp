//! Diagnosis of lock mismatches: the old frame stepper's kept ROM patches,
//! applied to a copy of the cartridge only to test whether a mismatch with the
//! existing lock is theirs. The oracle never runs with them
//! (`OracleId::patches` stays empty); a diagnosed run is not a record.

/// One kept patch of the old stepper (`tools/snesrev_patches.py`), as bytes.
#[derive(Clone, Copy, Debug)]
pub struct KeptPatch {
    pub name: &'static str,
    /// Why the old stepper kept it.
    pub reason: &'static str,
    pub writes: &'static [(u32, &'static [u8])],
}

/// The kept patches a diagnosis can apply.
pub const KEPT_PATCHES: [KeptPatch; 2] = [
    KeptPatch {
        name: "native-audio-handshake",
        reason: "$82:8A59/8A72/8A80: the sound state machine assumes the APU acknowledges at once",
        writes: &[
            (0x82_8a59, &[0x18, 0x18, 0x18, 0x80]),
            (0x82_8a72, &[0x18, 0x18, 0x18]),
            (0x82_8a80, &[0x18, 0x18, 0x18, 0x80]),
        ],
    },
    KeptPatch {
        name: "debug-enemy-set-names",
        reason: "$A0:896F: RTL skips the debug routine writing enemy-set names to $7E:D552-D5F1",
        writes: &[(0xa0_896f, &[0x6b])],
    },
];

/// The LoROM file offset of a CPU address in banks $80-$FF.
fn offset(address: u32) -> usize {
    usize::try_from(((address >> 16) & 0x7f) << 15 | (address & 0x7fff)).unwrap_or(usize::MAX)
}

/// Write `patch` into the cartridge image `rom`.
pub fn apply(rom: &mut [u8], patch: &KeptPatch) {
    for &(address, bytes) in patch.writes {
        let at = offset(address);
        rom[at..at + bytes.len()].copy_from_slice(bytes);
    }
}
