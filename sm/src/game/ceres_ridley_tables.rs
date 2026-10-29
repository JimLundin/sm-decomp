//! Ceres Ridley's bank $A6 tables through the imported `sm_data` records.
//!
//! Each accessor serves the record while the loaded image is the imported
//! allocation; any other image, and indices past a table, keep the original
//! indexed loads with their failures.
#![forbid(unsafe_code)]
use crate::game::imported::{Imported, ImportedGrid, ImportedWords};
use crate::game_state::Game;
use crate::rom::{RomImage, lorom_offset};
use sm_data::ceres_ridley::{
    ATTACK_ADDRESS, ATTACKS, BABY_METROID_FRAME_ADDRESS, BABY_METROID_FRAMES,
    BACKGROUND_FADE_ADDRESS, BACKGROUND_FADE_FRAMES, BODY_FADE_ADDRESS, BODY_FADE_COLOURS,
    EYE_FADE_ADDRESS, EYE_FADE_COLOURS, FADE_STEPS, GETAWAY_ADDRESS, GETAWAY_BYTES, GETAWAY_STEPS,
    Getaway, GetawayVelocity, WING_FRAME_ADDRESS, WING_FRAMES,
};

#[derive(Default)]
pub(crate) struct ImportedCeresRidley {
    background_fade: ImportedWords<BACKGROUND_FADE_ADDRESS, BACKGROUND_FADE_FRAMES>,
    attacks: ImportedWords<ATTACK_ADDRESS, ATTACKS>,
    baby_metroid_frames: ImportedWords<BABY_METROID_FRAME_ADDRESS, BABY_METROID_FRAMES>,
    wing_frames: ImportedWords<WING_FRAME_ADDRESS, WING_FRAMES>,
    getaway: Option<Imported<Getaway>>,
    eye_fade: ImportedGrid<EYE_FADE_ADDRESS, EYE_FADE_COLOURS, FADE_STEPS>,
    body_fade: ImportedGrid<BODY_FADE_ADDRESS, BODY_FADE_COLOURS, FADE_STEPS>,
}

pub(crate) fn import(image: &RomImage) -> ImportedCeresRidley {
    ImportedCeresRidley {
        background_fade: ImportedWords::import(image),
        attacks: ImportedWords::import(image),
        baby_metroid_frames: ImportedWords::import(image),
        wing_frames: ImportedWords::import(image),
        getaway: Imported::import(image, GETAWAY_ADDRESS, GETAWAY_BYTES, Getaway::import),
        eye_fade: ImportedGrid::import(image),
        body_fade: ImportedGrid::import(image),
    }
}

fn rom(game: &Game) -> &RomImage {
    game.loaded_rom.as_ref().expect("ROM must be loaded")
}

/// $A6:A4EB: the colour data for background fade-in `frame`, zero when done.
pub(crate) fn background_fade(game: &Game, frame: isize) -> u16 {
    game.ceres_ridley_tables
        .background_fade
        .word(rom(game), frame)
}

/// $A6:A743: the hovering function chosen by `choice`.
pub(crate) fn attack(game: &Game, choice: isize) -> u16 {
    game.ceres_ridley_tables.attacks.word(rom(game), choice)
}

/// $A6:ACDA: the baby metroid's tile transfers for animation `frame`.
pub(crate) fn baby_metroid_frame(game: &Game, frame: isize) -> u16 {
    game.ceres_ridley_tables
        .baby_metroid_frames
        .word(rom(game), frame)
}

/// $A6:AD45: the wings' tile transfers for animation `frame`.
pub(crate) fn wing_frame(game: &Game, frame: isize) -> u16 {
    game.ceres_ridley_tables.wing_frames.word(rom(game), frame)
}

/// $A6:AE4D: getaway step `index`'s zoom level, $FFFF once the getaway ends.
pub(crate) fn getaway_zoom(game: &Game, index: isize) -> u16 {
    let rom = rom(game);
    usize::try_from(index)
        .ok()
        .and_then(|index| Imported::from(&game.ceres_ridley_tables.getaway, rom)?.zoom(index))
        .unwrap_or_else(|| rom.table_word(lorom_offset(GETAWAY_ADDRESS) as usize, index))
}

/// $A6:AF2F and $A6:B00F: getaway step `index`'s Y and X velocities.
pub(crate) fn getaway_velocity(game: &Game, index: isize) -> GetawayVelocity {
    let rom = rom(game);
    usize::try_from(index)
        .ok()
        .and_then(|index| Imported::from(&game.ceres_ridley_tables.getaway, rom)?.velocity(index))
        .unwrap_or_else(|| {
            let list = |column: usize| {
                let start =
                    GETAWAY_ADDRESS as usize + 2 * (GETAWAY_STEPS + 1 + GETAWAY_STEPS * column);
                rom.table_word(lorom_offset(start as u32) as usize, index)
            };
            GetawayVelocity {
                y: list(0),
                x: list(1),
            }
        })
}

/// $A6:E2AA: the eye fade-in colour at word `index`.
pub(crate) fn eye_fade(game: &Game, index: isize) -> u16 {
    game.ceres_ridley_tables.eye_fade.word(rom(game), index)
}

/// $A6:E30A: the body fade-in colour at word `index`.
pub(crate) fn body_fade(game: &Game, index: isize) -> u16 {
    game.ceres_ridley_tables.body_fade.word(rom(game), index)
}
