//! Yapping Maw behavior over canonical actor/player state and synchronous effects.
#![forbid(unsafe_code)]
use super::{
    BASE_FACING_DOWN, BASE_FACING_UP, BASE_OFFSET, BITE_SOUND, BODY_PROJECTILE, COOLDOWN_FRAMES,
    DEMO_INPUT, GRABBED_INPUT, INTANGIBLE_FRAMES, LAST_SEGMENT, LEFT, LOCK_SAMUS, MAX_LENGTH, Maw,
    NEGATE_SPEED, NORMAL_INPUT, PALETTE_BITS, QUARTER_TURN, REACH, RIGHT, SIGN, SPEED_STEP,
    STRAIGHT, function, instruction as instruction_id, instruction_list,
};
use crate::actors::host::{Shot, abs16};
use crate::actors::session_slots::session_slot;
use crate::enemy_script::EnemyScript;
use crate::geometry::Point;
use crate::player_transitions::state::InputMode;
use sm_data::enemy_speeds::SpeedTable;

pub trait Host:
    crate::actors::access::ActorAccess
    + crate::actors::session_slots::SessionSlots
    + crate::effects::SoundEffects
{
    fn create_sprite(&mut self, x: u16, y: u16, kind: u16, appearance: u16) -> u16;
    fn spawn_projectile(&mut self, parameter: u16, key: u16, definition: u16) -> u16;
    fn distance_and_direction(&mut self, origin: Point, target: Point) -> (u16, u16);
    fn command_player(&mut self, command: u16);
    fn correct_previous_position(&mut self);
    fn write_maw_operand(&mut self, operand: u8);
    fn enemy_sine(&mut self, index: isize) -> u16;
    fn attack_list(&mut self, index: isize) -> u16;
    fn attachment_offset(&mut self, y: bool, native: bool, index: isize) -> u16;
    fn enemy_speed(&self, table: SpeedTable, index: isize) -> u16;
    fn update_maw_visibility(&mut self, key: i32);
    fn update_maw_segment(&mut self, part: u8);
    fn maw_appearance(&mut self, handle: u16, appearance: u16, key: u16);
    fn shot_reaction(&mut self, shot: Shot);
    fn free_projectile(&mut self, key: u16);
    fn retire_sprite(&mut self, handle: u16);
    fn update_frozen_enemy(&mut self);
    fn update_maw_palette(&mut self, handle: u16);
}

pub fn initialize<L: Host>(game: &mut L) {
    let maw = game.session().actors.controls.selected as i32;
    let sprite_owner = game.session().actors.controls.selected as i32;
    let copied = session_slot!(game, Maw[maw].base.x_pos);
    session_slot!(game, Maw[maw].base_x = copied);
    let copied = session_slot!(game, Maw[maw].base.y_pos);
    session_slot!(game, Maw[maw].base_y = copied);
    for field in 0..8 {
        session_slot!(game, Maw[maw + 2 * field].segment_x_offset = 0);
    }
    session_slot!(game, Maw[maw]; grabbing_samus = 0, proximity = session_slot!(game, Maw[maw].parameter_1), cooldown_timer = COOLDOWN_FRAMES);
    session_slot!(
        game,
        Maw[maw].base.current_instruction = instruction_list::INITIAL_FACING_UP
    );
    let (mut sprite_kind, mut displacement) = (BASE_FACING_UP, BASE_OFFSET);
    if session_slot!(game, Maw[maw].parameter_2) == 0 {
        session_slot!(
            game,
            Maw[maw].base.current_instruction = instruction_list::INITIAL_FACING_DOWN
        );
        sprite_kind = BASE_FACING_DOWN;
        displacement = BASE_OFFSET.wrapping_neg();
    }
    session_slot!(game, Maw[maw].base; instruction_timer = 1, timer = 0);
    session_slot!(game, Maw[maw]; function = function::NEUTRAL,
        initial_palette = session_slot!(game, Maw[maw].base.palette_index) & PALETTE_BITS);
    let mut part = LAST_SEGMENT as i16;
    session_slot!(game, Maw[maw].segment_index = LAST_SEGMENT);
    loop {
        game.spawn_projectile(
            part as u16,
            game.session().actors.controls.selected,
            BODY_PROJECTILE,
        );
        part = (session_slot!(game, Maw[maw].segment_index) as i16).wrapping_sub(1);
        session_slot!(game, Maw[maw].segment_index = part as u16);
        if part < 0 {
            break;
        }
    }
    let appearance = session_slot!(game, Maw[maw].base.vram_tiles_index)
        | session_slot!(game, Maw[maw].base.palette_index);
    session_slot!(game, Maw[maw].sprite_appearance = appearance);
    let x = session_slot!(game, Maw[maw].base.x_pos);
    let y = displacement.wrapping_add(session_slot!(game, Maw[maw].base.y_pos));
    let sprite = game.create_sprite(x, y, sprite_kind, appearance);
    session_slot!(game, Maw[sprite_owner].sprite_object = sprite);
}

