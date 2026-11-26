//! The cartridge's image layout and the ROM-supplied save format.

mod run;

use data::rom::{AddressError, Lorom, RomAddress};
use enum_map::EnumMap;
use save_files::SaveFormat;
use types::file_select::Slot::{First, Second, Third};
use types::save_files::{
    BatteryImage, Cartridge, CheckPair, CompletionMark, EncodedRun, MARK_BYTES,
    RememberedSelection, RetainedBytes, RetainedCartridgeData, RunSnapshot, SlotRecord,
};

use crate::state::sram::{Check, Checks, SaveFile, SaveRam, Unused};

const MAP_AREAS: usize = 6;
const PACKED_BYTES: usize = 327;

/// The original cartridge format, including map projection order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Format {
    completed: CompletionMark,
    incomplete: CompletionMark,
    projection: [Vec<u8>; MAP_AREAS],
}

/// A missing or malformed save-format asset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImportFault {
    /// The ROM data is missing.
    Address(AddressError),
    /// The two marks are indistinguishable.
    CompletionMarks,
    /// The area lists do not fill the packed map.
    ProjectionLength,
}

impl Format {
    /// Read format data from the cartridge ROM.
    ///
    /// # Errors
    /// Returns an error for missing marks, pointers or byte lists, or an invalid total length.
    pub fn load(rom: Lorom<'_>) -> Result<Self, ImportFault> {
        let read = |bank, address, len| {
            rom.read(RomAddress::new(bank, address), len)
                .map_err(ImportFault::Address)
        };
        let mark = |address| {
            let mut mark = [0; MARK_BYTES];
            mark.copy_from_slice(read(0x80, address, MARK_BYTES)?);
            Ok(CompletionMark(mark))
        };
        let completed = mark(0x82b9)?;
        let incomplete = mark(0x82ad)?;
        if completed == incomplete {
            return Err(ImportFault::CompletionMarks);
        }
        let counts = read(0x81, 0x8131, MAP_AREAS)?;
        let pointers = read(0x81, 0x82d6, MAP_AREAS * 2)?;
        let mut projection = std::array::from_fn(|_| Vec::new());
        for ((list, &count), &pointer) in projection
            .iter_mut()
            .zip(counts)
            .zip(pointers.as_chunks::<2>().0)
        {
            *list = read(0x81, u16::from_le_bytes(pointer), usize::from(count))?.to_vec();
        }
        if projection.iter().map(Vec::len).sum::<usize>() != PACKED_BYTES {
            return Err(ImportFault::ProjectionLength);
        }
        Ok(Self {
            completed,
            incomplete,
            projection,
        })
    }
}

const fn pair(check: Check) -> CheckPair {
    CheckPair {
        sum: check.sum,
        complement: check.complement,
    }
}

const fn check(pair: CheckPair) -> Check {
    Check {
        sum: pair.sum,
        complement: pair.complement,
    }
}

impl SaveFormat for Format {
    fn completed_mark(&self) -> CompletionMark {
        self.completed
    }

    fn incomplete_mark(&self) -> CompletionMark {
        self.incomplete
    }

    fn encode(&self, run: &RunSnapshot) -> EncodedRun {
        self.encode_run(run)
    }

    fn decode(&self, payload: &EncodedRun) -> RunSnapshot {
        self.decode_run(payload)
    }

    fn cartridge(&self, image: &BatteryImage) -> Cartridge {
        let mut bytes = [0; types::save_files::IMAGE_BYTES];
        bytes.copy_from_slice(image.as_slice());
        let ram = SaveRam::from_bytes(&bytes);
        let [a, b, c] = ram.files;
        let [pa, pb, pc] = ram.checks.files;
        let [ma, mb, mc] = ram.mirror.files;
        let slots = EnumMap::from_fn(|slot| {
            let (file, primary, mirror) = match slot {
                First => (a, pa, ma),
                Second => (b, pb, mb),
                Third => (c, pc, mc),
            };
            let mut payload = EncodedRun::zeroed();
            payload.as_mut_slice().copy_from_slice(&file.0);
            SlotRecord {
                payload,
                primary: pair(primary),
                mirror: pair(mirror),
            }
        });
        let mut retained = RetainedBytes::zeroed();
        let mut mark = [0; MARK_BYTES];
        for (dst, word) in retained
            .as_mut_slice()
            .as_chunks_mut::<2>()
            .0
            .iter_mut()
            .chain(mark.as_chunks_mut::<2>().0)
            .zip(ram.unused.0)
        {
            *dst = word.to_le_bytes();
        }
        let [slot_word, check_word] = ram.unused.0.last_chunk::<2>().copied().unwrap_or_default();
        Cartridge {
            slots,
            selection: RememberedSelection {
                slot_word,
                check_word,
            },
            mark: CompletionMark(mark),
            retained: RetainedCartridgeData {
                bytes: retained,
                spare_words: [
                    ram.checks.spare.sum,
                    ram.checks.spare.complement,
                    ram.mirror.spare.sum,
                    ram.mirror.spare.complement,
                ],
            },
        }
    }

