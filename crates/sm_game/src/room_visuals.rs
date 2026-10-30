//! Room visual effects: HDMA effect setup, Fireflea lighting, earthquakes, the Tourian statue and weather.
use crate::constants::bank_86::ADDR_K_EPROJ_TOURIAN_STATUE_DUST_CLOUDS;
use crate::hdma_objects::{HdmaEffect, spawn_bg3_scroll, spawn_hdma};
use crate::menus::prelude::HdmaObjectDefinition;
use crate::ram::RawRam as _;
use crate::ram::word;
use crate::visual_state::{TourianStatuePhase, Weather};

pub fn initialize_ceres_ridley<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    game.spawn_hdma(0x88, &HdmaObjectDefinition::new(0x40, 5, 0xd906));
    game.spawn_hdma(0x88, &HdmaObjectDefinition::new(0, 0x2c, 0xd91d));
}

pub fn initialize_bomb_torizo_haze<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    game.spawn_hdma(0x88, &HdmaObjectDefinition::new(2, 0x32, 0xdd4a));
    game.spawn_hdma(0x88, &HdmaObjectDefinition::new(0, 0x32, 0xdd62));
}

pub fn initialize_room_haze<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    let program = if game.boss_bit_for_current_area(1) & 1 != 0 {
        0xdeeb
    } else {
        0xded3
    };
    game.spawn_hdma(0x88, &HdmaObjectDefinition::new(0x40, 0x32, program));
}

pub fn initialize_title_gradient<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    game.spawn_hdma(0x88, &HdmaObjectDefinition::new(0, 0x32, 0xeb73));
    game.spawn_hdma(0x88, &HdmaObjectDefinition::new(0, 0x31, 0xeb85));
}

const FIREFLEA_FLASH: [u16; 12] = [
    0, 0x100, 0x200, 0x300, 0x400, 0x500, 0x600, 0x500, 0x400, 0x300, 0x200, 0x100,
];

const FIREFLEA_DARKNESS: [u16; 6] = [0, 0x600, 0xc00, 0x1200, 0x1800, 0x1900];

pub fn darken_fireflea_room<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    if word!(game.FIREFLEA_DARKNESS_LEVEL).wrapping_sub(12) & 0x8000 != 0 {
        word!(game.FIREFLEA_DARKNESS_LEVEL += 2);
    }
}

pub fn initialize_fireflea_lighting<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    word! {game; FIREFLEA_FLASHING_TIMER = 6; FIREFLEA_FLASHING_INDEX = 0; }
    game.spawn_hdma(0x88, &HdmaObjectDefinition::new(0x42, 0x11, 0xb0ac));
    crate::word_transfers!(game, ascending, 1; native(0x9e00) <- value(0));
    word! { game, 0; UNUSED_WORD_7E177C, FIREFLEA_DARKNESS_LEVEL }
    word!(game.UNUSED_WORD_7E1780 = 24);
    word!(game.UNUSED_WORD_7E1782 = FIREFLEA_FLASH[0]);
}

pub fn advance_fireflea_lighting<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    word!(game.FX_LAYER_BLENDING_CONFIG_C = word!(game.FX_LAYER_BLENDING_CONFIG_C) & 0xff00 | 0xc);
    if word!(game.TIME_IS_FROZEN_FLAG) != 0 {
        return;
    }
    if word!(game.FIREFLEA_FLASHING_TIMER -= 1, reread) == 0 {
        word!(game.FIREFLEA_FLASHING_TIMER = 6);
        let next = if word!(game.FIREFLEA_DARKNESS_LEVEL).wrapping_sub(10) & 0x8000 != 0 {
            let next = word!(game.FIREFLEA_FLASHING_INDEX).wrapping_add(1);
            if next >= 12 { 0 } else { next }
        } else {
            6
        };
        word!(game.FIREFLEA_FLASHING_INDEX = next);
    }
    let shade = (FIREFLEA_DARKNESS[usize::from(word!(game.FIREFLEA_DARKNESS_LEVEL) >> 1)]
        .wrapping_add(FIREFLEA_FLASH[usize::from(word!(game.FIREFLEA_FLASHING_INDEX))])
        >> 8) as u8;
    game.ram().write_byte(0x75, shade | 0x80);
    game.ram().write_byte(0x76, shade | 0x40);
    game.ram().write_byte(0x74, shade | 0x20);
}

/// The least earthquake timer a room shake sets.
const EARTHQUAKE_TIMER: u16 = 0x20;

