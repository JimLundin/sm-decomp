//! Sidehopper state machine over the current owned actor and player interval.
#![forbid(unsafe_code)]
use super::{
    MAX_FALLING_Y_SPEED_INDEX, QUADRATIC_SPEEDS, RANDOM_SEED, SIGN, SUPER_HOPS, Sidehopper,
    function,
};
use crate::actors::access::AccessMode::Native;
use crate::actors::api::{self, Axis};
use crate::actors::api::{MovementResult::Blocked, SlopePolicy::Ignore};
use crate::actors::combat::{self, current_slot};
use crate::actors::session_slots::SessionSlots;
use crate::enemy_script::EnemyScript;
use crate::session_slot;
use sm_data::{
    cartridge::RomImage,
    enemy_speeds::{SpeedTable, SpeedTable::Quadratic},
};

#[derive(Clone, Copy, Debug)]
pub enum SidehopperTable {
    LandedUp,
    LandedDown,
    HoppingUp,
    HoppingDown,
    HopFunctions,
    SmallDelta,
    SmallSpeed,
    SmallHeight,
    BigDelta,
    BigSpeed,
    BigHeight,
}

pub trait Host: SessionSlots + crate::actors::host::ActorMotion {
    fn sidehopper_table(&self, table: SidehopperTable, index: isize) -> u16;
    fn sidehopper_speed(&self, table: SpeedTable, index: isize) -> u16;
    fn sidehopper_variant(&mut self, definition: u16) -> u16;
    fn capture_hop_speeds(&self) -> RomImage;
    fn sidehopper_random(&mut self) -> u16;
    fn sidehopper_sound(&mut self, sound: u16);
    fn unknown_sidehopper_function(&mut self);
}

