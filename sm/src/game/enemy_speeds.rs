//! Enemy speed table reads through the imported `sm_data` block.
#![forbid(unsafe_code)]
use crate::rom::{RomImage, lorom_offset};
pub(crate) use sm_data::enemy_speeds::SpeedTable;
use sm_data::enemy_speeds::{ADDRESS, BYTES, EnemySpeeds};

/// The validated block and the allocation it was imported from.
pub(crate) struct ImportedSpeeds {
    image: RomImage,
    speeds: EnemySpeeds,
}

impl ImportedSpeeds {
    /// `None` for images too short to hold the block or holding other content;
    /// their reads keep the original loaded-image path.
    pub(crate) fn import(image: &RomImage) -> Option<Self> {
        let start = lorom_offset(ADDRESS) as usize;
        let bytes = (start..start + BYTES)
            .map(|at| (at < image.len()).then(|| image.read(at)))
            .collect::<Option<Vec<u8>>>()?;
        let speeds = EnemySpeeds::import(&bytes).ok()?;
        Some(Self {
            image: image.clone(),
            speeds,
        })
    }
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
    imported
        .as_ref()
        .filter(|imported| imported.image.same_allocation(rom))
        .and_then(|imported| imported.speeds.word(table, index))
        .unwrap_or_else(|| rom.table_word(lorom_offset(table.address()) as usize, index))
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