pub fn shake_room_with_sound<H: crate::menus::host::Host + ?Sized>(kind: u16, game: &mut H) {
    static SOUNDS: [u16; 17] = [
        0x46, 1, 0x46, 3, 0x46, 2, 0x46, 1, 0x46, 1, 0x46, 2, 0x46, 2, 0x46, 1, 0x8000,
    ];
    if word!(game.EARTHQUAKE_SFX_TIMER) & 0x8000 == 0
        && word!(game.EARTHQUAKE_SFX_TIMER -= 1, reread) & 0x8000 != 0
    {
        let mut index = word!(game.EARTHQUAKE_SFX_INDEX);
        let mut sound = SOUNDS[usize::from(index >> 1)];
        if sound & 0x8000 != 0 {
            index = 0;
            sound = SOUNDS[0];
        }
        game.queue_sound(1, (sound) as u8, 6);
        word!(
            game.EARTHQUAKE_SFX_TIMER = SOUNDS[usize::from(index >> 1) + 1]
                .wrapping_add(index & 1)
                .wrapping_add(word!(game.RANDOM_NUMBER) & 3)
        );
        word!(game.EARTHQUAKE_SFX_INDEX = index.wrapping_add(4));
    }
    crate::room_view::set_effect(game, crate::room_view::EffectWord::EarthquakeKind, kind);
    crate::room_view::set_effect(
        game,
        crate::room_view::EffectWord::EarthquakeTimer,
        crate::room_view::effects(game).earthquake_timer | EARTHQUAKE_TIMER,
    );
}

pub fn initialize_tourian_statue<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    if game.event_happened(0xa) & 1 != 0 {
        game.spawn_hardcoded_plm(6, 12, 0xb777);
        crate::word_transfers!(game, ascending, 1; native(0xcd20) <- value(514));
    }
    game.ram().write_byte(0x59, 74);
    crate::liquid::LiquidWords::set(game, |words| &mut words.liquid.rising, 0xc428);
    crate::liquid::LiquidWords::set(
        game,
        |words| &mut words.liquid.surface.whole,
        crate::liquid::LiquidWords::read(game).height,
    );
    game.spawn_hdma(0x88, &HdmaObjectDefinition::new(0x42, 0x11, 0xd856));
    game.spawn_hdma(0x88, &HdmaObjectDefinition::new(0x42, 0x10, 0xdcfa));
    game.spawn_bg3_scroll();
}

pub fn advance_tourian_statue<H: crate::menus::host::Host + ?Sized>(
    object: &mut crate::hdma::HdmaObject,
    phase: TourianStatuePhase,
    game: &mut H,
) {
    use TourianStatuePhase::*;
    if matches!(phase, Shaking | Sinking) {
        shake_room_with_sound(13, game);
    }
    match phase {
        Waiting => {
            if (6..=9).all(|event| game.event_happened(event) & 1 != 0) {
                crate::room_view::set_effect(
                    game,
                    crate::room_view::EffectWord::StatueAnimation,
                    crate::room_view::effects(game).statue_animation | (0x10),
                );
                if crate::room_view::effects(game).statue_animation & 0x8000 == 0 {
                    object.c = 300;
                    advance_statue_program(object);
                }
            }
        }
        Shaking => {
            object.c = object.c.wrapping_sub(1);
            if object.c & 0x8000 != 0 {
                // The spawns take their initialisation parameter from A: the
                // shake's earthquake timer 20h ($88:DC2C), then each spawn's
                // returned slot.
                let mut parameter = EARTHQUAKE_TIMER;
                for _ in 0..4 {
                    parameter = game.spawn_room_enemy_projectile(
                        ADDR_K_EPROJ_TOURIAN_STATUE_DUST_CLOUDS as u16,
                        parameter,
                    );
                }
                advance_statue_program(object);
            }
        }
        Sinking => {
            if word!(game.TIME_IS_FROZEN_FLAG) != 0 {
                return;
            }
            // A is the fraction and B the whole of the statue's scroll offset.
            let position = (u32::from(object.a) | u32::from(object.b) << 16).wrapping_sub(0x4000);
            object.a = position as u16;
            object.b = (position >> 16) as u16;
            if object.b == 0xff10 {
                {
                    game.spawn_hardcoded_plm(6, 12, 0xb773);
                }
                game.set_event_happened(0xa);
                advance_statue_program(object);
            }
        }
        Finished => {
            crate::room_view::set_effect(
                game,
                crate::room_view::EffectWord::StatueFinished,
                0x8000,
            );
            crate::word_transfers!(game, ascending, 1; native(0xcd20) <- value(514));
        }
    }
    let scroll = crate::camera::current(game).layer1[1].wrapping_add(object.b);
    crate::word_transfers!(game, ascending, 1; native(0x9e00) <- value(scroll));
}

fn advance_statue_program(object: &mut crate::hdma::HdmaObject) {
    object.instruction_timer = 1;
    object.instruction_list = object.instruction_list.wrapping_add(2);
}

