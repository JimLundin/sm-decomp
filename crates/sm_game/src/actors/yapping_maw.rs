//! Yapping Maw, the pincer worm that snaps Samus out of the air (bank
//! $A8:9F4F-$A8:A8EE): reach, articulated body, grip and attachments.
//! RAM words remain authoritative; operand writes run immediately through the caller.
use crate::actors::boss_lease::BossLease;
use crate::actors::host::{Shot, abs16};
use crate::actors::layouts::EnemyLayout104;
use crate::actors::slot_view::{SlotState, extended_var, slot_view};
use crate::cartridge::rom_table_word;
use crate::enemy_script::EnemyScript;
use crate::geometry::Point;
use crate::legacy_overflow;
use crate::player_transitions::state::InputMode;
use core::mem::offset_of;
use sm_data::enemy_speeds::SpeedTable;

/// A lease on a game with Yapping Maw's effects.
pub trait Lease: BossLease<Game: Host> + SlotState {}
impl<T: BossLease<Game: Host> + SlotState + ?Sized> Lease for T {}

slot_view! {
    /// Yapping Maw's slot: its function, bend angle (whole and sub), angular
    /// speed index, cooldown timer and grab proximity; its head's offsets, target
    /// and current length, target angle and the angle the maths uses, its base,
    /// origin and scratch positions; Samus's grabbed offsets, the instruction
    /// list index, intangibility timer and off-screen flag; the segment index,
    /// initial palette and the base's sprite object and appearance; whether it
    /// aims down and grabs Samus ($7E:801E and $7E:8020 in slot 0, Botwoon's
    /// death words there, which the view serves). The body segments' arrays are
    /// below.
    YappingMaw, YappingMawEdit {
        record: [
            function = var_a, angle = var_b, subangle = var_c, angular_speed_index = var_d,
            cooldown_timer = var_e, proximity = var_f, parameter_1 = parameter_1,
            parameter_2 = parameter_2,
        ],
        extended: [
            head_x_offset = var_06, head_y_offset = var_07, target_length = var_08,
            length = var_09, target_angle = var_0a, maths_target_angle = var_0b, base_x = var_0c,
            base_y = var_0d, origin_x = var_0e, origin_y = var_0f, scratch_x = var_10,
            scratch_y = var_11, samus_x_offset = var_32, samus_y_offset = var_33,
            aiming_down = var_2f, grabbing_samus = var_30, instruction_list_index = var_34,
            intangibility_timer = var_35, off_screen = var_36,
            segment_index = var_44, initial_palette = var_45, sprite_object = var_46,
            sprite_appearance = var_47,
        ],
    }
}

/// The extended words where the body segments' arrays start: each segment's
/// X and Y offsets (segment 1 first, the head's last, as `head_x_offset` and
/// `head_y_offset`), angles, X and Y velocities and projectiles.
const SEGMENT_OFFSETS: usize = extended_var(offset_of!(EnemyLayout104, var_00));
const SEGMENT_ANGLES: usize = extended_var(offset_of!(EnemyLayout104, var_21));
const SEGMENT_VELOCITIES: usize = extended_var(offset_of!(EnemyLayout104, var_27));
const SEGMENT_PROJECTILES: usize = extended_var(offset_of!(EnemyLayout104, var_40));

/// The X (`y` false) or Y offset word of `segment`.
fn axis_word(segment: usize, y: bool) -> usize {
    2 * segment + usize::from(y)
}

