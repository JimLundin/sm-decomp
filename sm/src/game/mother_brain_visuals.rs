//! Mother Brain encounter palettes, wall sealing, earthquakes and body flicker.
use crate::game::constants::bank_86::*;
use crate::game::enemy_projectiles::SpawnEprojWithRoomGfx;
use crate::game::imported::{Imported, word};
use crate::game::music::QueueSfx2_Max6;
use crate::game::palette_copies::copy_rom_colors_to_palette;
use crate::game::prelude::*;
use crate::game::room_mechanisms::spawn_hardcoded;
use crate::ram_map::{checked_word, slot};
use sm_data::mother_brain::{
    BACK_LEG_PALETTES_ADDRESS, BODY_PALETTES_ADDRESS, HealthPalettes, PALETTES_BYTES,
    RAINBOW_ADDRESS, RAINBOW_BYTES, RainbowColors, import_health_palettes, import_rainbow,
};

#[derive(Default)]
pub(crate) struct ImportedTables {
    body_palettes: Option<Imported<HealthPalettes>>,
    back_leg_palettes: Option<Imported<HealthPalettes>>,
    rainbow: Option<Imported<RainbowColors>>,
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
        rainbow: Imported::import(image, RAINBOW_ADDRESS, RAINBOW_BYTES, import_rainbow),
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
        if slot!(game, Enemy_MotherBrain[0].var_1F) < 2 {
            return;
        }
        let health = slot!(game, Enemy_MotherBrain[0x40].base.health);
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
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 9, game);
    game.eproj_spawn_pt = Point16U { x: 248, y: 152 };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 9, game);
    spawn_hardcoded(SpawnHardcodedPlmArgs::new(0xf, 0x4, 0xb673 as VoidP), game);
    spawn_hardcoded(SpawnHardcodedPlmArgs::new(0xf, 0x9, 0xb673 as VoidP), game);
    slot!(game, Enemy_MotherBrain[0].var_A = (0xa9881di32) as u16);
}
#[forbid(unsafe_code)]
pub fn start_earthquake(a: u16, game: &mut Game) {
    {
        word! {game; earthquake_type = a; earthquake_timer = 20; }
        QueueSfx2_Max6(0x25, game);
    }
}
#[forbid(unsafe_code)]
pub fn flicker_body(game: &mut Game) {
    if slot!(game, EnemyData[0].frame_counter) & 1 != 0 {
        checked_word!(game, 0x69; = checked_word!(game, 0x69;) | 2);
        slot!(game, EnemyData[0].properties, |properties| properties
            & !(kEnemyProps_Invisible as u16));
    } else {
        checked_word!(game, 0x69; = checked_word!(game, 0x69;) & !2);
        slot!(game, EnemyData[0].properties, |properties| properties
            | kEnemyProps_Invisible as u16);
    };
}

crate::forward_calls! { pub(crate) fn (game: &mut Game) -> u16;
    spawn_rising_hdma => crate::game::visual_runtime::spawn_hdma(0x88, &SpawnHdmaObject_Args::new(0, 0x2c, 0xe727), game);
    spawn_rainbow_hdma => crate::game::visual_runtime::spawn_hdma(0x88, &SpawnHdmaObject_Args::new(0x41, 0x26, 0xe751), game);
}

pub(crate) unsafe fn initialize_rainbow_beam(game: &mut Game) {
    game.ram().write_byte(0x74, 32);
    game.ram().write_byte(0x75, 71);
    game.ram().write_byte(0x76, 0x8f);
    word!(game.fx_layer_blending_config_c = 36);
    game.ram().write_byte(0x9c00, 100);
    word_transfers!(game, ascending, 1; native(0x9c01) <- value(0x9d00), native(0x9c03) <- value(0));
    word! {game; hdma_table_2[0] = 0; hdma_table_2[1] = 0x8020; hdma_table_2[2] = 0; }
    unsafe {
        crate::game::sm_a9::MotherBrain_CalculateRainbowBeamHdma(game);
    }
}

pub(crate) unsafe fn advance_rainbow_beam(game: &mut Game) {
    if word!(game.game_state) == 19 {
        word_transfers!(game, ascending, 1;
            native(0x18b4 + 2 * usize::from(word!(game.hdma_object_index) >> 1)) <- value(0));
    } else {
        word!(game.fx_layer_blending_config_c = 36);
        if word!(game.game_state) != 27 {
            unsafe {
                crate::game::sm_a9::MotherBrain_CalculateRainbowBeamHdma(game);
            }
            advance_rainbow_palette(game);
        }
    }
}

fn advance_rainbow_palette(game: &mut Game) {
    let index = (word!(game.hdma_object_A[0]) >> 1) as isize;
    let mut color = word(
        &game.mother_brain_visuals.rainbow,
        game,
        RAINBOW_ADDRESS,
        index,
    );
    if color & 0x8000 == 0 {
        word!(game.hdma_object_A[0] += 1);
        word!(game.hdma_object_A[0] += 1);
        word!(game.hdma_object_A[0] += 1);
        word!(game.hdma_object_A[0] += 1);
    } else {
        word!(game.hdma_object_A[0] = 0);
        color = word(&game.mother_brain_visuals.rainbow, game, RAINBOW_ADDRESS, 0);
    }
    game.ram().write_byte(0x74, color as u8 & 0x1f | 0x20);
    game.ram()
        .write_byte(0x75, (color >> 5) as u8 & 0x1f | 0x40);
    game.ram()
        .write_byte(0x76, (color >> 10) as u8 & 0x1f | 0x80);
}