fn upside_down(runtime: &impl Host, hopper: i32) -> usize {
    usize::from(session_slot!(runtime, Sidehopper[hopper].parameter_1) != 0)
}
fn negate_x_velocity(runtime: &mut impl Host, hopper: i32) {
    session_slot!(runtime, Sidehopper[hopper].x_velocity, |v| v.wrapping_neg());
}
/// Store the variant's list in `lists`; set enemy instruction list ($A3:ABBB) unless `stored_only`.
fn store_list(runtime: &mut impl Host, hopper: i32, lists: SidehopperTable, stored_only: bool) {
    let index = (session_slot!(runtime, Sidehopper[hopper].variant_index) >> 1) as isize;
    let list = runtime.sidehopper_table(lists, index);
    session_slot!(runtime, Sidehopper[hopper].new_instruction_list = list);
    if !stored_only {
        let key = combat::current(runtime);
        runtime.edit_actor(key, Native, |actor| actor.restart_animation(list));
    }
}
/// Calculate initial hop speed ($A3:AB9D), shared with beetoms: the first quadratic speed
/// index, stepping by `step`, whose summed distances (unaligned, a byte in) reach `height`.
pub fn initial_hop_phase(height: u16, step: u16, image: RomImage) -> u16 {
    let (mut phase, mut distance) = (0u16, 0u16);
    loop {
        phase = phase.wrapping_add(step);
        let at = usize::try_from(sm_data::cartridge::lorom_offset(QUADRATIC_SPEEDS))
            .expect("hop speeds offset")
            + 8 * usize::from(phase)
            + 1;
        distance = distance.wrapping_add(u16::from_ne_bytes([image.read(at), image.read(at + 1)]));
        if distance.wrapping_sub(height) & SIGN == 0 {
            return phase;
        }
    }
}
/// Initialisation AI - sidehopper / desgeega / super variants ($A3:AB09).
pub fn initialize(runtime: &mut impl Host) {
    runtime.session_mut().timing.random_number = RANDOM_SEED;
    runtime.refresh_inputs();
    runtime.sidehopper_random();
    let hopper = current_slot(runtime);
    session_slot!(runtime, Sidehopper[hopper]; falling = 0, ready_to_hop = 0, hop_table_index = 0);
    let definition = session_slot!(runtime, Sidehopper[hopper].base.enemy_ptr);
    let variant = runtime.sidehopper_variant(definition);
    let index = variant.wrapping_mul(2);
    session_slot!(runtime, Sidehopper[hopper].variant_index = index);
    let lists =
        [SidehopperTable::LandedUp, SidehopperTable::LandedDown][upside_down(runtime, hopper)];
    store_list(runtime, hopper, lists, false);
    if variant != 0 {
        session_slot!(runtime, Sidehopper[hopper].hop_table_index = SUPER_HOPS);
    }
    for (height_table, delta_table, big) in [
        (
            SidehopperTable::SmallHeight,
            SidehopperTable::SmallDelta,
            false,
        ),
        (SidehopperTable::BigHeight, SidehopperTable::BigDelta, true),
    ] {
        let index = (session_slot!(runtime, Sidehopper[hopper].hop_table_index) >> 1) as isize;
        let height = runtime.sidehopper_table(height_table, index);
        let step = runtime.sidehopper_table(delta_table, index);
        let phase = initial_hop_phase(height, step, runtime.capture_hop_speeds());
        match big {
            false => session_slot!(runtime, Sidehopper[hopper].small_hop_y_speed_index = phase),
            true => session_slot!(runtime, Sidehopper[hopper].big_hop_y_speed_index = phase),
        }
    }
    session_slot!(runtime, Sidehopper[hopper].function = function::HOP);
}
/// Move down (or up) by the Y speed index's quadratic speed, then right by the X velocity,
/// turning at a wall; whether each was blocked, skipping the second after a blocked first.
fn move_by_velocity(runtime: &mut impl Host, hopper: i32, up: bool) -> (bool, bool) {
    let index = 4 * session_slot!(runtime, Sidehopper[hopper].y_speed_index) as isize;
    let index = index + if up { 2 } else { 0 };
    let high = runtime.sidehopper_speed(Quadratic, index + 1);
    let speed = (u32::from(high) << 16) | u32::from(runtime.sidehopper_speed(Quadratic, index));
    let key = combat::current(runtime);
    if api::move_vertical(runtime, key, speed as i32) == Blocked {
        return (true, false);
    }
    let velocity = i32::from(session_slot!(runtime, Sidehopper[hopper].x_velocity) as i16);
    let blocked = api::move_horizontal(runtime, key, velocity << 16, Ignore) == Blocked;
    if blocked {
        negate_x_velocity(runtime, hopper);
    }
    (false, blocked)
}
/// Sidehopper movement - jumping - upside up ($A3:AD6D) or down ($A3:AE27).
fn jump(runtime: &mut impl Host, hopper: i32, upside_down: usize) {
    let falls = match move_by_velocity(runtime, hopper, upside_down == 0) {
        (true, _) => {
            negate_x_velocity(runtime, hopper);
            true
        }
        (false, true) => true,
        (false, false) => {
            let delta = session_slot!(runtime, Sidehopper[hopper].y_speed_index_delta);
            let index =
                session_slot!(runtime, Sidehopper[hopper].y_speed_index).wrapping_sub(delta);
            let top = index & SIGN != 0;
            let index = if top { 0 } else { index };
            session_slot!(runtime, Sidehopper[hopper].y_speed_index = index);
            top
        }
    };
    if falls {
        session_slot!(runtime, Sidehopper[hopper].falling = 1);
    }
}
/// Sidehopper movement - falling - upside up ($A3:ADD4) or down ($A3:AE8E).
fn fall(runtime: &mut impl Host, hopper: i32, upside_down: usize) {
    if move_by_velocity(runtime, hopper, upside_down != 0).0 {
        session_slot!(runtime, Sidehopper[hopper]; falling = 0, function = function::LANDED);
        return;
    }
    let delta = session_slot!(runtime, Sidehopper[hopper].y_speed_index_delta);
    session_slot!(runtime, Sidehopper[hopper].y_speed_index, |index| {
        let next = index.wrapping_add(delta);
        match next.wrapping_sub(MAX_FALLING_Y_SPEED_INDEX) & SIGN {
            0 => MAX_FALLING_Y_SPEED_INDEX,
            _ => next,
        }
    });
}
/// Main AI - sidehopper / desgeega / super variants ($A3:ABCF): run its function.
pub fn main_ai(runtime: &mut impl Host) {
    use function::*;
    let hopper = current_slot(runtime);
    let phase = session_slot!(runtime, Sidehopper[hopper].function);
    let next = match phase {
        HOP => {
            let index = (runtime.sidehopper_random() & 1) as isize;
            runtime.sidehopper_table(SidehopperTable::HopFunctions, index)
        }
        SMALL_HOP | BIG_HOP => {
            let (delta_table, speed_table) = [
                (SidehopperTable::SmallDelta, SidehopperTable::SmallSpeed),
                (SidehopperTable::BigDelta, SidehopperTable::BigSpeed),
            ][usize::from(phase == BIG_HOP)];
            let index = (session_slot!(runtime, Sidehopper[hopper].hop_table_index) >> 1) as isize;
            let delta = runtime.sidehopper_table(delta_table, index);
            let speed = runtime.sidehopper_table(speed_table, index);
            let initial = match phase {
                BIG_HOP => session_slot!(runtime, Sidehopper[hopper].big_hop_y_speed_index),
                _ => session_slot!(runtime, Sidehopper[hopper].small_hop_y_speed_index),
            };
            session_slot!(runtime, Sidehopper[hopper]; y_speed_index_delta = delta, x_velocity = speed,
                y_speed_index = initial);
            [HOP_UPSIDE_UP, HOP_UPSIDE_DOWN][upside_down(runtime, hopper)]
        }
        // Hop backwards, or forwards when Samus is to the right.
        HOP_UPSIDE_UP | HOP_UPSIDE_DOWN => {
            let key = combat::current(runtime);
            let forwards = usize::from(api::player_delta(runtime, key, Axis::X) & SIGN == 0);
            let hops = [
                [HOP_BACKWARDS_UPSIDE_UP, HOP_FORWARDS_UPSIDE_UP],
                [HOP_BACKWARDS_UPSIDE_DOWN, HOP_FORWARDS_UPSIDE_DOWN],
            ];
            hops[usize::from(phase == HOP_UPSIDE_DOWN)][forwards]
        }
        HOP_BACKWARDS_UPSIDE_UP
        | HOP_FORWARDS_UPSIDE_UP
        | HOP_BACKWARDS_UPSIDE_DOWN
        | HOP_FORWARDS_UPSIDE_DOWN => {
            let down = usize::from(phase >= HOP_BACKWARDS_UPSIDE_DOWN);
            let forwards = matches!(phase, HOP_FORWARDS_UPSIDE_UP | HOP_FORWARDS_UPSIDE_DOWN);
            if !forwards {
                negate_x_velocity(runtime, hopper);
            }
            store_list(
                runtime,
                hopper,
                [SidehopperTable::HoppingUp, SidehopperTable::HoppingDown][down],
                false,
            );
            let jumping = [
                [JUMPING_UPSIDE_UP, JUMPING_UPSIDE_UP_CLONE],
                [JUMPING_UPSIDE_DOWN, JUMPING_UPSIDE_DOWN_CLONE],
            ];
            jumping[down][usize::from(forwards)]
        }
        LANDED => {
            let down = upside_down(runtime, hopper) != 0;
            store_list(runtime, hopper, SidehopperTable::LandedUp, down);
            if down {
                store_list(runtime, hopper, SidehopperTable::LandedDown, false);
            }
            WAIT_TO_HOP
        }
        // Sidehopper function - jumping ($A3:AD0E, $A3:AD20, $A3:AD32, $A3:AD44).
        JUMPING_UPSIDE_UP
        | JUMPING_UPSIDE_UP_CLONE
        | JUMPING_UPSIDE_DOWN
        | JUMPING_UPSIDE_DOWN_CLONE => {
            let down = usize::from(phase >= JUMPING_UPSIDE_DOWN);
            match session_slot!(runtime, Sidehopper[hopper].falling) {
                0 => jump(runtime, hopper, down),
                _ => fall(runtime, hopper, down),
            }
            return;
        }
        WAIT_TO_HOP => match session_slot!(runtime, Sidehopper[hopper].ready_to_hop) {
            0 => return,
            _ => {
                session_slot!(runtime, Sidehopper[hopper].ready_to_hop = 0);
                HOP
            }
        },
        _ => return runtime.unknown_sidehopper_function(),
    };
    session_slot!(runtime, Sidehopper[hopper].function = next);
}
/// Instruction - queue sound [[Y]], sound library 2, max queued sounds allowed = 3 ($A3:AA68).
pub fn queue_sound(script: EnemyScript, runtime: &mut impl Host) -> EnemyScript {
    runtime.sidehopper_sound(script.word(0));
    script.offset(1)
}
/// Instruction - ready to hop ($A3:AAFE).
pub fn ready_to_hop(script: EnemyScript, runtime: &mut impl Host) -> EnemyScript {
    session_slot!(runtime, Sidehopper[current_slot(runtime)].ready_to_hop = 1);
    script
}
