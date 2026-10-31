//! Lower Norfair's spike platform, which waits, falls, rests and rises again, carrying
//! Samus while she stands on it (bank $A6:8B09-$A6:8CFA). Was `spikey_platforms`.
#![forbid(unsafe_code)]
use crate::actors::combat;
use crate::actors::host::{ActorLease, LeaseActors, abs16};
use crate::actors::layouts::part_view;
use crate::enemy_projectiles::host::Effects as _;
use crate::ram::words as W;
use crate::session_slot;
use sm_data::enemy_speeds::SpeedTable;

part_view! {
    /// The spike platform top's slot and extended RAM. Parameter 1's low byte selects its
    /// falling speed; parameter 2's low byte is its fall wait and high byte its drop.
    SpikePlatform: EnemyLayout40 {
        words: [function, fall_wait_timer, rising_target_y, falling_target_y, unread_speed_fraction,
            unread_speed, parameter_1, parameter_2],
        pad1: [u8; 26696],
        words: [fall_wait_reset, previous_y, speed_index, rise_wait_timer, var_04, var_05, var_06, var_07,
            var_08, var_09, var_0a, var_0b, var_0c, var_0d, var_0e, var_0f, var_10, var_11, var_12, var_13,
            var_14, var_15, var_16, var_17, var_18, var_19, var_1a, var_1b, var_1c, var_1d, var_1e, var_1f],
    }
}

/// Spike platform functions ($A6:8BB4-$A6:8C5D).
const WAITING_TO_FALL: u16 = 0x8bb4;
const FALLING: u16 = 0x8bdc;
const WAITING_TO_RISE: u16 = 0x8c4a;
const RISING: u16 = 0x8c5d;
/// Instruction list - spike platform ($A6:8B29).
const INSTRUCTIONS: u16 = 0x8b29;

/// The linear speed table ($A0:8187) word at word `index`.
fn speed_word(rt: &impl ActorLease, index: u16) -> u16 {
    rt.lease_enemy_speed(SpeedTable::Linear, usize::from(index).cast_signed())
}
/// The falling speed table index for parameter 1's low byte.
fn initial_speed(rt: &impl ActorLease, slot: i32) -> u16 {
    8 * u16::from(session_slot!(byte rt, SpikePlatform[slot].parameter_1[0]))
}

/// Initialisation AI - spike platform top ($A6:8B2F).
pub fn initialize<H: LeaseActors + ?Sized>(game: &mut H) {
    let mut lease = game.lease();
    let rt = &mut lease;
    let slot = combat::current_slot(rt);
    session_slot!(
        rt,
        SpikePlatform[slot].base.current_instruction = INSTRUCTIONS
    );
    session_slot!(rt, SpikePlatform[slot].function = WAITING_TO_FALL);
    let speed = initial_speed(rt, slot);
    session_slot!(rt, SpikePlatform[slot].speed_index = speed);
    let fraction = speed_word(rt, (speed >> 1) + 1);
    session_slot!(rt, SpikePlatform[slot].unread_speed_fraction = fraction);
    let whole = speed_word(rt, speed >> 1);
    session_slot!(rt, SpikePlatform[slot].unread_speed = whole);
    let drop = u16::from(session_slot!(byte rt, SpikePlatform[slot].parameter_2[1]));
    let y = session_slot!(rt, SpikePlatform[slot].base.y_pos);
    session_slot!(rt, SpikePlatform[slot]; falling_target_y = y.wrapping_add(drop), rising_target_y = y);
    let wait = u16::from(session_slot!(byte rt, SpikePlatform[slot].parameter_2[0]));
    session_slot!(rt, SpikePlatform[slot].fall_wait_reset = wait);
    session_slot!(rt, SpikePlatform[slot].fall_wait_timer = wait);
}

