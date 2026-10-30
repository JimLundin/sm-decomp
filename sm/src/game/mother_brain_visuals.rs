//! Mother Brain encounter palettes, wall sealing, earthquakes and body flicker.
use crate::game::constants::bank_86::*;
use crate::game::enemy_projectiles::spawn_with_room_gfx;
use crate::game::music::queue_sfx2_max6;
use crate::game::prelude::*;
use crate::game::room_mechanisms::spawn_hardcoded;
use crate::ram_map::{checked_word, slot};
use sm_data::mother_brain::{
    BACK_LEG_PALETTES_ADDRESS, BODY_PALETTES_ADDRESS, HealthPalettes, PALETTES_BYTES,
    import_health_palettes,
};
use sm_game::actors::mother_brain::{MotherBrainBody, MotherBrainBrain};
use sm_game::imported::{Imported, word};
use sm_game::palette_copies::copy_rom_colors_to_palette;

#[derive(Default)]
pub(crate) struct ImportedTables {
    body_palettes: Option<Imported<HealthPalettes>>,
    back_leg_palettes: Option<Imported<HealthPalettes>>,
}

pub(crate) fn import(image: &crate::rom::RomImage) -> ImportedTables {
    ImportedTables {
        body_palettes: Imported::import(
            image,
            BODY_PALETTES_ADDRESS,
            PALETTES_BYTES,
            import_health_palettes,
        ),
        back_leg_palettes: Imported::import(
            image,
            BACK_LEG_PALETTES_ADDRESS,
            PALETTES_BYTES,
            import_health_palettes,
        ),
    }
}
const RomPtr_AD: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0xad>;
pub enum Fade {
    ToBlack,
    FakeDeathToGray,
    FakeDeathFromGray,
    DrainedFromGray,
    DrainedToGray,
    RealDeathToGray,
    LightsOn,
}

pub unsafe fn fade(kind: Fade, frame: u16, game: &mut Game) -> u8 {
    unsafe {
        let (table, colors) = match kind {
            Fade::ToBlack => (0xade9e8, 14),
            Fade::FakeDeathToGray => (0xaded8a, 3),
            Fade::FakeDeathFromGray => (0xaded9c, 3),
            Fade::DrainedFromGray => (0xaded9c, 13),
            Fade::DrainedToGray => (0xadef87, 15),
            Fade::RealDeathToGray => (0xadf107, 15),
            Fade::LightsOn => (0xadf273, 14),
        };
        let source = game
            .legacy_native_rom_ptr()
            .offset(crate::rtl::lorom_offset(table))
            .cast::<u16>()
            .offset(frame as isize)
            .read_unaligned();
        if source == 0 {
            return 1;
        }
        match kind {
            Fade::FakeDeathToGray | Fade::FakeDeathFromGray => {
                copy_rom_colors_to_palette(0x122, 0xad, source, colors, game);
            }
            Fade::RealDeathToGray => copy_rom_colors_to_palette(0x1e2, 0xad, source, colors, game),
            Fade::LightsOn => {
                copy_rom_colors_to_palette(0x62, 0xad, source, colors, game);
                copy_rom_colors_to_palette(0xa2, 0xad, source.wrapping_add(28), colors, game);
            }
            _ => {
                copy_rom_colors_to_palette(0x82, 0xad, source, colors, game);
                copy_rom_colors_to_palette(0x122, 0xad, source, colors, game);
                let tail = source.wrapping_add(colors * 2);
                if matches!(kind, Fade::ToBlack) {
                    copy_rom_colors_to_palette(0x162, 0xad, tail, colors, game);
                } else {
                    copy_rom_colors_to_palette(0x168, 0xad, tail, 5, game);
                    // The original accent write targets $017c, outside the palette buffer.
                    let accent = *(RomPtr_AD(tail.wrapping_add(10), game) as *const u16);
                    game.ram().copy_in(0x17c, &accent.to_ne_bytes());
                }
            }
        }
        0
    }
}

pub unsafe fn update_health_palette(game: &mut Game) {
    {
        if slot!(game, MotherBrainBody[0].corpse_state) < 2 {
            return;
        }
        let health = slot!(game, MotherBrainBrain[0x40].base.health);
        let level = if health >= 9000 {
            0
        } else if health >= 5400 {
            1
        } else if health >= 1800 {
            2
        } else {
            3
        };
        for destination in [0x82, 0x122] {
            copy_rom_colors_to_palette(
                destination,
                0xad,
                word(
                    &game.mother_brain_visuals.body_palettes,
                    game,
                    BODY_PALETTES_ADDRESS,
                    level,
                ),
                15,
                game,
            );
        }
        copy_rom_colors_to_palette(
            0x162,
            0xad,
            word(
                &game.mother_brain_visuals.back_leg_palettes,
                game,
                BACK_LEG_PALETTES_ADDRESS,
                level,
            ),
            15,
            game,
        );
    }
}

pub unsafe fn lights_off(game: &mut Game) {
    {
        for destination in [0x62, 0xa2] {
            copy_rom_colors_to_palette(destination, 0xad, 61999, 14, game);
        }
    }
}

pub unsafe fn seal_wall(game: &mut Game) {
    game.eproj_spawn_pt = Point16U { x: 248, y: 72 };
    spawn_with_room_gfx(ADDR_K_EPROJ_DUST_CLOUD_EXPLOSION as u16, 9, game);
    game.eproj_spawn_pt = Point16U { x: 248, y: 152 };
    spawn_with_room_gfx(ADDR_K_EPROJ_DUST_CLOUD_EXPLOSION as u16, 9, game);
    spawn_hardcoded(SpawnHardcodedPlmArgs::new(0xf, 0x4, 0xb673 as VoidP), game);
    spawn_hardcoded(SpawnHardcodedPlmArgs::new(0xf, 0x9, 0xb673 as VoidP), game);
    slot!(game, MotherBrainBody[0].function = (0xa9881di32) as u16);
}
#[forbid(unsafe_code)]
pub fn start_earthquake(a: u16, game: &mut Game) {
    {
        word! {game; earthquake_type = a; earthquake_timer = 20; }
        queue_sfx2_max6(0x25, game);
    }
}
#[forbid(unsafe_code)]
pub fn flicker_body(game: &mut Game) {
    if slot!(game, EnemyData[0].frame_counter) & 1 != 0 {
        checked_word!(game, 0x69; = checked_word!(game, 0x69;) | 2);
        slot!(game, EnemyData[0].properties, |properties| properties
            & !(K_ENEMY_PROPS_INVISIBLE as u16));
    } else {
        checked_word!(game, 0x69; = checked_word!(game, 0x69;) & !2);
        slot!(game, EnemyData[0].properties, |properties| properties
            | K_ENEMY_PROPS_INVISIBLE as u16);
    };
}

crate::forward_calls! { pub(crate) fn (game: &mut Game) -> u16;
    spawn_rising_hdma => crate::game::visual_runtime::spawn_hdma(0x88, &HdmaObjectDefinition::new(0, 0x2c, 0xe727), game);
    spawn_rainbow_hdma => crate::game::visual_runtime::spawn_hdma(0x88, &HdmaObjectDefinition::new(0x41, 0x26, 0xe751), game);
}
