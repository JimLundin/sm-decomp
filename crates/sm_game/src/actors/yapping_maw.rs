//! Yapping Maw, the pincer worm that snaps Samus out of the air (bank
//! $A8:9F4F-$A8:A8EE): reach, articulated body, grip and attachments.
//! RAM words remain authoritative; operand writes run immediately through the caller.
#![forbid(unsafe_code)]
use crate::actors::host::Shot;
use crate::actors::layouts::part_view;
use crate::cartridge::rom_table_word;
use crate::enemy_script::EnemyScript;
use crate::geometry::Point;
use crate::ram::RawRam as _;
use crate::ram::{slot, word};
use sm_data::enemy_speeds::SpeedTable;

part_view! {
    /// Yapping Maw's view of its slot. Its body segments' words are arrays
    /// reached by slot offset: `segment_x_offset` and `segment_y_offset` at
    /// 4 bytes a segment (segment 1 first, the head last), `segment_angle` and
    /// `segment_projectile` at 2, the velocities at 4.
    Maw: EnemyLayout104 {
        words: [
            function, angle, subangle, angular_speed_index, cooldown_timer, proximity, parameter_1,
            parameter_2,
        ],
        pad1: [u8; 26696],
        words: [
            segment_x_offset, segment_y_offset, var_02, var_03, var_04, var_05, head_x_offset,
            head_y_offset, target_length, length, target_angle, maths_target_angle, base_x, base_y,
            origin_x, origin_y, scratch_x, scratch_y, var_12, var_13, var_14, var_15, var_16,
            var_17, var_18, var_19, var_1A, var_1B, var_1C, var_1D, var_1E, var_1F,
        ],
        pad2: [u8; 1984],
        words: [
            var_20, segment_angle, var_22, var_23, var_24, var_25, var_26, segment_x_velocity,
            segment_y_velocity, var_29, var_2A, var_2B, var_2C, var_2D, var_2E, aiming_down,
            grabbing_samus, var_31, samus_x_offset, samus_y_offset, instruction_list_index,
            intangibility_timer, off_screen, var_37, var_38, var_39, var_3A, var_3B, var_3C, var_3D,
            var_3E, var_3F,
        ],
        pad3: [u8; 1984],
        words: [
            segment_projectile, var_41, var_42, var_43, segment_index, initial_palette,
            sprite_object, sprite_appearance, var_48, var_49, var_4A, var_4B, var_4C, var_4D,
            var_4E, var_4F, var_50, var_51, var_52, var_53, var_54, var_55, var_56, var_57, var_58,
            var_59, var_5A, var_5B, var_5C, var_5D, var_5E, var_5F,
        ],
    }
}

/// Bank $A8 Yapping Maw functions, stored in `function`.
pub mod function {
    pub const NEUTRAL: u16 = 0xa235;
    pub const ATTACK: u16 = 0xa28c;
    pub const ATTACKING: u16 = 0xa445;
    pub const COOLDOWN: u16 = 0xa68a;
}

/// Bank $A8 Yapping Maw instruction lists.
pub mod instruction_list {
    pub const INITIAL_FACING_UP: u16 = 0x9f6f;
    pub const INITIAL_FACING_DOWN: u16 = 0x9fc7;
    pub const COOLDOWN_UP_RIGHT: u16 = 0xa01f;
    pub const COOLDOWN_UP: u16 = 0xa025;
    pub const COOLDOWN_UP_LEFT: u16 = 0xa03d;
    pub const COOLDOWN_DOWN_RIGHT: u16 = 0xa05b;
    pub const COOLDOWN_DOWN: u16 = 0xa061;
    pub const COOLDOWN_DOWN_LEFT: u16 = 0xa079;
}

/// Bank $A8 instructions Yapping Maw's scripts call.
pub mod instruction {
    pub const OFFSET_SAMUS_UP_RIGHT: u32 = 0xa8_a0c7;
    pub const OFFSET_SAMUS_UP_LEFT: u32 = 0xa8_a0d9;
    pub const OFFSET_SAMUS_DOWN_RIGHT: u32 = 0xa8_a0eb;
    pub const OFFSET_SAMUS_DOWN_LEFT: u32 = 0xa8_a0fd;
    pub const OFFSET_SAMUS_UP: u32 = 0xa8_a10f;
    pub const OFFSET_SAMUS_DOWN: u32 = 0xa8_a121;
    pub const QUEUE_SOUND_IF_ON_SCREEN: u32 = 0xa8_a133;
}

