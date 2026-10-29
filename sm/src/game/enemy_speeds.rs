//! Enemy speed table reads through the imported `sm_data` block.
#![forbid(unsafe_code)]
use crate::game::imported::Imported;
use crate::rom::{RomImage, lorom_offset};
pub(crate) use sm_data::enemy_speeds::SpeedTable;
use sm_data::enemy_speeds::{ADDRESS, ADDRESS32, BYTES, BYTES32, EnemySpeeds, Speeds32};

pub(crate) type ImportedSpeeds = Imported<EnemySpeeds>;
pub(crate) type ImportedSpeeds32 = Imported<Speeds32>;

/// The linear and quadratic tables; reads of other images keep the loaded-image path.
pub(crate) fn import(image: &RomImage) -> Option<ImportedSpeeds> {
    Imported::import(image, ADDRESS, BYTES, EnemySpeeds::import)
}

/// The 32-bit table at $A0:CBC7.
pub(crate) fn import32(image: &RomImage) -> Option<ImportedSpeeds32> {
    Imported::import(image, ADDRESS32, BYTES32, Speeds32::import)
}

/// One word at a signed word index from the table's start. The block serves it
/// only while `rom` is the imported allocation; otherwise, and outside the
/// block, the original indexed load runs with its failures.
pub(crate) fn word(
    imported: &Option<ImportedSpeeds>,
    rom: &Option<RomImage>,
    table: SpeedTable,
    index: isize,
) -> u16 {
    let rom = rom.as_ref().expect("ROM must be loaded");
    Imported::from(imported, rom)
        .and_then(|speeds| speeds.word(table, index))
        .unwrap_or_else(|| rom.table_word(lorom_offset(table.address()) as usize, index))
}

/// One native-order word at a byte offset from $A0:CBC7 in `native`. The table
/// serves it only while `native` is the imported allocation; otherwise, and
/// past the table, the original four byte reads run with their failures.
pub(crate) fn word32(imported: &Option<ImportedSpeeds32>, native: &RomImage, offset: usize) -> u32 {
    Imported::from(imported, native)
        .and_then(|speeds| speeds.word32_at(offset))
        .unwrap_or_else(|| {
            let start = lorom_offset(ADDRESS32) as usize + offset;
            u32::from_ne_bytes(core::array::from_fn(|i| native.read(start + i)))
        })
}

/// Borrow only the speed and ROM fields: callers sit inside slot writes.
macro_rules! enemy_speed {
    ($game:expr, $table:ident, $index:expr $(,)?) => {
        $crate::game::enemy_speeds::word(
            &$game.enemy_speeds,
            &$game.loaded_rom,
            $crate::game::enemy_speeds::SpeedTable::$table,
            $index,
        )
    };
}
pub(crate) use enemy_speed;
