//! Cinematic shooting-star records through the imported `sm_data` table.
#![forbid(unsafe_code)]
use crate::game::imported::Imported;
use crate::game_state::Game;
use crate::rom::{RomImage, lorom_offset};
pub(crate) use sm_data::shooting_stars::ShootingStar;
use sm_data::shooting_stars::{ADDRESS, BYTES, ShootingStars};

pub(crate) type ImportedShootingStars = Imported<ShootingStars>;

pub(crate) fn import(image: &RomImage) -> Option<ImportedShootingStars> {
    Imported::import(image, ADDRESS, BYTES, ShootingStars::import)
}

/// Star `index`'s record in the loaded image. The table serves it while that
/// image is the imported allocation; otherwise, and past the 40 stars, the
/// original four indexed loads run with their failures.
pub(crate) fn star(game: &Game, index: usize) -> ShootingStar {
    let rom = game.loaded_rom.as_ref().expect("ROM must be loaded");
    Imported::from(&game.shooting_stars, rom)
        .and_then(|stars| stars.star(index))
        .unwrap_or_else(|| {
            let word = |at: usize| {
                rom.table_word(lorom_offset(ADDRESS) as usize, (4 * index + at) as isize)
            };
            ShootingStar {
                x_acceleration: word(0),
                y_acceleration: word(1),
                frame_delay: word(2),
                start_delay: word(3),
            }
        })
}
