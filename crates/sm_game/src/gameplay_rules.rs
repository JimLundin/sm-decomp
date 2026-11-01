//! Per-frame gameplay bookkeeping: health and game time, screen shakes and
//! frame timers, on named RAM.
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
    let inventory = crate::session_cohort::load::<crate::inventory::Inventory, _>(game);
    if (inventory.health.current as i16) <= 0 {
        if (inventory.reserve_mode & 1) != 0 && inventory.reserve_health != 0 {
            word!(game.TIME_IS_FROZEN_FLAG = 0x8000);
            word!(game.GAME_STATE = K_GAME_STATE_27_RESERVE_TANKS_AUTO as u16);
            game.player_command(0x1b);
        } else if word!(game.GAME_STATE) as i32 != K_GAME_STATE_8_MAIN_GAMEPLAY as i32 {
            return;
        } else {
            word!(game.TIME_IS_FROZEN_FLAG = 0x8000);
            game.player_command(0x11);
            word!(game.GAME_STATE = K_GAME_STATE_19_SAMUS_NO_HEALTH as u16);
        }
    }
    // Clock words are frames, seconds, minutes, then hours in ascending RAM order.
    'carry: {
        for timer in (fields::GAME_TIME_FRAMES..fields::GAME_TIME_HOURS).step_by(2) {
            checked_word!(game, timer; += 1);
            if (checked_word!(game, timer;).wrapping_sub(60) as i16) < 0 {
                break 'carry;
            }
            checked_word!(game, timer; = 0);
        }
        word!(game.GAME_TIME_HOURS += 1);
    }
    if (word!(game.GAME_TIME_HOURS).wrapping_sub(100) as i16) >= 0 {
        word! { game, 59; GAME_TIME_FRAMES, GAME_TIME_SECONDS, GAME_TIME_MINUTES }
        word!(game.GAME_TIME_HOURS = 99);
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
    let quake = crate::session_cohort::load::<crate::room_view::RoomEffects, _>(game);
    if quake.earthquake_timer != 0
        && word!(game.TIME_IS_FROZEN_FLAG) == 0
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
            fields::REG_BG1HOFS,
            fields::REG_BG1VOFS,
            fields::REG_BG2HOFS,
            fields::REG_BG2VOFS,
        ]
        .into_iter()
        .enumerate()
        {
            checked_word!(game, register; += ROOM_SHAKES[index + offset].wrapping_mul(direction));
        }
        let timer = quake.earthquake_timer.wrapping_sub(1);
        crate::session_cohort::with(game, |_, room: &mut crate::room_view::RoomEffects| {
            room.earthquake_timer = timer
        });
        // Kinds from 18 also shake the active enemies.
        if (quake.earthquake_kind as i16).wrapping_sub(18) >= 0 {
            crate::actors::scheduling::shake_active_enemies(&mut game.runtime());
        }
    }
    word!(game.FRAME_COUNTER_EVERY_FRAME += 1);
    for timer in [
        fields::SAMUS_INVINCIBILITY_TIMER,
        fields::SAMUS_KNOCKBACK_TIMER,
        fields::PROJECTILE_INVINCIBILITY_TIMER,
    ] {
        let remaining = checked_word!(game, timer;);
        if remaining != 0 {
            checked_word!(game, timer; = remaining - 1);
        }
    }
    word!(game.INTERACTIVE_ENEMY_INDEXES[0] = u16::MAX);
    word!(game.ACTIVE_ENEMY_INDEXES[0] = u16::MAX);
}