fn neutral<L: Host>(game: &mut L) {
    let maw = game.session().actors.controls.selected as i32;
    let origin = Point {
        x: session_slot!(game, Maw[maw].base.x_pos),
        y: session_slot!(game, Maw[maw].base.y_pos),
    };
    let target = Point {
        x: game.session().player.x,
        y: game.session().player.y,
    };
    let (pair_distance, pair_direction) = game.distance_and_direction(origin, target);
    let distance = abs16(pair_distance);
    session_slot!(game, Maw[maw].target_length = distance);
    if distance.wrapping_sub(REACH) & SIGN != 0 {
        session_slot!(game, Maw[maw].intangibility_timer = INTANGIBLE_FRAMES);
    } else if (distance as i16).wrapping_sub(session_slot!(game, Maw[maw].proximity) as i16) < 0 {
        if session_slot!(game, Maw[maw].target_length).wrapping_sub(MAX_LENGTH) & SIGN == 0 {
            session_slot!(game, Maw[maw].target_length = MAX_LENGTH);
        }
        session_slot!(game, Maw[maw]; target_angle = pair_direction, function = function::ATTACK);
    }
}

fn project<L: Host>(angle: u16, distance: u16, game: &mut L) -> u16 {
    let sine = game.enemy_sine(-(angle as i32) as u8 as isize) as i16;
    let magnitude = if sine < 0 {
        -(sine as i32) as i16
    } else {
        sine
    };
    // The operand is physical byte one of the native signed word.
    let operand = magnitude.to_ne_bytes()[1];
    game.write_maw_operand(operand);
    if distance as u8 == 0 {
        return 0;
    }
    let product = u16::from(operand) * u16::from(distance as u8);
    if sine < 0 {
        ((2 * ((-(product as i32) as u16 as i32) >> 8)) | 0xff00) as u16
    } else {
        2 * (product >> 8)
    }
}

fn attack<L: Host>(game: &mut L) {
    crate::probe::hit("yapping_maw.attack");
    let maw = game.session().actors.controls.selected as i32;
    session_slot!(game, Maw[maw]; angle = 0, subangle = 0, angular_speed_index = 0);
    session_slot!(
        game,
        Maw[maw].length = session_slot!(game, Maw[maw].target_length) >> 1
    );
    let direction =
        QUARTER_TURN.wrapping_sub(session_slot!(game, Maw[maw].target_angle) as u8) as u16;
    session_slot!(game, Maw[maw]; maths_target_angle = direction, aiming_down = u16::from(direction >= STRAIGHT));
    let origin = game.session().actors.controls.selected as i32;
    let length = session_slot!(game, Maw[origin].length);
    let x = project(64, length, game);
    session_slot!(game, Maw[origin].origin_x = x);
    let length = session_slot!(game, Maw[origin].length) >> 1;
    let y = project(128, length, game);
    session_slot!(game, Maw[origin].origin_y = y);
    let direction =
        2 * ((session_slot!(game, Maw[maw].target_angle) as u8).wrapping_add(16) as u16 >> 5);
    session_slot!(game, Maw[maw].instruction_list_index = direction);
    session_slot!(game, Maw[maw].base; current_instruction = game.attack_list((direction >> 1) as isize), instruction_timer = 1, timer = 0);
    // These offsets use the native binding; animation above uses the loaded image.
    let x = game.attachment_offset(false, true, direction as isize);
    session_slot!(game, Maw[maw].samus_x_offset = x);
    let y = game.attachment_offset(true, true, direction as isize);
    session_slot!(game, Maw[maw]; samus_y_offset = y, function = function::ATTACKING);
}

