//! Choot, which waits for Samus, jumps and drifts down in a falling pattern
//! (bank $A2:D80C-$A2:E17A). Was `rising_falling`.
use crate::actors::access::AccessMode::Native;
use crate::actors::api::{self, Axis};
use crate::actors::combat;
use crate::actors::host::ActorLease;
use crate::actors::slot_view::{SlotState, slot_view};
use crate::player_projectiles::content::table;
use sm_data::enemy_speeds::SpeedTable::Quadratic;

slot_view! {
    /// A choot's slot: its function, Y speed index, falling pattern (row index,
    /// loops left, pointer and Y distance); its spawn position, where it starts
    /// falling, the falling pattern's origin, its initial Y speed index and its
    /// jump delay. Its parameters are the pattern and loops, and the delay.
    Choot, ChootEdit {
        record: [
            function = var_a, y_speed_index = var_b, pattern_index = var_c,
            pattern_loops = var_d, pattern = var_e, pattern_y_distance = var_f,
            parameter_1 = parameter_1, parameter_2 = parameter_2,
        ],
        extended: [
            spawn_x = var_00, spawn_y = var_01, falling_x = var_02, falling_y = var_03,
            origin_x = var_04, origin_y = var_05, initial_y_speed_index = var_06,
            jump_delay = var_07,
        ],
    }
}

/// A lease the choots run on: its slots are the Session's.
pub trait Lease: ActorLease + SlotState {}
impl<T: ActorLease + SlotState + ?Sized> Lease for T {}

const WAIT_FOR_SAMUS: u16 = 0xe035;
const PREPARE_TO_JUMP: u16 = 0xe04f;
const JUMPING: u16 = 0xe06a;
const FALLING: u16 = 0xe0cd;
const IDLE_LIST: u16 = 0xd82c;
const JUMPING_LIST: u16 = 0xd834;
const FALLING_LIST: u16 = 0xd840;
/// $A2:DF5E, $A2:DF6A: the falling pattern pointers and Y distances, by pattern.
const PATTERNS: u32 = 0xa2_df5e;
const PATTERN_Y_DISTANCES: u32 = 0xa2_df6a;
const PATTERN_END: u16 = 0x8000;
const SPEED_INDEX_STEP: u16 = 0x200;

