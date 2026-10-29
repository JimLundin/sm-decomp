//! The zebetites' bank $A6 tables through the imported `sm_data` records.
//!
//! Each accessor serves the record while the loaded image is the imported
//! allocation; any other image, and indices past a table, keep the original
//! indexed loads with their failures.
#![forbid(unsafe_code)]
use crate::game::imported::Imported;
use crate::game_state::Game;
use crate::rom::{RomImage, lorom_offset};
use sm_data::zebetites::{
    HEALTH_LIST_ADDRESS, HEALTH_LIST_BYTES, HEALTH_STAGES, HealthLists, HealthStages,
    PLACEMENT_ADDRESS, PLACEMENT_BYTES, PLACEMENTS, Placement, Placements, import_health_lists,
    import_placements,
};

#[derive(Default)]
pub(crate) struct ImportedZebetites {
    placements: Option<Imported<Placements>>,
    health_lists: Option<Imported<HealthStages>>,
}

pub(crate) fn import(image: &RomImage) -> ImportedZebetites {
    ImportedZebetites {
        placements: Imported::import(image, PLACEMENT_ADDRESS, PLACEMENT_BYTES, import_placements),
        health_lists: Imported::import(
            image,
            HEALTH_LIST_ADDRESS,
            HEALTH_LIST_BYTES,
            import_health_lists,
        ),
    }
}

fn rom(game: &Game) -> &RomImage {
    game.loaded_rom.as_ref().expect("ROM must be loaded")
}

/// Word `index` of the list `column` lists of `rows` words after `address`.
fn load(rom: &RomImage, address: u32, rows: usize, column: u32, index: isize) -> u16 {
    rom.table_word(
        lorom_offset(address + column * 2 * rows as u32) as usize,
        index,
    )
}

/// $A6:FC03: the placement for `destroyed` zebetites.
pub(crate) fn placement(game: &Game, destroyed: isize) -> Placement {
    let rom = rom(game);
    usize::try_from(destroyed)
        .ok()
        .and_then(|row| Imported::from(&game.zebetite_tables.placements, rom)?.row(row))
        .unwrap_or_else(|| {
            let list = |column| load(rom, PLACEMENT_ADDRESS, PLACEMENTS, column, destroyed);
            Placement {
                multi_part: list(0),
                height: list(1),
                instruction_list: list(2),
                x_position: list(3),
                y_position: list(4),
                y_position_alternate: list(5),
            }
        })
}

/// $A6:FD4A and $A6:FD54: the instruction lists for health `stage`.
pub(crate) fn health_lists(game: &Game, stage: isize) -> HealthLists {
    let rom = rom(game);
    usize::try_from(stage)
        .ok()
        .and_then(|row| Imported::from(&game.zebetite_tables.health_lists, rom)?.row(row))
        .unwrap_or_else(|| {
            let list = |column| load(rom, HEALTH_LIST_ADDRESS, HEALTH_STAGES, column, stage);
            HealthLists {
                big: list(0),
                small_pair: list(1),
            }
        })
}
