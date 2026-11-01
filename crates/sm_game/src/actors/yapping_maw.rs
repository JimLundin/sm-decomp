//! Yapping Maw, the pincer worm that snaps Samus out of the air (bank
//! $A8:9F4F-$A8:A8EE): reach, articulated body, grip and attachments.
//! RAM words remain authoritative; operand writes run immediately through the caller.
use crate::actors::boss_lease::BossLease;
use crate::actors::host::{Shot, abs16};
use crate::actors::layouts::part_view;
use crate::actors::session_slots::session_slot;
use crate::cartridge::rom_table_word;
use crate::enemy_script::EnemyScript;
use crate::geometry::Point;
use crate::ram::word;
use sm_data::enemy_speeds::SpeedTable;

/// A lease on a game with Yapping Maw's effects.
pub trait Lease: BossLease<Game: Host> {}
impl<T: BossLease<Game: Host> + ?Sized> Lease for T {}

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
            var_17, var_18, var_19, var_1a, var_1b, var_1c, var_1d, var_1e, var_1f,
        ],
        pad2: [u8; 1984],
        words: [
            var_20, segment_angle, var_22, var_23, var_24, var_25, var_26, segment_x_velocity,
            segment_y_velocity, var_29, var_2a, var_2b, var_2c, var_2d, var_2e, aiming_down,
            grabbing_samus, var_31, samus_x_offset, samus_y_offset, instruction_list_index,
            intangibility_timer, off_screen, var_37, var_38, var_39, var_3a, var_3b, var_3c, var_3d,
            var_3e, var_3f,
        ],
        pad3: [u8; 1984],
        words: [
            segment_projectile, var_41, var_42, var_43, segment_index, initial_palette,
            sprite_object, sprite_appearance,
        ],
        _: [u16; 24],
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
/// $A8:A0A7: Samus's offsets when grabbed, by direction.
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
/// The yapping maw's bite, sound 2Fh, library 2.
const BITE_SOUND: u8 = 0x2f;
const QUARTER_TURN: u8 = 0x40;
const SIGN: u16 = 0x8000;

/// What Yapping Maw needs beyond the shared actor operations.
pub trait Host:
    crate::actors::boss_lease::BossLease<Game = Self>
    + crate::actors::host::ActorHost
    + crate::runtime_access::RuntimeAccess
{
    /// Create a sprite object of `kind` at (`x`, `y`); its handle.
    fn create_sprite(&mut self, x: u16, y: u16, kind: u16, appearance: u16) -> u16;
    fn maw_appearance(&mut self, handle: u16, appearance: u16, enemy: u16);
    fn retire_sprite(&mut self, handle: u16);
    /// (distance, direction) from `origin` to `target`.
    fn distance_and_direction(&mut self, origin: Point, target: Point) -> (u16, u16);
    /// Run Samus command `command`.
    fn command_player(&mut self, command: u16);
    fn update_maw_visibility(&mut self, actor: i32);
    fn update_maw_segment(&mut self, part: u8);
    fn update_maw_palette(&mut self, handle: u16);
    fn update_frozen_enemy(&mut self);
}

