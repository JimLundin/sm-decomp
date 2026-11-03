//! Shutters and up/down movers (bank $A2:E978-$A2:F497): the growing shutter, the up/down
//! movers (shootable and destroyable shutters, the up/down mover platform) and the horizontal
//! shutter. Was `moving_platforms`. Each frame runs on one actor lease.
use crate::actors::api::{self, Axis, ShotHandling};
use crate::actors::combat;
use crate::actors::host::ActorLease;
use crate::actors::slot_view::{SlotState, slot_view};
use crate::controller_state::Word as Input;
use crate::player_projectiles::content::table;
use sm_data::enemy_speeds::SpeedTable::Linear;

slot_view! {
    /// An up/down mover's or horizontal shutter's slot. "Positive" is down or right.
    Mover, MoverEdit {
        record: [
            function = var_a, function_timer = var_b, positive_subvelocity = var_c,
            positive_velocity = var_d, negative_subvelocity = var_e, negative_velocity = var_f,
            parameter_1 = parameter_1, parameter_2 = parameter_2,
        ],
        extended: [
            speed_index = var_00, primary_direction = var_01, negative_rest_parameter = var_02,
            positive_rest_parameter = var_03, trigger_mode = var_04, distance = var_05,
            proximity = var_06, initial_function_index = var_07, negative_rest_time = var_08,
            positive_rest_time = var_09, moving_samus = var_0a, var_0b = var_0b,
            triggered = var_0c, previous_x = var_0d, previous_y = var_0e, minimum_y = var_0f,
            maximum_y = var_10, minimum_x = var_11, maximum_x = var_12, pushing_samus = var_13,
            samus_x = var_15, samus_x_fraction = var_16, reaction_direction = var_20,
        ],
    }
}

slot_view! {
    /// The growing shutter's slot: the Y position of each growth level, the speed, and the Y
    /// position at the start of the frame.
    Growing, GrowingEdit {
        record: [
            function = var_a, level_0_y = var_b, level_1_y = var_c, level_2_y = var_d,
            level_3_y = var_e, growth_level = var_f, parameter_1 = parameter_1,
            parameter_2 = parameter_2,
        ],
        extended: [speed = var_00, subspeed = var_01, previous_y = var_40],
    }
}

/// Which enemy: the growing shutter ($D4FF), the up/down mover platform ($D5FF), the shootable
/// or destroyable shutter ($D53F, $D5BF) and the horizontal shutter ($D57F).
#[derive(Clone, Copy, PartialEq)]
pub enum Kind {
    GrowingShutter,
    Platform,
    VerticalShutter,
    HorizontalShutter,
}
/// The touch, shot and power bomb reactions.
#[derive(Clone, Copy)]
pub enum Event {
    Touch,
    Shot,
    PowerBomb,
}

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    Initial,
    WaitForSamusToGetNear,
    MovingNegative,
    MovingPositive,
    StoppedNegative,
    StoppedPositive,
    Nothing,
}
use Phase::*;

/// A lease on a game whose slots are the Session's typed storage.
pub trait Lease: ActorLease + SlotState {}
impl<T: ActorLease + SlotState + ?Sized> Lease for T {}

/// An axis's functions, by phase.
struct Motion {
    horizontal: bool,
    functions: [u16; 7],
}
const VERTICAL: Motion = Motion {
    horizontal: false,
    functions: [0xef09, 0xef28, 0xef68, 0xefd4, 0xf040, 0xf072, 0xf099],
};
const HORIZONTAL: Motion = Motion {
    horizontal: true,
    functions: [0xf224, 0xf243, 0xf272, 0xf2e4, 0xf38c, 0xf3b0, 0xf3d4],
};
impl Motion {
    fn function(&self, phase: Phase) -> u16 {
        self.functions[phase as usize]
    }
    fn phase(&self, function: u16) -> Option<Phase> {
        [
            Initial,
            WaitForSamusToGetNear,
            MovingNegative,
            MovingPositive,
            StoppedNegative,
            StoppedPositive,
            Nothing,
        ]
        .into_iter()
        .find(|&phase| self.function(phase) == function)
    }
}
impl Kind {
    fn motion(self) -> &'static Motion {
        if self == Self::HorizontalShutter {
            &HORIZONTAL
        } else {
            &VERTICAL
        }
    }
}

