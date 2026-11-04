//! Lower Norfair's spike platform, which waits, falls, rests and rises again, carrying
//! Samus while she stands on it (bank $A6:8B09-$A6:8CFA).
#![forbid(unsafe_code)]
use crate::actors::combat;
use crate::actors::host::{ActorLease, abs16};
use crate::actors::slot_view::{SlotState, slot_view};
use sm_data::enemy_speeds::SpeedTable;

slot_view! {
    /// The spike platform top's slot and extended RAM. Parameter 1's low byte selects its
    /// falling speed; parameter 2's low byte is its fall wait and high byte its drop.
    SpikePlatform, SpikePlatformEdit {
        record: [
            function = var_a, fall_wait_timer = var_b, rising_target_y = var_c,
            falling_target_y = var_d, unread_speed_fraction = var_e, unread_speed = var_f,
            parameter_1 = parameter_1, parameter_2 = parameter_2,
        ],
        extended: [
            fall_wait_reset = var_00, previous_y = var_01, speed_index = var_02,
            rise_wait_timer = var_03,
        ],
    }
}

/// A lease the spike platform runs on: its slot is the Session's.
pub trait Lease: ActorLease + SlotState {}
impl<T: ActorLease + SlotState + ?Sized> Lease for T {}

/// Spike platform functions ($A6:8BB4-$A6:8C5D).
const WAITING_TO_FALL: u16 = 0x8bb4;
const FALLING: u16 = 0x8bdc;
const WAITING_TO_RISE: u16 = 0x8c4a;
const RISING: u16 = 0x8c5d;
/// Instruction list - spike platform ($A6:8B29).
const INSTRUCTIONS: u16 = 0x8b29;
/// The rise: half a pixel a frame, upwards (16.16).
const RISE_VELOCITY: i32 = -0x8000;

/// The linear speed table ($A0:8187) word at word `index`.
fn speed_word(rt: &impl ActorLease, index: u16) -> u16 {
    rt.lease_enemy_speed(SpeedTable::Linear, usize::from(index).cast_signed())
}
/// The falling speed table index for parameter 1's low byte.
fn initial_speed(parameter_1: u16) -> u16 {
    8 * (parameter_1 & 0xff)
}

/// Initialisation AI - spike platform top ($A6:8B2F).
pub fn initialize(lease: &mut impl crate::actors::species::TableLease) {
    let rt = lease;
    let k = combat::current(rt).byte_index();
    let speed = {
        let mut platform = SpikePlatformEdit::of(rt, k);
        platform.actor().instruction = INSTRUCTIONS;
        *platform.function() = WAITING_TO_FALL;
        let speed = initial_speed(*platform.parameter_1());
        *platform.speed_index() = speed;
        speed
    };
    let fraction = speed_word(rt, (speed >> 1) + 1);
    *SpikePlatformEdit::of(rt, k).unread_speed_fraction() = fraction;
    let whole = speed_word(rt, speed >> 1);
    let mut platform = SpikePlatformEdit::of(rt, k);
    *platform.unread_speed() = whole;
    let drop = *platform.parameter_2() >> 8;
    let y = platform.actor().y.whole;
    *platform.falling_target_y() = y.wrapping_add(drop);
    *platform.rising_target_y() = y;
    let wait = *platform.parameter_2() & 0xff;
    *platform.fall_wait_reset() = wait;
    *platform.fall_wait_timer() = wait;
}

/// Main AI - spike platform top ($A6:8BAD): its function.
pub fn run(lease: &mut impl crate::actors::species::TableLease) {
    crate::probe::hit("spike_platform.run");
    let rt = lease;
    let k = combat::current(rt).byte_index();
    let platform = SpikePlatform::of(rt, k);
    let start = platform.actor().y.whole;
    match platform.function() {
        WAITING_TO_FALL => {
            let mut platform = SpikePlatformEdit::of(rt, k);
            let timer = platform.fall_wait_timer().wrapping_sub(1);
            *platform.fall_wait_timer() = timer;
            if timer == 0 {
                let reset = *platform.fall_wait_reset();
                *platform.fall_wait_timer() = reset;
                let speed = initial_speed(*platform.parameter_1());
                *platform.speed_index() = speed;
                *platform.function() = FALLING;
            }
        }
        FALLING => {
            let speed = {
                let mut platform = SpikePlatformEdit::of(rt, k);
                *platform.previous_y() = start;
                *platform.speed_index()
            };
            let velocity = (u32::from(speed_word(rt, speed >> 1)) << 16)
                | u32::from(speed_word(rt, (speed >> 1) + 1));
            let landed = {
                let mut platform = SpikePlatformEdit::of(rt, k);
                platform.actor().y.advance(velocity.cast_signed());
                let target = *platform.falling_target_y();
                let landed = platform.actor().y.whole.wrapping_sub(target) & 0x8000 == 0;
                if landed {
                    *platform.rise_wait_timer() = 64;
                    *platform.function() = WAITING_TO_RISE;
                    platform.actor().y.whole = target;
                }
                landed
            };
            if landed {
                crate::enemy_projectiles::host::Effects::queue_sound(rt, 2, 6, 0x1b);
            }
            carry_samus(rt, k, start);
            let mut platform = SpikePlatformEdit::of(rt, k);
            let speed = *platform.speed_index();
            let fast = speed.wrapping_sub(504) & 0x8000 == 0;
            *platform.speed_index() = if fast { 512 } else { speed.wrapping_add(8) };
        }
        WAITING_TO_RISE => {
            let mut platform = SpikePlatformEdit::of(rt, k);
            let timer = platform.rise_wait_timer().wrapping_sub(1);
            *platform.rise_wait_timer() = timer;
            if timer == 0 {
                *platform.function() = RISING;
            }
        }
        RISING => {
            {
                let mut platform = SpikePlatformEdit::of(rt, k);
                *platform.previous_y() = start;
                platform.actor().y.advance(RISE_VELOCITY);
                let target = *platform.rising_target_y();
                if platform.actor().y.whole.wrapping_sub(target) & 0x8000 != 0 {
                    *platform.function() = WAITING_TO_FALL;
                    platform.actor().y.whole = target;
                }
            }
            carry_samus(rt, k, start);
        }
        _ => crate::diagnostics::Fatal::unreachable(rt),
    }
}

/// Check if spike platform is touching Samus from below ($A6:8CA1); if she stands on it,
/// she moves with it from `start`.
fn carry_samus(rt: &mut impl Lease, k: u16, start: u16) {
    let actor = SpikePlatform::of(rt, k).actor();
    let (x, y, width, height) = (actor.x.whole, actor.y.whole, actor.radius.x, actor.radius.y);
    let dx = abs16(rt.player_x().wrapping_sub(x));
    let samus_width = rt.session().player.contact.x_radius;
    let feet = rt.player_y().wrapping_add(5);
    if (dx >= samus_width && dx - samus_width >= width) || feet.wrapping_sub(y) & 0x8000 == 0 {
        return;
    }
    let (gap, samus_height) = (y.wrapping_sub(feet), rt.session().player.contact.radius);
    if gap < samus_height || gap - samus_height <= height {
        let displacement = &mut rt.session_mut().player.contact.external_y.whole;
        *displacement = displacement.wrapping_add(y.wrapping_sub(start));
    }
}