/// Set choot instruction list ($A2:E01E).
fn set_instruction_list(runtime: &mut impl Lease, list: u16) {
    let key = combat::current(runtime);
    runtime.edit_actor(key, Native, |actor| actor.restart_animation(list));
}
/// The quadratic speed at word `index`, a whole word above its subword.
fn speed(runtime: &impl Lease, index: usize) -> u32 {
    let high = runtime.lease_enemy_speed(Quadratic, (index + 1) as isize);
    (u32::from(high) << 16) | u32::from(runtime.lease_enemy_speed(Quadratic, index as isize))
}
/// Initialisation AI - choot ($A2:DF76).
pub fn initialize(runtime: &mut impl Lease) {
    set_instruction_list(runtime, IDLE_LIST);
    let choot = combat::current(runtime).byte_index();
    let parameter = {
        let mut edit = ChootEdit::of(runtime, choot);
        let (x, y) = (edit.actor().x.whole, edit.actor().y.whole);
        *edit.function() = WAIT_FOR_SAMUS;
        *edit.spawn_x() = x;
        *edit.spawn_y() = y;
        *edit.parameter_1()
    };
    let index = (parameter >> 8) as isize;
    let pattern = table(runtime, PATTERNS, index);
    let distance_at = table(runtime, PATTERN_Y_DISTANCES, index);
    let distance = runtime.lease_native_record(0xa2, distance_at).word(0);
    {
        let mut edit = ChootEdit::of(runtime, choot);
        *edit.pattern() = pattern;
        *edit.pattern_y_distance() = distance;
    }
    // Calculate choot jump height ($A2:DFCE) and initial choot jump speed ($A2:DFE9).
    let view = Choot::of(runtime, choot);
    let loops = view.parameter_1() & 0xff;
    let height = loops * (view.pattern_y_distance() & 0xff);
    let (mut covered, mut index) = (0u32, 0u16);
    loop {
        index = index.wrapping_add(SPEED_INDEX_STEP);
        covered = covered.wrapping_add(speed(runtime, 4 * usize::from(index >> 8)));
        if (covered >> 16).wrapping_sub(u32::from(height)) & 0x8000 == 0 {
            break;
        }
    }
    let mut edit = ChootEdit::of(runtime, choot);
    *edit.initial_y_speed_index() = index;
    let falling_y = edit.spawn_y().wrapping_sub(height);
    let falling_x = *edit.spawn_x();
    *edit.falling_y() = falling_y;
    *edit.falling_x() = falling_x;
    *edit.y_speed_index() = index;
}
/// Main AI - choot ($A2:E02E): run its function.
pub fn main_ai(runtime: &mut impl Lease) {
    let choot = combat::current(runtime).byte_index();
    match Choot::of(runtime, choot).function() {
        WAIT_FOR_SAMUS => {
            let key = combat::current(runtime);
            if api::player_within(runtime, key, Axis::X, 0x50) {
                let mut edit = ChootEdit::of(runtime, choot);
                let delay = *edit.parameter_2();
                *edit.jump_delay() = delay;
                *edit.function() = PREPARE_TO_JUMP;
            }
        }
        PREPARE_TO_JUMP => {
            let delay = Choot::of(runtime, choot).jump_delay().wrapping_sub(1);
            *ChootEdit::of(runtime, choot).jump_delay() = delay;
            if (delay as i16) < 0 {
                set_instruction_list(runtime, JUMPING_LIST);
                *ChootEdit::of(runtime, choot).function() = JUMPING;
            }
        }
        JUMPING => {
            let index = Choot::of(runtime, choot).y_speed_index() >> 8;
            let rise = speed(runtime, 4 * usize::from(index) + 2);
            let falling = {
                let mut edit = ChootEdit::of(runtime, choot);
                edit.actor().y.advance(rise as i32);
                let index = edit.y_speed_index();
                *index = index.wrapping_sub(SPEED_INDEX_STEP);
                if (*index as i16) < 0 {
                    let x = *edit.falling_x();
                    *edit.origin_x() = x;
                    edit.actor().x.whole = x;
                    let y = *edit.falling_y();
                    *edit.origin_y() = y;
                    edit.actor().y.whole = y;
                    let loops = (*edit.parameter_1() & 0xff).wrapping_sub(1);
                    *edit.pattern_index() = 0;
                    *edit.pattern_loops() = loops;
                    true
                } else {
                    false
                }
            };
            if falling {
                set_instruction_list(runtime, FALLING_LIST);
                *ChootEdit::of(runtime, choot).function() = FALLING;
            }
        }
        FALLING => {
            let view = Choot::of(runtime, choot);
            let step = 4 * (view.pattern_index() >> 8);
            let row_at = view.pattern().wrapping_add(step);
            let row = runtime.lease_native_record(0xa2, row_at);
            if row.word(0) != PATTERN_END {
                let mut edit = ChootEdit::of(runtime, choot);
                let x = row.word(0).wrapping_add(*edit.origin_x());
                let y = row.word(2).wrapping_add(*edit.origin_y());
                let actor = edit.actor();
                actor.x.whole = x;
                actor.y.whole = y;
                let index = edit.pattern_index();
                *index = index.wrapping_add(0x100);
                return;
            }
            let restart = {
                let mut edit = ChootEdit::of(runtime, choot);
                let distance = *edit.pattern_y_distance();
                let origin = edit.origin_y();
                *origin = origin.wrapping_add(distance);
                let loops = edit.pattern_loops().wrapping_sub(1);
                *edit.pattern_index() = 0;
                *edit.pattern_loops() = loops;
                if loops & 0x8000 != 0 {
                    let (x, y) = (*edit.spawn_x(), *edit.spawn_y());
                    let index = *edit.initial_y_speed_index();
                    *edit.y_speed_index() = index;
                    let actor = edit.actor();
                    actor.x.whole = x;
                    actor.x.fraction = 0;
                    actor.y.whole = y;
                    actor.y.fraction = 0;
                    true
                } else {
                    false
                }
            };
            if restart {
                set_instruction_list(runtime, IDLE_LIST);
                *ChootEdit::of(runtime, choot).function() = WAIT_FOR_SAMUS;
            }
        }
        _ => runtime.unreachable(),
    }
}