/// Play gate opening/closing sound effect if on-screen ($A2:EF5A).
fn movement_sound(runtime: &mut impl Lease) {
    if !api::current_outside_screen(runtime) {
        runtime.queue_sound(3, 6, 0xe);
    }
}

fn near(runtime: &mut impl Lease, distance: u16) -> bool {
    let key = combat::current(runtime);
    api::player_within(runtime, key, Axis::X, distance)
}

/// Initialisation AI ($A2:E9DA, $A2:EE05, $A2:EE12, $A2:F111).
pub fn initialize(kind: Kind, runtime: &mut impl Lease) {
    let e = combat::current(runtime).byte_index();
    if kind == Kind::GrowingShutter {
        let index = Growing::of(runtime, e).actor().instruction as i32
            + 2 * Growing::of(runtime, e).actor().extra_properties as i32;
        // $A2:EA4E: the initial functions.
        let function = table(runtime, 0xa2_ea4e, index as isize);
        *GrowingEdit::of(runtime, e).function() = function;
        let step = if Growing::of(runtime, e).actor().extra_properties != 0 {
            -8i16 as u16
        } else {
            8
        };
        let y = Growing::of(runtime, e).actor().y.whole;
        let levels = [
            y,
            y.wrapping_add(step),
            y.wrapping_add(step).wrapping_add(step),
            y.wrapping_add(step.wrapping_mul(3)),
        ];
        let index = {
            let mut growing = GrowingEdit::of(runtime, e);
            *growing.level_0_y() = levels[0];
            *growing.level_1_y() = levels[1];
            *growing.level_2_y() = levels[2];
            *growing.level_3_y() = levels[3];
            let actor = growing.actor();
            actor.extra_properties = 0;
            actor.instruction = 0xe998;
            *growing.growth_level() = 0;
            2 * isize::from(*growing.parameter_2() as u8)
        };
        let mut timer =
            |index| runtime.foreign(|game| crate::gameplay_tables::shutter_timer(game, index));
        let (speed, subspeed) = (timer(index), timer(index + 1));
        {
            let mut part = GrowingEdit::of(runtime, e);
            *part.speed() = speed;
            *part.subspeed() = subspeed;
        }
        return;
    }
    let p = kind.motion();
    let instruction = Mover::of(runtime, e).actor().instruction;
    let index = 4 * isize::from(instruction as u8);
    let [positive, positive_sub, negative, negative_sub] =
        [0, 1, 2, 3].map(|offset| runtime.lease_enemy_speed(Linear, index + offset));
    let [rest_negative, rest_positive] =
        Mover::of(runtime, e).actor().extra_properties.to_le_bytes();
    let [trigger, distance] = Mover::of(runtime, e).parameter_1().to_le_bytes();
    let proximity = Mover::of(runtime, e).parameter_2();
    {
        let mut part = MoverEdit::of(runtime, e);
        *part.speed_index() = u16::from(instruction as u8);
        *part.positive_velocity() = positive;
        *part.positive_subvelocity() = positive_sub;
        *part.negative_velocity() = negative;
        *part.negative_subvelocity() = negative_sub;
        *part.primary_direction() = instruction >> 8;
        *part.reaction_direction() = (instruction >> 8) ^ u16::from(p.horizontal);
        *part.negative_rest_parameter() = u16::from(rest_negative);
        *part.negative_rest_time() = 16 * u16::from(rest_negative);
        *part.positive_rest_parameter() = u16::from(rest_positive);
        *part.positive_rest_time() = 16 * u16::from(rest_positive);
        *part.trigger_mode() = u16::from(trigger);
        *part.initial_function_index() = 2 * u16::from(trigger);
        *part.distance() = u16::from(distance);
        *part.proximity() = proximity;
        *part.function_timer() = proximity;
    }
    let (position, upward) = if p.horizontal {
        (
            Mover::of(runtime, e).actor().x.whole,
            Mover::of(runtime, e).primary_direction() == 0,
        )
    } else {
        (
            Mover::of(runtime, e).actor().y.whole,
            Mover::of(runtime, e).primary_direction() == 0,
        )
    };
    let (low, high) = if upward {
        (position.wrapping_sub(u16::from(distance)), position)
    } else {
        (position, u16::from(distance).wrapping_add(position))
    };
    if p.horizontal {
        let mut part = MoverEdit::of(runtime, e);
        *part.minimum_x() = low;
        *part.maximum_x() = high;
    } else {
        let mut part = MoverEdit::of(runtime, e);
        *part.minimum_y() = low;
        *part.maximum_y() = high;
    }
    *MoverEdit::of(runtime, e).function() = p.function(Initial);
    if p.horizontal {
        let x = runtime.player_x();
        *MoverEdit::of(runtime, e).samus_x() = x;
    }
    MoverEdit::of(runtime, e).actor().extra_properties = 0;
    *MoverEdit::of(runtime, e).moving_samus() = 0;
    if p.horizontal {
        *MoverEdit::of(runtime, e).var_0b() = 0;
    }
    let list = match kind {
        Kind::Platform => 0xede7,
        Kind::VerticalShutter => 0xe9aa,
        _ => 0xe9d4,
    };
    MoverEdit::of(runtime, e).actor().instruction = list;
}