    fn image(&self, cartridge: &Cartridge) -> BatteryImage {
        let slots = [First, Second, Third].map(|slot| &cartridge.slots[slot]);
        let files = slots.map(|slot| {
            let mut file = SaveFile::default();
            file.0.copy_from_slice(slot.payload.as_slice());
            file
        });
        let primary = slots.map(|slot| check(slot.primary));
        let mirror = slots.map(|slot| check(slot.mirror));
        let mut unused = Unused::default();
        let words = cartridge
            .retained
            .bytes
            .as_slice()
            .as_chunks::<2>()
            .0
            .iter()
            .chain(cartridge.mark.0.as_chunks::<2>().0)
            .map(|bytes| u16::from_le_bytes(*bytes))
            .chain([
                cartridge.selection.slot_word,
                cartridge.selection.check_word,
            ]);
        for (out, word) in unused.0.iter_mut().zip(words) {
            *out = word;
        }
        let [sum, complement, mirror_sum, mirror_complement] = cartridge.retained.spare_words;
        let ram = SaveRam {
            checks: Checks {
                files: primary,
                spare: Check { sum, complement },
            },
            files,
            unused,
            mirror: Checks {
                files: mirror,
                spare: Check {
                    sum: mirror_sum,
                    complement: mirror_complement,
                },
            },
        };
        let mut image = BatteryImage::zeroed();
        image.as_mut_slice().copy_from_slice(&ram.to_bytes());
        image
    }
}

pub(crate) use run::fresh_words;
pub use run::{profile_bytes, profile_from_bytes, progress_from_bytes};

/// The platform's progress through the SRAM diagnostic before startup finishes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BootProgress {
    /// The descending clear sweep has reached this word.
    Clearing(u16),
    /// The descending mirror test has reached this word.
    Testing(u16),
    /// The descending backup restore has reached this word.
    Restoring(u16),
    /// Startup has cleared this many payload words in the consecutive invalid slots.
    ClearingFiles(u16),
}

impl BootProgress {
    /// Project the physical battery during the platform's in-progress operations.
    #[must_use]
    pub fn image(self, mut initial: BatteryImage) -> BatteryImage {
        let words = initial.as_mut_slice().as_chunks_mut::<2>().0;
        let limit = words.len();
        match self {
            Self::Clearing(first) => words
                .iter_mut()
                .skip(usize::from(first))
                .for_each(|word| *word = [0; 2]),
            Self::Testing(first) => {
                if let Some((cleared, testing)) = words.split_at_mut_checked(usize::from(first)) {
                    cleared.fill([0; 2]);
                    pattern(testing, testing.len());
                }
            }
            Self::Restoring(first) => {
                if let Some((testing, _)) = words.split_at_mut_checked(usize::from(first)) {
                    pattern(testing, limit);
                }
            }
            Self::ClearingFiles(count) => {
                let max = types::save_files::PAYLOAD_WORDS * 3;
                for word in words.iter_mut().skip(8).take(usize::from(count).min(max)) {
                    *word = [0; 2];
                }
            }
        }
        initial
    }
}

fn pattern(words: &mut [[u8; 2]], end: usize) {
    for (word, value) in words.iter_mut().zip((0..end).rev()) {
        *word = u16::try_from(value).unwrap_or_default().to_le_bytes();
    }
}

#[cfg(test)]
impl Format {
    pub(crate) fn test_format() -> Self {
        Self {
            completed: CompletionMark([1; MARK_BYTES]),
            incomplete: CompletionMark([2; MARK_BYTES]),
            projection: [
                vec![47, 12, 47],
                vec![33],
                Vec::new(),
                Vec::new(),
                Vec::new(),
                vec![0; 323],
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use types::Area;

    #[test]
    fn opaque_payload_and_battery_round_trip() {
        let format = Format::test_format();
        let image = BatteryImage::new(
            (0..types::save_files::IMAGE_BYTES)
                .map(|i| (i.wrapping_mul(7) ^ (i >> 4)).to_le_bytes()[0])
                .collect(),
        )
        .unwrap();
        assert_eq!(format.image(&format.cartridge(&image)), image);
        let payload = EncodedRun::new(
            (0..u16::try_from(types::save_files::PAYLOAD_WORDS).unwrap())
                .map(|word| word.wrapping_mul(9187).wrapping_add(39))
                .collect(),
        )
        .unwrap();
        assert_eq!(format.encode(&format.decode(&payload)), payload);
    }

    #[test]
    fn packed_maps_project_selected_cells_in_order_without_ceres() {
        let format = Format::test_format();
        let mut run = format.decode(&EncodedRun::zeroed());
        run.maps[Area::Crateria].as_mut_slice()[47] = 0x11;
        run.maps[Area::Crateria].as_mut_slice()[12] = 0x22;
        run.maps[Area::Crateria].as_mut_slice()[13] = 0x33;
        run.maps[Area::Brinstar].as_mut_slice()[33] = 0x44;
        run.maps[Area::Ceres].as_mut_slice()[47] = 0x55;
        let payload = format.encode(&run);
        assert_eq!(payload.as_slice()[174..176], [0x2211, 0x4411]);
        let decoded = format.decode(&payload);
        assert_eq!(decoded.maps[Area::Crateria].as_slice()[47], 0x11);
        assert_eq!(decoded.maps[Area::Crateria].as_slice()[13], 0);
        assert_eq!(decoded.maps[Area::Ceres].as_slice()[47], 0);
    }
}
