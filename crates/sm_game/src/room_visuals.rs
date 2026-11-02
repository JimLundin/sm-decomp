//! Room visual effects: HDMA effect setup, Fireflea lighting, earthquakes, the Tourian statue and weather.
use crate::constants::bank_86::ADDR_K_EPROJ_TOURIAN_STATUE_DUST_CLOUDS;
use crate::display::preparation::{Field as DisplayField, Store as DisplayStore};
use crate::fx_domain::native_order;
use crate::hdma_objects::{HdmaEffect, spawn_bg3_scroll, spawn_hdma};
use crate::menus::prelude::HdmaObjectDefinition;
use crate::ram::word;
use crate::scene::hdma_sources::LiveOwner;
use crate::session_access::SessionAccess;
use crate::visual_state::{TourianStatuePhase, Weather};

/// Statue transitions on the current Session; compatibility keeps the terminal
/// store available while the HDMA interpreter cannot admit a Session lease.
pub trait StatueHost: crate::runtime_access::RuntimeAccess {
    fn begin_statue_animation(&mut self) -> bool {
        let mut runtime = self.runtime();
        let animation = &mut runtime.session_mut().mechanisms.statue_animation;
        *animation |= 0x10;
        *animation & 0x8000 == 0
    }
    fn finish_statue_animation(&mut self);
}

pub fn initialize_ceres_ridley<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    game.spawn_hdma(0x88, &HdmaObjectDefinition::new(0x40, 5, 0xd906));
    game.spawn_hdma(0x88, &HdmaObjectDefinition::new(0, 0x2c, 0xd91d));
}

pub fn initialize_bomb_torizo_haze<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    game.spawn_hdma(0x88, &HdmaObjectDefinition::new(2, 0x32, 0xdd4a));
    game.spawn_hdma(0x88, &HdmaObjectDefinition::new(0, 0x32, 0xdd62));
}

pub fn initialize_room_haze<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    let program = if crate::progress::leased::check_boss_bit_for_cur_area(1, game) & 1 != 0 {
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

pub fn darken_fireflea_room<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    crate::fx_lighting::darken_current(game);
}

pub fn initialize_fireflea_lighting<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    crate::fx_lighting::begin_current(game);
    game.spawn_hdma(0x88, &HdmaObjectDefinition::new(0x42, 0x11, 0xb0ac));
    game.store_source_word(LiveOwner::WindowLeft, 0, native_order(), 0);
    crate::fx_lighting::finish_current(game);
}

pub fn advance_fireflea_lighting<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    // Time is an external owner; preserve this callback's original sampling point.
    // Blending is published before the freeze read, as in the original routine.
    let mut fx = crate::fx_domain::Controls::fx_controls(game);
    fx.blending = (fx.blending & 0xff00) | 0xc;
    crate::fx_domain::Controls::publish_fx_controls(game, fx, &[crate::fx_state::Field::Blending]);
    let frozen = crate::sequence_word!(game.TIME_IS_FROZEN_FLAG) != 0;
    crate::fx_lighting::advance_after_blending(game, frozen);
}

/// The least earthquake timer a room shake sets.
const EARTHQUAKE_TIMER: u16 = 0x20;