/// Main AI ($A2:EAB6, $A2:EED1, $A2:F1DE).
pub fn main_ai(kind: Kind, runtime: &mut impl Lease) {
    if kind == Kind::GrowingShutter {
        return grow(runtime);
    }
    let e = combat::current(runtime).byte_index();
    // Both original entry points dispatch both axes' functions.
    let function = Mover::of(runtime, e).function();
    let p = if VERTICAL.functions.contains(&function) {
        &VERTICAL
    } else {
        &HORIZONTAL
    };
    let Some(phase) = p.phase(function) else {
        return runtime.unreachable();
    };
    match phase {
        Initial | WaitForSamusToGetNear => {
            let trigger = if phase == WaitForSamusToGetNear {
                1
            } else {
                Mover::of(runtime, e).initial_function_index() >> 1
            };
            let ready = match trigger {
                0 => {
                    let timer = Mover::of(runtime, e).function_timer();
                    *MoverEdit::of(runtime, e).function_timer() = timer.wrapping_sub(1);
                    if timer == 1 {
                        let reload = Mover::of(runtime, e).proximity();
                        *MoverEdit::of(runtime, e).function_timer() = reload;
                    }
                    timer == 1
                }
                1 => {
                    let distance = Mover::of(runtime, e).proximity();
                    near(runtime, distance)
                }
                2 => true,
                3 | 4 => false,
                _ => panic!("mechanism trigger index out of bounds"),
            };
            if ready {
                let positive = Mover::of(runtime, e).primary_direction() != 0;
                let phase = if positive {
                    MovingPositive
                } else {
                    MovingNegative
                };
                *MoverEdit::of(runtime, e).function() = p.function(phase);
                if !p.horizontal {
                    movement_sound(runtime);
                }
            }
        }
        MovingNegative | MovingPositive => travel(runtime, p, phase == MovingPositive),
        StoppedNegative | StoppedPositive => {
            let timer = Mover::of(runtime, e).function_timer().wrapping_sub(1);
            *MoverEdit::of(runtime, e).function_timer() = timer;
            if (timer & 0x8000) != 0 {
                if !p.horizontal && phase == StoppedPositive {
                    movement_sound(runtime);
                }
                let next = if phase == StoppedNegative {
                    MovingPositive
                } else {
                    MovingNegative
                };
                *MoverEdit::of(runtime, e).function() = p.function(next);
                if Mover::of(runtime, e).trigger_mode() == 1
                    && (Mover::of(runtime, e).primary_direction() != 0)
                        == (phase == StoppedNegative)
                {
                    *MoverEdit::of(runtime, e).function() = p.function(WaitForSamusToGetNear);
                }
                if !p.horizontal
                    && phase == StoppedNegative
                    && Mover::of(runtime, e).actor().definition != 0xd83f
                {
                    movement_sound(runtime);
                }
            }
        }
        Nothing => {}
    }
    let p = kind.motion();
    // The enemy Samus collides with as a solid enemy, in all four directions.
    let collision = [3, 2, 1, 0]
        .map(|direction| {
            runtime.word(crate::ram::word_arrays::ENEMY_INDEX_COLLIDING_DIRS + 2 * direction)
        })
        .into_iter()
        .fold(0xffff, |all, index| all & index);
    let function = Mover::of(runtime, e).function();
    if function != p.function(MovingNegative)
        && function != p.function(MovingPositive)
        && collision != 0xffff
        && collision == runtime.session().actors.controls.selected
        && runtime.session().player.contact.contact_attack != 0
    {
        activate(runtime, p);
    }
    if p.horizontal {
        let (x, fraction) = (runtime.player_x(), runtime.player_x_fraction());
        let mut part = MoverEdit::of(runtime, e);
        *part.samus_x() = x;
        *part.samus_x_fraction() = fraction;
    }
}

