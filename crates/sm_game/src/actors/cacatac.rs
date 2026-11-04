//! Cacatac, the spiked cactus that paces and fires spikes, and Owtch, the spiked shell that
//! shares its patrol and burrows (bank $A2:9E6A-$A2:A59D).
use crate::actors::access::{AccessMode::Native, ActorKey};
use crate::actors::api::ShotHandling;
use crate::actors::combat;
use crate::actors::host::ActorLease;
use crate::actors::slot_view::{SlotMut, SlotRef, SlotState, slot_view};
use crate::enemy_script::EnemyScript;
use crate::player_projectiles::content::table;
use sm_data::enemy_speeds::SpeedTable::Linear;

/// A lease the cacatacs and owtches run on: their slots are the Session's.
pub trait Lease: ActorLease + SlotState {}
impl<T: ActorLease + SlotState + ?Sized> Lease for T {}

slot_view! {
    /// A cacatac's slot: its right and left velocities (16.16), direction (1 moving
    /// right) and function; its patrol's left and right bounds. Parameter 1's low byte
    /// is its initial direction and its high byte nonzero for an upside down cactus;
    /// parameter 2's bytes are its patrol range and speed.
    Cacatac, CacatacEdit {
        record: [
            right_subvelocity = var_a, right_velocity = var_b, left_subvelocity = var_c,
            left_velocity = var_d, direction = var_e, function = var_f,
            parameter_1 = parameter_1, parameter_2 = parameter_2,
        ],
        extended: [],
    }
}

/// The extended words of the patrol's left and right bounds: the cacatac's
/// `var_00` and `var_01`, the owtch's `var_01` and `var_02` (after its
/// underground timer).
fn bounds(owtch: bool) -> [usize; 2] {
    let left = usize::from(owtch);
    [left, left + 1]
}

