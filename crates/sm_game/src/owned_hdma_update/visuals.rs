//! Common room visual callbacks on the current Session.
#![forbid(unsafe_code)]
use crate::constants::bank_86::ADDR_K_EPROJ_TOURIAN_STATUE_DUST_CLOUDS;
use crate::display::preparation::{Field as DisplayField, Store as DisplayStore};
use crate::fx_domain::native_order;
use crate::hdma_objects::HdmaEffect;
use crate::scene::hdma_sources::LiveOwner;
use crate::visual_state::{TourianStatuePhase, Weather};
const EARTHQUAKE_TIMER: u16 = 0x20;
use super::Host as _;
use super::context::Context as Host;
pub fn advance_fireflea_lighting<H: Host>(game: &mut H) {
    // Time is an external owner; preserve this callback's original sampling point.
    // Blending is published before the freeze read, as in the original routine.
    let mut fx = crate::fx_domain::Controls::fx_controls(game);
    fx.blending = (fx.blending & 0xff00) | 0xc;
    crate::fx_domain::Controls::publish_fx_controls(game, fx, &[crate::fx_state::Field::Blending]);
    let frozen = game.session().timing.time_frozen != 0;
    crate::fx_lighting::advance_after_blending(game, frozen);
}

pub fn shake_room_with_sound<H: Host>(kind: u16, handler_carry: bool, game: &mut H) {
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
        crate::effects::SoundEffects::queue_sound(game, 1, (sound) as u8, 6);
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
                .wrapping_add(game.session().timing.random_number & 3)
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
    game.session_mut().feedback.earthquake_kind = kind;
    game.session_mut().feedback.earthquake_timer |= EARTHQUAKE_TIMER;
}

pub fn advance_tourian_statue<H: Host>(
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
            if (6..=9).all(|event| crate::progress::event_happened(game, event)) && {
                let animation = &mut game.session_mut().mechanisms.statue_animation;
                *animation |= 0x10;
                *animation & 0x8000 == 0
            } {
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
                    parameter = crate::enemy_projectiles::lifecycle::spawn_room(
                        game,
                        ADDR_K_EPROJ_TOURIAN_STATUE_DUST_CLOUDS as u16,
                        parameter,
                    );
                }
                advance_statue_program(hdma_index, game);
            }
        }
        Sinking => {
            if game.session().timing.time_frozen != 0 {
                return;
            }
            // A is the fraction and B the whole of the statue's scroll offset.
            let position = (u32::from(crate::hdma_word!(game, hdma_index, A))
                | u32::from(crate::hdma_word!(game, hdma_index, B)) << 16)
                .wrapping_sub(0x4000);
            crate::hdma_word!(game, hdma_index, A => position as u16);
            crate::hdma_word!(game, hdma_index, B => (position >> 16) as u16);
            if crate::hdma_word!(game, hdma_index, B) == 0xff10 {
                crate::room_mechanisms::creation::spawn(
                    game,
                    crate::room_mechanisms::host::SpawnSource::Hardcoded {
                        x: 6,
                        y: 12,
                        header: 0xb773,
                    },
                    &mut crate::room_mechanisms::host::Contact::Absent,
                );
                crate::progress::set_event(game, 0xa);
                advance_statue_program(hdma_index, game);
            }
        }
        Finished => {
            game.session_mut().mechanisms.statue_finished = 0x8000;
            game.statue_tiles(514);
        }
    }
    let scroll =
        game.session().room.camera.layer1[1].wrapping_add(crate::hdma_word!(game, hdma_index, B));
    game.store_source_word(LiveOwner::WindowLeft, 0, native_order(), scroll);
}

fn advance_statue_program<H: crate::hdma_objects::HdmaTable + ?Sized>(
    hdma_index: usize,
    game: &mut H,
) {
    crate::hdma_word!(game, hdma_index, InstructionTimer => 1);
    crate::hdma_word!(game, hdma_index, InstructionList => crate::hdma_word!(game, hdma_index, InstructionList).wrapping_add(2));
}

pub fn scroll_weather<H: Host>(slot: u16, weather: Weather, game: &mut H) {
    game.store_display_control_byte(DisplayField::Background3Map, 92, DisplayStore::Named);
    {
        let fx_value = game.room_blending().liquid;
        let mut fx = game.fx_controls();
        fx.blending = fx_value;
        game.publish_fx_controls(fx, &[crate::fx_state::Field::Blending]);
    };
    if game.session().timing.time_frozen != 0 {
        return;
    }
    game.effect(HdmaEffect::ScrollWeather(weather), usize::from(slot >> 1));
}
