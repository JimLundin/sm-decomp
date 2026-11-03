//! Skultera, Maridia's bony fish, which swims a wave left and right and turns
//! at walls (bank $A3:900A-$A3:94AA). Was `maridia_fish`.
use crate::actors::boss_lease::BossState;
use crate::actors::host::Slopes;
use crate::actors::slot_view::{SlotState, slot_view};
use crate::enemy_script::EnemyScript;
use sm_data::enemy_speeds::SpeedTable;

/// What Skultera needs beyond the shared actor operations; its slot is the
/// Session's.
pub trait Host: BossState + SlotState {
    fn unreachable(&mut self);
}

slot_view! {
    /// Skultera's view of its slot.
    Skultera, SkulteraEdit {
        record: [
            function = var_a, right_subvelocity = var_b, right_velocity = var_c,
            left_subvelocity = var_d, left_velocity = var_e, angle = var_f,
            parameter_1 = parameter_1, parameter_2 = parameter_2,
        ],
        extended: [
            radius = var_00, turn_finished = var_01, angle_delta = var_02,
            previous_y_offset = var_03, y_offset = var_04,
        ],
    }
}

/// Bank $A3 Skultera functions, stored in `function`.
pub mod function {
    pub const SWIMMING_LEFT: u16 = 0x9132;
    pub const SWIMMING_RIGHT: u16 = 0x91ab;
    pub const TURNING_RIGHT: u16 = 0x9224;
    pub const TURNING_LEFT: u16 = 0x9256;
}

/// Bank $A3 Skultera instruction lists.
pub mod instruction_list {
    pub const SWIMMING_LEFT: u16 = 0x902a;
    pub const TURNING_RIGHT: u16 = 0x903c;
    pub const SWIMMING_RIGHT: u16 = 0x9060;
    pub const TURNING_LEFT: u16 = 0x9072;
}

/// Bank $A3 instructions its scripts call.
pub mod instruction {
    pub const LAYER_6: u32 = 0xa3_9096;
    pub const LAYER_2: u32 = 0xa3_90a0;
    pub const SET_TURN_FINISHED: u32 = 0xa3_90aa;
}

