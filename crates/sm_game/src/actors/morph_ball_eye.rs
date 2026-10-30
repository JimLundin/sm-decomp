//! The morph ball eye, which opens and shines its ray on Samus once she has
//! the morph ball (bank $A8:8F8C-$A8:91DF): proximity, opening, tracking and
//! closing. The runtime owns beam hardware startup.
#![forbid(unsafe_code)]
use crate::actors::api::angle_from_delta;
use crate::actors::host::{ActorHost, ActorRuntime, Axis};
use crate::actors::layouts::part_view;
use crate::effects::SoundEffects;
use crate::ram::RawRam as _;
use crate::ram::{slot, word};
use crate::records::Records;

/// What the morph ball eye needs: the shared actor operations, sound and its
/// mount tables.
pub trait Host: ActorHost + ActorRuntime + SoundEffects + Records {}
impl<T: ActorHost + ActorRuntime + SoundEffects + Records + ?Sized> Host for T {}

part_view! {
    /// The eye's view of its slot.
    Eye: EnemyLayout8 {
        words: [var_A, var_B, activated, angle, function_timer, function, facing_left, mount],
    }
}

/// Bank $A8 morph ball eye functions, stored in `function`.
pub mod function {
    pub const WAIT_FOR_SAMUS: u16 = 0x90f1;
    pub const ACTIVATING: u16 = 0x912e;
    pub const ACTIVE: u16 = 0x9160;
    pub const DEACTIVATING: u16 = 0x91ce;
    /// $A8:91DC, an RTL: the mount's function.
    pub const MOUNT: u16 = 0x91dc;
}

/// Bank $A8 morph ball eye instruction lists, by facing.
pub mod instruction_list {
    /// $A8:8FAC: the active eye, one list of 4 bytes per 10h of angle.
    pub const ACTIVE: u16 = 0x8fac;
    pub const RIGHT_DEACTIVATING: u16 = 0x8ff0;
    pub const RIGHT_CLOSED: u16 = 0x8ffc;
    pub const LEFT_DEACTIVATING: u16 = 0x9002;
    pub const LEFT_CLOSED: u16 = 0x900e;
    pub const RIGHT_ACTIVATING: u16 = 0x9014;
    pub const LEFT_ACTIVATING: u16 = 0x9026;
}

const BANK: u32 = 0xa8;
/// Properties bit 13: process instructions; $A8:804D: no spritemap.
const PROCESS_INSTRUCTIONS: u16 = 0x2000;
const NO_SPRITEMAP: u16 = 0x804d;
/// Parameter 2 bit 15 makes a mount, whose low nibble faces right, left, up
/// or down, offsetting it by these ($A8:90CA, $A8:90D2).
const MOUNT_BIT: u16 = 0x8000;
const MOUNT_FACING: u16 = 15;
const MOUNT_X_OFFSETS: [i16; 4] = [-8, 8, 0, 0];
const MOUNT_Y_OFFSETS: [i16; 4] = [0, 0, -8, 8];
/// $7E:92FE: the beam's window 1 HDMA data table, cleared to FFh,00h.
const BEAM_WINDOW_TABLE: usize = 0x92fe;
/// Samus is near within 80h px rows and 80h px columns ($A8:9050), and stays
/// in view within B0h px columns.
const NEAR_ROWS: u16 = 0x80;
const NEAR_COLUMNS: u16 = 0x80;
const VIEW_COLUMNS: u16 = 0xb0;
const OPEN_FRAMES: u16 = 32;
/// The ray (17h) and silence (71h), library 2.
const RAY_SOUND: u8 = 0x17;
const SILENCE: u8 = 0x71;
const ANGLE_STEP: i32 = 0xf0;
/// Collected items bit 2: the morph ball.
const MORPH_BALL: u16 = 4;
const SIGN: u16 = 0x8000;