pub fn initialize_weather<H: crate::menus::host::Host + ?Sized>(weather: Weather, game: &mut H) {
    use crate::constants::bank_87::{ADDR_K_ANIMTILES_RAIN, ADDR_K_ANIMTILES_SPORES};
    let (control, program, tiles) = match weather {
        Weather::Rain => (0x43, 0xd96c, Some(ADDR_K_ANIMTILES_RAIN)),
        Weather::Spores => (0x42, 0xda2d, Some(ADDR_K_ANIMTILES_SPORES)),
        Weather::Fog => (0x43, 0xdb19, None),
    };
    game.ram().write_byte(0x5b, 92);
    game.spawn_hdma(0x88, &HdmaObjectDefinition::new(control, 0x11, program));
    if let Some(tiles) = tiles {
        game.spawn_bg3_scroll();
        game.spawn_animated_tiles(tiles as u16);
    }
}

pub fn initialize_ceres_elevator<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    spawn_hdma(0x88, &HdmaObjectDefinition::new(0x40, 5, 0xd939), game);
}

pub fn initialize_draygon_layer<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    spawn_hdma(0x88, &HdmaObjectDefinition::new(0, 0x2c, 0xdf4f), game);
    word!(game.ROOM_LOADING_IRQ_HANDLER = 12);
    spawn_bg3_scroll(game);
}

pub fn initialize_morph_eye_beam<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    spawn_hdma(0x88, &HdmaObjectDefinition::new(0x41, 0x26, 0xe8ec), game);
}

pub fn initialize_phantoon_motion<H: crate::menus::host::Host + ?Sized>(
    direction: u16,
    speed: u16,
    game: &mut H,
) {
    game.hdma_effect(HdmaEffect::SetPhantoonMotion { direction, speed }, 0);
    spawn_hdma(0x88, &HdmaObjectDefinition::new(0x42, 0xf, 0xe4a8), game);
}

pub fn initialize_intro_wave<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    word!(game.WAVY_SAMUS_ENABLED = 1);
    crate::word_transfers!(game, ascending, 1; native(0xd9e) <- value(0x4000));
    word!(game.LOOP_COUNTER_TRANSFER_ENEMIES_TO_VRAM = 8);
    game.hdma_effect(
        HdmaEffect::InitializeWavePair {
            base: 0xd82,
            primary: 0x9800,
        },
        0,
    );
    spawn_hdma(0x88, &HdmaObjectDefinition::new(0x42, 0x11, 0xec8a), game);
}

pub fn scroll_weather<H: crate::menus::host::Host + ?Sized>(
    slot: u16,
    weather: Weather,
    game: &mut H,
) {
    use crate::ram::checked_word;
    game.ram().write_byte(0x5b, 92);
    checked_word!(game.FX_LAYER_BLENDING_CONFIG_C = checked_word!(game.FX_LAYER_BLENDING_CONFIG_B));
    if checked_word!(game.TIME_IS_FROZEN_FLAG) != 0 {
        return;
    }
    game.hdma_effect(HdmaEffect::ScrollWeather(weather), slot);
}

/// Crocomire's phase step, its scroll HDMA, Samus's collision and its palette.
pub fn advance_crocomire<H>(game: &mut H)
where
    H: crate::menus::host::Host + crate::actors::crocomire_reactions::Host + ?Sized,
{
    use crate::actors::crocomire_reactions as crocomire;
    use crate::actors::layouts::EnemyLayout72 as Crocomire;
    use crate::ram::slot;
    static DEFINITION: HdmaObjectDefinition = HdmaObjectDefinition::new(0x42, 0x10, 0x9563);
    if let Some(request) = crocomire::execute_phase(game) {
        game.queue_sound(1, request as u16 as u8, 6);
        crocomire::prepare_scroll_hdma(game);
        slot!(
            game,
            Crocomire[0].var_1f = spawn_hdma(0xa4, &DEFINITION, game)
        );
    }
    crocomire::samus_collision(game);
    crocomire::update_palette(game);
}

/// Position Crocomire's second body part and align its BG2 image with the camera.
pub fn align_crocomire_background<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    use crate::actors::layouts::EnemyLayout72 as Crocomire;
    use crate::ram::slot;
    let primary = i32::from(word!(game.CUR_ENEMY_INDEX));
    let secondary = i32::from(word!(game.CUR_ENEMY_INDEX)) + 64;
    slot!(
        game,
        Crocomire[secondary].base.x_pos = slot!(game, Crocomire[secondary].var_a)
            .wrapping_add(slot!(game, Crocomire[primary].base.x_pos))
    );
    slot!(
        game,
        Crocomire[secondary].base.y_pos = slot!(game, Crocomire[primary].base.y_pos)
    );
    let relative = slot!(game, Crocomire[primary].base.x_pos)
        .wrapping_sub(crate::camera::current(game).layer1[0]) as i16;
    let offset = 51u16.wrapping_sub(relative as u16);
    word!(
        game.REG_BG2HOFS =
            if (-128..384).contains(&relative) && (offset as i16).unsigned_abs() < 284 {
                offset
            } else {
                256
            }
    );
}