fn current(game: &(impl Host + ?Sized)) -> u16 {
    game.current_enemy()
}
/// The wave's Y offset at the fish's angle.
fn y_offset(fish: u16, game: &mut (impl Host + ?Sized)) -> u16 {
    let angle = Skultera::of(game, fish).angle();
    let radius = Skultera::of(game, fish).radius();
    game.scaled_sine(angle, radius)
}
/// Initialisation AI - Skultera ($A3:90B5).
pub fn initialize(game: &mut (impl Host + ?Sized)) {
    let fish = current(game);
    SkulteraEdit::of(game, fish).actor().instruction = instruction_list::SWIMMING_LEFT;
    *SkulteraEdit::of(game, fish).function() = function::SWIMMING_LEFT;
    if Skultera::of(game, fish).parameter_1().to_le_bytes()[1] == 0 {
        SkulteraEdit::of(game, fish).actor().instruction = instruction_list::SWIMMING_RIGHT;
        *SkulteraEdit::of(game, fish).function() = function::SWIMMING_RIGHT;
    }
    let index = 4 * isize::from(Skultera::of(game, fish).parameter_1().to_le_bytes()[0]);
    let value = game.enemy_speed(SpeedTable::Linear, index);
    *SkulteraEdit::of(game, fish).right_velocity() = value;
    let value = game.enemy_speed(SpeedTable::Linear, index + 1);
    *SkulteraEdit::of(game, fish).right_subvelocity() = value;
    let value = game.enemy_speed(SpeedTable::Linear, index + 2);
    *SkulteraEdit::of(game, fish).left_velocity() = value;
    let value = game.enemy_speed(SpeedTable::Linear, index + 3);
    *SkulteraEdit::of(game, fish).left_subvelocity() = value;
    let value = u16::from(Skultera::of(game, fish).parameter_2().to_le_bytes()[0]);
    *SkulteraEdit::of(game, fish).radius() = value;
    let value = u16::from(Skultera::of(game, fish).parameter_2().to_le_bytes()[1]);
    {
        let mut edit = SkulteraEdit::of(game, fish);
        *edit.angle_delta() = value;
        *edit.angle() = 0;
        *edit.turn_finished() = 0;
    }
    let offset = y_offset(fish, game);
    *SkulteraEdit::of(game, fish).previous_y_offset() = offset;
}
fn set_instruction_list(fish: u16, instruction: u16, game: &mut (impl Host + ?Sized)) {
    let mut edit = SkulteraEdit::of(game, fish);
    edit.actor().instruction_delay = 1;
    edit.actor().timer = 0;
    edit.actor().instruction = instruction;
}
fn turn(fish: u16, left: bool, game: &mut (impl Host + ?Sized)) {
    let (function, list) = if left {
        (function::TURNING_RIGHT, instruction_list::TURNING_RIGHT)
    } else {
        (function::TURNING_LEFT, instruction_list::TURNING_LEFT)
    };
    *SkulteraEdit::of(game, fish).function() = function;
    set_instruction_list(fish, list, game);
}
/// Skultera function - swimming left ($A3:9132) or right ($A3:91AB): swim on
/// the wave, turning at walls and blocks.
fn swim(fish: u16, left: bool, game: &mut (impl Host + ?Sized)) {
    let k = game.current_enemy();
    let amount = if left {
        (u32::from(Skultera::of(game, fish).left_velocity()) << 16)
            | u32::from(Skultera::of(game, fish).left_subvelocity())
    } else {
        (u32::from(Skultera::of(game, fish).right_velocity()) << 16)
            | u32::from(Skultera::of(game, fish).right_subvelocity())
    } as i32;
    if game.move_horizontal(k, amount, Slopes::Ignore) {
        turn(fish, left, game);
    } else {
        let offset = y_offset(fish, game);
        *SkulteraEdit::of(game, fish).y_offset() = offset;
        let k = game.current_enemy();
        let amount = i32::from(
            (offset as i16).wrapping_sub(Skultera::of(game, fish).previous_y_offset() as i16),
        ) << 16;
        if game.move_vertical(k, amount) {
            turn(fish, left, game);
        } else {
            let angle = Skultera::of(game, fish).angle_delta().to_le_bytes()[0]
                .wrapping_add(Skultera::of(game, fish).angle() as u8);
            *SkulteraEdit::of(game, fish).angle() = u16::from(angle);
        }
    }
    let value = Skultera::of(game, fish).y_offset();
    *SkulteraEdit::of(game, fish).previous_y_offset() = value;
}
/// Skultera function - turning right ($A3:9224) or left ($A3:9256): once the
/// turn has finished, swim the other way.
fn finish_turn(fish: u16, right: bool, game: &mut (impl Host + ?Sized)) {
    if Skultera::of(game, fish).turn_finished() != 0 {
        *SkulteraEdit::of(game, fish).turn_finished() = 0;
        let (function, list) = if right {
            (function::SWIMMING_RIGHT, instruction_list::SWIMMING_RIGHT)
        } else {
            (function::SWIMMING_LEFT, instruction_list::SWIMMING_LEFT)
        };
        *SkulteraEdit::of(game, fish).function() = function;
        {
            let mut edit = SkulteraEdit::of(game, fish);
            let old = *edit.angle_delta();
            *edit.angle_delta() = old.wrapping_neg();
        }
        set_instruction_list(fish, list, game);
    }
}
/// Main AI - Skultera ($A3:912B). Was `advance`.
pub fn main_ai(game: &mut (impl Host + ?Sized)) {
    let fish = current(game);
    match Skultera::of(game, fish).function() {
        function::SWIMMING_LEFT => swim(fish, true, game),
        function::SWIMMING_RIGHT => swim(fish, false, game),
        function::TURNING_RIGHT => finish_turn(fish, true, game),
        function::TURNING_LEFT => finish_turn(fish, false, game),
        _ => game.unreachable(),
    }
}
/// Run one of Skultera's script instructions (one of [`instruction`]).
pub fn instruction(
    address: u32,
    script: EnemyScript,
    game: &mut (impl Host + ?Sized),
) -> EnemyScript {
    let fish = current(game);
    match address {
        instruction::LAYER_6 => SkulteraEdit::of(game, fish).actor().layer = 6,
        instruction::LAYER_2 => SkulteraEdit::of(game, fish).actor().layer = 2,
        instruction::SET_TURN_FINISHED => *SkulteraEdit::of(game, fish).turn_finished() = 1,
        _ => unreachable!("unrecognized Skultera instruction"),
    }
    script
}
