//! The morph ball eye, which opens and shines its ray on Samus once she has
//! the morph ball (bank $A8:8F8C-$A8:91DF): proximity, opening, tracking and
//! closing. The runtime owns beam hardware startup.
use crate::actors::api::angle_from_delta;
use crate::actors::boss_lease::BossState;
use crate::actors::host::Axis;
use crate::actors::slot_view::{SlotState, slot_view};

/// What the morph ball eye needs beyond a boss's state: its beam.
pub trait Host: BossState + SlotState {
    /// Start the eye's beam effect.
    fn start_beam(&mut self);
}

slot_view! {
    /// The eye's view of its slot.
    Eye, EyeEdit {
        record: [
            activated = var_c, angle = var_d, function_timer = var_e, function = var_f,
            facing_left = parameter_1, mount = parameter_2,
        ],
        extended: [],
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

fn face(eye: u16, right: u16, left: u16, game: &mut (impl Host + ?Sized)) {
    let instruction = if Eye::of(game, eye).facing_left() & 1 == 0 {
        right
    } else {
        left
    };
    EyeEdit::of(game, eye).actor().instruction = instruction;
}
fn samus_near(k: u16, columns: u16, game: &mut (impl Host + ?Sized)) -> bool {
    game.player_within(k, Axis::Y, NEAR_ROWS) && game.player_within(k, Axis::X, columns)
}
fn aim(eye: u16, game: &mut (impl Host + ?Sized)) -> u16 {
    let angle = angle_from_delta(
        game.player_x()
            .wrapping_sub(Eye::of(game, eye).actor().x.whole),
        game.player_y()
            .wrapping_sub(Eye::of(game, eye).actor().y.whole),
    );
    *EyeEdit::of(game, eye).angle() = angle;
    angle
}
/// Initialisation AI - morph ball eye ($A8:9058): an eye, or a mount facing
/// the way its parameter says.
pub fn initialize(game: &mut (impl Host + ?Sized)) {
    let eye = game.current_enemy();
    EyeEdit::of(game, eye).actor().properties |= PROCESS_INSTRUCTIONS;
    {
        let mut edit = EyeEdit::of(game, eye);
        edit.actor().spritemap = NO_SPRITEMAP;
        edit.actor().instruction_delay = 1;
        edit.actor().timer = 0;
    }
    if Eye::of(game, eye).mount() & MOUNT_BIT != 0 {
        let index = (Eye::of(game, eye).mount() & MOUNT_FACING) as usize;
        {
            let mut edit = EyeEdit::of(game, eye);
            let old = edit.actor().x.whole;
            edit.actor().x.whole = old.wrapping_add(MOUNT_X_OFFSETS[index] as u16);
        }
        {
            let mut edit = EyeEdit::of(game, eye);
            let old = edit.actor().y.whole;
            edit.actor().y.whole = old.wrapping_add(MOUNT_Y_OFFSETS[index] as u16);
        }
        *EyeEdit::of(game, eye).function() = function::MOUNT;
        let list = crate::enemy_lists::eye_instruction_list(game, index as isize);
        EyeEdit::of(game, eye).actor().instruction = list;
        crate::word_transfers!(game, descending, 256; native(BEAM_WINDOW_TABLE) <- value(255));
    } else {
        *EyeEdit::of(game, eye).function() = function::WAIT_FOR_SAMUS;
        face(
            eye,
            instruction_list::RIGHT_CLOSED,
            instruction_list::LEFT_CLOSED,
            game,
        );
    }
}
/// Main AI - morph ball eye ($A8:90E2): its function, once the morph ball is
/// collected; an eye that has opened starts its beam and tracks Samus.
pub fn main_ai(game: &mut (impl Host + ?Sized)) {
    if game.session().inventory.equipment.owned & MORPH_BALL == 0 {
        return;
    }
    let k = game.current_enemy();
    let function = Eye::of(game, k).function();
    if run_function(function, k, game) {
        game.start_beam();
        start_tracking(k, game);
    }
}
/// Morph ball eye functions ($A8:90F1, $A8:912E, $A8:9160, $A8:91CE): true
/// when the eye has opened and its beam should start, after its sound and
/// before hardware startup and [`start_tracking`]. Was `prepare_update`.
pub fn run_function(function: u16, k: u16, game: &mut (impl Host + ?Sized)) -> bool {
    let eye = k;
    match function {
        function::MOUNT => (),
        function::WAIT_FOR_SAMUS => {
            if samus_near(k, NEAR_COLUMNS, game) {
                *EyeEdit::of(game, eye).function_timer() = OPEN_FRAMES;
                EyeEdit::of(game, eye).actor().instruction_delay = 1;
                face(
                    eye,
                    instruction_list::RIGHT_ACTIVATING,
                    instruction_list::LEFT_ACTIVATING,
                    game,
                );
                *EyeEdit::of(game, eye).function() = function::ACTIVATING;
            }
        }
        function::ACTIVE => {
            if samus_near(k, VIEW_COLUMNS, game) {
                let angle = aim(eye, game);
                let list = (((angle as i32 & ANGLE_STEP) >> 2)
                    + i32::from(instruction_list::ACTIVE)) as u16;
                EyeEdit::of(game, eye).actor().instruction = list;
            } else {
                game.queue_sound(1, SILENCE, 6);
                {
                    let mut edit = EyeEdit::of(game, eye);
                    *edit.activated() = 0;
                    *edit.function_timer() = OPEN_FRAMES;
                }
                face(
                    eye,
                    instruction_list::RIGHT_DEACTIVATING,
                    instruction_list::LEFT_DEACTIVATING,
                    game,
                );
                *EyeEdit::of(game, eye).function() = function::DEACTIVATING;
            }
            EyeEdit::of(game, eye).actor().instruction_delay = 1;
        }
        function::ACTIVATING | function::DEACTIVATING => {
            let expired = Eye::of(game, eye).function_timer() == 1;
            {
                let mut edit = EyeEdit::of(game, eye);
                let old = *edit.function_timer();
                *edit.function_timer() = old.wrapping_sub(1);
            }
            let negative = Eye::of(game, eye).function_timer() & SIGN != 0;
            if expired || negative {
                if function == function::ACTIVATING {
                    game.queue_sound(1, RAY_SOUND, 6);
                    return true;
                }
                *EyeEdit::of(game, eye).function() = function::WAIT_FOR_SAMUS;
            }
        }
        _ => unreachable!("morph ball eye function {function:04x}"),
    }
    false
}
/// The end of activating, after the beam starts: the eye is active and aims at
/// Samus. Was `finish_beam`.
pub fn start_tracking(k: u16, game: &mut (impl Host + ?Sized)) {
    let eye = k;
    *EyeEdit::of(game, eye).function() = function::ACTIVE;
    aim(eye, game);
}
