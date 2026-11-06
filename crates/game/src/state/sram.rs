//! The cartridge's battery-backed save RAM.
//!
//! It holds three save files, each vouched for by a checksum and its complement, kept twice
//! (bank $81 log, `$81:8000` saves a file, `$81:8085` loads one, `$81:812B` places them).
//!
//! The image is the battery file's byte layout read as little-endian words, in order: the
//! checks, the three files, a stretch the save code never touches, and the checks' mirror.
//! Slice 0 neither saves nor loads, so a file's contents stay opaque words here.

use canonical::Canonical;

/// The save RAM's size in bytes.
pub const SAVE_RAM_BYTES: usize = 0x2000;

/// The save files.
pub const SAVE_FILES: usize = 3;

/// A save file's words: the copy of the file's work RAM the save code takes.
pub const SAVE_FILE_WORDS: usize = 0x32e;

/// The image's words.
const SAVE_RAM_WORDS: usize = SAVE_RAM_BYTES / 2;

/// Words in one copy of the checks: a sum and a complement per file and a fourth pair, so
/// each column is four words wide.
const CHECKS_WORDS: usize = 2 * (SAVE_FILES + 1);

/// The words between the last file and the mirrored checks.
pub const UNUSED_WORDS: usize = SAVE_RAM_WORDS - 2 * CHECKS_WORDS - SAVE_FILES * SAVE_FILE_WORDS;

/// One save file's words, as the save code copies them out of work RAM.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SaveFile(pub [u16; SAVE_FILE_WORDS]);

impl Default for SaveFile {
    fn default() -> Self {
        Self([0; SAVE_FILE_WORDS])
    }
}

impl SaveFile {
    /// The file's checksum: its words summed, wrapping (`$81:805A`).
    #[must_use]
    pub fn checksum(&self) -> u16 {
        self.0.iter().fold(0, |sum, word| sum.wrapping_add(*word))
    }
}

/// A file's check: its checksum and the checksum's complement.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Check {
    /// The checksum the save stored.
    pub sum: u16,
    /// Its complement.
    pub complement: u16,
}

impl Check {
    /// The check the save code writes for `file`.
    #[must_use]
    pub fn of(file: &SaveFile) -> Self {
        let sum = file.checksum();
        Self {
            sum,
            complement: !sum,
        }
    }

    /// Whether this check vouches for `file`: the load accepts a file when either copy of its
    /// check matches (`$81:80B9`, then the mirror at `$81:80CC`).
    #[must_use]
    pub fn vouches_for(self, file: &SaveFile) -> bool {
        self == Self::of(file)
    }
}

/// One copy of the checks.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Checks {
    /// Each file's check.
    pub files: [Check; SAVE_FILES],
    /// The fourth pair of each column, which no save writes; kept so the image round-trips.
    pub spare: Check,
}

impl Checks {
    /// The checks' words in image order: the sums column, then the complements column.
    fn words(&self) -> impl Iterator<Item = u16> + '_ {
        let checks = || self.files.iter().chain([&self.spare]);
        checks()
            .map(|check| check.sum)
            .chain(checks().map(|check| check.complement))
    }

    fn read(words: &mut impl Iterator<Item = u16>) -> Self {
        let sums: [u16; SAVE_FILES + 1] = take(words);
        let complements: [u16; SAVE_FILES + 1] = take(words);
        let check = |index: usize| Check {
            sum: sums.get(index).copied().unwrap_or_default(),
            complement: complements.get(index).copied().unwrap_or_default(),
        };
        Self {
            files: std::array::from_fn(check),
            spare: check(SAVE_FILES),
        }
    }
}

/// The words no save or load touches.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Unused(pub [u16; UNUSED_WORDS]);

impl Default for Unused {
    fn default() -> Self {
        Self([0; UNUSED_WORDS])
    }
}

