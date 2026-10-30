//! Sidehoppers, desgeegas and their super variants hop small or big about
//! Samus, on the floor or upside down on the ceiling (bank $A3:AA48-$A3:B3A0).
#![forbid(unsafe_code)]
use crate::actors::access::AccessMode::Native;
use crate::actors::api::{self, Axis, MovementResult::Blocked, SlopePolicy::Ignore};
use crate::actors::combat::{self, current_slot};
use crate::actors::host::{ActorLease, LeaseActors};
use crate::actors::layouts::part_view;
use crate::enemy_projectiles::host::Effects as _;
use crate::enemy_script::EnemyScript;
use crate::player_projectiles::content::{self, table};
use crate::player_projectiles::host::Host as _;
use crate::session_slot;
use sm_data::{cartridge::RomImage, enemy_speeds::SpeedTable::Quadratic};

part_view! {
    /// A sidehopper's slot and extended RAM; a non-zero first parameter is upside down.
    Sidehopper: EnemyLayout40 {
        words: [var_A, function, y_speed_index, x_velocity, y_speed_index_delta, var_F, parameter_1, parameter_2],
        pad1: [u8; 26696],
        words: [
            new_instruction_list, small_hop_y_speed_index, big_hop_y_speed_index, falling, ready_to_hop,
            hop_table_index, variant_index, var_07, var_08, var_09, var_0A, var_0B, var_0C, var_0D, var_0E,
            var_0F, var_10, var_11, var_12, var_13, var_14, var_15, var_16, var_17, var_18, var_19, var_1A,
            var_1B, var_1C, var_1D, var_1E, var_1F,
        ],
    }
}

/// Bank $A3 sidehopper functions; hopping backwards sets each jumping original, forwards its clone.
pub mod function {
    pub const HOP: u16 = 0xabd6;
    pub const SMALL_HOP: u16 = 0xabe6;
    pub const BIG_HOP: u16 = 0xac13;
    pub const HOP_UPSIDE_UP: u16 = 0xac40;
    pub const HOP_UPSIDE_DOWN: u16 = 0xac56;
    pub const HOP_BACKWARDS_UPSIDE_UP: u16 = 0xac6c;
    pub const HOP_FORWARDS_UPSIDE_UP: u16 = 0xac8f;
    pub const HOP_BACKWARDS_UPSIDE_DOWN: u16 = 0xaca8;
    pub const HOP_FORWARDS_UPSIDE_DOWN: u16 = 0xaccb;
    pub const LANDED: u16 = 0xace4;
    pub const JUMPING_UPSIDE_UP: u16 = 0xad0e;
    pub const JUMPING_UPSIDE_UP_CLONE: u16 = 0xad20;
    pub const JUMPING_UPSIDE_DOWN: u16 = 0xad32;
    pub const JUMPING_UPSIDE_DOWN_CLONE: u16 = 0xad44;
    pub const WAIT_TO_HOP: u16 = 0xad56;
}

/// $A3:AAC2: landed then hopping instruction lists, upside up then down, by variant.
const LANDED_LISTS: [u32; 2] = [0xa3_aac2, 0xa3_aaca];
const HOPPING_LISTS: [u32; 2] = [0xa3_aad2, 0xa3_aada];
/// $A3:AAE2: the small and big hop functions, drawn at random.
const HOP_FUNCTIONS: u32 = 0xa3_aae2;
/// $A3:AAE6: small and big hop Y speed index deltas, X speeds 4 bytes on, heights 8 on.
const HOP_TABLES: [u32; 2] = [0xa3_aae6, 0xa3_aaf2];
const X_SPEED: u32 = 4;
const HEIGHT: u32 = 8;
/// $A2:838F: the quadratic speed table, 8 bytes an entry.
const QUADRATIC_SPEEDS: u32 = 0xa2_838f;
/// Initialisation seeds the random number with 25h; super variants use the
/// second hop table entry; falling speeds up to Y speed table index 40h.
const RANDOM_SEED: u16 = 37;
const SUPER_HOPS: u16 = 2;
const MAX_FALLING_Y_SPEED_INDEX: u16 = 64;
const SIGN: u16 = 0x8000;