fn face(eye: i32, right: u16, left: u16, game: &mut (impl Host + ?Sized)) {
    let instruction = if slot!(game, Eye[eye].facing_left) & 1 == 0 {
        right
    } else {
        left
    };
    slot!(game, Eye[eye].base.current_instruction = instruction);
}
fn samus_near(k: u16, columns: u16, game: &mut (impl Host + ?Sized)) -> bool {
    game.player_within(k, Axis::Y, NEAR_ROWS) && game.player_within(k, Axis::X, columns)
}
fn aim(eye: i32, game: &mut (impl Host + ?Sized)) -> u16 {
    let angle = angle_from_delta(
        game.player_x()
            .wrapping_sub(slot!(game, Eye[eye].base.x_pos)),
        game.player_y()
            .wrapping_sub(slot!(game, Eye[eye].base.y_pos)),
    );
    slot!(game, Eye[eye].angle = angle);
    angle
}
/// Initialisation AI - morph ball eye ($A8:9058): an eye, or a mount facing
/// the way its parameter says.
pub fn initialize(game: &mut (impl Host + ?Sized)) {
    let eye = word!(game.cur_enemy_index) as i32;
    slot!(game, Eye[eye].base.properties, |old| old
        | PROCESS_INSTRUCTIONS);
    slot!(game, Eye[eye].base; spritemap_pointer = NO_SPRITEMAP, instruction_timer = 1, timer = 0);
    if slot!(game, Eye[eye].mount) & MOUNT_BIT != 0 {
        let index = (slot!(game, Eye[eye].mount) & MOUNT_FACING) as usize;
        slot!(game, Eye[eye].base.x_pos, |old| old
            .wrapping_add(MOUNT_X_OFFSETS[index] as u16));
        slot!(game, Eye[eye].base.y_pos, |old| old
            .wrapping_add(MOUNT_Y_OFFSETS[index] as u16));
        slot!(game, Eye[eye].function = function::MOUNT);
        slot!(
            game,
            Eye[eye].base.current_instruction =
                crate::enemy_lists::eye_instruction_list(game, index as isize)
        );
        crate::word_transfers!(game, descending, 256; native(BEAM_WINDOW_TABLE) <- value(255));
    } else {
        slot!(game, Eye[eye].function = function::WAIT_FOR_SAMUS);
        face(
            eye,
            instruction_list::RIGHT_CLOSED,
            instruction_list::LEFT_CLOSED,
            game,
        );
    }
}
/// Main AI - morph ball eye ($A8:90E2): its function, once the morph ball is
/// collected. Was `state_address`.
pub fn main_ai(game: &mut (impl Host + ?Sized)) -> Option<u32> {
    if crate::inventory::get(game, crate::inventory::Word::CollectedItems) & MORPH_BALL == 0 {
        return None;
    }
    let eye = word!(game.cur_enemy_index) as i32;
    Some(BANK << 16 | u32::from(slot!(game, Eye[eye].function)))
}
/// Morph ball eye functions ($A8:90F1, $A8:912E, $A8:9160, $A8:91CE): true
/// when the eye has opened and its beam should start, after its sound and
/// before hardware startup and [`start_tracking`]. Was `prepare_update`.
pub fn run_function(function: u16, k: u16, game: &mut (impl Host + ?Sized)) -> bool {
    let eye = k as i32;
    match function {
        function::WAIT_FOR_SAMUS => {
            if samus_near(k, NEAR_COLUMNS, game) {
                slot!(game, Eye[eye].function_timer = OPEN_FRAMES);
                slot!(game, Eye[eye].base.instruction_timer = 1);
                face(
                    eye,
                    instruction_list::RIGHT_ACTIVATING,
                    instruction_list::LEFT_ACTIVATING,
                    game,
                );
                slot!(game, Eye[eye].function = function::ACTIVATING);
            }
        }
        function::ACTIVE => {
            if samus_near(k, VIEW_COLUMNS, game) {
                let angle = aim(eye, game);
                slot!(
                    game,
                    Eye[eye].base.current_instruction = (((angle as i32 & ANGLE_STEP) >> 2)
                        + i32::from(instruction_list::ACTIVE))
                        as u16
                );
            } else {
                game.queue_sound(1, SILENCE, 6);
                slot!(game, Eye[eye]; activated = 0, function_timer = OPEN_FRAMES);
                face(
                    eye,
                    instruction_list::RIGHT_DEACTIVATING,
                    instruction_list::LEFT_DEACTIVATING,
                    game,
                );
                slot!(game, Eye[eye].function = function::DEACTIVATING);
            }
            slot!(game, Eye[eye].base.instruction_timer = 1);
        }
        function::ACTIVATING | function::DEACTIVATING => {
            let expired = slot!(game, Eye[eye].function_timer) == 1;
            slot!(game, Eye[eye].function_timer, |old| old.wrapping_sub(1));
            let negative = slot!(game, Eye[eye].function_timer) & SIGN != 0;
            if expired || negative {
                if function == function::ACTIVATING {
                    game.queue_sound(1, RAY_SOUND, 6);
                    return true;
                }
                slot!(game, Eye[eye].function = function::WAIT_FOR_SAMUS);
            }
        }
        _ => unreachable!("morph ball eye function {function:04x}"),
    }
    false
}
/// The end of activating, after the beam starts: the eye is active and aims at
/// Samus. Was `finish_beam`.
pub fn start_tracking(k: u16, game: &mut (impl Host + ?Sized)) {
    let eye = k as i32;
    slot!(game, Eye[eye].function = function::ACTIVE);
    aim(eye, game);
}