/// The save RAM.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SaveRam {
    /// The checks.
    pub checks: Checks,
    /// The files.
    pub files: [SaveFile; SAVE_FILES],
    /// What lies between the files and the mirror.
    pub unused: Unused,
    /// The checks again, which the load falls back on when the first copy doesn't match.
    pub mirror: Checks,
}

/// The next `N` words; an exhausted iterator gives zeros, which the fixed image size rules out.
fn take<const N: usize>(words: &mut impl Iterator<Item = u16>) -> [u16; N] {
    std::array::from_fn(|_| words.next().unwrap_or_default())
}

impl SaveRam {
    /// The save RAM a battery image holds.
    #[must_use]
    pub fn from_bytes(bytes: &[u8; SAVE_RAM_BYTES]) -> Self {
        let mut words = bytes
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| u16::from_le_bytes(*pair));
        let checks = Checks::read(&mut words);
        let files = std::array::from_fn(|_| SaveFile(take(&mut words)));
        let unused = Unused(take(&mut words));
        let mirror = Checks::read(&mut words);
        Self {
            checks,
            files,
            unused,
            mirror,
        }
    }

    /// The battery image, byte for byte as [`Self::from_bytes`] read it.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; SAVE_RAM_BYTES] {
        let words = self
            .checks
            .words()
            .chain(self.files.iter().flat_map(|file| file.0))
            .chain(self.unused.0)
            .chain(self.mirror.words());
        let mut bytes = [0; SAVE_RAM_BYTES];
        bytes
            .as_chunks_mut::<2>()
            .0
            .iter_mut()
            .zip(words)
            .for_each(|(pair, word)| *pair = word.to_le_bytes());
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::{Check, SAVE_FILE_WORDS, SAVE_RAM_BYTES, SaveFile, SaveRam, UNUSED_WORDS};

    fn image() -> [u8; SAVE_RAM_BYTES] {
        let mut bytes = [0; SAVE_RAM_BYTES];
        bytes.iter_mut().zip(0_u32..).for_each(|(byte, index)| {
            *byte = (index.wrapping_mul(7) ^ (index >> 8)).to_le_bytes()[0]
        });
        bytes
    }

    #[test]
    fn the_image_round_trips() {
        let bytes = image();
        assert_eq!(SaveRam::from_bytes(&bytes).to_bytes(), bytes);
    }

    #[test]
    fn the_layout_is_checks_files_unused_mirror() {
        assert_eq!(UNUSED_WORDS, 1638);
        let mut bytes = [0; SAVE_RAM_BYTES];
        // The sums column's first word, the complements column's (four words on), the first
        // file's first word (eight words on) and the second file's.
        bytes[0] = 1;
        bytes[8] = 2;
        bytes[16] = 3;
        bytes[16 + 2 * SAVE_FILE_WORDS] = 4;
        bytes[SAVE_RAM_BYTES - 16] = 5;
        bytes[SAVE_RAM_BYTES - 2] = 6;
        let ram = SaveRam::from_bytes(&bytes);
        assert_eq!(
            ram.checks.files[0],
            Check {
                sum: 1,
                complement: 2
            }
        );
        assert_eq!(ram.files[0].0[0], 3);
        assert_eq!(ram.files[1].0[0], 4);
        assert_eq!(ram.mirror.files[0].sum, 5);
        assert_eq!(ram.mirror.spare.complement, 6);
    }

    #[test]
    fn a_saved_files_check_vouches_for_it() {
        let mut file = SaveFile::default();
        file.0[0] = 0xfff0;
        file.0[1] = 0x0020;
        assert_eq!(file.checksum(), 0x0010);
        let check = Check::of(&file);
        assert_eq!(
            check,
            Check {
                sum: 0x0010,
                complement: 0xffef
            }
        );
        assert!(check.vouches_for(&file));
        // Erased RAM's zero complement doesn't vouch for a zero sum.
        assert!(!Check::default().vouches_for(&SaveFile::default()));
    }
}