/// Main AI - spike platform top ($A6:8BAD): its function.
pub fn run<H: LeaseActors + ?Sized>(game: &mut H) {
    let mut lease = game.lease();
    let rt = &mut lease;
    let slot = combat::current_slot(rt);
    let start = session_slot!(rt, SpikePlatform[slot].base.y_pos);
    match session_slot!(rt, SpikePlatform[slot].function) {
        WAITING_TO_FALL => {
            let timer = session_slot!(rt, SpikePlatform[slot].fall_wait_timer).wrapping_sub(1);
            session_slot!(rt, SpikePlatform[slot].fall_wait_timer = timer);
            if timer == 0 {
                let reset = session_slot!(rt, SpikePlatform[slot].fall_wait_reset);
                session_slot!(rt, SpikePlatform[slot].fall_wait_timer = reset);
                let speed = initial_speed(rt, slot);
                session_slot!(rt, SpikePlatform[slot]; speed_index = speed, function = FALLING);
            }
        }
        FALLING => {
            session_slot!(rt, SpikePlatform[slot].previous_y = start);
            let speed = session_slot!(rt, SpikePlatform[slot].speed_index);
            let velocity = (u32::from(speed_word(rt, speed >> 1)) << 16)
                | u32::from(speed_word(rt, (speed >> 1) + 1));
            session_slot!(rt, SpikePlatform[slot].base(y_pos, y_subpos) += velocity);
            let target = session_slot!(rt, SpikePlatform[slot].falling_target_y);
            if session_slot!(rt, SpikePlatform[slot].base.y_pos).wrapping_sub(target) & 0x8000 == 0
            {
                session_slot!(rt, SpikePlatform[slot]; rise_wait_timer = 64, function = WAITING_TO_RISE);
                session_slot!(rt, SpikePlatform[slot].base.y_pos = target);
                rt.queue_sound(2, 6, 0x1b);
            }
            carry_samus(rt, slot, start);
            let speed = session_slot!(rt, SpikePlatform[slot].speed_index);
            let fast = speed.wrapping_sub(504) & 0x8000 == 0;
            session_slot!(
                rt,
                SpikePlatform[slot].speed_index = if fast { 512 } else { speed.wrapping_add(8) }
            );
        }
        WAITING_TO_RISE => {
            let timer = session_slot!(rt, SpikePlatform[slot].rise_wait_timer).wrapping_sub(1);
            session_slot!(rt, SpikePlatform[slot].rise_wait_timer = timer);
            if timer == 0 {
                session_slot!(rt, SpikePlatform[slot].function = RISING);
            }
        }
        RISING => {
            session_slot!(rt, SpikePlatform[slot].previous_y = start);
            session_slot!(
                rt,
                SpikePlatform[slot].base(y_pos, y_subpos) += 0x8000u32.wrapping_neg()
            );
            let target = session_slot!(rt, SpikePlatform[slot].rising_target_y);
            if session_slot!(rt, SpikePlatform[slot].base.y_pos).wrapping_sub(target) & 0x8000 != 0
            {
                session_slot!(rt, SpikePlatform[slot].function = WAITING_TO_FALL);
                session_slot!(rt, SpikePlatform[slot].base.y_pos = target);
            }
            carry_samus(rt, slot, start);
        }
        _ => rt.unreachable(),
    }
}

/// Check if spike platform is touching Samus from below ($A6:8CA1); if she stands on it,
/// she moves with it from `start`.
fn carry_samus(rt: &mut impl ActorLease, slot: i32, start: u16) {
    let (x, y) = (
        session_slot!(rt, SpikePlatform[slot].base.x_pos),
        session_slot!(rt, SpikePlatform[slot].base.y_pos),
    );
    let width = session_slot!(rt, SpikePlatform[slot].base.x_width);
    let height = session_slot!(rt, SpikePlatform[slot].base.y_height);
    let dx = abs16(rt.player_x().wrapping_sub(x));
    let samus_width = rt.word(W::SAMUS_X_RADIUS);
    let feet = rt.player_y().wrapping_add(5);
    if (dx >= samus_width && dx - samus_width >= width) || feet.wrapping_sub(y) & 0x8000 == 0 {
        return;
    }
    let (gap, samus_height) = (y.wrapping_sub(feet), rt.word(W::SAMUS_Y_RADIUS));
    if gap < samus_height || gap - samus_height <= height {
        let moved = rt
            .word(W::EXTRA_SAMUS_Y_DISPLACEMENT)
            .wrapping_add(y.wrapping_sub(start));
        rt.write_slice_le(W::EXTRA_SAMUS_Y_DISPLACEMENT, moved);
    }
}
