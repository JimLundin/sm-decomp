//! Botwoon's holes and speed levels in bank $B3 through the imported `sm_data` records.
//!
//! Each accessor serves the record while the loaded image is the imported
//! allocation; any other image or index keeps the original indexed loads with
//! their failures.
use crate::imported::Imported;
use crate::records::Records;
use sm_data::botwoon_holes::{self, BotwoonHoles, Hole};
use sm_data::botwoon_speeds::{self, BotwoonSpeed, BotwoonSpeeds};
use sm_data::cartridge::{RomImage, lorom_offset};

#[derive(Default)]
pub struct ImportedBotwoon {
    speeds: Option<Imported<BotwoonSpeeds>>,
    holes: Option<Imported<BotwoonHoles>>,
}

pub fn import(image: &RomImage) -> ImportedBotwoon {
    ImportedBotwoon {
        speeds: Imported::import(
            image,
            botwoon_speeds::ADDRESS,
            botwoon_speeds::BYTES,
            BotwoonSpeeds::import,
        ),
        holes: Imported::import(
            image,
            botwoon_holes::ADDRESS,
            botwoon_holes::BYTES,
            BotwoonHoles::import,
        ),
    }
}

/// The word at signed word `index` of the table at `address`.
fn table_word(rom: &RomImage, address: u32, index: isize) -> u16 {
    rom.table_word(lorom_offset(address) as usize, index)
}

/// The speed level whose two words start at word `index` of $B3:94BB.
pub fn speed_level(game: &(impl Records + ?Sized), index: isize) -> BotwoonSpeed {
    let rom = game.loaded_image();
    Imported::from(&game.records().botwoon_tables.speeds, rom)
        .filter(|_| index % 2 == 0)
        .and_then(|speeds| speeds.level(usize::try_from(index / 2).ok()?))
        .unwrap_or_else(|| BotwoonSpeed {
            speed: table_word(rom, botwoon_speeds::ADDRESS, index),
            body_travel_time: table_word(rom, botwoon_speeds::ADDRESS, index + 1),
        })
}

/// The hole whose four words start at word `index` of $B3:949B.
pub fn hole(game: &(impl Records + ?Sized), index: isize) -> Hole {
    let rom = game.loaded_image();
    Imported::from(&game.records().botwoon_tables.holes, rom)
        .filter(|_| index % 4 == 0)
        .and_then(|holes| holes.hole(usize::try_from(index / 4).ok()?))
        .unwrap_or_else(|| Hole {
            left: table_word(rom, botwoon_holes::ADDRESS, index),
            right: table_word(rom, botwoon_holes::ADDRESS, index + 1),
            top: table_word(rom, botwoon_holes::ADDRESS, index + 2),
            bottom: table_word(rom, botwoon_holes::ADDRESS, index + 3),
        })
}
