//! Mother Brain encounter palettes, fades, wall sealing, body flicker and HDMA spawns.
#![forbid(unsafe_code)]
use crate::actors::boss_fields_owned::mother_brain_part as slot_view;
use crate::actors::boss_palette_owned::copy_rom_colors_to_palette;
use crate::actors::mother_brain_fight_owned::Fade;
use crate::actors::owned_execution::PreparedActorState;
use crate::constants::bank_86::ADDR_K_EPROJ_DUST_CLOUD_EXPLOSION;
use crate::constants::enemy_properties::K_ENEMY_PROPS_INVISIBLE;
use crate::display::hdma_spawn::HdmaObjectDefinition;
use sm_data::mother_brain::{BACK_LEG_PALETTES_ADDRESS, BODY_PALETTES_ADDRESS};
/// Mother Brain's palette lines in the palette buffer, by byte offset, and
/// the bank every colour source lives in.
const BRAIN_LINE: u16 = 0x62;
const LEGS_LINE: u16 = 0xa2;
const BODY_LINE: u16 = 0x82;
const BODY_ALIAS_LINE: u16 = 0x122;
const BACK_LEG_LINE: u16 = 0x162;
const PALETTE_BANK: u8 = 0xad;
/// The lights-off colours ($AD:F22F) for the brain and leg lines.
const LIGHTS_OFF_COLORS: u16 = 0xf22f;
/// The main-screen designation mirror the body flicker toggles BG2 in.
const MAIN_SCREEN: usize = 0x69;

/// Step fade `kind` to `frame`; returns 1 once its colour list ends.
pub fn fade(kind: Fade, frame: u16, game: &mut PreparedActorState<'_>) -> u8 {
    let (table, colors) = match kind {
        Fade::ToBlack => (0xad_e9e8, 14),
        Fade::FakeDeathToGray => (0xad_ed8a, 3),
        Fade::FakeDeathFromGray => (0xad_ed9c, 3),
        Fade::DrainedFromGray => (0xad_ed9c, 13),
        Fade::DrainedToGray => (0xad_ef87, 15),
        Fade::RealDeathToGray => (0xad_f107, 15),
        Fade::LightsOn => (0xad_f273, 14),
    };
    let base = sm_data::cartridge::lorom_offset(table) as usize;
    let source = game.allocations.native().table_word(base, frame as isize);
    if source == 0 {
        return 1;
    }
    let mut copy =
        |line, source, count| copy_rom_colors_to_palette(line, PALETTE_BANK, source, count, game);
    match kind {
        Fade::FakeDeathToGray | Fade::FakeDeathFromGray => copy(BODY_ALIAS_LINE, source, colors),
        Fade::RealDeathToGray => copy(0x1e2, source, colors),
        Fade::LightsOn => {
            copy(BRAIN_LINE, source, colors);
            copy(LEGS_LINE, source.wrapping_add(28), colors);
        }
        Fade::ToBlack | Fade::DrainedFromGray | Fade::DrainedToGray => {
            copy(BODY_LINE, source, colors);
            copy(BODY_ALIAS_LINE, source, colors);
            let tail = source.wrapping_add(colors * 2);
            if matches!(kind, Fade::ToBlack) {
                copy(BACK_LEG_LINE, tail, colors);
            } else {
                copy(0x168, tail, 5);
                // The original accent write targets $017c, outside the palette buffer.
                let accent = game
                    .native_actor_record(PALETTE_BANK, tail.wrapping_add(10))
                    .word(0);
                let [first, second] = accent.to_ne_bytes();
                game.set_current_byte(0x17c, first);
                game.set_current_byte(0x17d, second);
            }
        }
    }
    0
}

/// Mother Brain's body and back-leg palettes for her current health band.
pub fn update_health_palette(game: &mut PreparedActorState<'_>) {
    if slot_view!(game, MotherBrainBody[0].corpse_state) < 2 {
        return;
    }
    let band = match slot_view!(game, MotherBrainBrain[0x40].base.health) {
        9000.. => 0,
        5400.. => 1,
        1800.. => 2,
        _ => 3,
    };
    for line in [BODY_LINE, BODY_ALIAS_LINE] {
        let source = game.loaded_table_word(BODY_PALETTES_ADDRESS, band);
        copy_rom_colors_to_palette(line, PALETTE_BANK, source, 15, game);
    }
    let source = game.loaded_table_word(BACK_LEG_PALETTES_ADDRESS, band);
    copy_rom_colors_to_palette(BACK_LEG_LINE, PALETTE_BANK, source, 15, game);
}

/// The lights-off colours on the brain and leg lines.
pub fn lights_off(game: &mut PreparedActorState<'_>) {
    for line in [BRAIN_LINE, LEGS_LINE] {
        copy_rom_colors_to_palette(line, PALETTE_BANK, LIGHTS_OFF_COLORS, 14, game);
    }
}

/// Seal the wall behind Mother Brain: two dust clouds, two wall blocks, then
/// her next function.
pub fn seal_wall(game: &mut PreparedActorState<'_>) {
    for y in [72, 152] {
        crate::actors::mother_brain_fight_owned::set_projectile_spawn_point(game, 248, y);
        crate::enemy_projectiles::lifecycle::spawn_room(
            game,
            u16::try_from(ADDR_K_EPROJ_DUST_CLOUD_EXPLOSION).expect("dust selector"),
            9,
        );
    }
    for y in [4, 9] {
        crate::room_mechanisms::creation::spawn(
            game,
            crate::room_mechanisms::host::SpawnSource::Hardcoded {
                x: 0xf,
                y,
                header: 0xb673,
            },
            &mut crate::room_mechanisms::host::Contact::Absent,
        );
    }
    slot_view!(game, MotherBrainBody[0].function = 0x881d);
}

/// Flicker Mother Brain's body layer and visibility on alternate frames.
pub fn flicker_body(game: &mut PreparedActorState<'_>) {
    let invisible = K_ENEMY_PROPS_INVISIBLE as u16;
    let screen = u16::from_le_bytes([
        game.current_byte(MAIN_SCREEN),
        game.current_byte(MAIN_SCREEN + 1),
    ]);
    if slot_view!(game, EnemyBase[0].frame_counter) & 1 != 0 {
        let [first, second] = (screen | 2).to_le_bytes();
        game.set_current_byte(MAIN_SCREEN, first);
        game.set_current_byte(MAIN_SCREEN + 1, second);
        slot_view!(game, EnemyBase[0].properties, |properties| properties
            & !invisible);
    } else {
        let [first, second] = (screen & !2).to_le_bytes();
        game.set_current_byte(MAIN_SCREEN, first);
        game.set_current_byte(MAIN_SCREEN + 1, second);
        slot_view!(game, EnemyBase[0].properties, |properties| properties
            | invisible);
    }
}

/// The rising and rainbow beams' HDMA objects; each returns its slot.
pub fn spawn_rising_hdma(game: &mut PreparedActorState<'_>) -> u16 {
    crate::actors::boss_hdma_owned::spawn_hdma_owned(
        0x88,
        &HdmaObjectDefinition::new(0, 0x2c, 0xe727),
        game,
    )
}
pub fn spawn_rainbow_hdma(game: &mut PreparedActorState<'_>) -> u16 {
    crate::actors::boss_hdma_owned::spawn_hdma_owned(
        0x88,
        &HdmaObjectDefinition::new(0x41, 0x26, 0xe751),
        game,
    )
}
