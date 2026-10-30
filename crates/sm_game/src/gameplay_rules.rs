//! Per-frame gameplay bookkeeping: health and game time, screen shakes and
//! frame timers, on named RAM.
#![forbid(unsafe_code)]
use crate::constants::game_states::*;
use crate::ram::{checked_word, word, words as fields};
use crate::runtime_access::RuntimeAccess;

/// Named RAM, the Samus pose command and the owned-Session view.
pub trait Host:
    crate::ram::CompatibilityRam + crate::menus::host::MenuRuntime + RuntimeAccess
{
}
impl<T> Host for T where
    T: crate::ram::CompatibilityRam + crate::menus::host::MenuRuntime + RuntimeAccess + ?Sized
{
}

pub fn finish_health_and_time<H: Host + ?Sized>(game: &mut H) {
    let inventory = crate::inventory::current(game);
    if (inventory.health.current as i16) <= 0 {
        if (inventory.reserve_mode & 1) != 0 && inventory.reserve_health != 0 {
            word!(game.time_is_frozen_flag = 0x8000);
            word!(game.game_state = kGameState_27_ReserveTanksAuto as u16);
            game.player_command(0x1b);
        } else if word!(game.game_state) as i32 != kGameState_8_MainGameplay as i32 {
            return;
        } else {
            word!(game.time_is_frozen_flag = 0x8000);
            game.player_command(0x11);
            word!(game.game_state = kGameState_19_SamusNoHealth as u16);
        }
    }
    // Clock words are frames, seconds, minutes, then hours in ascending RAM order.
    'carry: {
        for timer in (fields::game_time_frames..fields::game_time_hours).step_by(2) {
            checked_word!(game, timer; += 1);
            if (checked_word!(game, timer;).wrapping_sub(60) as i16) < 0 {
                break 'carry;
            }
            checked_word!(game, timer; = 0);
        }
        word!(game.game_time_hours += 1);
    }
    if (word!(game.game_time_hours).wrapping_sub(100) as i16) >= 0 {
        word! { game, 59; game_time_frames, game_time_seconds, game_time_minutes }
        word!(game.game_time_hours = 99);
    }
}

/// Each earthquake kind's BG1 x, BG1 y, BG2 x and BG2 y shake amplitudes.
const ROOM_SHAKES: [u16; 144] = [
    1, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 2, 0, 0, 0, 0, 2, 0, 0, 2, 2, 0, 0, 3, 0, 0, 0, 0, 3, 0, 0,
    3, 3, 0, 0, 1, 0, 1, 0, 0, 1, 0, 1, 1, 1, 1, 1, 2, 0, 2, 0, 0, 2, 0, 2, 2, 2, 2, 2, 3, 0, 3, 0,
    0, 3, 0, 3, 3, 3, 3, 3, 1, 0, 1, 0, 0, 1, 0, 1, 1, 1, 1, 1, 2, 0, 2, 0, 0, 2, 0, 2, 2, 2, 2, 2,
    3, 0, 3, 0, 0, 3, 0, 3, 3, 3, 3, 3, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 2, 0, 0, 0, 0, 2,
    0, 0, 2, 2, 0, 0, 3, 0, 0, 0, 0, 3, 0, 0, 3, 3,
];

pub fn finish_frame<H: Host + ?Sized>(game: &mut H) {
    let quake = crate::room_view::effects(game);
    if quake.earthquake_timer != 0
        && word!(game.time_is_frozen_flag) == 0
        && (quake.earthquake_kind as i16).wrapping_sub(36) < 0
    {
        let index = usize::from(quake.earthquake_kind) * 4;
        // The shake alternates direction every two frames.
        let direction = if quake.earthquake_timer & 2 != 0 {
            u16::MAX
        } else {
            1
        };
        for (offset, register) in [
            fields::reg_BG1HOFS,
            fields::reg_BG1VOFS,
            fields::reg_BG2HOFS,
            fields::reg_BG2VOFS,
        ]
        .into_iter()
        .enumerate()
        {
            checked_word!(game, register; += ROOM_SHAKES[index + offset].wrapping_mul(direction));
        }
        crate::room_view::set_effect(
            game,
            crate::room_view::EffectWord::EarthquakeTimer,
            quake.earthquake_timer.wrapping_sub(1),
        );
        // Kinds from 18 also shake the active enemies.
        if (quake.earthquake_kind as i16).wrapping_sub(18) >= 0 {
            crate::actors::scheduling::shake_active_enemies(&mut game.runtime());
        }
    }
    word!(game.frame_counter_every_frame += 1);
    for timer in [
        fields::samus_invincibility_timer,
        fields::samus_knockback_timer,
        fields::projectile_invincibility_timer,
    ] {
        let remaining = checked_word!(game, timer;);
        if remaining != 0 {
            checked_word!(game, timer; = remaining - 1);
        }
    }
    word!(game.interactive_enemy_indexes[0] = u16::MAX);
    word!(game.active_enemy_indexes[0] = u16::MAX);
}