pub fn shake_room_with_sound<H: crate::menus::host::Host + ?Sized>(
    kind: u16,
    handler_carry: bool,
    game: &mut H,
) {
    static SOUNDS: [u16; 17] = [
        0x46, 1, 0x46, 3, 0x46, 2, 0x46, 1, 0x46, 1, 0x46, 2, 0x46, 2, 0x46, 1, 0x8000,
    ];
    if game.fx_controls().earthquake_sound_timer & 0x8000 == 0 && {
        let fx_value = game.fx_controls().earthquake_sound_timer.wrapping_sub(1);
        let mut fx = game.fx_controls();
        fx.earthquake_sound_timer = fx_value;
        game.publish_fx_controls(fx, &[crate::fx_state::Field::EarthquakeSoundTimer]);
        game.fx_controls().earthquake_sound_timer
    } & 0x8000
        != 0
    {
        let mut index = game.fx_controls().earthquake_sound_index;
        let mut sound = SOUNDS[usize::from(index >> 1)];
        if sound & 0x8000 != 0 {
            index = 0;
            sound = SOUNDS[0];
        }
        game.queue_sound(1, (sound) as u8, 6);
        // The timer add ($88:B245) takes the HDMA object handler's carry: clear after its
        // CPX #$0C, but for the first object the one the handler was entered with. The main game
        // loop's PLP restores that every frame from power-on, where loading the save slots
        // ($80:8261) leaves it set.
        let carry = u16::from(
            crate::hdma_objects::HdmaTable::hdma_word(game, crate::hdma::Word::Selected) == 0
                && handler_carry,
        );
        {
            let fx_value = SOUNDS[usize::from(index >> 1) + 1]
                .wrapping_add(word!(game.RANDOM_NUMBER) & 3)
                .wrapping_add(carry);
            let mut fx = game.fx_controls();
            fx.earthquake_sound_timer = fx_value;
            game.publish_fx_controls(fx, &[crate::fx_state::Field::EarthquakeSoundTimer]);
        };
        {
            let fx_value = index.wrapping_add(4);
            let mut fx = game.fx_controls();
            fx.earthquake_sound_index = fx_value;
            game.publish_fx_controls(fx, &[crate::fx_state::Field::EarthquakeSoundIndex]);
        };
    }
    crate::gameplay_context::set_effect(game, crate::room_view::EffectWord::EarthquakeKind, kind);
    crate::gameplay_context::set_effect(
        game,
        crate::room_view::EffectWord::EarthquakeTimer,
        crate::gameplay_context::room_effects(game).earthquake_timer | EARTHQUAKE_TIMER,
    );
}

pub fn initialize_tourian_statue<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    if crate::progress::leased::event_has_happened(0xa, game) & 1 != 0 {
        crate::room_mechanisms::creation::spawn_hardcoded(6, 12, 0xb777, game);
        crate::word_transfers!(game, ascending, 1; native(0xcd20) <- value(514));
    }
    game.store_display_control_byte(DisplayField::Screens1, 74, DisplayStore::Named);
    crate::liquid::LiquidWords::set(game, |words| &mut words.liquid.rising, 0xc428);
    crate::liquid::LiquidWords::set(
        game,
        |words| &mut words.liquid.surface.whole,
        crate::cinematic_access::load::<crate::liquid::LiquidWords, _>(game).height,
    );
    game.spawn_hdma(0x88, &HdmaObjectDefinition::new(0x42, 0x11, 0xd856));
    game.spawn_hdma(0x88, &HdmaObjectDefinition::new(0x42, 0x10, 0xdcfa));
    spawn_bg3_scroll(game);
}

pub fn advance_tourian_statue<H: crate::menus::host::Host + StatueHost + ?Sized>(
    hdma_index: usize,
    phase: TourianStatuePhase,
    handler_carry: bool,
    game: &mut H,
) {
    use TourianStatuePhase::*;
    if matches!(phase, Shaking | Sinking) {
        shake_room_with_sound(13, handler_carry, game);
    }
    match phase {
        Waiting => {
            if (6..=9)
                .all(|event| crate::progress::leased::event_has_happened(event, game) & 1 != 0)
                && game.begin_statue_animation()
            {
                crate::hdma_word!(game, hdma_index, C => 300);
                advance_statue_program(hdma_index, game);
            }
        }
        Shaking => {
            crate::hdma_word!(game, hdma_index, C => crate::hdma_word!(game, hdma_index, C).wrapping_sub(1));
            if crate::hdma_word!(game, hdma_index, C) & 0x8000 != 0 {
                // The spawns take their initialisation parameter from A: the
                // shake's earthquake timer 20h ($88:DC2C), then each spawn's
                // returned slot.
                let mut parameter = EARTHQUAKE_TIMER;
                for _ in 0..4 {
                    parameter = crate::enemy_projectiles::lifecycle::spawn_room_leased(
                        ADDR_K_EPROJ_TOURIAN_STATUE_DUST_CLOUDS as u16,
                        parameter,
                        game,
                    );
                }
                advance_statue_program(hdma_index, game);
            }
        }
        Sinking => {
            if crate::sequence_word!(game.TIME_IS_FROZEN_FLAG) != 0 {
                return;
            }
            // A is the fraction and B the whole of the statue's scroll offset.
            let position = (u32::from(crate::hdma_word!(game, hdma_index, A))
                | u32::from(crate::hdma_word!(game, hdma_index, B)) << 16)
                .wrapping_sub(0x4000);
            crate::hdma_word!(game, hdma_index, A => position as u16);
            crate::hdma_word!(game, hdma_index, B => (position >> 16) as u16);
            if crate::hdma_word!(game, hdma_index, B) == 0xff10 {
                crate::room_mechanisms::creation::spawn_hardcoded(6, 12, 0xb773, game);
                crate::progress::leased::mark_event(0xa, game);
                advance_statue_program(hdma_index, game);
            }
        }
        Finished => {
            game.finish_statue_animation();
            crate::word_transfers!(game, ascending, 1; native(0xcd20) <- value(514));
        }
    }
    let scroll =
        crate::camera::current(game).layer1[1].wrapping_add(crate::hdma_word!(game, hdma_index, B));
    game.store_source_word(LiveOwner::WindowLeft, 0, native_order(), scroll);
}