fn reorient_segment<L: Host>(part: i32, game: &mut L) {
    let maw = game.session().actors.controls.selected as i32;
    let distance = session_slot!(game, Maw[maw + 4 * part].segment_x_offset);
    let x = project(0u16.wrapping_sub(64), distance, game);
    session_slot!(game, Maw[maw].scratch_x = x);
    let y = project(0, distance, game);
    session_slot!(game, Maw[maw].scratch_y = y);
    let angle = session_slot!(game, Maw[maw].maths_target_angle).wrapping_sub(64);
    let x = project(angle, distance, game);
    session_slot!(
        game,
        Maw[maw + 4 * part].segment_x_velocity =
            x.wrapping_sub(session_slot!(game, Maw[maw].scratch_x))
    );
    let angle = session_slot!(game, Maw[maw].maths_target_angle);
    let y = project(angle, distance, game);
    session_slot!(
        game,
        Maw[maw + 4 * part].segment_y_velocity =
            y.wrapping_sub(session_slot!(game, Maw[maw].scratch_y))
    );
}

fn move_samus_with_pincers<L: Host>(game: &mut L) {
    game.command_player(LOCK_SAMUS);
    let maw = game.session().actors.controls.selected as i32;
    game.session_mut().player.x = session_slot!(game, Maw[maw].samus_x_offset)
        .wrapping_add(session_slot!(game, Maw[maw].base.x_pos));
    game.session_mut().player.y = session_slot!(game, Maw[maw].samus_y_offset)
        .wrapping_add(session_slot!(game, Maw[maw].base.y_pos));
    game.correct_previous_position();
}