/// The yapping maw body projectile ($86:EC95), one per segment, 4 segments.
const BODY_PROJECTILE: u16 = 0xec95;
const LAST_SEGMENT: u16 = 3;
/// The base sprite object, 8 px below (facing up, 39h) or above (facing down, 38h).
const BASE_FACING_UP: u16 = 0x39;
const BASE_FACING_DOWN: u16 = 0x38;
const BASE_OFFSET: u16 = 8;
const PALETTE_BITS: u16 = 0xe00;
const COOLDOWN_FRAMES: u16 = 0x40;
/// Samus is in reach within 20h px, and the maw stretches at most 40h.
const REACH: u16 = 0x20;
const MAX_LENGTH: u16 = 0x40;
const INTANGIBLE_FRAMES: u16 = 0x30;
/// $A0:B1C3: the sine table; $A8:A097: the attack instruction lists, and
/// $A8:A0A7: Samus's offsets when grabbed, by direction.
const SINE: u32 = 0xa0_b1c3;
const ATTACK_INSTRUCTION_LISTS: u32 = 0xa8_a097;
const SAMUS_X_OFFSETS: u32 = 0xa8_a0a7;
const SAMUS_Y_OFFSETS: u32 = 0xa8_a0a9;
/// `instruction_list_index` values for the diagonals: up-right and up-left (or
/// down-right and down-left facing down).
const RIGHT: u16 = 4;
const LEFT: u16 = 12;
/// The bend straightens out at angle 80h.
const STRAIGHT: u16 = 0x80;
const SPEED_STEP: u16 = 8;
const NEGATE_SPEED: u16 = 4;
/// Samus pose input handlers: grabbed ($E90E), normal ($E913) and demo ($E91D).
const GRABBED_INPUT: u16 = 0xe90e;
const NORMAL_INPUT: u16 = 0xe913;
const DEMO_INPUT: u16 = 0xe91d;
/// Samus command 3: locked in place.
const LOCK_SAMUS: u16 = 3;
/// $7E:1997: the enemy projectile IDs.
const PROJECTILE_IDS: usize = 0x1997;
/// The yapping maw's bite, sound 2Fh, library 2.
const BITE_SOUND: u8 = 0x2f;
const QUARTER_TURN: u8 = 0x40;
const SIGN: u16 = 0x8000;

/// What Yapping Maw needs beyond the shared actor operations.
pub trait Host:
    crate::actors::host::ActorHost + crate::actors::host::ActorRuntime + crate::effects::SoundEffects
{
    /// Create a sprite object of `kind` at (`x`, `y`); its handle.
    fn create_sprite(&mut self, x: u16, y: u16, kind: u16, appearance: u16) -> u16;
    fn maw_appearance(&mut self, handle: u16, appearance: u16, enemy: u16);
    fn retire_sprite(&mut self, handle: u16);
    /// (distance, direction) from `origin` to `target`.
    fn distance_and_direction(&mut self, origin: Point, target: Point) -> (u16, u16);
    /// Run Samus command `command`.
    fn command_player(&mut self, command: u16);
    fn correct_previous_position(&mut self);
    fn update_maw_visibility(&mut self, actor: i32);
    fn update_maw_segment(&mut self, part: u8);
    fn update_maw_palette(&mut self, handle: u16);
    fn update_frozen_enemy(&mut self);
}

fn abs16(value: u16) -> u16 {
    (value as i16).unsigned_abs()
}

