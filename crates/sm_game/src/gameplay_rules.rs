//! Per-frame gameplay bookkeeping: health and game time, screen shakes and
//! frame timers, on the gameplay pass's lease.
use crate::constants::game_states::*;
use crate::runtime_access::PlayerLease;

/// The lease the bookkeeping runs on: the gameplay pass's, with the game-loop
/// and room-effect words.
pub trait Host: PlayerLease + crate::gameplay_context::Access {}
impl<T> Host for T where T: PlayerLease + crate::gameplay_context::Access + ?Sized {}

pub fn finish_health_and_time<H: Host>(lease: &mut H) {
    let inventory = crate::gameplay_context::inventory(lease);
    if (inventory.health.current as i16) <= 0 {
        if (inventory.reserve_mode & 1) != 0 && inventory.reserve_health != 0 {
            freeze_time(lease);
            crate::gameplay_word!(lease.GAME_STATE = K_GAME_STATE_27_RESERVE_TANKS_AUTO as u16);
            lease.native_player_command(0x1b);
        } else if lease.session().game_loop.mode as i32 != K_GAME_STATE_8_MAIN_GAMEPLAY as i32 {
            return;
        } else {
            freeze_time(lease);
            lease.native_player_command(0x11);
            crate::gameplay_word!(lease.GAME_STATE = K_GAME_STATE_19_SAMUS_NO_HEALTH as u16);
        }
    }
    // Clock words are frames, seconds, minutes, then hours in ascending RAM order.
    'carry: {
        for timer in [
            crate::game_loop::Word::GameTimeFrames,
            crate::game_loop::Word::GameTimeSeconds,
            crate::game_loop::Word::GameTimeMinutes,
        ]
        .map(crate::gameplay_context::Word::Loop)
        {
            let value =
                crate::gameplay_context::Access::gameplay_word(lease, timer).wrapping_add(1);
            crate::gameplay_context::Access::set_gameplay_word(lease, timer, value);
            if (crate::gameplay_context::Access::gameplay_word(lease, timer).wrapping_sub(60)
                as i16)
                < 0
            {
                break 'carry;
            }
            crate::gameplay_context::Access::set_gameplay_word(lease, timer, 0);
        }
        crate::gameplay_word!(lease.GAME_TIME_HOURS += 1);
    }
    if (lease.session().game_loop.clock.hours.wrapping_sub(100) as i16) >= 0 {
        crate::gameplay_word! { lease, 59; GAME_TIME_FRAMES, GAME_TIME_SECONDS, GAME_TIME_MINUTES }
        crate::gameplay_word!(lease.GAME_TIME_HOURS = 99);
    }
}

/// Freeze time ($0A78) for the reserve tanks or the death sequence.
fn freeze_time<H: Host>(lease: &mut H) {
    lease.session_mut().timing.time_frozen = 0x8000;
    lease.words_stored();
}

/// Each earthquake kind's BG1 x, BG1 y, BG2 x and BG2 y shake amplitudes.
const ROOM_SHAKES: [u16; 144] = [
    1, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 2, 0, 0, 0, 0, 2, 0, 0, 2, 2, 0, 0, 3, 0, 0, 0, 0, 3, 0, 0,
    3, 3, 0, 0, 1, 0, 1, 0, 0, 1, 0, 1, 1, 1, 1, 1, 2, 0, 2, 0, 0, 2, 0, 2, 2, 2, 2, 2, 3, 0, 3, 0,
    0, 3, 0, 3, 3, 3, 3, 3, 1, 0, 1, 0, 0, 1, 0, 1, 1, 1, 1, 1, 2, 0, 2, 0, 0, 2, 0, 2, 2, 2, 2, 2,
    3, 0, 3, 0, 0, 3, 0, 3, 3, 3, 3, 3, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 2, 0, 0, 0, 0, 2,
    0, 0, 2, 2, 0, 0, 3, 0, 0, 0, 0, 3, 0, 0, 3, 3,
];

pub fn finish_frame<H: Host>(lease: &mut H) {
    let quake = crate::gameplay_context::room_effects(lease);
    if quake.earthquake_timer != 0
        && lease.session().timing.time_frozen == 0
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
            crate::display_scroll::Word::Bg1H,
            crate::display_scroll::Word::Bg1V,
            crate::display_scroll::Word::Bg2H,
            crate::display_scroll::Word::Bg2V,
        ]
        .into_iter()
        .enumerate()
        {
            let old = lease.scroll_word(register);
            let value = old.wrapping_add(ROOM_SHAKES[index + offset].wrapping_mul(direction));
            lease.set_scroll_word(register, value);
        }
        crate::gameplay_context::set_effect(
            lease,
            crate::room_view::EffectWord::EarthquakeTimer,
            quake.earthquake_timer.wrapping_sub(1),
        );
        // Kinds from 18 also shake the active enemies.
        if (quake.earthquake_kind as i16).wrapping_sub(18) >= 0 {
            crate::actors::scheduling::shake_active_enemies(lease);
        }
    }
    // Samus's invincibility and knockback timers, the frame counter, the
    // projectile protection timer and the first interactive and active enemy
    // entries are Session fields no other step here reads.
    let session = lease.session_mut();
    for timer in [
        &mut session.player.invincibility,
        &mut session.player.knockback_timer,
    ] {
        if *timer != 0 {
            *timer -= 1;
        }
    }
    session.timing.frame_counter = session.timing.frame_counter.wrapping_add(1);
    let protection = &mut session.projectiles.controls.protection;
    if *protection != 0 {
        *protection -= 1;
    }
    session.actors.schedule.interactive[0] = u16::MAX;
    session.actors.schedule.active[0] = u16::MAX;
    lease.words_stored();
}