/// Up/down mover function - moving up and down ($A2:EF68, $A2:EFD4), horizontal shutter
/// function - moving left and right ($A2:F272, $A2:F2E4): carry Samus, stop at the end.
fn travel(runtime: &mut impl Lease, p: &Motion, positive: bool) {
    let e = combat::current(runtime).byte_index();
    let mover = Mover::of(runtime, e);
    let (velocity, subvelocity) = if positive {
        (mover.positive_velocity(), mover.positive_subvelocity())
    } else {
        (mover.negative_velocity(), mover.negative_subvelocity())
    };
    let speed = (u32::from(velocity) << 16) | u32::from(subvelocity);
    let key = combat::current(runtime);
    let (position, limit) = if p.horizontal {
        let x = Mover::of(runtime, e).actor().x.whole;
        {
            let mut part = MoverEdit::of(runtime, e);
            *part.previous_x() = x;
            *part.pushing_samus() = 0;
        }
        let samus_side = (runtime.player_x() as i16).wrapping_sub(x as i16) >= 0;
        if api::overlaps_player_with_padding(runtime, key) && samus_side == positive {
            *MoverEdit::of(runtime, e).pushing_samus() = 1;
        }
        MoverEdit::of(runtime, e)
            .actor()
            .x
            .advance(speed.cast_signed());
        if Mover::of(runtime, e).pushing_samus() != 0 {
            let contact = &mut runtime.session_mut().player.contact;
            (contact.external_x.fraction, contact.external_x.whole) =
                (speed as u16, (speed >> 16) as u16);
            eject_samus(runtime, positive);
        }
        let limit = if positive {
            Mover::of(runtime, e).maximum_x()
        } else {
            Mover::of(runtime, e).minimum_x()
        };
        (Mover::of(runtime, e).actor().x.whole, limit)
    } else {
        let y = Mover::of(runtime, e).actor().y.whole;
        {
            let mut part = MoverEdit::of(runtime, e);
            *part.previous_y() = y;
            *part.moving_samus() = 0;
        }
        if api::touches_player_from_above(runtime, key) {
            *MoverEdit::of(runtime, e).moving_samus() = 1;
        }
        MoverEdit::of(runtime, e)
            .actor()
            .y
            .advance(speed.cast_signed());
        if Mover::of(runtime, e).moving_samus() != 0 {
            let mover = Mover::of(runtime, e);
            let moved = mover.actor().y.whole.wrapping_sub(mover.previous_y());
            runtime.session_mut().player.contact.external_y.whole = moved;
        }
        let limit = if positive {
            Mover::of(runtime, e).maximum_y()
        } else {
            Mover::of(runtime, e).minimum_y()
        };
        (Mover::of(runtime, e).actor().y.whole, limit)
    };
    let past = if positive {
        position.wrapping_sub(limit)
    } else {
        limit.wrapping_sub(position)
    };
    if (past as i16) >= 0 {
        let rest = if positive {
            Mover::of(runtime, e).positive_rest_time()
        } else {
            Mover::of(runtime, e).negative_rest_time()
        };
        if rest == 4080 {
            *MoverEdit::of(runtime, e).function() = p.function(Nothing);
        } else {
            let phase = if positive {
                StoppedPositive
            } else {
                StoppedNegative
            };
            let mut part = MoverEdit::of(runtime, e);
            *part.function_timer() = rest;
            *part.function() = p.function(phase);
        }
    }
}