fn upside_down(runtime: &impl ActorLease, hopper: i32) -> usize {
    usize::from(session_slot!(runtime, Sidehopper[hopper].parameter_1) != 0)
}
fn negate_x_velocity(runtime: &mut impl ActorLease, hopper: i32) {
    session_slot!(runtime, Sidehopper[hopper].x_velocity, |v| v.wrapping_neg());
}
/// Store the variant's list in `lists`; set enemy instruction list ($A3:ABBB) unless `stored_only`.
fn store_list(runtime: &mut impl ActorLease, hopper: i32, lists: u32, stored_only: bool) {
    let index = (session_slot!(runtime, Sidehopper[hopper].variant_index) >> 1) as isize;
    let list = table(runtime, lists, index);
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
        let at = content::offset(QUADRATIC_SPEEDS) + 8 * usize::from(phase) + 1;
        distance = distance.wrapping_add(u16::from_ne_bytes([image.read(at), image.read(at + 1)]));
        if distance.wrapping_sub(height) & SIGN == 0 {
            return phase;
        }
    }
}
/// Initialisation AI - sidehopper / desgeega / super variants ($A3:AB09).
pub fn initialize<H: LeaseActors + ?Sized>(game: &mut H) {
    let mut lease = game.lease();
    let runtime = &mut lease;
    runtime.write_slice_le(crate::ram::words::random_number, RANDOM_SEED);
    runtime.next_random();
    let hopper = current_slot(runtime);
    session_slot!(runtime, Sidehopper[hopper]; falling = 0, ready_to_hop = 0, hop_table_index = 0);
    let definition = session_slot!(runtime, Sidehopper[hopper].base.enemy_ptr);
    let variant = runtime.record(0xa0, definition).word(0x2a);
    let index = variant.wrapping_mul(2);
    session_slot!(runtime, Sidehopper[hopper].variant_index = index);
    let lists = LANDED_LISTS[upside_down(runtime, hopper)];
    store_list(runtime, hopper, lists, false);
    if variant != 0 {
        session_slot!(runtime, Sidehopper[hopper].hop_table_index = SUPER_HOPS);
    }
    for (hop, big) in HOP_TABLES.into_iter().zip([false, true]) {
        let index = (session_slot!(runtime, Sidehopper[hopper].hop_table_index) >> 1) as isize;
        let height = table(runtime, hop + HEIGHT, index);
        let step = table(runtime, hop, index);
        let phase = initial_hop_phase(height, step, runtime.native_image());
        match big {
            false => session_slot!(runtime, Sidehopper[hopper].small_hop_y_speed_index = phase),
            true => session_slot!(runtime, Sidehopper[hopper].big_hop_y_speed_index = phase),
        }
    }
    session_slot!(runtime, Sidehopper[hopper].function = function::HOP);
}
/// Move down (or up) by the Y speed index's quadratic speed, then right by the X velocity,
/// turning at a wall; whether each was blocked, skipping the second after a blocked first.
fn move_by_velocity(runtime: &mut impl ActorLease, hopper: i32, up: bool) -> (bool, bool) {
    let index = 4 * session_slot!(runtime, Sidehopper[hopper].y_speed_index) as isize;
    let index = index + if up { 2 } else { 0 };
    let high = runtime.lease_enemy_speed(Quadratic, index + 1);
    let speed = (u32::from(high) << 16) | u32::from(runtime.lease_enemy_speed(Quadratic, index));
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
fn jump(runtime: &mut impl ActorLease, hopper: i32, upside_down: usize) {
    runtime.probe(["hop.Sidehopper_Func_19", "hop.Sidehopper_Func_21"][upside_down]);
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
fn fall(runtime: &mut impl ActorLease, hopper: i32, upside_down: usize) {
    runtime.probe(["hop.Sidehopper_Func_20", "hop.Sidehopper_Func_22"][upside_down]);
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
pub fn main_ai<H: LeaseActors + ?Sized>(game: &mut H) {
    use function::*;
    let mut lease = game.lease();
    let runtime = &mut lease;
    let hopper = current_slot(runtime);
    let phase = session_slot!(runtime, Sidehopper[hopper].function);
    let next = match phase {
        HOP => {
            let index = (runtime.next_random() & 1) as isize;
            table(runtime, HOP_FUNCTIONS, index)
        }
        SMALL_HOP | BIG_HOP => {
            let hop = HOP_TABLES[usize::from(phase == BIG_HOP)];
            let index = (session_slot!(runtime, Sidehopper[hopper].hop_table_index) >> 1) as isize;
            let delta = table(runtime, hop, index);
            let speed = table(runtime, hop + X_SPEED, index);
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
            store_list(runtime, hopper, HOPPING_LISTS[down], false);
            let jumping = [
                [JUMPING_UPSIDE_UP, JUMPING_UPSIDE_UP_CLONE],
                [JUMPING_UPSIDE_DOWN, JUMPING_UPSIDE_DOWN_CLONE],
            ];
            jumping[down][usize::from(forwards)]
        }
        LANDED => {
            let down = upside_down(runtime, hopper) != 0;
            store_list(runtime, hopper, LANDED_LISTS[0], down);
            if down {
                store_list(runtime, hopper, LANDED_LISTS[1], false);
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
        _ => return runtime.unreachable(),
    };
    session_slot!(runtime, Sidehopper[hopper].function = next);
}
/// Instruction - queue sound [[Y]], sound library 2, max queued sounds allowed = 3 ($A3:AA68).
pub fn queue_sound<H: LeaseActors + ?Sized>(script: EnemyScript, game: &mut H) -> EnemyScript {
    game.lease().queue_sound(2, 3, script.word(0));
    script.offset(1)
}
/// Instruction - ready to hop ($A3:AAFE).
pub fn ready_to_hop<H: LeaseActors + ?Sized>(script: EnemyScript, game: &mut H) -> EnemyScript {
    let mut lease = game.lease();
    let runtime = &mut lease;
    session_slot!(runtime, Sidehopper[current_slot(runtime)].ready_to_hop = 1);
    script
}