impl YappingMaw<'_> {
    /// Word `word` of the segments' offsets ([`axis_word`]).
    fn segment_offset(&self, word: usize) -> u16 {
        self.0.extended(SEGMENT_OFFSETS + word)
    }
    fn segment_angle(&self, segment: usize) -> u16 {
        self.0.extended(SEGMENT_ANGLES + segment)
    }
    fn segment_projectile(&self, segment: usize) -> u16 {
        self.0.extended(SEGMENT_PROJECTILES + segment)
    }
}
impl<H: SlotState + ?Sized> YappingMawEdit<'_, H> {
    /// Word `word` of the segments' offsets ([`axis_word`]).
    fn segment_offset(&mut self, word: usize) -> &mut u16 {
        self.0.extended(SEGMENT_OFFSETS + word)
    }
    fn segment_angle(&mut self, segment: usize) -> &mut u16 {
        self.0.extended(SEGMENT_ANGLES + segment)
    }
    /// Word `word` of the segments' velocities ([`axis_word`]).
    fn segment_velocity(&mut self, word: usize) -> &mut u16 {
        self.0.extended(SEGMENT_VELOCITIES + word)
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
/// Samus command 3: locked in place.
const LOCK_SAMUS: u16 = 3;
/// $7E:1997: the enemy projectile IDs.
/// The yapping maw's bite, sound 2Fh, library 2.
const BITE_SOUND: u8 = 0x2f;
const QUARTER_TURN: u8 = 0x40;
const SIGN: u16 = 0x8000;

/// What Yapping Maw needs beyond the shared actor operations.
pub trait Host: crate::ram::WorkRamAccess + crate::runtime_access::RuntimeAccess {
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
    let maw = game.current_enemy();
    let sprite_owner = game.current_enemy();
    {
        let mut edit = YappingMawEdit::of(game, maw);
        let copied = edit.actor().x.whole;
        *edit.base_x() = copied;
        let copied = edit.actor().y.whole;
        *edit.base_y() = copied;
        for word in 0..8 {
            *edit.segment_offset(word) = 0;
        }
    }
    *YappingMawEdit::of(game, maw).grabbing_samus() = 0;
    let mut edit = YappingMawEdit::of(game, maw);
    let proximity = *edit.parameter_1();
    *edit.proximity() = proximity;
    *edit.cooldown_timer() = COOLDOWN_FRAMES;
    edit.actor().instruction = instruction_list::INITIAL_FACING_UP;
    let (mut sprite_kind, mut displacement) = (BASE_FACING_UP, BASE_OFFSET);
    if *edit.parameter_2() == 0 {
        edit.actor().instruction = instruction_list::INITIAL_FACING_DOWN;
        sprite_kind = BASE_FACING_DOWN;
        displacement = BASE_OFFSET.wrapping_neg();
    }
    let actor = edit.actor();
    actor.instruction_delay = 1;
    actor.timer = 0;
    *edit.function() = function::NEUTRAL;
    let palette = edit.actor().palette & PALETTE_BITS;
    *edit.initial_palette() = palette;
    let mut part = LAST_SEGMENT as i16;
    *edit.segment_index() = LAST_SEGMENT;
    drop(edit);
    loop {
        game.spawn_projectile(part as u16, game.current_enemy(), BODY_PROJECTILE);
        part = (YappingMaw::of(game, maw).segment_index() as i16).wrapping_sub(1);
        *YappingMawEdit::of(game, maw).segment_index() = part as u16;
        if part < 0 {
            break;
        }
    }
    let view = YappingMaw::of(game, maw);
    let appearance = view.actor().tile | view.actor().palette;
    *YappingMawEdit::of(game, maw).sprite_appearance() = appearance;
    let view = YappingMaw::of(game, maw);
    let x = view.actor().x.whole;
    let y = displacement.wrapping_add(view.actor().y.whole);
    let sprite = game.foreign(|game| game.create_sprite(x, y, sprite_kind, appearance));
    *YappingMawEdit::of(game, sprite_owner).sprite_object() = sprite;
}
/// Yapping maw function - neutral ($A8:A235): wait for Samus in reach.
fn neutral<L: Lease>(game: &mut L) {
    let maw = game.current_enemy();
    let view = YappingMaw::of(game, maw);
    let origin = Point {
        x: view.actor().x.whole,
        y: view.actor().y.whole,
    };
    let target = Point {
        x: game.player_x(),
        y: game.player_y(),
    };
    let (pair_distance, pair_direction) =
        game.foreign(|game| game.distance_and_direction(origin, target));
    let distance = abs16(pair_distance);
    let proximity = YappingMaw::of(game, maw).proximity();
    let mut edit = YappingMawEdit::of(game, maw);
    *edit.target_length() = distance;
    if distance.wrapping_sub(REACH) & SIGN != 0 {
        *edit.intangibility_timer() = INTANGIBLE_FRAMES;
    } else if (distance as i16).wrapping_sub(proximity as i16) < 0 {
        if edit.target_length().wrapping_sub(MAX_LENGTH) & SIGN == 0 {
            *edit.target_length() = MAX_LENGTH;
        }
        *edit.target_angle() = pair_direction;
        *edit.function() = function::ATTACK;
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
    let maw = game.current_enemy();
    let direction = {
        let mut edit = YappingMawEdit::of(game, maw);
        *edit.angle() = 0;
        *edit.subangle() = 0;
        *edit.angular_speed_index() = 0;
        let length = *edit.target_length() >> 1;
        *edit.length() = length;
        let direction = QUARTER_TURN.wrapping_sub(*edit.target_angle() as u8) as u16;
        *edit.maths_target_angle() = direction;
        direction
    };
    *YappingMawEdit::of(game, maw).aiming_down() = u16::from(direction >= STRAIGHT);
    let origin = game.current_enemy();
    let length = YappingMaw::of(game, origin).length();
    let x = project(64, length, write_operand, game);
    *YappingMawEdit::of(game, origin).origin_x() = x;
    let length = YappingMaw::of(game, origin).length() >> 1;
    let y = project(128, length, write_operand, game);
    *YappingMawEdit::of(game, origin).origin_y() = y;
    let direction =
        2 * ((YappingMaw::of(game, maw).target_angle() as u8).wrapping_add(16) as u16 >> 5);
    *YappingMawEdit::of(game, maw).instruction_list_index() = direction;
    let list = crate::actor_tables::word(game, |t| &t.maw_attack_lists, (direction >> 1) as isize);
    {
        let mut edit = YappingMawEdit::of(game, maw);
        let actor = edit.actor();
        actor.instruction = list;
        actor.instruction_delay = 1;
        actor.timer = 0;
    }
    // These offsets use the native binding; animation above uses the loaded image.
    let x = game.native_image().table_word(
        sm_data::cartridge::lorom_offset(SAMUS_X_OFFSETS) as usize,
        direction as isize,
    );
    *YappingMawEdit::of(game, maw).samus_x_offset() = x;
    let y = game.native_image().table_word(
        sm_data::cartridge::lorom_offset(SAMUS_Y_OFFSETS) as usize,
        direction as isize,
    );
    let mut edit = YappingMawEdit::of(game, maw);
    *edit.samus_y_offset() = y;
    *edit.function() = function::ATTACKING;
}
fn reorient_segment<L: Lease>(
    part: usize,
    write_operand: &mut impl FnMut(u8, &mut L),
    game: &mut L,
) {
    let maw = game.current_enemy();
    let distance = YappingMaw::of(game, maw).segment_offset(axis_word(part, false));
    let x = project(0u16.wrapping_sub(64), distance, write_operand, game);
    *YappingMawEdit::of(game, maw).scratch_x() = x;
    let y = project(0, distance, write_operand, game);
    *YappingMawEdit::of(game, maw).scratch_y() = y;
    let angle = YappingMaw::of(game, maw)
        .maths_target_angle()
        .wrapping_sub(64);
    let x = project(angle, distance, write_operand, game);
    {
        let mut edit = YappingMawEdit::of(game, maw);
        let scratch = *edit.scratch_x();
        *edit.segment_velocity(axis_word(part, false)) = x.wrapping_sub(scratch);
    }
    let angle = YappingMaw::of(game, maw).maths_target_angle();
    let y = project(angle, distance, write_operand, game);
    let mut edit = YappingMawEdit::of(game, maw);
    let scratch = *edit.scratch_y();
    *edit.segment_velocity(axis_word(part, true)) = y.wrapping_sub(scratch);
}
/// Move Samus with yapping maw pincers ($A8:A665). Was `carry_player`.
fn move_samus_with_pincers<L: Lease>(game: &mut L) {
    game.foreign(|game| game.command_player(LOCK_SAMUS));
    let maw = game.current_enemy();
    let view = YappingMaw::of(game, maw);
    let x = view.samus_x_offset().wrapping_add(view.actor().x.whole);
    game.set_player_x(x);
    let view = YappingMaw::of(game, maw);
    let y = view.samus_y_offset().wrapping_add(view.actor().y.whole);
    game.set_player_y(y);
    crate::player_position::correct_previous_position(game);
}
/// Yapping maw function - attacking ($A8:A445), updating the angular speed
/// ($A8:A63E). Was `stretch`.
fn attacking<L: Lease>(write_operand: &mut impl FnMut(u8, &mut L), game: &mut L) {
    let maw = game.current_enemy();
    let step = YappingMaw::of(game, maw).angle() >> 2;
    let positive = YappingMaw::of(game, maw).aiming_down() != 0;
    {
        let mut edit = YappingMawEdit::of(game, maw);
        for part in 0..4 {
            let bend = step.wrapping_mul(part as u16 + 1);
            *edit.segment_angle(part) = if positive {
                STRAIGHT.wrapping_add(bend)
            } else {
                STRAIGHT.wrapping_sub(bend)
            };
        }
    }
    let radius = YappingMaw::of(game, maw).length();
    for part in 0..4 {
        let angle = YappingMaw::of(game, maw)
            .segment_angle(part)
            .wrapping_sub(64);
        let x = project(angle, radius, write_operand, game);
        let mut edit = YappingMawEdit::of(game, maw);
        let origin = *edit.origin_x();
        *edit.segment_offset(axis_word(part, false)) = x.wrapping_sub(origin);
    }
    let radius = YappingMaw::of(game, maw).length() >> 1;
    for part in 0..4 {
        let angle = YappingMaw::of(game, maw).segment_angle(part);
        let y = project(angle, radius, write_operand, game);
        let mut edit = YappingMawEdit::of(game, maw);
        let origin = *edit.origin_y();
        *edit.segment_offset(axis_word(part, true)) = y.wrapping_sub(origin);
    }
    for part in 0..4 {
        reorient_segment(part, write_operand, game);
    }
    {
        // The original adds after one CLC ($A8:A532), so each add's carry
        // reaches the next offset.
        let mut edit = YappingMawEdit::of(game, maw);
        let mut carry = false;
        for word in 0..8 {
            let velocity = *edit.segment_velocity(word);
            let offset = edit.segment_offset(word);
            let (sum, first) = offset.overflowing_add(velocity);
            let (sum, second) = sum.overflowing_add(u16::from(carry));
            carry = first || second;
            *offset = sum;
        }
        let x = edit.head_x_offset().wrapping_add(*edit.base_x());
        edit.actor().x.whole = x;
        let y = edit.head_y_offset().wrapping_add(*edit.base_y());
        edit.actor().y.whole = y;
    }
    let velocity_owner = game.current_enemy();
    let speed = YappingMaw::of(game, velocity_owner).angular_speed_index() as isize >> 1;
    let high = game.enemy_speed(SpeedTable::Quadratic, speed + 1);
    let low = game.enemy_speed(SpeedTable::Quadratic, speed);
    {
        let mut edit = YappingMawEdit::of(game, velocity_owner);
        let angle = (u32::from(*edit.angle()) << 16) | u32::from(*edit.subangle());
        let sum = angle.wrapping_add((u32::from(high) << 16) | u32::from(low));
        *edit.subangle() = sum as u16;
        *edit.angle() = (sum >> 16) as u16;
        let index = edit.angular_speed_index();
        *index = index.wrapping_add(SPEED_STEP);
    }
    let angle = YappingMaw::of(game, maw).angle();
    if angle & SIGN == 0 {
        if angle.wrapping_sub(STRAIGHT) & SIGN == 0 {
            let mut edit = YappingMawEdit::of(game, maw);
            *edit.angle() = STRAIGHT;
            *edit.subangle() = 0;
            let index = edit.angular_speed_index();
            *index = index.wrapping_add(NEGATE_SPEED);
        }
    } else {
        let mut edit = YappingMawEdit::of(game, maw);
        *edit.function() = function::COOLDOWN;
        *edit.intangibility_timer() = INTANGIBLE_FRAMES;
        let direction = *edit.instruction_list_index();
        edit.actor().instruction = if direction == RIGHT {
            instruction_list::COOLDOWN_UP_RIGHT
        } else if direction == LEFT {
            instruction_list::COOLDOWN_UP_LEFT
        } else {
            instruction_list::COOLDOWN_UP
        };
        if *edit.parameter_2() != 0 {
            return;
        }
        let direction = *edit.instruction_list_index();
        if direction == RIGHT {
            edit.actor().instruction = instruction_list::COOLDOWN_DOWN_RIGHT;
        } else if direction == LEFT {
            edit.actor().instruction = instruction_list::COOLDOWN_DOWN_LEFT;
        } else {
            edit.actor().instruction = instruction_list::COOLDOWN_DOWN;
            return;
        }
    }
    if YappingMaw::of(game, maw).grabbing_samus() != 0 {
        move_samus_with_pincers(game);
    }
}
/// Yapping maw function - cooldown ($A8:A68A). Was `release_player`.
fn cooldown<L: Lease>(game: &mut L) {
    let maw = game.current_enemy();
    if YappingMaw::of(game, maw).grabbing_samus() != 0 {
        move_samus_with_pincers(game);
    }
    let timer = {
        let mut edit = YappingMawEdit::of(game, maw);
        let timer = edit.cooldown_timer();
        *timer = timer.wrapping_sub(1);
        *timer
    };
    if timer & SIGN != 0 && !demo_input(game) {
        set_input(game, InputMode::Normal);
        *YappingMawEdit::of(game, maw).grabbing_samus() = 0;
        let mut edit = YappingMawEdit::of(game, maw);
        *edit.intangibility_timer() = INTANGIBLE_FRAMES;
        *edit.cooldown_timer() = COOLDOWN_FRAMES;
        *edit.function() = function::NEUTRAL;
    }
}
/// Whether Samus's input handler ($0A60) is the demo's ($E91D).
fn demo_input<L: Lease>(game: &L) -> bool {
    game.session().player.transitions.input_mode.to_raw() == InputMode::Demo.to_raw()
}
/// Store Samus's input handler ($0A60).
fn set_input<L: Lease>(game: &mut L, mode: InputMode) {
    game.session_mut().player.transitions.input_mode = mode;
    game.words_stored();
}
/// Give Samus back the normal input handler ($E913) unless the demo drives her.
fn release_input<L: Lease>(game: &mut L) {
    if !demo_input(game) {
        set_input(game, InputMode::Normal);
    }
}
/// Main AI - Yapping Maw ($A8:A211): run its function. False selects the
/// existing runtime's unknown-address failure before attachments update. Was
/// `advance`.
pub fn main_ai<L: Lease>(game: &mut L, mut write_operand: impl FnMut(u8, &mut L)) -> bool {
    crate::probe::hit("yapping_maw.main_ai");
    let maw = game.current_enemy();
    {
        let mut edit = YappingMawEdit::of(game, maw);
        let timer = edit.intangibility_timer();
        *timer = timer.wrapping_sub(1);
    }
    game.foreign(|game| game.update_maw_visibility(i32::from(maw)));
    match YappingMaw::of(game, maw).function() {
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
    let maw = game.current_enemy();
    if YappingMaw::of(game, maw).intangibility_timer() & SIGN != 0
        && YappingMaw::of(game, maw).grabbing_samus() == 0
    {
        *YappingMawEdit::of(game, maw).intangibility_timer() = 0;
        *YappingMawEdit::of(game, maw).grabbing_samus() = 1;
        // The grabbed handler ($E90E) ignores the controller.
        set_input(game, InputMode::Disabled);
    }
}
/// The base sprite object takes the maw's initial palette.
pub fn appearance<L: Lease>(game: &mut L) {
    let enemy = game.current_enemy();
    let (appearance, handle) = if usize::from(enemy) < 32 * 64 && enemy.is_multiple_of(64) {
        let maw = YappingMaw::of(game, enemy);
        (maw.initial_palette(), maw.sprite_object())
    } else {
        // LEGACY(original-overflow): an enemy index outside the slot table
        // reads the RAM at $7E:0F78 + index + the word's offset.
        let at = |offset| 0xf78 + usize::from(enemy) + offset;
        (
            legacy_overflow::slot_word(game, at(offset_of!(EnemyLayout104, var_45))),
            legacy_overflow::slot_word(game, at(offset_of!(EnemyLayout104, var_46))),
        )
    };
    game.foreign(|game| game.maw_appearance(handle, appearance, enemy));
}
/// Enemy shot - Yapping Maw ($A8:A7BD).
pub fn shot<L: Lease>(game: &mut L) {
    game.shot_reaction(Shot::Finalize);
    let maw = game.current_enemy();
    if YappingMaw::of(game, maw).actor().health != 0 {
        if YappingMaw::of(game, maw).actor().frozen != 0 {
            release_input(game);
            *YappingMawEdit::of(game, maw).grabbing_samus() = 0;
        }
    } else {
        for part in 0..4 {
            let projectile = YappingMaw::of(game, maw).segment_projectile(part);
            game.foreign(|game| crate::enemy_projectiles::table::free(projectile, game));
        }
        let handle = YappingMaw::of(game, maw).sprite_object();
        game.foreign(|game| game.retire_sprite(handle));
        release_input(game);
        let maw = game.current_enemy();
        *YappingMawEdit::of(game, maw).grabbing_samus() = 0;
    }
}
/// Frozen AI - Yapping Maw ($A8:A835).
pub fn frozen<L: Lease>(game: &mut L) {
    game.foreign(|game| game.update_frozen_enemy());
    let maw = game.current_enemy();
    for part in 0..4 {
        let handle = YappingMaw::of(game, maw).segment_projectile(part);
        game.foreign(|game| game.update_maw_palette(handle));
    }
    appearance(game);
}
/// Run one of Yapping Maw's script instructions (one of [`instruction`]).
pub fn instruction<L: Lease>(address: u32, script: EnemyScript, game: &mut L) -> EnemyScript {
    use instruction::*;
    if address == QUEUE_SOUND_IF_ON_SCREEN {
        if YappingMaw::of(game, game.current_enemy()).off_screen() == 0 {
            game.queue_sound(1, BITE_SOUND, 6);
        }
    } else {
        let maw = game.current_enemy();
        let table = match address {
            OFFSET_SAMUS_UP_RIGHT => SAMUS_X_OFFSETS + 4,
            OFFSET_SAMUS_UP_LEFT => SAMUS_X_OFFSETS + 28,
            OFFSET_SAMUS_DOWN_RIGHT => SAMUS_X_OFFSETS + 12,
            OFFSET_SAMUS_DOWN_LEFT => SAMUS_X_OFFSETS + 20,
            OFFSET_SAMUS_UP => SAMUS_X_OFFSETS,
            OFFSET_SAMUS_DOWN => SAMUS_X_OFFSETS + 16,
            _ => unreachable!("Yapping Maw instruction {address:06x}"),
        };
        let x = rom_table_word!(game, table, 0);
        *YappingMawEdit::of(game, maw).samus_x_offset() = x;
        let y = rom_table_word!(game, table + 2, 0);
        *YappingMawEdit::of(game, maw).samus_y_offset() = y;
    }
    script
}
