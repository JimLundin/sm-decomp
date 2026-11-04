//! Bull, Maridia's gas bag, which waits, then speeds at Samus and slows when she slips
//! off its heading; a shot knocks it away (bank $A8:D821-$A8:DBC6). Was `maridia_puffer`.
#![forbid(unsafe_code)]
use crate::actors::boss_lease::BossLease;
use crate::actors::host::{Shot, abs16, sign_extend8};
use crate::actors::slot_view::{SlotState, slot_view};
use crate::gameplay_tables::enemy_sine;

/// A lease on a game with the bulls' effects; its slots are the Session's.
pub trait Lease: BossLease + SlotState {}
impl<T: BossLease + SlotState + ?Sized> Lease for T {}

slot_view! {
    /// A bull's slot: its function, acceleration and its step, speed, activation and
    /// acceleration timers; its max speed, angles to Samus and of travel, the shot
    /// reaction's disable flag and timer, the acceleration and deceleration intervals,
    /// and its health before a shot. Parameter 1 picks the intervals and 2 the max speed.
    Bull, BullEdit {
        record: [
            function = var_a, acceleration = var_b, acceleration_delta = var_c, speed = var_d,
            activation_timer = var_e, acceleration_timer = var_f, parameter_1 = parameter_1,
            parameter_2 = parameter_2,
        ],
        extended: [
            max_speed = var_00, angle_to_samus = var_01, angle = var_02,
            shot_reaction_disabled = var_03, acceleration_interval = var_05,
            deceleration_interval = var_06, shot_reaction_timer = var_07, previous_health = var_40,
        ],
    }
}

const MOVEMENT_DELAY: u16 = 0xd92b;
const TARGET_SAMUS: u16 = 0xd940;
const ACCELERATING: u16 = 0xd963;
const DECELERATING: u16 = 0xd97c;
/// The knockback angle by the shot's direction.
const RECOIL_ANGLES: [u16; 10] = [0xc0, 0xe0, 0, 0x20, 0x40, 0x40, 0x60, 0x80, 0xa0, 0xc0];
/// $A8:D885: the max speeds; $A8:D895: the acceleration then deceleration intervals.
const MAX_SPEEDS: [u16; 8] = [0x3ff, 0x4ff, 0x5ff, 0x6ff, 0x7ff, 0x8ff, 0x9ff, 0xaff];
const INTERVALS: [u16; 26] = [
    3, 1, 4, 1, 5, 2, 6, 2, 7, 2, 8, 3, 9, 3, 0xa, 4, 0xb, 4, 0xc, 5, 0xd, 5, 0xe, 6, 0xf, 6,
];

