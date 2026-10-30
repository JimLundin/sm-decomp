//! Skultera, Maridia's bony fish, which swims a wave left and right and turns
//! at walls (bank $A3:900A-$A3:94AA). Was `maridia_fish`.
#![forbid(unsafe_code)]
use crate::actors::host::{ActorHost, ActorRuntime, Slopes};
use crate::actors::layouts::part_view;
use crate::enemy_script::EnemyScript;
use crate::ram::RawRam as _;
use crate::ram::{slot, word};
use sm_data::enemy_speeds::SpeedTable;

/// What Skultera needs beyond the shared actor operations.
pub trait Host: ActorHost + ActorRuntime {
    /// `amplitude` times the sine of `angle`.
    fn actor_sine(&mut self, angle: u16, amplitude: u16) -> u16;
    fn unreachable(&mut self);
}

part_view! {
    /// Skultera's view of its slot.
    Skultera: EnemyLayout40 {
        words: [
            function, right_subvelocity, right_velocity, left_subvelocity, left_velocity, angle,
            parameter_1, parameter_2,
        ],
        pad1: [u8; 26696],
        words: [
            radius, turn_finished, angle_delta, previous_y_offset, y_offset, var_05, var_06, var_07,
            var_08, var_09, var_0a, var_0b, var_0c, var_0d, var_0e, var_0f, var_10, var_11, var_12,
            var_13, var_14, var_15, var_16, var_17, var_18, var_19, var_1a, var_1b, var_1c, var_1d,
            var_1e, var_1f,
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

fn current(game: &(impl Host + ?Sized)) -> i32 {
    i32::from(word!(game.cur_enemy_index))
}
/// The wave's Y offset at the fish's angle.
fn y_offset(fish: i32, game: &mut (impl Host + ?Sized)) -> u16 {
    let angle = slot!(game, Skultera[fish].angle);
    let radius = slot!(game, Skultera[fish].radius);
    game.actor_sine(angle, radius)
}
/// Initialisation AI - Skultera ($A3:90B5).
pub fn initialize(game: &mut (impl Host + ?Sized)) {
    let fish = current(game);
    slot!(
        game,
        Skultera[fish].base.current_instruction = instruction_list::SWIMMING_LEFT
    );
    slot!(game, Skultera[fish].function = function::SWIMMING_LEFT);
    if slot!(byte game, Skultera[fish].parameter_1[1]) == 0 {
        slot!(
            game,
            Skultera[fish].base.current_instruction = instruction_list::SWIMMING_RIGHT
        );
        slot!(game, Skultera[fish].function = function::SWIMMING_RIGHT);
    }
    let index = 4 * isize::from(slot!(byte game, Skultera[fish].parameter_1[0]));
    let value = game.enemy_speed(SpeedTable::Linear, index);
    slot!(game, Skultera[fish].right_velocity = value);
    let value = game.enemy_speed(SpeedTable::Linear, index + 1);
    slot!(game, Skultera[fish].right_subvelocity = value);
    let value = game.enemy_speed(SpeedTable::Linear, index + 2);
    slot!(game, Skultera[fish].left_velocity = value);
    let value = game.enemy_speed(SpeedTable::Linear, index + 3);
    slot!(game, Skultera[fish].left_subvelocity = value);
    let value = u16::from(slot!(byte game, Skultera[fish].parameter_2[0]));
    slot!(game, Skultera[fish].radius = value);
    let value = u16::from(slot!(byte game, Skultera[fish].parameter_2[1]));
    slot!(game, Skultera[fish].angle_delta = value);
    slot!(game, Skultera[fish].angle = 0);
    slot!(game, Skultera[fish].turn_finished = 0);
    let offset = y_offset(fish, game);
    slot!(game, Skultera[fish].previous_y_offset = offset);
}
fn set_instruction_list(fish: i32, instruction: u16, game: &mut (impl Host + ?Sized)) {
    slot!(game, Skultera[fish].base.instruction_timer = 1);
    slot!(game, Skultera[fish].base.timer = 0);
    slot!(game, Skultera[fish].base.current_instruction = instruction);
}
fn turn(fish: i32, left: bool, game: &mut (impl Host + ?Sized)) {
    let (function, list) = if left {
        (function::TURNING_RIGHT, instruction_list::TURNING_RIGHT)
    } else {
        (function::TURNING_LEFT, instruction_list::TURNING_LEFT)
    };
    slot!(game, Skultera[fish].function = function);
    set_instruction_list(fish, list, game);
}
/// Skultera function - swimming left ($A3:9132) or right ($A3:91AB): swim on
/// the wave, turning at walls and blocks.
fn swim(fish: i32, left: bool, game: &mut (impl Host + ?Sized)) {
    let k = word!(game.cur_enemy_index);
    let amount = if left {
        (u32::from(slot!(game, Skultera[fish].left_velocity)) << 16)
            | u32::from(slot!(game, Skultera[fish].left_subvelocity))
    } else {
        (u32::from(slot!(game, Skultera[fish].right_velocity)) << 16)
            | u32::from(slot!(game, Skultera[fish].right_subvelocity))
    } as i32;
    if game.move_horizontal(k, amount, Slopes::Ignore) {
        turn(fish, left, game);
    } else {
        let offset = y_offset(fish, game);
        slot!(game, Skultera[fish].y_offset = offset);
        let k = word!(game.cur_enemy_index);
        let amount = i32::from(
            (offset as i16).wrapping_sub(slot!(game, Skultera[fish].previous_y_offset) as i16),
        ) << 16;
        if game.move_vertical(k, amount) {
            turn(fish, left, game);
        } else {
            let angle = slot!(byte game, Skultera[fish].angle_delta[0]).wrapping_add(slot!(
                game,
                Skultera[fish].angle
            )
                as u8);
            slot!(game, Skultera[fish].angle = u16::from(angle));
        }
    }
    slot!(
        game,
        Skultera[fish].previous_y_offset = slot!(game, Skultera[fish].y_offset)
    );
}
/// Skultera function - turning right ($A3:9224) or left ($A3:9256): once the
/// turn has finished, swim the other way.
fn finish_turn(fish: i32, right: bool, game: &mut (impl Host + ?Sized)) {
    if slot!(game, Skultera[fish].turn_finished) != 0 {
        slot!(game, Skultera[fish].turn_finished = 0);
        let (function, list) = if right {
            (function::SWIMMING_RIGHT, instruction_list::SWIMMING_RIGHT)
        } else {
            (function::SWIMMING_LEFT, instruction_list::SWIMMING_LEFT)
        };
        slot!(game, Skultera[fish].function = function);
        slot!(game, Skultera[fish].angle_delta, |old| old.wrapping_neg());
        set_instruction_list(fish, list, game);
    }
}
/// Main AI - Skultera ($A3:912B). Was `advance`.
pub fn main_ai(game: &mut (impl Host + ?Sized)) {
    let fish = current(game);
    match slot!(game, Skultera[fish].function) {
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
        instruction::LAYER_6 => slot!(game, Skultera[fish].base.layer = 6),
        instruction::LAYER_2 => slot!(game, Skultera[fish].base.layer = 2),
        instruction::SET_TURN_FINISHED => slot!(game, Skultera[fish].turn_finished = 1),
        _ => unreachable!("unrecognized Skultera instruction"),
    }
    script
}
