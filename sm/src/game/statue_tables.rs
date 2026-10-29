//! The statues' bank $AA tables through the imported `sm_data` records.
//!
//! Each accessor serves the record while the loaded image is the imported
//! allocation; any other image, and indices outside a table, keep the original
//! indexed loads with their failures.
#![forbid(unsafe_code)]
use crate::game::imported::{Imported, load, rom, row};
use crate::game_state::Game;
use crate::rom::RomImage;
use sm_data::statues::*;

#[derive(Default)]
pub(crate) struct ImportedStatues {
    tourian: Option<Imported<TourianStatues>>,
    chozo: Option<Imported<ChozoStatues>>,
    movements: Option<Imported<Movements>>,
}

pub(crate) fn import(image: &RomImage) -> ImportedStatues {
    ImportedStatues {
        tourian: Imported::import(
            image,
            TOURIAN_STATUE_ADDRESS,
            TOURIAN_STATUE_BYTES,
            import_tourian_statues,
        ),
        chozo: Imported::import(
            image,
            CHOZO_STATUE_ADDRESS,
            CHOZO_STATUE_BYTES,
            import_chozo_statues,
        ),
        movements: Imported::import(image, MOVEMENT_ADDRESS, MOVEMENT_BYTES, import_movements),
    }
}

/// Tourian entrance statue `index` at $AA:D810.
pub(crate) fn tourian_statue(game: &Game, index: isize) -> Statue {
    let rom = rom(game);
    row(&game.statue_tables.tourian, rom, index).unwrap_or_else(|| Statue {
        instruction_list: load(rom, TOURIAN_STATUE_ADDRESS, index),
    })
}

/// Chozo statue `index` at $AA:E7A2.
pub(crate) fn chozo_statue(game: &Game, index: isize) -> Statue {
    let rom = rom(game);
    row(&game.statue_tables.chozo, rom, index).unwrap_or_else(|| Statue {
        instruction_list: load(rom, CHOZO_STATUE_ADDRESS, index),
    })
}

/// Chozo statue movement `index` at $AA:E630/$AA:E670/$AA:E6B0.
pub(crate) fn movement(game: &Game, index: isize) -> Movement {
    let rom = rom(game);
    row(&game.statue_tables.movements, rom, index).unwrap_or_else(|| Movement {
        x_velocity: load(rom, MOVEMENT_ADDRESS, index),
        samus_x_offset: load(rom, SAMUS_X_OFFSET_ADDRESS, index),
        samus_y_offset: load(rom, SAMUS_Y_OFFSET_ADDRESS, index),
    })
}