/// Initialisation AI - Yapping Maw ($A8:A148).
pub fn initialize<L: Lease>(game: &mut L) {
    let maw = game.current_enemy() as i32;
    let sprite_owner = game.current_enemy() as i32;
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
        game.spawn_projectile(part as u16, game.current_enemy(), BODY_PROJECTILE);
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
    let sprite = game.foreign(|game| game.create_sprite(x, y, sprite_kind, appearance));
    session_slot!(game, Maw[sprite_owner].sprite_object = sprite);
}
/// Yapping maw function - neutral ($A8:A235): wait for Samus in reach.
fn neutral<L: Lease>(game: &mut L) {
    let maw = game.current_enemy() as i32;
    let origin = Point {
        x: session_slot!(game, Maw[maw].base.x_pos),
        y: session_slot!(game, Maw[maw].base.y_pos),
    };
    let target = Point {
        x: game.player_x(),
        y: game.player_y(),
    };
    let (pair_distance, pair_direction) =
        game.foreign(|game| game.distance_and_direction(origin, target));
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
fn project<L: Lease>(
    angle: u16,
    distance: u16,
    write_operand: &mut impl FnMut(u8, &mut L),
    game: &mut L,
) -> u16 {
    let sine = crate::gameplay_tables::enemy_sine(game, -(angle as i32) as u8 as isize) as i16;
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
        ((2 * ((-(product as i32) as u16 as i32) >> 8)) | 0xff00) as u16
    } else {
        2 * (product >> 8)
    }
}
/// Yapping maw function - attack ($A8:A28C), calculating the origin
/// ($A8:A310). Was `start_reach`.
fn attack<L: Lease>(write_operand: &mut impl FnMut(u8, &mut L), game: &mut L) {
    crate::probe::hit("yapping_maw.attack");
    let maw = game.current_enemy() as i32;
    session_slot!(game, Maw[maw]; angle = 0, subangle = 0, angular_speed_index = 0);
    session_slot!(
        game,
        Maw[maw].length = session_slot!(game, Maw[maw].target_length) >> 1
    );
    let direction =
        QUARTER_TURN.wrapping_sub(session_slot!(game, Maw[maw].target_angle) as u8) as u16;
    session_slot!(game, Maw[maw]; maths_target_angle = direction, aiming_down = u16::from(direction >= STRAIGHT));
    let origin = game.current_enemy() as i32;
    let length = session_slot!(game, Maw[origin].length);
    let x = project(64, length, write_operand, game);
    session_slot!(game, Maw[origin].origin_x = x);
    let length = session_slot!(game, Maw[origin].length) >> 1;
    let y = project(128, length, write_operand, game);
    session_slot!(game, Maw[origin].origin_y = y);
    let direction =
        2 * ((session_slot!(game, Maw[maw].target_angle) as u8).wrapping_add(16) as u16 >> 5);
    session_slot!(game, Maw[maw].instruction_list_index = direction);
    session_slot!(game, Maw[maw].base; current_instruction = crate::actor_tables::word(game, |t| &t.maw_attack_lists, (direction >> 1) as isize), instruction_timer = 1, timer = 0);
    // These offsets use the native binding; animation above uses the loaded image.
    let x = game.native_image().table_word(
        sm_data::cartridge::lorom_offset(SAMUS_X_OFFSETS) as usize,
        direction as isize,
    );
    session_slot!(game, Maw[maw].samus_x_offset = x);
    let y = game.native_image().table_word(
        sm_data::cartridge::lorom_offset(SAMUS_Y_OFFSETS) as usize,
        direction as isize,
    );
    session_slot!(game, Maw[maw]; samus_y_offset = y, function = function::ATTACKING);
}
fn reorient_segment<L: Lease>(part: i32, write_operand: &mut impl FnMut(u8, &mut L), game: &mut L) {
    let maw = game.current_enemy() as i32;
    let distance = session_slot!(game, Maw[maw + 4 * part].segment_x_offset);
    let x = project(0u16.wrapping_sub(64), distance, write_operand, game);
    session_slot!(game, Maw[maw].scratch_x = x);
    let y = project(0, distance, write_operand, game);
    session_slot!(game, Maw[maw].scratch_y = y);
    let angle = session_slot!(game, Maw[maw].maths_target_angle).wrapping_sub(64);
    let x = project(angle, distance, write_operand, game);
    session_slot!(
        game,
        Maw[maw + 4 * part].segment_x_velocity =
            x.wrapping_sub(session_slot!(game, Maw[maw].scratch_x))
    );
    let angle = session_slot!(game, Maw[maw].maths_target_angle);
    let y = project(angle, distance, write_operand, game);
    session_slot!(
        game,
        Maw[maw + 4 * part].segment_y_velocity =
            y.wrapping_sub(session_slot!(game, Maw[maw].scratch_y))
    );
}
/// Move Samus with yapping maw pincers ($A8:A665). Was `carry_player`.
fn move_samus_with_pincers<L: Lease>(game: &mut L) {
    game.foreign(|game| game.command_player(LOCK_SAMUS));
    let maw = game.current_enemy() as i32;
    game.set_player_x(
        session_slot!(game, Maw[maw].samus_x_offset)
            .wrapping_add(session_slot!(game, Maw[maw].base.x_pos)),
    );
    game.set_player_y(
        session_slot!(game, Maw[maw].samus_y_offset)
            .wrapping_add(session_slot!(game, Maw[maw].base.y_pos)),
    );
    crate::player_position::correct_previous_position(game);
}
/// Yapping maw function - attacking ($A8:A445), updating the angular speed
/// ($A8:A63E). Was `stretch`.
fn attacking<L: Lease>(write_operand: &mut impl FnMut(u8, &mut L), game: &mut L) {
    let maw = game.current_enemy() as i32;
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
        let x = project(angle, radius, write_operand, game);
        session_slot!(
            game,
            Maw[maw + 4 * part].segment_x_offset =
                x.wrapping_sub(session_slot!(game, Maw[maw].origin_x))
        );
    }
    let radius = session_slot!(game, Maw[maw].length) >> 1;
    for part in 0..4 {
        let angle = session_slot!(game, Maw[maw + 2 * part].segment_angle);
        let y = project(angle, radius, write_operand, game);
        session_slot!(
            game,
            Maw[maw + 4 * part].segment_y_offset =
                y.wrapping_sub(session_slot!(game, Maw[maw].origin_y))
        );
    }
    for part in 0..4 {
        reorient_segment(part, write_operand, game);
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
    let velocity_owner = game.current_enemy() as i32;
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
/// Yapping maw function - cooldown ($A8:A68A). Was `release_player`.
fn cooldown<L: Lease>(game: &mut L) {
    let maw = game.current_enemy() as i32;
    if session_slot!(game, Maw[maw].grabbing_samus) != 0 {
        move_samus_with_pincers(game);
    }
    session_slot!(game, Maw[maw].cooldown_timer, |old| old.wrapping_sub(1));
    if session_slot!(game, Maw[maw].cooldown_timer) & SIGN != 0
        && word!(game.SAMUS_INPUT_HANDLER) != DEMO_INPUT
    {
        word!(game.SAMUS_INPUT_HANDLER = NORMAL_INPUT);
        session_slot!(game, Maw[maw]; grabbing_samus = 0, intangibility_timer = INTANGIBLE_FRAMES,
            cooldown_timer = COOLDOWN_FRAMES, function = function::NEUTRAL);
    }
}
/// Main AI - Yapping Maw ($A8:A211): run its function. False selects the
/// existing runtime's unknown-address failure before attachments update. Was
/// `advance`.
pub fn main_ai<L: Lease>(game: &mut L, mut write_operand: impl FnMut(u8, &mut L)) -> bool {
    crate::probe::hit("yapping_maw.main_ai");
    let maw = game.current_enemy() as i32;
    session_slot!(game, Maw[maw].intangibility_timer, |old| old
        .wrapping_sub(1));
    game.foreign(|game| game.update_maw_visibility(maw));
    match session_slot!(game, Maw[maw].function) {
        function::NEUTRAL => neutral(game),
        function::ATTACK => attack(&mut write_operand, game),
        function::ATTACKING => attacking(&mut write_operand, game),
        function::COOLDOWN => cooldown(game),
        _ => return false,
    }
    true
}
pub fn update_segments<L: Lease>(game: &mut L) {
    for part in (0..4).rev() {
        game.foreign(|game| game.update_maw_segment(part));
    }
}
/// Enemy touch - Yapping Maw ($A8:A799): grab Samus.
pub fn touch<L: Lease>(game: &mut L) {
    let maw = game.current_enemy() as i32;
    if session_slot!(game, Maw[maw].intangibility_timer) & SIGN != 0
        && session_slot!(game, Maw[maw].grabbing_samus) == 0
    {
        session_slot!(game, Maw[maw]; intangibility_timer = 0, grabbing_samus = 1);
        word!(game.SAMUS_INPUT_HANDLER = GRABBED_INPUT);
    }
}
pub fn appearance<L: Lease>(game: &mut L) {
    let maw = game.current_enemy() as i32;
    let enemy = game.current_enemy();
    let appearance = session_slot!(game, Maw[maw].initial_palette);
    let handle = session_slot!(game, Maw[maw].sprite_object);
    game.foreign(|game| game.maw_appearance(handle, appearance, enemy));
}
/// Enemy shot - Yapping Maw ($A8:A7BD).
pub fn shot<L: Lease>(game: &mut L) {
    game.shot_reaction(Shot::Finalize);
    let maw = game.current_enemy() as i32;
    if session_slot!(game, Maw[maw].base.health) != 0 {
        if session_slot!(game, Maw[maw].base.frozen_timer) != 0 {
            if word!(game.SAMUS_INPUT_HANDLER) != DEMO_INPUT {
                word!(game.SAMUS_INPUT_HANDLER = NORMAL_INPUT);
            }
            session_slot!(game, Maw[maw].grabbing_samus = 0);
        }
    } else {
        for part in 0..4 {
            let projectile = session_slot!(game, Maw[maw + 2 * part].segment_projectile);
            game.foreign(|game| crate::enemy_projectiles::table::free(projectile, game));
        }
        let handle = session_slot!(game, Maw[maw].sprite_object);
        game.foreign(|game| game.retire_sprite(handle));
        if word!(game.SAMUS_INPUT_HANDLER) != DEMO_INPUT {
            word!(game.SAMUS_INPUT_HANDLER = NORMAL_INPUT);
        }
        session_slot!(game, Maw[game.current_enemy() as i32].grabbing_samus = 0);
    }
}
/// Frozen AI - Yapping Maw ($A8:A835).
pub fn frozen<L: Lease>(game: &mut L) {
    game.foreign(|game| game.update_frozen_enemy());
    let maw = game.current_enemy() as i32;
    for part in 0..4 {
        let handle = session_slot!(game, Maw[maw + 2 * part].segment_projectile);
        game.foreign(|game| game.update_maw_palette(handle));
    }
    appearance(game);
}
/// Run one of Yapping Maw's script instructions (one of [`instruction`]).
pub fn instruction<L: Lease>(address: u32, script: EnemyScript, game: &mut L) -> EnemyScript {
    use instruction::*;
    if address == QUEUE_SOUND_IF_ON_SCREEN {
        if session_slot!(game, Maw[game.current_enemy() as i32].off_screen) == 0 {
            game.queue_sound(1, BITE_SOUND, 6);
        }
    } else {
        let maw = game.current_enemy() as i32;
        let table = match address {
            OFFSET_SAMUS_UP_RIGHT => SAMUS_X_OFFSETS + 4,
            OFFSET_SAMUS_UP_LEFT => SAMUS_X_OFFSETS + 28,
            OFFSET_SAMUS_DOWN_RIGHT => SAMUS_X_OFFSETS + 12,
            OFFSET_SAMUS_DOWN_LEFT => SAMUS_X_OFFSETS + 20,
            OFFSET_SAMUS_UP => SAMUS_X_OFFSETS,
            OFFSET_SAMUS_DOWN => SAMUS_X_OFFSETS + 16,
            _ => unreachable!("Yapping Maw instruction {address:06x}"),
        };
        session_slot!(
            game,
            Maw[maw].samus_x_offset = rom_table_word!(game, table, 0)
        );
        session_slot!(
            game,
            Maw[maw].samus_y_offset = rom_table_word!(game, table + 2, 0)
        );
    }
    script
}