const MOVING_LEFT: u16 = 0x9fba;
const MOVING_RIGHT: u16 = 0x9fec;
const ATTACKING: u16 = 0xa01b; // An RTS: wait for the attack's instructions.
const LISTS: [[u16; 2]; 2] = [[0x9eda, 0x9f00], [0x9e8a, 0x9eb0]]; // Idle, attack; down, up.
fn current(runtime: &impl Lease) -> u16 {
    combat::current(runtime).byte_index()
}
fn set_instruction_list(runtime: &mut impl Lease, attacking: bool) {
    let up = usize::from(Cacatac::of(runtime, current(runtime)).parameter_1() >> 8 != 0);
    let key = combat::current(runtime);
    let list = LISTS[up][usize::from(attacking)];
    runtime.edit_actor(key, Native, |actor| actor.restart_animation(list));
}
pub(crate) fn set_bounds(runtime: &mut impl Lease, owtch: bool, distances: u32, range: u16) {
    let k = current(runtime);
    let distance = table(runtime, distances, range as isize);
    let key = combat::current(runtime);
    let x = runtime.resolve_actor(key, Native).x.whole;
    let [left, right] = bounds(owtch);
    let mut slot = SlotMut::of(runtime, k);
    *slot.extended(right) = distance.wrapping_add(x);
    *slot.extended(left) = x.wrapping_sub(distance);
}
pub(crate) fn set_velocities(runtime: &mut impl Lease, speed: u16) {
    let speeds = [0, 1, 2, 3].map(|i| runtime.lease_enemy_speed(Linear, 4 * speed as isize + i));
    let k = current(runtime);
    let mut cacatac = CacatacEdit::of(runtime, k);
    *cacatac.right_velocity() = speeds[0];
    *cacatac.right_subvelocity() = speeds[1];
    *cacatac.left_velocity() = speeds[2];
    *cacatac.left_subvelocity() = speeds[3];
}
/// Initialisation AI - cacatac ($A2:9F48).
pub fn initialize(runtime: &mut impl Lease) {
    let k = current(runtime);
    CacatacEdit::of(runtime, k).actor().spritemap = 0x804d;
    set_instruction_list(runtime, false);
    let [direction, _] = Cacatac::of(runtime, k).parameter_1().to_le_bytes();
    let function = table(runtime, 0xa2_9f42, isize::from(direction));
    let mut cacatac = CacatacEdit::of(runtime, k);
    *cacatac.direction() = u16::from(direction);
    *cacatac.function() = function;
    let [range, speed] = cacatac.parameter_2().to_le_bytes();
    drop(cacatac);
    set_bounds(runtime, false, 0xa2_9f36, u16::from(range));
    set_velocities(runtime, u16::from(speed));
}
/// Move by the left or right velocity; at `bound`, turn, and a random draw below `odds`.
pub(crate) fn patrol(runtime: &mut impl Lease, owtch: bool, right: bool) {
    let k = current(runtime);
    let x = {
        let mut cacatac = CacatacEdit::of(runtime, k);
        let speed = if right {
            (u32::from(*cacatac.right_velocity()) << 16) | u32::from(*cacatac.right_subvelocity())
        } else {
            (u32::from(*cacatac.left_velocity()) << 16) | u32::from(*cacatac.left_subvelocity())
        };
        cacatac.actor().x.advance(speed.cast_signed());
        cacatac.actor().x.whole
    };
    let bound = SlotRef::of(runtime, k).extended(bounds(owtch)[usize::from(right)]);
    if (x.wrapping_sub(bound) as i16 >= 0) == right {
        let mut cacatac = CacatacEdit::of(runtime, k);
        if owtch {
            let left = cacatac.direction().wrapping_sub(1);
            *cacatac.direction() = if right { 0 } else { left };
        } else {
            *cacatac.function() = if right { MOVING_LEFT } else { MOVING_RIGHT };
            *cacatac.direction() = u16::from(!right);
        }
    }
    // Maybe make cacatac attack ($A2:A01C) or owtch sink ($A2:A553).
    runtime.next_random();
    let frame = Cacatac::of(runtime, k).actor().frame as u8;
    let random = runtime.word(crate::ram::words::RANDOM_NUMBER) as u8;
    if frame.wrapping_add(random) < if owtch { 6 } else { 3 } {
        if owtch {
            *CacatacEdit::of(runtime, k).direction() = 3;
        } else {
            *CacatacEdit::of(runtime, k).function() = ATTACKING;
            set_instruction_list(runtime, true);
        }
    }
}
/// Cacatac function - moving left ($A2:9FBA) or right ($A2:9FEC).
pub fn moving(right: bool, runtime: &mut impl crate::actors::species::TableLease) {
    patrol(runtime, false, right);
}
/// Instruction - function = moving left/right ($A2:A095), by the direction of enemy `k`.
pub fn resume_moving(
    k: u16,
    j: EnemyScript,
    runtime: &mut impl crate::actors::species::TableLease,
) -> EnemyScript {
    runtime.edit_actor(ActorKey::from_legacy(k), Native, |actor| {
        actor.pre_instruction = [MOVING_LEFT, MOVING_RIGHT][usize::from(actor.variables[4] != 0)]
    });
    j
}
/// Owtch ($A2:A38B-$A2:A59D).
pub mod owtch {
    use super::*;
    slot_view! {
        /// An owtch's slot: its velocities, as the cacatac's; its function index and
        /// how far it has sunk; its underground timer. Parameter 1's low byte is its
        /// initial function index and its high byte its underground time; parameter
        /// 2's bytes are its speed and patrol range.
        Owtch, OwtchEdit {
            record: [
                function_index = var_e, sink_offset = var_f, parameter_1 = parameter_1,
                parameter_2 = parameter_2,
            ],
            extended: [underground_timer = var_00],
        }
    }
    /// Function indices; -2 and -1 set the moving left and right lists.
    const UNDERGROUND: u16 = 2;
    const SINKING: u16 = 3;
    const RISING: u16 = 4;
    const SINK_DEPTH: u16 = 16;
    /// Set owtch instruction list pointer - moving left ($A2:A48A) or right ($A2:A49D).
    fn set_instruction_list(runtime: &mut impl Lease, right: bool) {
        let key = combat::current(runtime);
        let list = if right { 0xa3bd } else { 0xa3ab };
        runtime.edit_actor(key, Native, |actor| actor.restart_animation(list));
    }
    fn reload_underground_timer(runtime: &mut impl Lease, owtch: u16) {
        let index = Owtch::of(runtime, owtch).parameter_1() >> 8;
        let timer = table(runtime, 0xa2_a3ed, index as isize);
        *OwtchEdit::of(runtime, owtch).underground_timer() = timer;
    }
    /// Initialisation AI - owtch ($A2:A3F9).
    pub fn initialize(runtime: &mut impl crate::actors::species::TableLease) {
        let owtch = current(runtime);
        let index = {
            let mut edit = OwtchEdit::of(runtime, owtch);
            edit.actor().spritemap = 0x804d;
            let index = *edit.parameter_1() & 0xff;
            *edit.function_index() = index;
            index
        };
        set_instruction_list(runtime, index & 1 != 0);
        let [speed, range] = Owtch::of(runtime, owtch).parameter_2().to_le_bytes();
        set_velocities(runtime, u16::from(speed));
        set_bounds(runtime, true, 0xa2_a3dd, u16::from(range));
        reload_underground_timer(runtime, owtch);
        let mut edit = OwtchEdit::of(runtime, owtch);
        let underground = *edit.function_index() == UNDERGROUND;
        *edit.sink_offset() = SINK_DEPTH * u16::from(underground);
        if underground {
            let y = &mut edit.actor().y.whole;
            *y = y.wrapping_add(SINK_DEPTH);
        }
    }
    /// Main AI - owtch ($A2:A47E): run its function index.
    pub fn main_ai(runtime: &mut impl crate::actors::species::TableLease) {
        let owtch = current(runtime);
        let index = Owtch::of(runtime, owtch).function_index() as i16;
        match index {
            -2 | -1 => set_instruction_list(runtime, index == -1),
            0 | 1 => patrol(runtime, true, index == 1),
            2 => {
                let mut edit = OwtchEdit::of(runtime, owtch);
                let timer = edit.underground_timer().wrapping_sub(1);
                *edit.underground_timer() = timer;
                if timer == 0 {
                    *edit.function_index() = RISING;
                }
            }
            3 | 4 => {
                // Owtch function index 3 - sinking ($A2:A517) or 4 - rising ($A2:A53E).
                let sinking = index as u16 == SINKING;
                let step = if sinking { 1 } else { u16::MAX };
                let previous = {
                    let mut edit = OwtchEdit::of(runtime, owtch);
                    let y = &mut edit.actor().y.whole;
                    *y = y.wrapping_add(step);
                    let previous = *edit.sink_offset();
                    *edit.sink_offset() = previous.wrapping_add(step);
                    previous
                };
                if sinking && previous.wrapping_add(1).wrapping_sub(SINK_DEPTH) & 0x8000 == 0 {
                    *OwtchEdit::of(runtime, owtch).function_index() = UNDERGROUND;
                    reload_underground_timer(runtime, owtch);
                } else if !sinking && previous == 1 {
                    let direction =
                        u16::from(runtime.lease_byte(crate::ram::words::RANDOM_NUMBER + 1)) & 1;
                    *OwtchEdit::of(runtime, owtch).function_index() = direction;
                }
            }
            // The source indexes its table signed; past either end fails.
            _ => panic!("owtch function index {index}"),
        }
    }
    /// Instruction - enemy function index = 0 (moving left, $A2:A56D) or 1 ($A2:A571).
    pub fn set_function_index(
        ea: u32,
        k: u16,
        j: EnemyScript,
        runtime: &mut impl crate::actors::species::TableLease,
    ) -> EnemyScript {
        let index = u16::from(ea == 0xa2_a571);
        let key = ActorKey::from_legacy(k);
        runtime.edit_actor(key, Native, |actor| actor.variables[4] = index);
        j
    }
    /// Enemy shot - owtch ($A2:A579): hurt only while moving.
    pub fn shot(runtime: &mut impl crate::actors::species::TableLease) {
        let owtch = current(runtime);
        if Owtch::of(runtime, owtch).function_index().wrapping_sub(1) & 0x8000 != 0 {
            combat::shot_reaction(runtime, ShotHandling::Finalize);
        }
    }
}