/// Initialisation AI - Yapping Maw ($A8:A148).
pub fn initialize<H: Host + ?Sized>(game: &mut H) {
    let maw = word!(game.cur_enemy_index) as i32;
    let sprite_owner = word!(game.cur_enemy_index) as i32;
    slot!(game, Maw[maw].base_x = slot!(game, Maw[maw].base.x_pos));
    slot!(game, Maw[maw].base_y = slot!(game, Maw[maw].base.y_pos));
    for field in 0..8 {
        slot!(game, Maw[maw + 2 * field].segment_x_offset = 0);
    }
    slot!(game, Maw[maw]; grabbing_samus = 0, proximity = slot!(game, Maw[maw].parameter_1), cooldown_timer = COOLDOWN_FRAMES);
    slot!(
        game,
        Maw[maw].base.current_instruction = instruction_list::INITIAL_FACING_UP
    );
    let (mut sprite_kind, mut displacement) = (BASE_FACING_UP, BASE_OFFSET);
    if slot!(game, Maw[maw].parameter_2) == 0 {
        slot!(
            game,
            Maw[maw].base.current_instruction = instruction_list::INITIAL_FACING_DOWN
        );
        sprite_kind = BASE_FACING_DOWN;
        displacement = BASE_OFFSET.wrapping_neg();
    }
    slot!(game, Maw[maw].base; instruction_timer = 1, timer = 0);
    slot!(game, Maw[maw]; function = function::NEUTRAL,
        initial_palette = slot!(game, Maw[maw].base.palette_index) & PALETTE_BITS);
    let mut part = LAST_SEGMENT as i16;
    slot!(game, Maw[maw].segment_index = LAST_SEGMENT);
    loop {
        game.spawn_projectile(part as u16, word!(game.cur_enemy_index), BODY_PROJECTILE);
        part = (slot!(game, Maw[maw].segment_index) as i16).wrapping_sub(1);
        slot!(game, Maw[maw].segment_index = part as u16);
        if part < 0 {
            break;
        }
    }
    let appearance =
        slot!(game, Maw[maw].base.vram_tiles_index) | slot!(game, Maw[maw].base.palette_index);
    slot!(game, Maw[maw].sprite_appearance = appearance);
    let x = slot!(game, Maw[maw].base.x_pos);
    let y = displacement.wrapping_add(slot!(game, Maw[maw].base.y_pos));
    let sprite = game.create_sprite(x, y, sprite_kind, appearance);
    slot!(game, Maw[sprite_owner].sprite_object = sprite);
}
/// Yapping maw function - neutral ($A8:A235): wait for Samus in reach.
fn neutral<H: Host + ?Sized>(game: &mut H) {
    let maw = word!(game.cur_enemy_index) as i32;
    let origin = Point {
        x: slot!(game, Maw[maw].base.x_pos),
        y: slot!(game, Maw[maw].base.y_pos),
    };
    let target = Point {
        x: game.player_x(),
        y: game.player_y(),
    };
    let (pair_distance, pair_direction) = game.distance_and_direction(origin, target);
    let distance = abs16(pair_distance);
    slot!(game, Maw[maw].target_length = distance);
    if distance.wrapping_sub(REACH) & SIGN != 0 {
        slot!(game, Maw[maw].intangibility_timer = INTANGIBLE_FRAMES);
    } else if (distance as i16).wrapping_sub(slot!(game, Maw[maw].proximity) as i16) < 0 {
        if slot!(game, Maw[maw].target_length).wrapping_sub(MAX_LENGTH) & SIGN == 0 {
            slot!(game, Maw[maw].target_length = MAX_LENGTH);
        }
        slot!(game, Maw[maw]; target_angle = pair_direction, function = function::ATTACK);
    }
}
fn project<H: Host + ?Sized>(
    angle: u16,
    distance: u16,
    write_operand: &mut impl FnMut(u8, &mut H),
    game: &mut H,
) -> u16 {
    let sine = rom_table_word!(game, SINE, -(angle as i32) as u8 as isize) as i16;
    let magnitude = if sine < 0 {
        -(sine as i32) as i16
    } else {
        sine
    };
    // The operand is physical byte one of the native signed word.
    let operand = magnitude.to_ne_bytes()[1];
    write_operand(operand, game);
    if distance as u8 == 0 {
        return 0;
    }
    let product = u16::from(operand) * u16::from(distance as u8);
    if sine < 0 {
        (2 * ((-(product as i32) as u16 as i32) >> 8) | 0xff00) as u16
    } else {
        2 * (product >> 8)
    }
}
/// Yapping maw function - attack ($A8:A28C), calculating the origin
/// ($A8:A310). Was `start_reach`.
fn attack<H: Host + ?Sized>(write_operand: &mut impl FnMut(u8, &mut H), game: &mut H) {
    let maw = word!(game.cur_enemy_index) as i32;
    slot!(game, Maw[maw]; angle = 0, subangle = 0, angular_speed_index = 0);
    slot!(
        game,
        Maw[maw].length = slot!(game, Maw[maw].target_length) >> 1
    );
    let direction = QUARTER_TURN.wrapping_sub(slot!(game, Maw[maw].target_angle) as u8) as u16;
    slot!(game, Maw[maw]; maths_target_angle = direction, aiming_down = u16::from(direction >= STRAIGHT));
    let origin = word!(game.cur_enemy_index) as i32;
    let x = project(64, slot!(game, Maw[origin].length), write_operand, game);
    slot!(game, Maw[origin].origin_x = x);
    let y = project(
        128,
        slot!(game, Maw[origin].length) >> 1,
        write_operand,
        game,
    );
    slot!(game, Maw[origin].origin_y = y);
    let direction = 2 * ((slot!(game, Maw[maw].target_angle) as u8).wrapping_add(16) as u16 >> 5);
    slot!(game, Maw[maw].instruction_list_index = direction);
    slot!(game, Maw[maw].base; current_instruction = rom_table_word!(game, ATTACK_INSTRUCTION_LISTS, (direction >> 1) as isize), instruction_timer = 1, timer = 0);
    // These offsets use the native binding; animation above uses the loaded image.
    let x = game.native_image().table_word(
        sm_data::cartridge::lorom_offset(SAMUS_X_OFFSETS) as usize,
        direction as isize,
    );
    slot!(game, Maw[maw].samus_x_offset = x);
    let y = game.native_image().table_word(
        sm_data::cartridge::lorom_offset(SAMUS_Y_OFFSETS) as usize,
        direction as isize,
    );
    slot!(game, Maw[maw].samus_y_offset = y);
    slot!(game, Maw[maw].function = function::ATTACKING);
}
fn reorient_segment<H: Host + ?Sized>(
    part: i32,
    write_operand: &mut impl FnMut(u8, &mut H),
    game: &mut H,
) {
    let maw = word!(game.cur_enemy_index) as i32;
    let distance = slot!(game, Maw[maw + 4 * part].segment_x_offset);
    let x = project(0u16.wrapping_sub(64), distance, write_operand, game);
    slot!(game, Maw[maw].scratch_x = x);
    let y = project(0, distance, write_operand, game);
    slot!(game, Maw[maw].scratch_y = y);
    let angle = slot!(game, Maw[maw].maths_target_angle).wrapping_sub(64);
    let x = project(angle, distance, write_operand, game);
    slot!(
        game,
        Maw[maw + 4 * part].segment_x_velocity = x.wrapping_sub(slot!(game, Maw[maw].scratch_x))
    );
    let angle = slot!(game, Maw[maw].maths_target_angle);
    let y = project(angle, distance, write_operand, game);
    slot!(
        game,
        Maw[maw + 4 * part].segment_y_velocity = y.wrapping_sub(slot!(game, Maw[maw].scratch_y))
    );
}
/// Move Samus with yapping maw pincers ($A8:A665). Was `carry_player`.
fn move_samus_with_pincers<H: Host + ?Sized>(game: &mut H) {
    game.command_player(LOCK_SAMUS);
    let maw = word!(game.cur_enemy_index) as i32;
    game.set_player_x(
        slot!(game, Maw[maw].samus_x_offset).wrapping_add(slot!(game, Maw[maw].base.x_pos)),
    );
    game.set_player_y(
        slot!(game, Maw[maw].samus_y_offset).wrapping_add(slot!(game, Maw[maw].base.y_pos)),
    );
    game.correct_previous_position();
}
/// Yapping maw function - attacking ($A8:A445), updating the angular speed
/// ($A8:A63E). Was `stretch`.
fn attacking<H: Host + ?Sized>(write_operand: &mut impl FnMut(u8, &mut H), game: &mut H) {
    let maw = word!(game.cur_enemy_index) as i32;
    let step = slot!(game, Maw[maw].angle) >> 2;
    let positive = slot!(game, Maw[maw].aiming_down) != 0;
    for part in 0..4 {
        let angle = if positive {
            STRAIGHT.wrapping_add(step.wrapping_mul(part as u16 + 1))
        } else {
            STRAIGHT.wrapping_sub(step.wrapping_mul(part as u16 + 1))
        };
        slot!(game, Maw[maw + 2 * part].segment_angle = angle);
    }
    let radius = slot!(game, Maw[maw].length);
    for part in 0..4 {
        let angle = slot!(game, Maw[maw + 2 * part].segment_angle).wrapping_sub(64);
        let x = project(angle, radius, write_operand, game);
        slot!(
            game,
            Maw[maw + 4 * part].segment_x_offset = x.wrapping_sub(slot!(game, Maw[maw].origin_x))
        );
    }
    let radius = slot!(game, Maw[maw].length) >> 1;
    for part in 0..4 {
        let angle = slot!(game, Maw[maw + 2 * part].segment_angle);
        let y = project(angle, radius, write_operand, game);
        slot!(
            game,
            Maw[maw + 4 * part].segment_y_offset = y.wrapping_sub(slot!(game, Maw[maw].origin_y))
        );
    }
    for part in 0..4 {
        reorient_segment(part, write_operand, game);
    }
    for field in 0..8 {
        slot!(game, Maw[maw + 2 * field].segment_x_offset, |old| old
            .wrapping_add(slot!(
                game,
                Maw[maw + 2 * field].segment_x_velocity
            )));
    }
    slot!(
        game,
        Maw[maw].base.x_pos =
            slot!(game, Maw[maw].head_x_offset).wrapping_add(slot!(game, Maw[maw].base_x))
    );
    slot!(
        game,
        Maw[maw].base.y_pos =
            slot!(game, Maw[maw].head_y_offset).wrapping_add(slot!(game, Maw[maw].base_y))
    );
    let velocity_owner = word!(game.cur_enemy_index) as i32;
    let speed = slot!(game, Maw[velocity_owner].angular_speed_index) as isize >> 1;
    slot!(
        game,
        Maw[velocity_owner](angle, subangle) += {
            let high = game.enemy_speed(SpeedTable::Quadratic, speed + 1);
            let low = game.enemy_speed(SpeedTable::Quadratic, speed);
            (u32::from(high) << 16) | u32::from(low)
        }
    );
    slot!(game, Maw[velocity_owner].angular_speed_index, |old| old
        .wrapping_add(SPEED_STEP));
    if slot!(game, Maw[maw].angle) & SIGN == 0 {
        if slot!(game, Maw[maw].angle).wrapping_sub(STRAIGHT) & SIGN == 0 {
            slot!(game, Maw[maw]; angle = STRAIGHT, subangle = 0);
            slot!(game, Maw[maw].angular_speed_index, |old| old
                .wrapping_add(NEGATE_SPEED));
        }
    } else {
        slot!(game, Maw[maw]; function = function::COOLDOWN, intangibility_timer = INTANGIBLE_FRAMES);
        let direction = slot!(game, Maw[maw].instruction_list_index);
        slot!(
            game,
            Maw[maw].base.current_instruction = if direction == RIGHT {
                instruction_list::COOLDOWN_UP_RIGHT
            } else if direction == LEFT {
                instruction_list::COOLDOWN_UP_LEFT
            } else {
                instruction_list::COOLDOWN_UP
            }
        );
        if slot!(game, Maw[maw].parameter_2) != 0 {
            return;
        }
        if slot!(game, Maw[maw].instruction_list_index) == RIGHT {
            slot!(
                game,
                Maw[maw].base.current_instruction = instruction_list::COOLDOWN_DOWN_RIGHT
            );
        } else if slot!(game, Maw[maw].instruction_list_index) == LEFT {
            slot!(
                game,
                Maw[maw].base.current_instruction = instruction_list::COOLDOWN_DOWN_LEFT
            );
        } else {
            slot!(
                game,
                Maw[maw].base.current_instruction = instruction_list::COOLDOWN_DOWN
            );
            return;
        }
    }
    if slot!(game, Maw[maw].grabbing_samus) != 0 {
        move_samus_with_pincers(game);
    }
}
/// Yapping maw function - cooldown ($A8:A68A). Was `release_player`.
fn cooldown<H: Host + ?Sized>(game: &mut H) {
    let maw = word!(game.cur_enemy_index) as i32;
    if slot!(game, Maw[maw].grabbing_samus) != 0 {
        move_samus_with_pincers(game);
    }
    slot!(game, Maw[maw].cooldown_timer, |old| old.wrapping_sub(1));
    if slot!(game, Maw[maw].cooldown_timer) & SIGN != 0
        && word!(game.samus_input_handler) != DEMO_INPUT
    {
        word!(game.samus_input_handler = NORMAL_INPUT);
        slot!(game, Maw[maw]; grabbing_samus = 0, intangibility_timer = INTANGIBLE_FRAMES,
            cooldown_timer = COOLDOWN_FRAMES, function = function::NEUTRAL);
    }
}
/// Main AI - Yapping Maw ($A8:A211): run its function. False selects the
/// existing runtime's unknown-address failure before attachments update. Was
/// `advance`.
pub fn main_ai<H: Host + ?Sized>(game: &mut H, mut write_operand: impl FnMut(u8, &mut H)) -> bool {
    let maw = word!(game.cur_enemy_index) as i32;
    slot!(game, Maw[maw].intangibility_timer, |old| old
        .wrapping_sub(1));
    game.update_maw_visibility(maw);
    match slot!(game, Maw[maw].function) {
        function::NEUTRAL => neutral(game),
        function::ATTACK => attack(&mut write_operand, game),
        function::ATTACKING => attacking(&mut write_operand, game),
        function::COOLDOWN => cooldown(game),
        _ => return false,
    }
    true
}
pub fn update_segments<H: Host + ?Sized>(game: &mut H) {
    for part in (0..4).rev() {
        game.update_maw_segment(part);
    }
}
/// Enemy touch - Yapping Maw ($A8:A799): grab Samus.
pub fn touch<H: Host + ?Sized>(game: &mut H) {
    let maw = word!(game.cur_enemy_index) as i32;
    if slot!(game, Maw[maw].intangibility_timer) & SIGN != 0
        && slot!(game, Maw[maw].grabbing_samus) == 0
    {
        slot!(game, Maw[maw]; intangibility_timer = 0, grabbing_samus = 1);
        word!(game.samus_input_handler = GRABBED_INPUT);
    }
}
pub fn appearance<H: Host + ?Sized>(game: &mut H) {
    let maw = word!(game.cur_enemy_index) as i32;
    let enemy = word!(game.cur_enemy_index);
    let appearance = slot!(game, Maw[maw].initial_palette);
    let handle = slot!(game, Maw[maw].sprite_object);
    game.maw_appearance(handle, appearance, enemy);
}
/// Enemy shot - Yapping Maw ($A8:A7BD).
pub fn shot<H: Host + ?Sized>(game: &mut H) {
    game.shot_reaction(Shot::Finalize);
    let maw = word!(game.cur_enemy_index) as i32;
    if slot!(game, Maw[maw].base.health) != 0 {
        if slot!(game, Maw[maw].base.frozen_timer) != 0 {
            if word!(game.samus_input_handler) != DEMO_INPUT {
                word!(game.samus_input_handler = NORMAL_INPUT);
            }
            slot!(game, Maw[maw].grabbing_samus = 0);
        }
    } else {
        for part in 0..4 {
            let at =
                PROJECTILE_IDS + usize::from(slot!(game, Maw[maw + 2 * part].segment_projectile));
            game.ram().copy_in(at, &0u16.to_ne_bytes());
        }
        let handle = slot!(game, Maw[maw].sprite_object);
        game.retire_sprite(handle);
        if word!(game.samus_input_handler) != DEMO_INPUT {
            word!(game.samus_input_handler = NORMAL_INPUT);
        }
        slot!(
            game,
            Maw[word!(game.cur_enemy_index) as i32].grabbing_samus = 0
        );
    }
}
/// Frozen AI - Yapping Maw ($A8:A835).
pub fn frozen<H: Host + ?Sized>(game: &mut H) {
    game.update_frozen_enemy();
    let maw = word!(game.cur_enemy_index) as i32;
    for part in 0..4 {
        let handle = slot!(game, Maw[maw + 2 * part].segment_projectile);
        game.update_maw_palette(handle);
    }
    appearance(game);
}
/// Run one of Yapping Maw's script instructions (one of [`instruction`]).
pub fn instruction<H: Host + ?Sized>(
    address: u32,
    script: EnemyScript,
    game: &mut H,
) -> EnemyScript {
    use instruction::*;
    if address == QUEUE_SOUND_IF_ON_SCREEN {
        if slot!(game, Maw[word!(game.cur_enemy_index) as i32].off_screen) == 0 {
            game.queue_sound(1, BITE_SOUND, 6);
        }
    } else {
        let maw = word!(game.cur_enemy_index) as i32;
        let table = match address {
            OFFSET_SAMUS_UP_RIGHT => SAMUS_X_OFFSETS + 4,
            OFFSET_SAMUS_UP_LEFT => SAMUS_X_OFFSETS + 28,
            OFFSET_SAMUS_DOWN_RIGHT => SAMUS_X_OFFSETS + 12,
            OFFSET_SAMUS_DOWN_LEFT => SAMUS_X_OFFSETS + 20,
            OFFSET_SAMUS_UP => SAMUS_X_OFFSETS,
            OFFSET_SAMUS_DOWN => SAMUS_X_OFFSETS + 16,
            _ => unreachable!("Yapping Maw instruction {address:06x}"),
        };
        slot!(
            game,
            Maw[maw].samus_x_offset = rom_table_word!(game, table, 0)
        );
        slot!(
            game,
            Maw[maw].samus_y_offset = rom_table_word!(game, table + 2, 0)
        );
    }
    script
}