/// Eject Samus if pressing right or left ($A2:F356, $A2:F371).
fn eject_samus(runtime: &mut impl Lease, right: bool) {
    let button = if right { 0x200 } else { 0x100 };
    if (*Input::Joypad1Lastkeys.get(&runtime.session().control) & button) != 0 {
        let contact = &mut runtime.session_mut().player.contact;
        contact.external_x.whole =
            contact
                .external_x
                .whole
                .wrapping_add(if right { 4 } else { -4i16 as u16 });
        contact.external_y.fraction = 0;
    }
}

/// Activate up/down mover / horizontal shutter ($A2:EF44, $A2:F25F) and the mover reactions.
fn activate(runtime: &mut impl Lease, p: &Motion) {
    let e = combat::current(runtime).byte_index();
    if (Mover::of(runtime, e)
        .initial_function_index()
        .wrapping_sub(6)
        & 0x8000)
        == 0
    {
        if Mover::of(runtime, e).initial_function_index() != 8 {
            if Mover::of(runtime, e).triggered() != 0 {
                return;
            }
            *MoverEdit::of(runtime, e).triggered() = 1;
        }
        let function = Mover::of(runtime, e).function();
        if function == p.function(MovingNegative) || function == p.function(MovingPositive) {
            return;
        }
        if p.horizontal {
            *MoverEdit::of(runtime, e).reaction_direction() ^= 1;
        }
        let positive = Mover::of(runtime, e).reaction_direction() != 0;
        let phase = if positive {
            MovingPositive
        } else {
            MovingNegative
        };
        *MoverEdit::of(runtime, e).function() = p.function(phase);
        if !p.horizontal {
            *MoverEdit::of(runtime, e).reaction_direction() ^= 1;
        }
    }
    if !p.horizontal {
        movement_sound(runtime);
    }
}

/// Enemy touch, shot and power bomb reactions ($A2:F09D-$A2:F0B6, $A2:F3D8-$A2:F41A).
pub fn reaction(kind: Kind, event: Event, runtime: &mut impl Lease) {
    if let Event::Shot = event {
        match kind {
            Kind::VerticalShutter => combat::shot_reaction(runtime, ShotHandling::Finalize),
            Kind::HorizontalShutter => combat::shot_reaction(runtime, ShotHandling::ReactionOnly),
            _ => {}
        }
    }
    if kind == Kind::HorizontalShutter && matches!(event, Event::Touch) {
        let e = combat::current(runtime).byte_index();
        if Mover::of(runtime, e).function() == HORIZONTAL.function(Nothing) {
            let right = (runtime.player_x() as i16)
                .wrapping_sub(Mover::of(runtime, e).actor().x.whole as i16)
                >= 0;
            let button = if right { 0x200 } else { 0x100 };
            if (*Input::Joypad1Lastkeys.get(&runtime.session().control) & button) != 0 {
                let contact = &mut runtime.session_mut().player.contact;
                (contact.external_x.whole, contact.external_y.fraction) =
                    (if right { 4 } else { -4i16 as u16 }, 0);
            }
        }
    } else {
        activate(runtime, kind.motion());
    }
}

