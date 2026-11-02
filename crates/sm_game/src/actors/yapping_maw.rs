//! Yapping Maw, the pincer worm that snaps Samus out of the air (bank
//! $A8:9F4F-$A8:A8EE): reach, articulated body, grip and attachments.
//! RAM words remain authoritative; operand writes run immediately through the caller.
use crate::actors::boss_lease::BossLease;
use crate::actors::layouts::part_view;
use crate::enemy_script::EnemyScript;
use crate::geometry::Point;

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
    game.foreign(|game| owned::initialize(&mut crate::runtime_access::RuntimeAccess::runtime(game)))
}
/// Yapping maw function - neutral ($A8:A235): wait for Samus in reach.

/// Yapping maw function - attack ($A8:A28C), calculating the origin
/// ($A8:A310). Was `start_reach`.

/// Move Samus with yapping maw pincers ($A8:A665). Was `carry_player`.

/// Yapping maw function - attacking ($A8:A445), updating the angular speed
/// ($A8:A63E). Was `stretch`.

/// Yapping maw function - cooldown ($A8:A68A). Was `release_player`.

/// Main AI - Yapping Maw ($A8:A211): run its function. False selects the
/// existing runtime's unknown-address failure before attachments update. Was
/// `advance`.
pub fn main_ai<L: Lease>(game: &mut L) -> bool {
    game.foreign(|game| owned::main_ai(&mut crate::runtime_access::RuntimeAccess::runtime(game)))
}
pub fn update_segments<L: Lease>(game: &mut L) {
    game.foreign(|game| {
        owned::update_segments(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    })
}
/// Enemy touch - Yapping Maw ($A8:A799): grab Samus.
pub fn touch<L: Lease>(game: &mut L) {
    game.foreign(|game| owned::touch(&mut crate::runtime_access::RuntimeAccess::runtime(game)))
}
pub fn appearance<L: Lease>(game: &mut L) {
    game.foreign(|game| owned::appearance(&mut crate::runtime_access::RuntimeAccess::runtime(game)))
}
/// Enemy shot - Yapping Maw ($A8:A7BD).
pub fn shot<L: Lease>(game: &mut L) {
    game.foreign(|game| owned::shot(&mut crate::runtime_access::RuntimeAccess::runtime(game)))
}
/// Frozen AI - Yapping Maw ($A8:A835).
pub fn frozen<L: Lease>(game: &mut L) {
    game.foreign(|game| owned::frozen(&mut crate::runtime_access::RuntimeAccess::runtime(game)))
}
/// Run one of Yapping Maw's script instructions (one of [`instruction`]).
pub fn instruction<L: Lease>(address: u32, script: EnemyScript, game: &mut L) -> EnemyScript {
    game.foreign(|game| {
        owned::instruction(
            address,
            script,
            &mut crate::runtime_access::RuntimeAccess::runtime(game),
        )
    })
}

#[path = "yapping_maw_owned.rs"]
pub mod owned;