fn attacking<L: Host>(game: &mut L) {
    let maw = game.session().actors.controls.selected as i32;
    let step = session_slot!(game, Maw[maw].angle) >> 2;
    let positive = session_slot!(game, Maw[maw].aiming_down) != 0;
    for part in 0..4 {
        let angle = if positive {
            STRAIGHT.wrapping_add(step.wrapping_mul(part as u16 + 1))
        } else {
            STRAIGHT.wrapping_sub(step.wrapping_mul(part as u16 + 1))
        };
        session_slot!(game, Maw[maw + 2 * part].segment_angle = angle);
    }
    let radius = session_slot!(game, Maw[maw].length);
    for part in 0..4 {
        let angle = session_slot!(game, Maw[maw + 2 * part].segment_angle).wrapping_sub(64);
        let x = project(angle, radius, game);
        session_slot!(
            game,
            Maw[maw + 4 * part].segment_x_offset =
                x.wrapping_sub(session_slot!(game, Maw[maw].origin_x))
        );
    }
    let radius = session_slot!(game, Maw[maw].length) >> 1;
    for part in 0..4 {
        let angle = session_slot!(game, Maw[maw + 2 * part].segment_angle);
        let y = project(angle, radius, game);
        session_slot!(
            game,
            Maw[maw + 4 * part].segment_y_offset =
                y.wrapping_sub(session_slot!(game, Maw[maw].origin_y))
        );
    }
    for part in 0..4 {
        reorient_segment(part, game);
    }
    // The original adds after one CLC ($A8:A532), so each add's carry
    // reaches the next offset.
    let mut carry = false;
    for field in 0..8 {
        let velocity = session_slot!(game, Maw[maw + 2 * field].segment_x_velocity);
        let offset = session_slot!(game, Maw[maw + 2 * field].segment_x_offset);
        let (sum, first) = offset.overflowing_add(velocity);
        let (sum, second) = sum.overflowing_add(u16::from(carry));
        carry = first || second;
        session_slot!(game, Maw[maw + 2 * field].segment_x_offset = sum);
    }
    session_slot!(
        game,
        Maw[maw].base.x_pos = session_slot!(game, Maw[maw].head_x_offset)
            .wrapping_add(session_slot!(game, Maw[maw].base_x))
    );
    session_slot!(
        game,
        Maw[maw].base.y_pos = session_slot!(game, Maw[maw].head_y_offset)
            .wrapping_add(session_slot!(game, Maw[maw].base_y))
    );
    let velocity_owner = game.session().actors.controls.selected as i32;
    let speed = session_slot!(game, Maw[velocity_owner].angular_speed_index) as isize >> 1;
    session_slot!(
        game,
        Maw[velocity_owner](angle, subangle) += {
            let high = game.enemy_speed(SpeedTable::Quadratic, speed + 1);
            let low = game.enemy_speed(SpeedTable::Quadratic, speed);
            (u32::from(high) << 16) | u32::from(low)
        }
    );
    session_slot!(game, Maw[velocity_owner].angular_speed_index, |old| old
        .wrapping_add(SPEED_STEP));
    if session_slot!(game, Maw[maw].angle) & SIGN == 0 {
        if session_slot!(game, Maw[maw].angle).wrapping_sub(STRAIGHT) & SIGN == 0 {
            session_slot!(game, Maw[maw]; angle = STRAIGHT, subangle = 0);
            session_slot!(game, Maw[maw].angular_speed_index, |old| old
                .wrapping_add(NEGATE_SPEED));
        }
    } else {
        session_slot!(game, Maw[maw]; function = function::COOLDOWN, intangibility_timer = INTANGIBLE_FRAMES);
        let direction = session_slot!(game, Maw[maw].instruction_list_index);
        session_slot!(
            game,
            Maw[maw].base.current_instruction = if direction == RIGHT {
                instruction_list::COOLDOWN_UP_RIGHT
            } else if direction == LEFT {
                instruction_list::COOLDOWN_UP_LEFT
            } else {
                instruction_list::COOLDOWN_UP
            }
        );
        if session_slot!(game, Maw[maw].parameter_2) != 0 {
            return;
        }
        if session_slot!(game, Maw[maw].instruction_list_index) == RIGHT {
            session_slot!(
                game,
                Maw[maw].base.current_instruction = instruction_list::COOLDOWN_DOWN_RIGHT
            );
        } else if session_slot!(game, Maw[maw].instruction_list_index) == LEFT {
            session_slot!(
                game,
                Maw[maw].base.current_instruction = instruction_list::COOLDOWN_DOWN_LEFT
            );
        } else {
            session_slot!(
                game,
                Maw[maw].base.current_instruction = instruction_list::COOLDOWN_DOWN
            );
            return;
        }
    }
    if session_slot!(game, Maw[maw].grabbing_samus) != 0 {
        move_samus_with_pincers(game);
    }
}

fn cooldown<L: Host>(game: &mut L) {
    let maw = game.session().actors.controls.selected as i32;
    if session_slot!(game, Maw[maw].grabbing_samus) != 0 {
        move_samus_with_pincers(game);
    }
    session_slot!(game, Maw[maw].cooldown_timer, |old| old.wrapping_sub(1));
    if session_slot!(game, Maw[maw].cooldown_timer) & SIGN != 0
        && game.session().player.transitions.input_mode.to_raw() != DEMO_INPUT
    {
        game.session_mut().player.transitions.input_mode = InputMode::from_raw(NORMAL_INPUT);
        session_slot!(game, Maw[maw]; grabbing_samus = 0, intangibility_timer = INTANGIBLE_FRAMES,
            cooldown_timer = COOLDOWN_FRAMES, function = function::NEUTRAL);
    }
}

pub fn main_ai<L: Host>(game: &mut L) -> bool {
    crate::probe::hit("yapping_maw.main_ai");
    let maw = game.session().actors.controls.selected as i32;
    session_slot!(game, Maw[maw].intangibility_timer, |old| old
        .wrapping_sub(1));
    game.update_maw_visibility(maw);
    match session_slot!(game, Maw[maw].function) {
        function::NEUTRAL => neutral(game),
        function::ATTACK => attack(game),
        function::ATTACKING => attacking(game),
        function::COOLDOWN => cooldown(game),
        _ => return false,
    }
    true
}