fn advance_statue_program<H: crate::hdma_objects::HdmaTable + ?Sized>(
    hdma_index: usize,
    game: &mut H,
) {
    crate::hdma_word!(game, hdma_index, InstructionTimer => 1);
    crate::hdma_word!(game, hdma_index, InstructionList => crate::hdma_word!(game, hdma_index, InstructionList).wrapping_add(2));
}

pub fn initialize_weather<H: crate::menus::host::Host + ?Sized>(weather: Weather, game: &mut H) {
    use crate::constants::bank_87::{ADDR_K_ANIMTILES_RAIN, ADDR_K_ANIMTILES_SPORES};
    let (control, program, tiles) = match weather {
        Weather::Rain => (0x43, 0xd96c, Some(ADDR_K_ANIMTILES_RAIN)),
        Weather::Spores => (0x42, 0xda2d, Some(ADDR_K_ANIMTILES_SPORES)),
        Weather::Fog => (0x43, 0xdb19, None),
    };
    game.store_display_control_byte(DisplayField::Background3Map, 92, DisplayStore::Named);
    game.spawn_hdma(0x88, &HdmaObjectDefinition::new(control, 0x11, program));
    if let Some(tiles) = tiles {
        spawn_bg3_scroll(game);
        crate::animated_tiles::execution::leased::spawn(tiles as u16, game);
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
    crate::hdma_effects::apply_leased(HdmaEffect::SetPhantoonMotion { direction, speed }, 0, game);
    spawn_hdma(0x88, &HdmaObjectDefinition::new(0x42, 0xf, 0xe4a8), game);
}

pub fn initialize_intro_wave<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    word!(game.WAVY_SAMUS_ENABLED = 1);
    crate::word_transfers!(game, ascending, 1; native(0xd9e) <- value(0x4000));
    word!(game.LOOP_COUNTER_TRANSFER_ENEMIES_TO_VRAM = 8);
    crate::hdma_effects::apply_leased(
        HdmaEffect::InitializeWavePair {
            base: 0xd82,
            primary: 0x9800,
        },
        0,
        game,
    );
    spawn_hdma(0x88, &HdmaObjectDefinition::new(0x42, 0x11, 0xec8a), game);
}

pub fn scroll_weather<H: crate::menus::host::Host + ?Sized>(
    slot: u16,
    weather: Weather,
    game: &mut H,
) {
    game.store_display_control_byte(DisplayField::Background3Map, 92, DisplayStore::Named);
    {
        let fx_value = game.room_blending().liquid;
        let mut fx = game.fx_controls();
        fx.blending = fx_value;
        game.publish_fx_controls(fx, &[crate::fx_state::Field::Blending]);
    };
    if crate::sequence_word!(game.TIME_IS_FROZEN_FLAG) != 0 {
        return;
    }
    crate::hdma_effects::apply_leased(HdmaEffect::ScrollWeather(weather), slot, game);
}