/// Growing shutter functions ($A2:EABD-$A2:ED32): wait for the timer or for Samus to get near,
/// then grow or shrink a growth level at a time, carrying Samus standing on it.
fn grow(runtime: &mut impl Lease) {
    let e = combat::current(runtime).byte_index();
    let function = Growing::of(runtime, e).function();
    match function {
        0xeabd | 0xead1 | 0xeae7 | 0xeafd => {
            let ready = if function == 0xead1 || function == 0xeae7 {
                let distance = Growing::of(runtime, e).parameter_1();
                near(runtime, distance)
            } else if Growing::of(runtime, e).parameter_1() != 0 {
                let mut growing = GrowingEdit::of(runtime, e);
                let timer = growing.parameter_1();
                *timer = timer.wrapping_sub(1);
                false
            } else {
                true
            };
            if ready {
                movement_sound(runtime);
                let upwards = function == 0xeabd || function == 0xead1;
                *GrowingEdit::of(runtime, e).function() = if upwards { 0xec13 } else { 0xeb11 };
            }
        }
        0xeb11 | 0xec13 => {
            let upwards = function == 0xec13;
            if upwards {
                let y = Growing::of(runtime, e).actor().y.whole;
                *GrowingEdit::of(runtime, e).previous_y() = y;
            }
            let level = Growing::of(runtime, e).growth_level();
            let level = usize::from(if upwards { level & 0x7fff } else { level });
            assert!(level < 5, "timed shutter stage out of bounds");
            if level < 4 {
                let growing = Growing::of(runtime, e);
                let speed = (u32::from(growing.speed()) << 16) | u32::from(growing.subspeed());
                let amount: u32 = if upwards { speed.wrapping_neg() } else { speed };
                GrowingEdit::of(runtime, e)
                    .actor()
                    .y
                    .advance(amount.cast_signed());
                let stop = [
                    Growing::of(runtime, e).level_0_y(),
                    Growing::of(runtime, e).level_1_y(),
                    Growing::of(runtime, e).level_2_y(),
                    Growing::of(runtime, e).level_3_y(),
                ][level];
                let limit = stop.wrapping_add(if upwards { -16i16 as u16 } else { 16 });
                let past =
                    (limit as i16).wrapping_sub(Growing::of(runtime, e).actor().y.whole as i16);
                if (upwards && past >= 0) || (!upwards && past < 0) {
                    let y = if level == 3 {
                        limit
                    } else if upwards {
                        limit.wrapping_add(7)
                    } else {
                        stop.wrapping_add(9)
                    };
                    let mut growing = GrowingEdit::of(runtime, e);
                    growing.actor().y.whole = y;
                    let grown = growing.growth_level();
                    *grown = grown.wrapping_add(1);
                    if level != 3 {
                        let list = [0xe99e, 0xe9a4, 0xe9aa][level];
                        let actor = growing.actor();
                        actor.instruction_delay = 1;
                        actor.instruction = list;
                        actor.radius.y = [16, 24, 32][level];
                    }
                }
            }
            let key = combat::current(runtime);
            if upwards && api::touches_player_from_above(runtime, key) {
                let growing = Growing::of(runtime, e);
                let moved = growing.actor().y.whole.wrapping_sub(growing.previous_y());
                if (moved as i16) < 0 {
                    let carry = &mut runtime.session_mut().player.contact.external_y.whole;
                    *carry = carry.wrapping_add(moved);
                }
            }
        }
        _ => runtime.unreachable(),
    }
}