pub fn update_segments<L: Host>(game: &mut L) {
    for part in (0..4).rev() {
        game.update_maw_segment(part);
    }
}

pub fn touch<L: Host>(game: &mut L) {
    let maw = game.session().actors.controls.selected as i32;
    if session_slot!(game, Maw[maw].intangibility_timer) & SIGN != 0
        && session_slot!(game, Maw[maw].grabbing_samus) == 0
    {
        session_slot!(game, Maw[maw]; intangibility_timer = 0, grabbing_samus = 1);
        game.session_mut().player.transitions.input_mode = InputMode::from_raw(GRABBED_INPUT);
    }
}

pub fn appearance<L: Host>(game: &mut L) {
    let maw = game.session().actors.controls.selected as i32;
    let enemy = game.session().actors.controls.selected;
    let appearance = session_slot!(game, Maw[maw].initial_palette);
    let handle = session_slot!(game, Maw[maw].sprite_object);
    game.maw_appearance(handle, appearance, enemy);
}

pub fn shot<L: Host>(game: &mut L) {
    game.shot_reaction(Shot::Finalize);
    let maw = game.session().actors.controls.selected as i32;
    if session_slot!(game, Maw[maw].base.health) != 0 {
        if session_slot!(game, Maw[maw].base.frozen_timer) != 0 {
            if game.session().player.transitions.input_mode.to_raw() != DEMO_INPUT {
                game.session_mut().player.transitions.input_mode =
                    InputMode::from_raw(NORMAL_INPUT);
            }
            session_slot!(game, Maw[maw].grabbing_samus = 0);
        }
    } else {
        for part in 0..4 {
            let projectile = session_slot!(game, Maw[maw + 2 * part].segment_projectile);
            game.free_projectile(projectile);
        }
        let handle = session_slot!(game, Maw[maw].sprite_object);
        game.retire_sprite(handle);
        if game.session().player.transitions.input_mode.to_raw() != DEMO_INPUT {
            game.session_mut().player.transitions.input_mode = InputMode::from_raw(NORMAL_INPUT);
        }
        session_slot!(
            game,
            Maw[game.session().actors.controls.selected as i32].grabbing_samus = 0
        );
    }
}

pub fn frozen<L: Host>(game: &mut L) {
    game.update_frozen_enemy();
    let maw = game.session().actors.controls.selected as i32;
    for part in 0..4 {
        let handle = session_slot!(game, Maw[maw + 2 * part].segment_projectile);
        game.update_maw_palette(handle);
    }
    appearance(game);
}

pub fn instruction<L: Host>(address: u32, script: EnemyScript, game: &mut L) -> EnemyScript {
    use instruction_id::*;
    if address == QUEUE_SOUND_IF_ON_SCREEN {
        if session_slot!(
            game,
            Maw[game.session().actors.controls.selected as i32].off_screen
        ) == 0
        {
            game.queue_sound(1, BITE_SOUND, 6);
        }
    } else {
        let maw = game.session().actors.controls.selected as i32;
        let table = match address {
            OFFSET_SAMUS_UP_RIGHT => 4,
            OFFSET_SAMUS_UP_LEFT => 28,
            OFFSET_SAMUS_DOWN_RIGHT => 12,
            OFFSET_SAMUS_DOWN_LEFT => 20,
            OFFSET_SAMUS_UP => 0,
            OFFSET_SAMUS_DOWN => 16,
            _ => unreachable!("Yapping Maw instruction {address:06x}"),
        };
        session_slot!(
            game,
            Maw[maw].samus_x_offset = game.attachment_offset(false, false, (table / 2) as isize)
        );
        session_slot!(
            game,
            Maw[maw].samus_y_offset = game.attachment_offset(true, false, (table / 2) as isize)
        );
    }
    script
}