/// Initialisation AI - bull ($A8:D8C9).
pub fn initialize(game: &mut impl Lease) {
    let bull = game.current_enemy();
    {
        let mut edit = BullEdit::of(game, bull);
        edit.actor().instruction_delay = 1;
        edit.actor().timer = 0;
        edit.actor().instruction = 0xd841;
    }
    let index = 4u16.wrapping_mul(Bull::of(game, bull).parameter_1()) as usize >> 1;
    let max_speed = MAX_SPEEDS[Bull::of(game, bull).parameter_2() as usize];
    let mut edit = BullEdit::of(game, bull);
    *edit.acceleration_interval() = INTERVALS[index];
    *edit.acceleration_timer() = INTERVALS[index];
    *edit.deceleration_interval() = INTERVALS[index + 1];
    *edit.activation_timer() = 16;
    *edit.function() = MOVEMENT_DELAY;
    *edit.max_speed() = max_speed;
}
/// The angle to Samus, from the positive X axis.
fn aim(game: &mut impl Lease) -> (u16, u16) {
    let angle = (game.player_angle(game.current_enemy()) as u8).wrapping_sub(64) as u16;
    let bull = game.current_enemy();
    *BullEdit::of(game, bull).angle_to_samus() = angle;
    (bull, angle)
}
/// Bull X and Y movement ($A8:D9DB, $A8:DA28): along its angle at its speed.
fn drift(game: &mut impl Lease) {
    for horizontal in [true, false] {
        let bull = game.current_enemy();
        let quarter = if horizontal { 64 } else { 0 };
        let angle = (Bull::of(game, bull).angle() as u8).wrapping_add(quarter);
        let component = enemy_sine(game, angle as isize);
        let speed = u32::from(Bull::of(game, bull).speed());
        let mut amount = u32::from(abs16(component) >> 8) * speed;
        if component & 0x8000 != 0 {
            amount = amount.wrapping_neg();
        }
        let mut edit = BullEdit::of(game, bull);
        let actor = edit.actor();
        let position = if horizontal {
            &mut actor.x
        } else {
            &mut actor.y
        };
        position.advance(amount.cast_signed());
    }
}
/// Accelerate ($A8:DA71) or decelerate ($A8:DA92), each interval.
fn change_speed(game: &mut impl Lease, accelerating: bool) {
    let bull = game.current_enemy();
    let previous = Bull::of(game, bull).acceleration_timer();
    let timer = previous.wrapping_sub(1);
    *BullEdit::of(game, bull).acceleration_timer() = timer;
    if previous == 1 {
        let interval = match accelerating {
            true => Bull::of(game, bull).acceleration_interval(),
            false => Bull::of(game, bull).deceleration_interval(),
        };
        *BullEdit::of(game, bull).acceleration_timer() = interval;
        let direction = if accelerating { 1 } else { u16::MAX };
        let delta = Bull::of(game, bull)
            .acceleration_delta()
            .wrapping_mul(direction);
        {
            let mut edit = BullEdit::of(game, bull);
            let old = *edit.acceleration();
            *edit.acceleration() = old.wrapping_add(delta);
        }
        let acceleration = Bull::of(game, bull).acceleration().wrapping_mul(direction);
        let mut edit = BullEdit::of(game, bull);
        let old = *edit.speed();
        *edit.speed() = old.wrapping_add(acceleration);
    }
}
/// Main AI - bull ($A8:D90B): re-enable the shot reaction, and run its function.
pub fn main_ai(game: &mut impl Lease) {
    let bull = game.current_enemy();
    let timer = Bull::of(game, bull).shot_reaction_timer().wrapping_sub(1);
    *BullEdit::of(game, bull).shot_reaction_timer() = timer;
    if timer == 0 {
        let mut edit = BullEdit::of(game, bull);
        *edit.shot_reaction_timer() = 1;
        *edit.shot_reaction_disabled() = 0;
    }
    match Bull::of(game, bull).function() {
        MOVEMENT_DELAY => {
            let previous = Bull::of(game, bull).activation_timer();
            *BullEdit::of(game, bull).activation_timer() = previous.wrapping_sub(1);
            if previous == 1 {
                let mut edit = BullEdit::of(game, bull);
                *edit.activation_timer() = 16;
                *edit.function() = TARGET_SAMUS;
            }
        }
        TARGET_SAMUS => {
            let (bull, angle) = aim(game);
            let mut edit = BullEdit::of(game, bull);
            *edit.angle() = angle;
            *edit.function() = ACCELERATING;
            *edit.acceleration_delta() = 24;
        }
        ACCELERATING => {
            let speed = Bull::of(game, bull).speed() as i16;
            if speed.wrapping_sub(Bull::of(game, bull).max_speed() as i16) < 0 {
                change_speed(game, true);
            }
            drift(game);
            // Trigger braking if too far off target ($A8:D9AA).
            let (bull, angle) = aim(game);
            let off_target = angle.wrapping_sub(Bull::of(game, bull).angle());
            let turn = abs16(sign_extend8(off_target));
            if (turn as i32 - 48) & 0x8000 == 0 {
                let mut edit = BullEdit::of(game, bull);
                *edit.function() = DECELERATING;
                *edit.acceleration_delta() = 24;
            }
        }
        DECELERATING => {
            let moving = (Bull::of(game, bull).speed() as i16) > 0;
            if moving && (Bull::of(game, bull).acceleration() as i16) > 0 {
                drift(game);
                change_speed(game, false);
            } else {
                let mut edit = BullEdit::of(game, bull);
                *edit.function() = MOVEMENT_DELAY;
                *edit.acceleration() = 0;
                *edit.acceleration_delta() = 0;
                *edit.speed() = 0;
            }
        }
        _ => game.unreachable(),
    }
}
/// Enemy shot - bull: knocked away from the shot unless it hurt or was recently shot.
pub fn shot(game: &mut impl Lease) {
    let bull = game.current_enemy();
    let health = Bull::of(game, bull).actor().health;
    *BullEdit::of(game, bull).previous_health() = health;
    game.shot_reaction(Shot::Finalize);
    let hurt = Bull::of(game, bull).actor().health != health;
    if !hurt && Bull::of(game, bull).shot_reaction_disabled() == 0 {
        {
            let mut edit = BullEdit::of(game, bull);
            edit.actor().instruction_delay = 1;
            edit.actor().timer = 0;
            edit.actor().instruction = 0xd855;
        }
        let direction = game.shot_direction() & 0xf;
        *BullEdit::of(game, bull).angle() = RECOIL_ANGLES[direction as usize];
        let mut edit = BullEdit::of(game, bull);
        *edit.acceleration() = 256;
        *edit.speed() = 1536;
        *edit.function() = DECELERATING;
        *edit.shot_reaction_timer() = 48;
        *edit.shot_reaction_disabled() = 1;
    }
}
