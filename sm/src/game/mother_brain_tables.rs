//! Mother Brain's bank $A9/$AD tables through the imported `sm_data` records.
//!
//! Each accessor serves the record while the loaded image is the imported
//! allocation; any other image, and indices outside a table, keep the original
//! indexed loads with their failures.
#![forbid(unsafe_code)]
use crate::game::imported::{Imported, load, rom, row};
use crate::game_state::Game;
use crate::rom::RomImage;
use sm_data::mother_brain::*;

#[derive(Default)]
pub(crate) struct ImportedMotherBrain {
    fake_death_explosions: Option<Imported<FakeDeathExplosions>>,
    brain_attacks: Option<Imported<BrainAttacks>>,
    brain_palette_sets: Option<Imported<BrainPaletteSets>>,
    walk_forwards: Option<Imported<Walks>>,
    walk_backwards: Option<Imported<Walks>>,
    falling_tubes: Option<Imported<FallingTubes>>,
    rainbow_beam_palettes: Option<Imported<RainbowBeamPalettes>>,
}

pub(crate) fn import(image: &RomImage) -> ImportedMotherBrain {
    ImportedMotherBrain {
        fake_death_explosions: Imported::import(
            image,
            FAKE_DEATH_EXPLOSION_ADDRESS,
            FAKE_DEATH_EXPLOSION_BYTES,
            import_fake_death_explosions,
        ),
        brain_attacks: Imported::import(
            image,
            BRAIN_ATTACK_ADDRESS,
            BRAIN_ATTACK_BYTES,
            import_brain_attacks,
        ),
        brain_palette_sets: Imported::import(
            image,
            BRAIN_PALETTE_SET_ADDRESS,
            BRAIN_PALETTE_SET_BYTES,
            import_brain_palette_sets,
        ),
        walk_forwards: Imported::import(image, WALK_FORWARDS_ADDRESS, WALK_BYTES, import_walks),
        walk_backwards: Imported::import(image, WALK_BACKWARDS_ADDRESS, WALK_BYTES, import_walks),
        falling_tubes: Imported::import(
            image,
            FALLING_TUBE_ADDRESS,
            FALLING_TUBE_BYTES,
            import_falling_tubes,
        ),
        rainbow_beam_palettes: Imported::import(
            image,
            RAINBOW_BEAM_PALETTE_ADDRESS,
            RAINBOW_BEAM_PALETTE_BYTES,
            import_rainbow_beam_palettes,
        ),
    }
}

/// Explosion `index` of the fake death at $A9:8929.
pub(crate) fn fake_death_explosion(game: &Game, index: isize) -> ExplosionPosition {
    let rom = rom(game);
    row(&game.mother_brain_tables.fake_death_explosions, rom, index).unwrap_or_else(|| {
        ExplosionPosition {
            x: load(rom, FAKE_DEATH_EXPLOSION_ADDRESS, 2 * index),
            y: load(rom, FAKE_DEATH_EXPLOSION_ADDRESS, 2 * index + 1),
        }
    })
}

/// Brain attack `index` at $A9:B6D4.
pub(crate) fn brain_attack(game: &Game, index: isize) -> BrainAttack {
    let rom = rom(game);
    row(&game.mother_brain_tables.brain_attacks, rom, index).unwrap_or_else(|| BrainAttack {
        instruction_list: load(rom, BRAIN_ATTACK_ADDRESS, index),
    })
}

/// Brain palette set `index` at $A9:D260.
pub(crate) fn brain_palette_set(game: &Game, index: isize) -> BrainPaletteSet {
    let rom = rom(game);
    row(&game.mother_brain_tables.brain_palette_sets, rom, index).unwrap_or_else(|| {
        BrainPaletteSet {
            colors: load(rom, BRAIN_PALETTE_SET_ADDRESS, index),
        }
    })
}

/// Forwards walk `index` at $A9:C61E.
pub(crate) fn walk_forwards(game: &Game, index: isize) -> Walk {
    let rom = rom(game);
    row(&game.mother_brain_tables.walk_forwards, rom, index).unwrap_or_else(|| Walk {
        instruction_list: load(rom, WALK_FORWARDS_ADDRESS, index),
    })
}

/// Backwards walk `index` at $A9:C664.
pub(crate) fn walk_backwards(game: &Game, index: isize) -> Walk {
    let rom = rom(game);
    row(&game.mother_brain_tables.walk_backwards, rom, index).unwrap_or_else(|| Walk {
        instruction_list: load(rom, WALK_BACKWARDS_ADDRESS, index),
    })
}

/// Falling tube `index` at $A9:8B7B.
pub(crate) fn falling_tube(game: &Game, index: isize) -> FallingTube {
    let rom = rom(game);
    row(&game.mother_brain_tables.falling_tubes, rom, index).unwrap_or_else(|| FallingTube {
        function: load(rom, FALLING_TUBE_ADDRESS, index),
    })
}

/// Rainbow beam palette `index` at $AD:E434.
pub(crate) fn rainbow_beam_palette(game: &Game, index: isize) -> RainbowBeamPalette {
    let rom = rom(game);
    row(&game.mother_brain_tables.rainbow_beam_palettes, rom, index).unwrap_or_else(|| {
        RainbowBeamPalette {
            colors: load(rom, RAINBOW_BEAM_PALETTE_ADDRESS, index),
        }
    })
}
