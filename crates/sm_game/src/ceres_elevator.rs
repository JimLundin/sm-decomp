//! The Ceres elevator rotation through the imported `sm_data` table.
#![forbid(unsafe_code)]
use crate::imported::Imported;
use crate::records::Records;
use sm_data::cartridge::{RomImage, lorom_offset};
use sm_data::ceres_elevator::{ADDRESS, BYTES, Rotation, RotationStep};

pub type ImportedRotation = Imported<Rotation>;

pub fn import(image: &RomImage) -> Option<ImportedRotation> {
    Imported::import(image, ADDRESS, BYTES, Rotation::import)
}

/// The step starting at word index `word` from $89:AD5F in the loaded image.
/// The table serves whole steps of the imported allocation; any other image or
/// index keeps the original three indexed loads with their failures.
pub fn step(game: &(impl Records + ?Sized), word: isize) -> RotationStep {
    let rom = game.loaded_image();
    usize::try_from(word)
        .ok()
        .filter(|word| word % 3 == 0)
        .and_then(|word| Imported::from(&game.records().ceres_rotation, rom)?.step(word / 3))
        .unwrap_or_else(|| {
            let load = |at: isize| rom.table_word(lorom_offset(ADDRESS) as usize, word + at);
            RotationStep {
                frames: load(0),
                mode7_b: load(1),
                mode7_a: load(2),
            }
        })
}
