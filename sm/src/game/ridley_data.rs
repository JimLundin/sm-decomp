//! Ridley's bank $A6 tables through the imported `sm_data` records.
//!
//! Each accessor serves the record while the loaded image is the imported
//! allocation; any other image, and indices past a table, keep the original
//! indexed loads with their failures.
#![forbid(unsafe_code)]
use crate::game::imported::{Imported, ImportedGrid, ImportedWords};
use crate::game_state::Game;
use crate::rom::{RomImage, lorom_offset};
use sm_data::ridley::{
    EMERGENCY_GLOW_ADDRESS, EMERGENCY_GLOW_COLOURS, EMERGENCY_GLOW_STEPS, FACINGS, FEET_DISTANCES,
    FEET_X_ADDRESS, FEET_Y_ADDRESS, HOVER_BEHIND_ADDRESS, HOVER_IN_FRONT_ADDRESS,
    LUNGE_ACCELERATION_ADDRESS, LUNGE_ACCELERATIONS, LUNGE_SIDE_ADDRESS, POGO_TARGET_ADDRESS,
    TAIL_ANGLE_ADDRESS, TAIL_ANGLE_BYTES, TAIL_TIP_SPRITEMAP_ADDRESS, TAIL_TIP_SPRITEMAPS,
    TailAngleLimits, TailAngles, WING_SPRITEMAP_ADDRESS, WING_SPRITEMAPS, import_tail_angles,
};

#[derive(Default)]
pub(crate) struct ImportedRidley {
    hover_behind: ImportedWords<HOVER_BEHIND_ADDRESS, FACINGS>,
    hover_in_front: ImportedWords<HOVER_IN_FRONT_ADDRESS, FACINGS>,
    pogo_target: ImportedWords<POGO_TARGET_ADDRESS, FACINGS>,
    feet_x: ImportedWords<FEET_X_ADDRESS, FACINGS>,
    feet_y: ImportedWords<FEET_Y_ADDRESS, FEET_DISTANCES>,
    lunge_side: ImportedWords<LUNGE_SIDE_ADDRESS, FACINGS>,
    lunge_acceleration: ImportedWords<LUNGE_ACCELERATION_ADDRESS, LUNGE_ACCELERATIONS>,
    tail_angles: Option<Imported<TailAngles>>,
    wing_spritemaps: ImportedWords<WING_SPRITEMAP_ADDRESS, WING_SPRITEMAPS>,
    tail_tip_spritemaps: ImportedWords<TAIL_TIP_SPRITEMAP_ADDRESS, TAIL_TIP_SPRITEMAPS>,
    emergency_glow:
        ImportedGrid<EMERGENCY_GLOW_ADDRESS, EMERGENCY_GLOW_COLOURS, EMERGENCY_GLOW_STEPS>,
}

pub(crate) fn import(image: &RomImage) -> ImportedRidley {
    ImportedRidley {
        hover_behind: ImportedWords::import(image),
        hover_in_front: ImportedWords::import(image),
        pogo_target: ImportedWords::import(image),
        feet_x: ImportedWords::import(image),
        feet_y: ImportedWords::import(image),
        lunge_side: ImportedWords::import(image),
        lunge_acceleration: ImportedWords::import(image),
        tail_angles: Imported::import(
            image,
            TAIL_ANGLE_ADDRESS,
            TAIL_ANGLE_BYTES,
            import_tail_angles,
        ),
        wing_spritemaps: ImportedWords::import(image),
        tail_tip_spritemaps: ImportedWords::import(image),
        emergency_glow: ImportedGrid::import(image),
    }
}

fn rom(game: &Game) -> &RomImage {
    game.loaded_rom.as_ref().expect("ROM must be loaded")
}

/// $A6:B60D: the hover target X behind Ridley for `facing`.
pub(crate) fn hover_behind(game: &Game, facing: isize) -> u16 {
    game.ridley_tables.hover_behind.word(rom(game), facing)
}

/// $A6:B63B: the hover target X in front of Ridley for `facing`.
pub(crate) fn hover_in_front(game: &Game, facing: isize) -> u16 {
    game.ridley_tables.hover_in_front.word(rom(game), facing)
}

/// $A6:B6C8: the pogo's target X for `facing`.
pub(crate) fn pogo_target(game: &Game, facing: isize) -> u16 {
    game.ridley_tables.pogo_target.word(rom(game), facing)
}

/// $A6:B9D5: the feet's X offset for `facing`.
pub(crate) fn feet_x(game: &Game, facing: isize) -> u16 {
    game.ridley_tables.feet_x.word(rom(game), facing)
}

/// $A6:B9DB: the feet's Y offset at word `index`.
pub(crate) fn feet_y(game: &Game, index: isize) -> u16 {
    game.ridley_tables.feet_y.word(rom(game), index)
}

/// $A6:BB48: the lunge's facing sign for `facing`.
pub(crate) fn lunge_side(game: &Game, facing: isize) -> u16 {
    game.ridley_tables.lunge_side.word(rom(game), facing)
}

/// $A6:BB4E: the lunge's acceleration factor for acceleration `index`.
pub(crate) fn lunge_acceleration(game: &Game, index: isize) -> u16 {
    game.ridley_tables.lunge_acceleration.word(rom(game), index)
}

/// $A6:CC12 and $A6:CC18: the tail's angle limits for `facing`.
pub(crate) fn tail_angles(game: &Game, facing: isize) -> TailAngleLimits {
    let rom = rom(game);
    usize::try_from(facing)
        .ok()
        .and_then(|row| Imported::from(&game.ridley_tables.tail_angles, rom)?.row(row))
        .unwrap_or_else(|| {
            let list = |column: u32| {
                rom.table_word(
                    lorom_offset(TAIL_ANGLE_ADDRESS + column * 2 * FACINGS as u32) as usize,
                    facing,
                )
            };
            TailAngleLimits {
                min_clockwise: list(0),
                max_anticlockwise: list(1),
            }
        })
}

/// $A6:DB02: the wing spritemap at word `index`.
pub(crate) fn wing_spritemap(game: &Game, index: isize) -> u16 {
    game.ridley_tables.wing_spritemaps.word(rom(game), index)
}

/// $A6:DCBA: the tail tip spritemap for angle `sector`.
pub(crate) fn tail_tip_spritemap(game: &Game, sector: isize) -> u16 {
    game.ridley_tables
        .tail_tip_spritemaps
        .word(rom(game), sector)
}

/// $A6:C1DF: the emergency text glow colour at word `index`.
pub(crate) fn emergency_glow(game: &Game, index: isize) -> u16 {
    game.ridley_tables.emergency_glow.word(rom(game), index)
}
