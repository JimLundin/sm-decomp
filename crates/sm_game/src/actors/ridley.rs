//! Ridley, Norfair's dragon (bank $A6:A0F5-$A6:EFB0): the fight's state
//! machine over the body's function word, the hover, swoop, pogo and lunge,
//! grabbing Samus, the tail, the baby metroid he carries out of Ceres, the
//! death sequence and its explosions.
//!
//! Was `Ridley_*`, `BabyMetroid_*` and `RidleysExplosion_*` in the original
//! decompilation. Ridley is enemy 0; the `Ridley` view names his slot and his
//! three extra RAM blocks at $7E:7800, $7E:8000 and $7E:8800 over the same
//! bytes as `EnemyLayout104`. The tail is seven segments of ten words in
//! `tilemap_stuff` ($7E:2000), named in `tail`. Ceres Ridley's own functions
//! and its escape sequence are in `ceres_ridley`; `Host::run_function`
//! dispatches both through the bank's function table.
use crate::actors::api::angle_from_delta;
use crate::actors::boss_lease::BossLease;
use crate::actors::deaths::Deaths as _;
use crate::actors::host::BossEffects as _;
use crate::actors::host::{Death, Shot, abs16, below};
use crate::actors::layouts::{EnemyBase, same_words};
use crate::actors::ridley_tables::{set_pogo_speeds, set_tail_segments_from_list};
use crate::actors::session_slots::{SlotAccess, session_slot};
use crate::constants::enemy_properties::{
    K_ENEMY_PROPS_DELETED, K_ENEMY_PROPS_INVISIBLE, K_ENEMY_PROPS_TANGIBLE,
};
use crate::display_refresh::mult8x8;
use crate::enemy_projectiles::drops::DropSource;
use crate::enemy_projectiles::shared_pre::{x_velocity_at_angle, y_velocity_at_angle};
use crate::enemy_script::EnemyScript;
use crate::geometry::Point;
use crate::palette_copies::copy_rom_colors_to_palette;
use crate::ram::RawRam as _;
use crate::ram::word;
use crate::sprite_drawing::Clip;
use crate::{ridley_data, ridley_explosion_data, sprite_output};
use sm_data::cartridge::lorom_offset;
use sm_data::ridley_explosion::{
    HAND_ADDRESS, LEGS_ADDRESS, MOUTH_ADDRESS, TORSO_ADDRESS, WINGS_ADDRESS,
};

/// A lease on a game with Ridley's effects.
pub trait Lease: BossLease<Game: Host> {}
impl<T: BossLease<Game: Host> + ?Sized> Lease for T {}

/// What Ridley needs beyond the shared boss effects, deaths and sprites.
pub trait Host:
    crate::actors::host::BossEffects
    + crate::actors::deaths::Deaths
    + crate::records::Records
    + crate::cartridge::NativeImage
    + crate::sprite_output::SpriteHost
    + BossLease<Game = Self>
{
    /// Run the bank $A6 Ridley function at `function`, Norfair's or Ceres's.
    fn run_function(&mut self, function: u16);
    /// Spawn the enemy population at `bank:population`.
    fn spawn_population(&mut self, bank: u8, population: u16) -> u16;
    /// Mark the current area's boss bits `mask` as defeated.
    fn set_boss_bit(&mut self, mask: u16);
    /// `amplitude` times the sine of `angle` through the PPU multiplier.
    fn hardware_sine(&mut self, amplitude: u16, angle: u16) -> u16;
    /// `amplitude` times the cosine of `angle` through the PPU multiplier.
    fn hardware_cosine(&mut self, amplitude: u16, angle: u16) -> u16;
    /// Print the original "Warning: X undefined" line.
    fn warn_x_undefined(&mut self);
    /// Move the room's liquid to `target` at `velocity` after `delay` frames.
    fn move_liquid(&mut self, target: u16, velocity: u16, delay: u16);
}

/// What Ridley's state steps read and write: his slot words and the named RAM
/// words, on the Session inside a lease and on the RAM image otherwise.
pub trait State: SlotAccess + crate::ram::CompatibilityRam {}
impl<T: SlotAccess + crate::ram::CompatibilityRam + ?Sized> State for T {}

/// Ridley's view of enemy 0 and his extra RAM blocks, Norfair's and Ceres's.
/// `fade_index` is also the escape timer tile transfer pointer and the self
/// destruct step's timer, `function_timer` the self destruct sequence step,
/// and `swoop_angle` the getaway's mode 7 angle. The `pad_*` words are unused.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Ridley {
    pub base: EnemyBase,
    pub function: u16,
    pub x_velocity: u16,
    pub y_velocity: u16,
    pad_0fae: [u16; 1],
    pub fade_index: u16,
    pub function_timer: u16,
    pub parameter_1: u16,
    pub parameter_2: u16,
    pad1: [u8; 26696],
    pub timer: u16,
    pub fight_mode: u16,
    pub movement_enabled: u16,
    pub baby_metroid_instruction: u16,
    pub baby_metroid_instruction_timer: u16,
    pub ribs_pointer: u16,
    pub ribs_timer: u16,
    pub wings_frame: u16,
    pub wings_timer_delta: u16,
    pub wings_timer: u16,
    pub swoop_angle: u16,
    pub swoop_speed: u16,
    pub palette: u16,
    pub hit_counter: u16,
    pub whip_sound_counter: u16,
    pub roaring: u16,
    pub facing: u16,
    pad_7822: [u16; 1],
    pub acceleration_index: u16,
    pad_7826: [u16; 1],
    pub grabbed_x_offset: u16,
    pub grabbed_y_offset: u16,
    pad_782c: [u16; 1],
    pub target_x: u16,
    pub target_y: u16,
    pub fireball_x_velocity: u16,
    pub fireball_y_velocity: u16,
    pub holding_samus: u16,
    pub tail_damage: u16,
    pub feet_distance_index: u16,
    pub intangibility_timer: u16,
    pub hit_boundary: u16,
    pad2: [u8; 1984],
    pub min_y: u16,
    pub max_y: u16,
    pub min_x: u16,
    pub max_x: u16,
    pub grabbed_lag_timer: u16,
    pub death_lunges: u16,
    pub pogo_bounces: u16,
    pub small_explosion_timer: u16,
    pub small_explosion_index: u16,
    pad_8012: [u16; 7],
    pub mode7_zoom: u16,
    /// Set with the zoom, never read.
    pub unread_8022: u16,
    pub getaway_finished: u16,
    pub getaway_index: u16,
    pub mode7_x: u16,
    pub mode7_y: u16,
    pub baby_metroid_frame: u16,
    pub mode7_wings_frame: u16,
    pub emergency_glow_index: u16,
    pad_8032: [u16; 2],
    pub typewriter_list: u16,
    pub typewriter_tilemap: u16,
    pub typewriter_timer: u16,
    pub typewriter_timer_reset: u16,
    pub typewriter_stroke_timer: u16,
    pad3: [u8; 1984],
    pub baby_metroid_function: u16,
    pad_8802: [u16; 1],
    pub baby_metroid_x: u16,
    pub baby_metroid_y_subpixel: u16,
    pub baby_metroid_y: u16,
    pub baby_metroid_unread: u16,
    pub baby_metroid_y_velocity: u16,
    pad_880e: [u16; 25],
}

/// An explosion enemy's view of its slot: `part` is its initialisation index,
/// the part of Ridley it starts on.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Explosion {
    pub base: EnemyBase,
    pad_0fa8: [u16; 1],
    pub x_velocity: u16,
    pub y_velocity: u16,
    pad_0fae: [u16; 2],
    pub lifetime: u16,
    pub part: u16,
    pad_0fb6: [u16; 1],
}

/// The Ceres door's view of enemy 1, which Ridley's graphics hook draws.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct CeresDoor {
    pub base: EnemyBase,
    pad_0fa8: [u16; 1],
    pub drawn_by_ridley: u16,
    pad_0fac: [u16; 6],
}

same_words!(Ridley: EnemyLayout104 {
    function = var_a, x_velocity = var_b, y_velocity = var_c, fade_index = var_e,
    function_timer = var_f, parameter_1 = parameter_1, parameter_2 = parameter_2,
    timer = var_00, fight_mode = var_01, movement_enabled = var_02,
    baby_metroid_instruction = var_03, baby_metroid_instruction_timer = var_04,
    ribs_pointer = var_05, ribs_timer = var_06, wings_frame = var_07, wings_timer_delta = var_08,
    wings_timer = var_09, swoop_angle = var_0a, swoop_speed = var_0b, palette = var_0c,
    hit_counter = var_0d, whip_sound_counter = var_0e, roaring = var_0f, facing = var_10,
    acceleration_index = var_12, grabbed_x_offset = var_14, grabbed_y_offset = var_15,
    target_x = var_17, target_y = var_18, fireball_x_velocity = var_19,
    fireball_y_velocity = var_1a, holding_samus = var_1b, tail_damage = var_1c,
    feet_distance_index = var_1d, intangibility_timer = var_1e, hit_boundary = var_1f,
    min_y = var_20, max_y = var_21, min_x = var_22, max_x = var_23, grabbed_lag_timer = var_24,
    death_lunges = var_25, pogo_bounces = var_26, small_explosion_timer = var_27,
    small_explosion_index = var_28, mode7_zoom = var_30, unread_8022 = var_31,
    getaway_finished = var_32, getaway_index = var_33, mode7_x = var_34, mode7_y = var_35,
    baby_metroid_frame = var_36, mode7_wings_frame = var_37, emergency_glow_index = var_38,
    typewriter_list = var_3b, typewriter_tilemap = var_3c, typewriter_timer = var_3d,
    typewriter_timer_reset = var_3e, typewriter_stroke_timer = var_3f,
    baby_metroid_function = var_40, baby_metroid_x = var_42, baby_metroid_y_subpixel = var_43,
    baby_metroid_y = var_44, baby_metroid_unread = var_45, baby_metroid_y_velocity = var_46,
});
same_words!(Explosion: EnemyLayout8 {
    x_velocity = var_b, y_velocity = var_c, lifetime = var_f, part = parameter_1,
});
same_words!(CeresDoor: EnemyLayout8 { drawn_by_ridley = var_b });

pub(crate) const RIDLEY: i32 = 0;
const CERES_DOOR: i32 = 0x40;

/// Ridley facing directions.
pub(crate) const FACING_LEFT: u16 = 0;
const FACING_FORWARD: u16 = 1;
pub(crate) const FACING_RIGHT: u16 = 2;
/// The fight mode of the death sequence; 0 is the intro or retreat, 1 the
/// fight and 2 dodging a power bomb.
const DEATH_SEQUENCE: u16 = 0xffff;
/// Ridley's area; anywhere else he is Ceres Ridley.
pub(crate) const NORFAIR: u16 = 2;
/// Samus commands.
const LOCK_SAMUS: u16 = 0;
const UNLOCK_SAMUS: u16 = 1;
/// A whip target angle with bit 15 set means no whip.
const NO_WHIP_TARGET: u16 = 0xffff;
/// Enemy definitions are in bank $A0; their contact damage word.
const ENEMY_DEFINITION_BANK: u8 = 0xa0;
const ENEMY_DEFINITION_DAMAGE: usize = 6;
/// The projectile type words.
const PROJECTILE_TYPES: usize = 0xc18;

/// Bank $A6 Ridley functions, stored in the body's `function` word.
pub mod function {
    pub const LIFTOFF: u16 = 0xb2f3;
    pub const DECIDE_ACTION: u16 = 0xb321;
    pub const LUNGE_MISSED: u16 = 0xb3ec;
    pub const LUNGE_MISSED_MOVE_TO_POSITION: u16 = 0xb3f8;
    pub const SWOOP: u16 = 0xb441;
    pub const SWOOP_MOVE_TO_POSITION: u16 = 0xb455;
    pub const SWOOP_AIMING_DOWN: u16 = 0xb493;
    pub const SWOOP_AIMING_HORIZONTAL: u16 = 0xb4d1;
    pub const SWOOP_AIMING_UP: u16 = 0xb516;
    pub const SWOOP_AIMING_UP_FASTER: u16 = 0xb554;
    pub const SWOOP_DECELERATE: u16 = 0xb594;
    pub const HOVER: u16 = 0xb5c4;
    pub const HOVER_TO_WALL_BEHIND: u16 = 0xb5e5;
    pub const HOVER_TO_WALL_IN_FRONT: u16 = 0xb613;
    pub const POGO_FLY_TO_POSITION: u16 = 0xb6a7;
    pub const POGO_WAIT_AT_POSITION: u16 = 0xb6dd;
    pub const POGO_DESCENDING: u16 = 0xb70e;
    pub const POGO_ASCENDING: u16 = 0xb7b9;
    pub const LUNGE: u16 = 0xbab7;
    pub const GRABBED_SAMUS: u16 = 0xbb8f;
    pub const GRABBED_SAMUS_RISING: u16 = 0xbbc4;
    pub const GRABBED_SAMUS_RISING_TO_MIDDLE: u16 = 0xbbf1;
    pub const GRABBED_SAMUS_RELEASE: u16 = 0xbc2e;
    pub const DODGE_POWER_BOMB: u16 = 0xbd4e;
    pub const FAKE_RETREAT_MOVE_TO_POSITION: u16 = 0xbd9a;
    pub const FAKE_RETREAT_RISING: u16 = 0xbdbc;
    pub const DROP_BABY_METROID: u16 = 0xbdf2;
    pub const RETRIEVE_BABY_METROID: u16 = 0xbe03;
    pub const DYING_GRABBED_SAMUS: u16 = 0xc538;
    pub const DEATH_ROAR: u16 = 0xc53e;
    pub const DEATH_MOVE_TO_DEATH_SPOT: u16 = 0xc551;
    pub const LOWER_ACID_AND_RELEASE_SAMUS: u16 = 0xc588;
    pub const START_FALLING_APART: u16 = 0xc5a8;
    pub const FALLING_APART: u16 = 0xc5c8;
    pub const STILL_FALLING_APART: u16 = 0xc5da;
    /// The RTS after the death sequence.
    pub const DEAD: u16 = 0xc600;
    /// Ceres Ridley's liftoff, which the liftoff instruction starts.
    pub const CERES_LIFTOFF_ACCELERATING: u16 = 0xa6af;
    /// Ceres Ridley's real retreat, once he holds the baby metroid again.
    pub const CERES_REAL_RETREAT_RISING: u16 = 0xa971;
}

/// Bank $A6 baby metroid instructions.
pub mod baby_metroid_instruction {
    pub const GOTO_IF_STILL: u16 = 0xbfc9;
    pub const LOAD_PALETTE: u16 = 0xbfe1;
    pub const GOTO_IF_MOVING: u16 = 0xbff2;
    pub const GOTO: u16 = 0xbff8;
}

/// Bank $A6 Ridley instructions.
pub mod instruction {
    pub const ROAR: u16 = 0xe4be;
    pub const STOP_ROARING: u16 = 0xe4ca;
    pub const GOTO_IF_SAMUS_HEALTH_LOW: u16 = 0xe4d2;
    pub const GOTO_BY_HOLDING_SAMUS: u16 = 0xe4ee;
    pub const GOTO_IF_NOT_HOLDING_SAMUS: u16 = 0xe4f8;
    pub const SET_FEET_DISTANCE: u16 = 0xe501;
    pub const GOTO_IF_NOT_FACING_LEFT: u16 = 0xe517;
    pub const MOVE_BY: u16 = 0xe51f;
    pub const FLIP_LEFT: u16 = 0xe71c;
    pub const FACE_FORWARD: u16 = 0xe727;
    pub const FLIP_RIGHT: u16 = 0xe72f;
    pub const CALCULATE_FIREBALL_VELOCITIES: u16 = 0xe84d;
    pub const FIREBALL_WITH_AFTERBURN: u16 = 0xe904;
    pub const FIREBALL_WITHOUT_AFTERBURN: u16 = 0xe909;
    pub const START_CERES_LIFTOFF: u16 = 0xe969;
    pub const START_NORFAIR_LIFTOFF: u16 = 0xe976;
}

/// Bank $A6 baby metroid functions, stored in `baby_metroid_function`.
pub mod baby_metroid_function {
    pub const HELD: u16 = 0xbe9c;
    pub const HELD_BY_FEET: u16 = 0xbeb3;
    pub const START_FALLING: u16 = 0xbeca;
    pub const FALLING: u16 = 0xbedc;
    /// The RTS once the baby metroid has landed.
    pub const NOTHING: u16 = 0xbf19;
}

/// Bank $A6 instruction lists.
pub mod instruction_list {
    pub const RETRIEVE_BABY_METROID: u16 = 0xe658;
    pub const DEATH_ROAR: u16 = 0xe6c8;
    pub const TURN_TO_FACE_RIGHT: u16 = 0xe6f0;
    pub const TURN_TO_FACE_LEFT: u16 = 0xe706;
    pub const SHOOT_FIREBALLS: u16 = 0xe73a;
}

/// Bank $A6 explosion instruction lists, by tail segment.
pub mod explosion_instruction_list {
    pub const TAIL_SEGMENTS_0_1: u16 = 0xca47;
    pub const TAIL_SEGMENTS_2_3: u16 = 0xca4d;
    pub const TAIL_SEGMENTS_4_5: u16 = 0xca53;
}

/// Bank $A6 fight action choices: eight Ridley functions each, picked at
/// random by `decide_action`.
pub mod actions {
    pub const POGO_OR_SWOOP: u16 = 0xb38c;
    pub const SWOOP_OR_POGO: u16 = 0xb39c;
    pub const LUNGE_OR_POGO: u16 = 0xb3ac;
    pub const POGO: u16 = 0xb3bc;
    pub const HOVER: u16 = 0xb3cc;
    pub const LUNGE: u16 = 0xb3dc;
}

/// Ridley tail data ($A6:D36E): seven words each, by tail segment.
pub mod tail_data {
    pub const DIRECTIONS: u16 = 0xd36e;
    pub const DISTANCES: u16 = 0xd37c;
    pub const ANGLES: u16 = 0xd38a;
    pub const X_POSITIONS: u16 = 0xd398;
    pub const Y_POSITIONS: u16 = 0xd3a6;
}

/// Ridley's tail RAM as `tilemap_stuff` word indices: the shared words, then
/// seven segments of ten words from $7E:2020. Segment 0 is the tail's base
/// and segment 6 its tip; each field function takes the segment.
pub(crate) mod tail {
    pub const FUNCTION: usize = 0;
    pub const IDLE_WHIP: usize = 1;
    pub const WHIP_REQUEST: usize = 2;
    pub const WHIP_ANGLE_OFFSET: usize = 5;
    pub const POGO_DOWN_ACCELERATION: usize = 6;
    pub const POGO_UP_ACCELERATION: usize = 7;
    pub const POGO_TAIL_TIMER: usize = 8;
    pub const EXTENSION_SPEED: usize = 9;
    pub const ANGLE_DELTA: usize = 10;
    pub const MIN_CLOCKWISE_ANGLE: usize = 11;
    pub const MAX_ANTICLOCKWISE_ANGLE: usize = 12;
    pub const WHIP_TARGET_CLOCKWISE: usize = 13;
    pub const WHIP_TARGET_ANTICLOCKWISE: usize = 14;
    pub const IDEAL_SEGMENT_ANGLE: usize = 15;
    const SEGMENTS: usize = 16;
    /// Words per segment.
    pub const SEGMENT: usize = 10;

    const fn field(segment: usize, offset: usize) -> usize {
        SEGMENTS + SEGMENT * segment + offset
    }
    pub const fn active(segment: usize) -> usize {
        field(segment, 0)
    }
    pub const fn stagger_angle(segment: usize) -> usize {
        field(segment, 1)
    }
    pub const fn direction(segment: usize) -> usize {
        field(segment, 2)
    }
    pub const fn distance(segment: usize) -> usize {
        field(segment, 3)
    }
    pub const fn target_distance(segment: usize) -> usize {
        field(segment, 4)
    }
    pub const fn angle(segment: usize) -> usize {
        field(segment, 5)
    }
    pub const fn x(segment: usize) -> usize {
        field(segment, 6)
    }
    pub const fn y(segment: usize) -> usize {
        field(segment, 7)
    }
    pub const fn x_offset(segment: usize) -> usize {
        field(segment, 8)
    }
    pub const fn y_offset(segment: usize) -> usize {
        field(segment, 9)
    }
}

/// The tail RAM's first byte, for the byte reads of segment angles.
const TAIL_RAM: usize = 0x2000;
/// The Ceres door spritemap ($A6:A329).
const CERES_DOOR_SPRITEMAP: u16 = 0xa329;
/// The enemy graphics drawn hook's long pointer.
const GRAPHICS_DRAWN_HOOK: usize = 0x178c;
/// Ridley's feet tiles in bank $B0 ($A6:DAD0): top and bottom.
const FEET_UNCLENCHED_TILES: u16 = 0xdad0;
const FEET_CLENCHED_TILES: u16 = 0xdad4;
/// The ribs animation table ($A6:DA71).
const RIBS_ANIMATION: u16 = 0xda71;
/// The Ridley tail tip spritemap pointers ($A6:C7BA), for the tail tip explosion.
const TAIL_TIP_EXPLOSION_LISTS: u32 = 0xa6_c7ba;
/// Ridley tail segment spritemaps ($A6:DC90).
const TAIL_SEGMENT_SPRITEMAPS: [u16; 3] = [0xdc90, 0xdc97, 0xdc9e];
/// Bank $86 enemy projectiles.
const DUST_CLOUD: u16 = 0xe509;
const FIREBALL: u16 = 0x9642;
/// Samus's movement type byte.
const SAMUS_MOVEMENT_TYPE: usize = 0xa1f;
const SPIN_JUMPING: u8 = 3;
/// The level data's block words ($7F:0002).
const LEVEL_DATA: isize = 0x10002;
/// The OAM buffer.
const OAM: usize = 0x370;
/// The VRAM write queue.
const VRAM_WRITE_QUEUE: usize = 0xd0;
/// One VRAM write queue entry: size, long source, destination.
const VRAM_WRITE_ENTRY: u16 = 7;
/// Ridley's tiles live in bank $B0.
const TILE_BANK: u16 = 0xb000;
/// Projectile type bits.
const MISSILE: u8 = 1;
const SUPER_MISSILE: u8 = 2;

/// The original's `Rect16U`: a centre and X and Y radii.
#[derive(Clone, Copy)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
}

/// Check for enemy collision with rectangle ($A9:EF06, as the baby Metroid's):
/// whether enemy `k`'s hitbox is clear of `rect`.
pub(crate) fn separated_from_rect<L: Lease + ?Sized>(k: u16, rect: Rect, game: &mut L) -> bool {
    let e = i32::from(k);
    let width = (session_slot!(game, Ridley[e].base.x_width) as i32 + rect.w as i32 + 1) as u16;
    let dx = abs16(
        rect.x
            .wrapping_sub(session_slot!(game, Ridley[e].base.x_pos)),
    );
    if dx < width {
        let height =
            (session_slot!(game, Ridley[e].base.y_height) as i32 + rect.h as i32 + 1) as u16;
        return abs16(
            rect.y
                .wrapping_sub(session_slot!(game, Ridley[e].base.y_pos)),
        ) >= height;
    }
    true
}

fn current<L: Lease + ?Sized>(game: &L) -> i32 {
    game.current_enemy() as i32
}

/// Count down Ridley's function timer; true once it is negative.
fn timer_expired<L: Lease + ?Sized>(game: &mut L) -> bool {
    session_slot!(
        game,
        Ridley[RIDLEY].function_timer =
            session_slot!(game, Ridley[RIDLEY].function_timer).wrapping_sub(1)
    );
    (session_slot!(game, Ridley[RIDLEY].function_timer) & 0x8000) != 0
}

/// The work RAM address of tail word `index`.
const fn tail_address(index: usize) -> u16 {
    (TAIL_RAM + 2 * index) as u16
}

fn tail_low_byte<L: Lease + ?Sized>(game: &L, index: usize) -> u16 {
    u16::from(game.slot_byte(TAIL_RAM + index * 2))
}

fn snes_divide(a: u16, b: u8) -> u16 {
    (if b as i32 == 0 {
        0xffff
    } else {
        a as i32 / b as i32
    }) as u16
}

/// The tail word `index` through raw work RAM, low byte first.
fn tail_word(game: &(impl SlotAccess + ?Sized), index: usize) -> u16 {
    let at = TAIL_RAM + index * 2;
    u16::from_le_bytes([game.slot_byte(at), game.slot_byte(at + 1)])
}

fn set_tail_word(game: &mut (impl SlotAccess + ?Sized), index: usize, value: u16) {
    let at = TAIL_RAM + index * 2;
    let [low, high] = value.to_le_bytes();
    game.set_slot_byte(at, low);
    game.set_slot_byte(at + 1, high);
}

/// Queue two VRAM transfers of `size` bytes from bank $B0, storing as the
/// original does: both bank words, both sources, both destinations, both
/// sizes, then the queue tail and the terminating size.
fn queue_tile_pair<L: Lease + ?Sized>(
    game: &mut L,
    sources: [u16; 2],
    destinations: [u16; 2],
    size: u16,
) {
    let tail = word!(game.VRAM_WRITE_QUEUE_TAIL);
    let entry = |n: usize| VRAM_WRITE_QUEUE + usize::from(tail) + n * usize::from(VRAM_WRITE_ENTRY);
    let store = |game: &mut L, at: usize, value: u16| {
        let bytes = value.to_ne_bytes();
        game.set_slot_byte(at, bytes[0]);
        game.set_slot_byte(at + 1, bytes[1]);
    };
    for n in 0..2 {
        store(game, entry(n) + 3, TILE_BANK);
    }
    for (n, &source) in sources.iter().enumerate() {
        store(game, entry(n) + 2, source);
    }
    for (n, &destination) in destinations.iter().enumerate() {
        store(game, entry(n) + 5, destination);
    }
    for n in 0..2 {
        store(game, entry(n), size);
    }
    word!(game.VRAM_WRITE_QUEUE_TAIL = tail.wrapping_add(2 * VRAM_WRITE_ENTRY));
    store(game, entry(2), 0);
}

/// The two-direction homing reversal of the Ridley and baby Metroid accelerations.
pub enum HomingReversal {
    ExtraForce(u16),
    DoubleStep,
}

/// One axis of the homing accelerations. Distance, acceleration and the limit
/// tests wrap at sixteen bits; in particular, the limit is a signed-difference
/// test, not an ordinary clamp.
pub fn homing_velocity(
    distance: i16,
    mut velocity: u16,
    divisor: u8,
    reversal: HomingReversal,
    limit: u16,
) -> u16 {
    let step = snes_divide(distance.unsigned_abs(), divisor).max(1);
    let reversing = (velocity as i16 >= 0) == (distance > 0);
    let extra = if reversing {
        match reversal {
            HomingReversal::ExtraForce(force) => step.wrapping_add(force),
            HomingReversal::DoubleStep => step.wrapping_mul(2),
        }
    } else {
        0
    };
    if distance > 0 {
        velocity = velocity.wrapping_sub(extra).wrapping_sub(step);
        if velocity.wrapping_add(limit) & 0x8000 != 0 {
            velocity = 0u16.wrapping_sub(limit);
        }
    } else {
        velocity = velocity.wrapping_add(extra).wrapping_add(step);
        if velocity.wrapping_sub(limit) & 0x8000 == 0 {
            velocity = limit;
        }
    }
    velocity
}

/// Move the current enemy by its X and Y velocities: subpixels in the low
/// bytes, pixels in the signed high bytes. Was `MoveEnemyWithVelocity`.
fn move_with_velocity<L: Lease + ?Sized>(game: &mut L) {
    let e = game.current_enemy() as i32;
    let x_velocity = session_slot!(game, Explosion[e].x_velocity);
    let carry = session_slot!(byte game, Explosion[e].base.x_subpos[1]) as i32
        + x_velocity.to_ne_bytes()[0] as i32;
    session_slot!(byte game, Explosion[e].base.x_subpos[1] = carry as u8);
    session_slot!(
        game,
        Explosion[e].base.x_pos = (session_slot!(game, Explosion[e].base.x_pos) as i32
            + ((x_velocity >> 8) as i8 as i32 + (carry >> 8)))
            as u16
    );
    let y_velocity = session_slot!(game, Explosion[e].y_velocity);
    let carry = session_slot!(byte game, Explosion[e].base.y_subpos[1]) as i32
        + y_velocity.to_ne_bytes()[0] as i32;
    session_slot!(byte game, Explosion[e].base.y_subpos[1] = carry as u8);
    session_slot!(
        game,
        Explosion[e].base.y_pos = (session_slot!(game, Explosion[e].base.y_pos) as i32
            + ((y_velocity >> 8) as i8 as i32 + (carry >> 8)))
            as u16
    );
}

/// Handle baby metroid ($A6:A2DC): run its function, then hook the Ceres
/// door and baby metroid drawing once Ridley moves, or draw them now. Was
/// `Ridley_A2DC`.
pub fn handle_baby_metroid<L: Lease + ?Sized>(game: &mut L) {
    run_baby_metroid_function(game);
    if session_slot!(game, Ridley[RIDLEY].movement_enabled) != 0 {
        // The hook is the long pointer $A6:A2F2, `draw_baby_metroid_and_door`.
        game.set_slot_byte(GRAPHICS_DRAWN_HOOK, 0xf2);
        game.set_slot_byte(GRAPHICS_DRAWN_HOOK + 1, 0xa2);
        game.set_slot_byte(GRAPHICS_DRAWN_HOOK + 2, 0xa6);
    } else {
        draw_baby_metroid_and_door(game);
    }
}

/// Enemy graphics drawn hook - Ceres Ridley - draw baby metroid and door
/// ($A6:A2F2). Was `Ridley_A2F2`.
pub fn draw_baby_metroid_and_door<L: Lease + ?Sized>(game: &mut L) {
    /// Ceres Ridley door X offsets during earthquake ($A6:A321).
    const EARTHQUAKE_X_OFFSETS: [i16; 4] = [0, -1024, -4, -1];
    if word!(game.CERES_STATUS) == 0 {
        draw_baby_metroid(game);
    }
    if session_slot!(game, CeresDoor[CERES_DOOR].drawn_by_ridley) != 0 {
        let offset = EARTHQUAKE_X_OFFSETS[(word!(game.EARTHQUAKE_TIMER) & 3) as usize];
        let x = session_slot!(game, CeresDoor[CERES_DOOR].base.x_pos).wrapping_add(offset as u16);
        let y = session_slot!(game, CeresDoor[CERES_DOOR].base.y_pos);
        game.foreign(|game| {
            sprite_output::paletted(
                0xa6,
                CERES_DOOR_SPRITEMAP,
                (x, y),
                1024,
                Clip::Bounded,
                game,
            )
        });
    }
}

/// Main AI - enemy $E17F (Ridley) ($A6:B227). Was `Ridley_Main`.
pub fn main<L: Lease + ?Sized>(lease: &mut L) {
    // The state steps and Ridley's function run on the lease, the tail,
    // drawing and ribs in one trip to the game.
    let mut lag = (session_slot!(lease, Ridley[RIDLEY].grabbed_lag_timer) as i16).wrapping_sub(4);
    if lag < 0 {
        lag = 0;
    }
    session_slot!(lease, Ridley[RIDLEY].grabbed_lag_timer = lag as u16);
    set_tangibility(lease);
    lunge_if_power_bomb(lease);
    let function = session_slot!(lease, Ridley[RIDLEY].function);
    if !run_ridley_function(lease, function) {
        lease.foreign(|game| game.run_function(function));
    }
    if session_slot!(lease, Ridley[RIDLEY].movement_enabled) != 0 {
        hurt_flash(lease);
        move_ridley(lease);
        update_wings(lease);
        lease.foreign(|game| {
            handle_tail(game);
            tail_projectile_collision(game);
            draw_tail(game);
            draw_wings(game);
            handle_ribs(game);
            if session_slot!(game, Ridley[RIDLEY].holding_samus) != 0 {
                move_samus_with_feet(game);
            }
        });
    }
    health_palette(lease);
}

/// Run Ridley's bank $A6 function `function` on this lease; false when it
/// is not one of Norfair Ridley's.
pub(crate) fn run_ridley_function<L: Lease + ?Sized>(game: &mut L, function: u16) -> bool {
    match function {
        function::LIFTOFF => liftoff(game),
        function::DECIDE_ACTION => decide_action(game),
        function::LUNGE_MISSED => lunge_missed(game),
        function::LUNGE_MISSED_MOVE_TO_POSITION => lunge_missed_move_to_position(game),
        function::SWOOP => swoop(game),
        function::SWOOP_MOVE_TO_POSITION => swoop_move_to_position(game),
        function::SWOOP_AIMING_DOWN => swoop_aiming_down(game),
        function::SWOOP_AIMING_HORIZONTAL => swoop_aiming_horizontal(game),
        function::SWOOP_AIMING_UP => swoop_aiming_up(game),
        function::SWOOP_AIMING_UP_FASTER => swoop_aiming_up_faster(game),
        function::SWOOP_DECELERATE => swoop_decelerate(game),
        function::HOVER => hover(game),
        function::HOVER_TO_WALL_BEHIND => hover_to_wall_behind(game),
        function::HOVER_TO_WALL_IN_FRONT => hover_to_wall_in_front(game),
        function::POGO_FLY_TO_POSITION => pogo_fly_to_position(game),
        function::POGO_WAIT_AT_POSITION => pogo_wait_at_position(game),
        function::POGO_DESCENDING => pogo_descending(game),
        function::POGO_ASCENDING => pogo_ascending(game),
        function::LUNGE => lunge(game),
        function::GRABBED_SAMUS => grabbed_samus(game),
        function::GRABBED_SAMUS_RISING => grabbed_samus_rising(game),
        function::GRABBED_SAMUS_RISING_TO_MIDDLE => grabbed_samus_rising_to_middle(game),
        function::GRABBED_SAMUS_RELEASE => grabbed_samus_release(game),
        function::DODGE_POWER_BOMB => dodge_power_bomb(game),
        function::FAKE_RETREAT_MOVE_TO_POSITION => fake_retreat_move_to_position(game),
        function::FAKE_RETREAT_RISING => fake_retreat_rising(game),
        function::DROP_BABY_METROID => drop_baby_metroid(game),
        function::RETRIEVE_BABY_METROID => retrieve_baby_metroid(game),
        function::DYING_GRABBED_SAMUS => dying_grabbed_samus(game),
        function::DEATH_ROAR => death_roar(game),
        function::DEATH_MOVE_TO_DEATH_SPOT => death_move_to_death_spot(game),
        function::LOWER_ACID_AND_RELEASE_SAMUS => lower_acid_and_release_samus(game),
        function::START_FALLING_APART => start_falling_apart(game),
        function::FALLING_APART => falling_apart(game),
        function::STILL_FALLING_APART => still_falling_apart(game),
        function::DEAD => {}
        _ => return false,
    }
    true
}

/// Time is frozen AI - enemy $E17F (Ridley) ($A6:B28A). Was `Ridley_Func_2`.
pub fn time_frozen<L: Lease + ?Sized>(game: &mut L) {
    session_slot!(game, Ridley[RIDLEY].grabbed_lag_timer = 0);
    session_slot!(game, Ridley[RIDLEY].base.frame_counter = 1);
    hurt(game);
}

/// Hurt AI - enemy $E17F (Ridley) ($A6:B297): on even frames Ridley acts as in
/// `main`; then the grabbed Samus lags until the timer reaches its limit.
/// Was `Ridley_Hurt`.
pub fn hurt<L: Lease + ?Sized>(game: &mut L) {
    let animate = if session_slot!(game, Ridley[RIDLEY].base.frame_counter) & 1 != 0 {
        true
    } else {
        set_tangibility(game);
        lunge_if_power_bomb(game);
        game.foreign(|game| game.run_function(session_slot!(game, Ridley[RIDLEY].function)));
        if session_slot!(game, Ridley[RIDLEY].movement_enabled) != 0 {
            move_ridley(game);
            update_wings(game);
            handle_tail(game);
            true
        } else {
            false
        }
    };
    if animate {
        tail_projectile_collision(game);
        hurt_flash(game);
        draw_tail(game);
        draw_wings(game);
        health_palette(game);
        handle_ribs(game);
        let mut lag = session_slot!(game, Ridley[RIDLEY].grabbed_lag_timer).wrapping_add(1);
        if (lag as i16).wrapping_sub(GRABBED_HURT_LAG as i16) >= 0 {
            lag = GRABBED_HURT_LAG;
        }
        session_slot!(game, Ridley[RIDLEY].grabbed_lag_timer = lag);
        if (session_slot!(game, Ridley[RIDLEY].grabbed_lag_timer) as i16)
            .wrapping_sub(GRABBED_HURT_LAG as i16)
            < 0
        {
            return;
        }
    }
    if session_slot!(game, Ridley[RIDLEY].holding_samus) != 0 {
        move_samus_with_feet(game);
    }
}

/// Ridley grabbed Samus movement hurt AI lag time ($A6:B288).
const GRABBED_HURT_LAG: u16 = 8;

/// Ridley function - liftoff ($A6:B2F3). Was `Ridley_Func_3`.
pub fn liftoff<L: Lease + ?Sized>(game: &mut L) {
    let rect = Rect {
        x: 64,
        y: 256,
        w: 8,
        h: 8,
    };
    accelerate(0, 14, rect.x, rect.y, game);
    if !separated_from_rect(0, rect, game) {
        session_slot!(game, Ridley[RIDLEY].fight_mode = 1);
        session_slot!(game, Ridley[RIDLEY].function = function::DECIDE_ACTION);
        decide_action(game);
    }
}

/// Ridley function - decide action ($A6:B321): a random one of the eight
/// choices. Was `Ridley_Func_3b`.
pub fn decide_action<L: Lease + ?Sized>(game: &mut L) {
    let choices = fight_action_choices(game);
    let choice = 2 * (game.roll_random() & 7) as usize;
    let next = crate::cartridge::native_record(0xa6, choices, game).word(choice);
    session_slot!(game, Ridley[RIDLEY].function = next);
    game.foreign(|game| game.run_function(session_slot!(game, Ridley[RIDLEY].function)));
}

/// Get fight action choices ($A6:B335). Was `Ridley_Func_4`.
pub fn fight_action_choices<L: Lease + ?Sized>(game: &mut L) -> u16 {
    if game.slot_byte(SAMUS_MOVEMENT_TYPE) == SPIN_JUMPING {
        return actions::HOVER;
    }
    let health = session_slot!(game, Ridley[RIDLEY].base.health);
    if health == 0 {
        session_slot!(
            game,
            Ridley[RIDLEY].death_lunges =
                session_slot!(game, Ridley[RIDLEY].death_lunges).wrapping_add(1)
        );
        actions::LUNGE
    } else if below(health, 14400) {
        actions::POGO_OR_SWOOP
    } else if samus_in_pogo_zone(game) {
        actions::POGO
    } else if samus_not_spinning(game) {
        actions::LUNGE_OR_POGO
    } else if below(session_slot!(game, Ridley[RIDLEY].base.health), 9000) {
        actions::SWOOP_OR_POGO
    } else {
        actions::POGO_OR_SWOOP
    }
}

/// Ridley function - lunge missed - setup ($A6:B3EC). Was `Ridley_Func_5`.
pub fn lunge_missed<L: Lease + ?Sized>(game: &mut L) {
    session_slot!(
        game,
        Ridley[RIDLEY].function = function::LUNGE_MISSED_MOVE_TO_POSITION
    );
    session_slot!(game, Ridley[RIDLEY].function_timer = 128);
    lunge_missed_move_to_position(game);
}

/// Ridley function - lunge missed - move to position ($A6:B3F8). Was
/// `Ridley_Func_6`.
pub fn lunge_missed_move_to_position<L: Lease + ?Sized>(game: &mut L) {
    if timer_expired(game) || {
        let x = if session_slot!(game, Ridley[RIDLEY].facing) != 0 {
            96
        } else {
            192
        };
        let rect = Rect {
            x,
            y: 256,
            w: 8,
            h: 8,
        };
        accelerate(0, acceleration_factor(game), rect.x, rect.y, game);
        !separated_from_rect(0, rect, game)
    } {
        session_slot!(game, Ridley[RIDLEY].function = function::DECIDE_ACTION);
    }
}

/// Y = [$B439 + [Ridley acceleration index] * 2] ($A6:B42E): the acceleration
/// factor for lunge missed and dodging power bomb. Was `Ridley_Func_7`.
pub fn acceleration_factor<L: Lease + ?Sized>(game: &mut L) -> u16 {
    const FACTORS: [u16; 4] = [4, 8, 0xa, 0xc];
    FACTORS[session_slot!(game, Ridley[RIDLEY].acceleration_index) as usize]
}

/// Ridley action - swoop ($A6:B441). Was `Ridley_Func_8`.
pub fn swoop<L: Lease + ?Sized>(game: &mut L) {
    session_slot!(
        game,
        Ridley[RIDLEY].function = function::SWOOP_MOVE_TO_POSITION
    );
    session_slot!(game, Ridley[RIDLEY].timer = 10);
    session_slot!(game, Ridley[RIDLEY].swoop_angle = 0);
    swoop_move_to_position(game);
}

/// Ridley function - swoop - move to position ($A6:B455). Was `Ridley_B455`.
pub fn swoop_move_to_position<L: Lease + ?Sized>(game: &mut L) {
    let x = if session_slot!(game, Ridley[RIDLEY].facing) != 0 {
        64
    } else {
        192
    };
    let rect = Rect {
        x,
        y: 128,
        w: 8,
        h: 8,
    };
    accelerate(0, 1, rect.x, rect.y, game);
    if !separated_from_rect(0, rect, game) {
        session_slot!(game, Ridley[RIDLEY].function = function::SWOOP_AIMING_DOWN);
        session_slot!(game, Ridley[RIDLEY].timer = 32);
        session_slot!(game, Ridley[RIDLEY].swoop_angle = 0);
    }
}

/// One swoop stage: steer by `angle_step` towards `target_angle` at
/// `target_speed`, mirrored when facing left, until the timer runs out.
fn swoop_stage<L: Lease + ?Sized>(
    game: &mut L,
    angle_step: u16,
    target_angle: u16,
    target_speed: u16,
) -> bool {
    if session_slot!(game, Ridley[RIDLEY].facing) != 0 {
        calculate_swoop_velocities(angle_step, target_angle, target_speed, game);
    } else {
        calculate_swoop_velocities(
            angle_step.wrapping_neg(),
            target_angle.wrapping_neg(),
            target_speed,
            game,
        );
    }
    let timer = session_slot!(game, Ridley[RIDLEY].timer);
    if timer != 0 {
        session_slot!(game, Ridley[RIDLEY].timer = timer.wrapping_sub(1));
        false
    } else {
        true
    }
}

/// Ridley function - swoop - descending - aiming down ($A6:B493). Was
/// `Ridley_Func_9`.
pub fn swoop_aiming_down<L: Lease + ?Sized>(game: &mut L) {
    if swoop_stage(game, 32, 512, 1152) {
        session_slot!(
            game,
            Ridley[RIDLEY].function = function::SWOOP_AIMING_HORIZONTAL
        );
        session_slot!(game, Ridley[RIDLEY].timer = 20);
    }
}

/// Ridley function - swoop - descending - aiming horizontal ($A6:B4D1). Was
/// `Ridley_Func_10`.
pub fn swoop_aiming_horizontal<L: Lease + ?Sized>(game: &mut L) {
    if swoop_stage(game, 320, 0x4000, 1280) {
        session_slot!(game, Ridley[RIDLEY].function = function::SWOOP_AIMING_UP);
        session_slot!(game, Ridley[RIDLEY].timer = 16);
        word!(game.TILEMAP_STUFF[tail::WHIP_REQUEST] = 1);
    }
}

/// Ridley function - swoop - ascending - aiming up ($A6:B516). Was
/// `Ridley_Func_11`.
pub fn swoop_aiming_up<L: Lease + ?Sized>(game: &mut L) {
    if swoop_stage(game, 512, 30720, 768) {
        session_slot!(
            game,
            Ridley[RIDLEY].function = function::SWOOP_AIMING_UP_FASTER
        );
        session_slot!(game, Ridley[RIDLEY].timer = 32);
    }
}

/// Ridley function - swoop - ascending - aiming up faster ($A6:B554). Was
/// `Ridley_Func_12`.
pub fn swoop_aiming_up_faster<L: Lease + ?Sized>(game: &mut L) {
    if swoop_stage(game, 1024, 30720, 768) {
        session_slot!(game, Ridley[RIDLEY].function = function::SWOOP_DECELERATE);
        session_slot!(game, Ridley[RIDLEY].timer = 32);
        face_room_middle(game);
    }
}

/// Ridley function - swoop - ascending - decelerate ($A6:B594). Was
/// `Ridley_Func_13`.
pub fn swoop_decelerate<L: Lease + ?Sized>(game: &mut L) {
    calculate_swoop_velocities(0, 0x8000, 448, game);
    let timer = session_slot!(game, Ridley[RIDLEY].timer);
    if timer != 0 {
        session_slot!(game, Ridley[RIDLEY].timer = timer.wrapping_sub(1));
    } else {
        let next = if samus_not_spinning(game) {
            function::LUNGE
        } else {
            function::DECIDE_ACTION
        };
        session_slot!(game, Ridley[RIDLEY].function = next);
    }
}

/// Ridley action - hover ($A6:B5C4). Was `Ridley_Func_14`.
pub fn hover<L: Lease + ?Sized>(game: &mut L) {
    word! {game; TILEMAP_STUFF[tail::IDEAL_SEGMENT_ANGLE] = 11; TILEMAP_STUFF[tail::EXTENSION_SPEED] = 384; }
    session_slot!(
        game,
        Ridley[RIDLEY].function = function::HOVER_TO_WALL_BEHIND
    );
    session_slot!(
        game,
        Ridley[RIDLEY].function_timer = ((word!(game.RANDOM_NUMBER) & 0x1f) + 32)
    );
    hover_to_wall_behind(game);
}

/// Ridley function - hover - move to wall behind Ridley ($A6:B5E5). Was
/// `Ridley_Func_15`.
pub fn hover_to_wall_behind<L: Lease + ?Sized>(game: &mut L) {
    let x = ridley_data::hover_behind(game, session_slot!(game, Ridley[RIDLEY].facing) as isize);
    hover_movement(game, x, function::HOVER_TO_WALL_IN_FRONT);
}

/// Ridley function - hover - move to wall in front of Ridley ($A6:B613). Was
/// `Ridley_Func_16`.
pub fn hover_to_wall_in_front<L: Lease + ?Sized>(game: &mut L) {
    let x = ridley_data::hover_in_front(game, session_slot!(game, Ridley[RIDLEY].facing) as isize);
    hover_movement(game, x, function::HOVER_TO_WALL_BEHIND);
}

/// Ridley hover movement ($A6:B641): hover towards `x`; pogo unless Samus
/// spin jumps, and cross to the other wall (`timeout`) when the function
/// timer runs out. Was `advance_ridley_instruction_with_timeout`.
fn hover_movement<L: Lease + ?Sized>(game: &mut L, x: u16, timeout: u16) {
    if hover_towards(x, game) {
        set_up_pogo_from_hover(game);
    } else if timer_expired(game) {
        session_slot!(game, Ridley[RIDLEY].function = timeout);
        session_slot!(game, Ridley[RIDLEY].function_timer = 128);
        face_room_middle(game);
    }
}

/// Ridley hover movement ($A6:B641): the movement towards (`x`, Samus's Y, at
/// most 352), with a tail whip and sometimes fireballs while Samus spin
/// jumps; true when she does not, so the pogo starts. Was `Ridley_Func_17`.
pub fn hover_towards<L: Lease + ?Sized>(x: u16, game: &mut L) -> bool {
    let mut y = game.player_y();
    if !below(game.player_y(), 352) {
        y = 352;
    }
    accelerate(0, acceleration_factor(game), x, y, game);
    word!(game.TILEMAP_STUFF[tail::WHIP_REQUEST] = 1);
    if game.slot_byte(SAMUS_MOVEMENT_TYPE) != SPIN_JUMPING {
        return true;
    }
    if (word!(game.RANDOM_NUMBER) as u8) >= 0x80
        && session_slot!(game, Ridley[RIDLEY].roaring) == 0
        && session_slot!(game, Ridley[RIDLEY].facing) != FACING_FORWARD
    {
        set_instruction_list(instruction_list::SHOOT_FIREBALLS, game);
    }
    false
}

/// Set up Ridley pogo from hover ($A6:B68B). Was `Ridley_Func_18`.
pub fn set_up_pogo_from_hover<L: Lease + ?Sized>(game: &mut L) {
    word! {game; TILEMAP_STUFF[tail::EXTENSION_SPEED] = 240; TILEMAP_STUFF[tail::IDEAL_SEGMENT_ANGLE] = 16; TILEMAP_STUFF[tail::FUNCTION] = 1; }
    session_slot!(
        game,
        Ridley[RIDLEY].function = function::POGO_FLY_TO_POSITION
    );
}

/// Ridley action / function - pogo - fly to position ($A6:B6A7). Was
/// `Ridley_Func_19`.
pub fn pogo_fly_to_position<L: Lease + ?Sized>(game: &mut L) {
    if below(session_slot!(game, Ridley[RIDLEY].base.y_pos), 288) {
        face_room_middle(game);
        session_slot!(
            game,
            Ridley[RIDLEY].function = function::POGO_WAIT_AT_POSITION
        );
        session_slot!(game, Ridley[RIDLEY].function_timer = 32);
        pogo_wait_at_position(game);
    } else {
        let x = ridley_data::pogo_target(game, session_slot!(game, Ridley[RIDLEY].facing) as isize);
        accelerate(0, 0, x, 288, game);
    }
}

/// Ridley function - pogo - wait at position ($A6:B6DD). Was `Ridley_Func_20`.
pub fn pogo_wait_at_position<L: Lease + ?Sized>(game: &mut L) {
    accelerate(
        0,
        0,
        session_slot!(game, Ridley[RIDLEY].base.x_pos),
        288,
        game,
    );
    if timer_expired(game) {
        set_up_pogo_tail(game);
        game.foreign(set_pogo_speeds);
        session_slot!(game, Ridley[RIDLEY].function = function::POGO_DESCENDING);
        session_slot!(
            game,
            Ridley[RIDLEY].function_timer = ((word!(game.RANDOM_NUMBER) & 0x3f) + 128)
        );
    }
}

/// Ridley function - pogo - descending ($A6:B70E): grab Samus, or bounce off
/// the floor with a dust cloud, a quake and every other time fireballs. Was
/// `Ridley_Func_21`.
pub fn pogo_descending<L: Lease + ?Sized>(game: &mut L) {
    if samus_grabbable(game) {
        set_neutral_tail(game);
        grab_samus_from_pogo(game);
        return;
    }
    let mut y_velocity = (word!(game.TILEMAP_STUFF[tail::POGO_DOWN_ACCELERATION]) as i16)
        .wrapping_add(session_slot!(game, Ridley[RIDLEY].y_velocity) as i16);
    if !below(y_velocity as u16, 1536) {
        y_velocity = 1536;
    }
    session_slot!(game, Ridley[RIDLEY].y_velocity = y_velocity as u16);
    if tail_hits_floor(game) {
        let x = word!(game.TILEMAP_STUFF[tail::x(6)]);
        let y = word!(game.TILEMAP_STUFF[tail::y(6)]).wrapping_add(12);
        game.place_projectile_spawn(x, y);
        game.spawn_room_projectile(DUST_CLOUD, 9);
        game.queue_sound(1, 0x76, 6);
        word! {game; EARTHQUAKE_TYPE = 13; EARTHQUAKE_TIMER = 4; }
        set_pogo_x_direction(game);
        game.foreign(|game| {
            set_pogo_speeds(game);
            set_tail_segments_from_list(
                tail_address(tail::distance(0)),
                tail_data::DISTANCES,
                game,
            );
        });
        for segment in 0..7 {
            word!(game.TILEMAP_STUFF[tail::target_distance(segment)] = 3072);
        }
        word!(game.TILEMAP_STUFF[tail::FUNCTION] = 4);
        let mut bounces = session_slot!(game, Ridley[RIDLEY].pogo_bounces).wrapping_add(1);
        if !below(session_slot!(game, Ridley[RIDLEY].pogo_bounces), 1) {
            if session_slot!(game, Ridley[RIDLEY].facing) != FACING_FORWARD {
                set_instruction_list(instruction_list::SHOOT_FIREBALLS, game);
            }
            bounces = 0;
        }
        session_slot!(game, Ridley[RIDLEY].pogo_bounces = bounces);
        session_slot!(game, Ridley[RIDLEY].function = function::POGO_ASCENDING);
    }
}

/// Ridley function - pogo - ascending ($A6:B7B9). Was `Ridley_Func_22`.
pub fn pogo_ascending<L: Lease + ?Sized>(game: &mut L) {
    if !samus_in_pogo_zone(game) || {
        let expired =
            (session_slot!(game, Ridley[RIDLEY].function_timer) as i16).wrapping_sub(1) < 0;
        session_slot!(
            game,
            Ridley[RIDLEY].function_timer =
                session_slot!(game, Ridley[RIDLEY].function_timer).wrapping_sub(1)
        );
        expired
    } {
        set_neutral_tail(game);
        session_slot!(game, Ridley[RIDLEY].function = function::DECIDE_ACTION);
    } else {
        let y_velocity = session_slot!(game, Ridley[RIDLEY].y_velocity);
        let rising = (word!(game.TILEMAP_STUFF[tail::POGO_UP_ACCELERATION]) as i16)
            .wrapping_add(y_velocity as i16)
            < 0;
        session_slot!(
            game,
            Ridley[RIDLEY].y_velocity =
                word!(game.TILEMAP_STUFF[tail::POGO_UP_ACCELERATION]).wrapping_add(y_velocity)
        );
        if !rising {
            session_slot!(game, Ridley[RIDLEY].y_velocity = 0);
            session_slot!(game, Ridley[RIDLEY].function = function::POGO_DESCENDING);
        }
    }
}

/// Check for tail collision with floor ($A6:B7E7): the tip, then segments 5
/// down to 2, just below each. Was `Ridley_Func_23`.
pub fn tail_hits_floor<L: Lease + ?Sized>(game: &mut L) -> bool {
    [(6, 16), (5, 18), (4, 18), (3, 18), (2, 18)]
        .into_iter()
        .any(|(segment, below)| {
            let x = word!(game.TILEMAP_STUFF[tail::x(segment)]);
            let y = word!(game.TILEMAP_STUFF[tail::y(segment)]).wrapping_add(below);
            game.foreign(|game| solid_block_at(x, y, game))
        })
}

/// Set neutral Ridley tail ($A6:B84D). Was `Ridley_Func_24`.
pub fn set_neutral_tail<S: State + ?Sized>(game: &mut S) {
    word! { game, 1; TILEMAP_STUFF[tail::FUNCTION], TILEMAP_STUFF[tail::ANGLE_DELTA] }
}

/// Check if Samus is grabbable ($A6:B859). Was `Ridley_Func_25`.
pub fn samus_grabbable<L: Lease + ?Sized>(game: &mut L) -> bool {
    samus_not_spinning(game) && samus_touches_feet(4, 4, game)
}

/// Check for Samus collision with Ridley feet ($A6:B865), with feet radii
/// `x_radius` and `y_radius`. Was `Ridley_Func_26`.
pub fn samus_touches_feet<L: Lease + ?Sized>(x_radius: u16, y_radius: u16, game: &mut L) -> bool {
    let rect = Rect {
        x: session_slot!(game, Ridley[RIDLEY].base.x_pos).wrapping_add(ridley_data::feet_x(
            game,
            session_slot!(game, Ridley[RIDLEY].facing) as isize,
        )),
        y: session_slot!(game, Ridley[RIDLEY].base.y_pos).wrapping_add(ridley_data::feet_y(
            game,
            (session_slot!(game, Ridley[RIDLEY].feet_distance_index) >> 1) as isize,
        )),
        w: x_radius,
        h: y_radius,
    };
    samus_in_rect(rect, game)
}

/// Ridley grabs Samus from pogo ($A6:B889): Y velocity = -max(200h, |Y
/// velocity|). Was `Ridley_Func_27`.
pub fn grab_samus_from_pogo<L: Lease + ?Sized>(game: &mut L) {
    let mut y_velocity = session_slot!(game, Ridley[RIDLEY].y_velocity) as i16;
    if y_velocity >= 0 {
        y_velocity = -(y_velocity as i32) as i16;
    }
    if (y_velocity as i32 + 512) & 0x8000 == 0 {
        y_velocity = -512;
    }
    session_slot!(game, Ridley[RIDLEY].y_velocity = y_velocity as u16);
    set_neutral_tail(game);
    session_slot!(game, Ridley[RIDLEY].function = function::GRABBED_SAMUS);
    grabbed_samus(game);
}

/// Set Ridley pogo X movement direction ($A6:B8A9): towards the room inside
/// at the bounds, otherwise mostly towards Samus. Was `Ridley_Func_28`.
pub fn set_pogo_x_direction<L: Lease + ?Sized>(game: &mut L) {
    if session_slot!(game, Ridley[RIDLEY].x_velocity) == 0 {
        let x_velocity = if session_slot!(game, Ridley[RIDLEY].base.x_pos) & 0x80 != 0 {
            -0xc0i32
        } else {
            0xc0
        };
        session_slot!(game, Ridley[RIDLEY].x_velocity = x_velocity as u16);
    }
    let facing_away = ((session_slot!(game, Ridley[RIDLEY].x_velocity) as i32
        ^ (session_slot!(game, Ridley[RIDLEY].base.x_pos) as i32 - game.player_x() as i32))
        & 0x8000)
        == 0;
    let x = session_slot!(game, Ridley[RIDLEY].base.x_pos);
    if (x as i16).wrapping_sub(session_slot!(game, Ridley[RIDLEY].min_x) as i16) < 0 {
        if (session_slot!(game, Ridley[RIDLEY].x_velocity) & 0x8000) == 0 {
            return;
        }
    } else if (x as i16).wrapping_sub(session_slot!(game, Ridley[RIDLEY].max_x) as i16) >= 0 {
        if (session_slot!(game, Ridley[RIDLEY].x_velocity) & 0x8000) != 0 {
            return;
        }
    } else if facing_away == (word!(game.RANDOM_NUMBER) < 0x555) {
        return;
    }
    session_slot!(
        game,
        Ridley[RIDLEY].x_velocity = -(session_slot!(game, Ridley[RIDLEY].x_velocity) as i32) as u16
    );
}

/// Move Samus with Ridley's feet ($A6:B9E1): the grabbed offsets settle 4px
/// a frame towards the feet. Was `Ridley_Func_30`.
pub fn move_samus_with_feet<L: Lease + ?Sized>(game: &mut L) {
    if let Some(offset) = settle_offset(session_slot!(game, Ridley[RIDLEY].grabbed_x_offset)) {
        session_slot!(game, Ridley[RIDLEY].grabbed_x_offset = offset);
    }
    if let Some(offset) = settle_offset(session_slot!(game, Ridley[RIDLEY].grabbed_y_offset)) {
        session_slot!(game, Ridley[RIDLEY].grabbed_y_offset = offset);
    }
    word! { game;
        SAMUS_X_POS = (session_slot!(game, Ridley[RIDLEY].grabbed_x_offset) as i32
            + session_slot!(game, Ridley[RIDLEY].base.x_pos) as i32
            + ridley_data::feet_x(game, session_slot!(game, Ridley[RIDLEY].facing) as isize) as i32)
            as u16;
        SAMUS_Y_POS = (session_slot!(game, Ridley[RIDLEY].grabbed_y_offset) as i32
            + session_slot!(game, Ridley[RIDLEY].base.y_pos) as i32
            + ridley_data::feet_y(game, (session_slot!(game, Ridley[RIDLEY].feet_distance_index) >> 1) as isize) as i32)
            as u16;
    }
}

/// A grabbed offset 4px closer to 0, or `None` when it is already 0. Was
/// `reduce_ridley_axis_speed`.
fn settle_offset(offset: u16) -> Option<u16> {
    if offset == 0 {
        return None;
    }
    let magnitude = (abs16(offset) as i16).wrapping_sub(4);
    Some(if magnitude < 0 {
        0
    } else if offset & 0x8000 != 0 {
        -(magnitude as i32) as u16
    } else {
        magnitude as u16
    })
}

/// Set grabbed Samus X/Y offset ($A6:BA54). Was `Ridley_Func_31`.
pub fn set_grabbed_samus_offset<L: Lease + ?Sized>(game: &mut L) {
    session_slot!(
        game,
        Ridley[RIDLEY].grabbed_x_offset = (game.player_x() as i32
            - (session_slot!(game, Ridley[RIDLEY].base.x_pos) as i32
                + ridley_data::feet_x(game, session_slot!(game, Ridley[RIDLEY].facing) as isize)
                    as i32)) as u16
    );
    session_slot!(
        game,
        Ridley[RIDLEY].grabbed_y_offset = (game.player_y() as i32
            - (session_slot!(game, Ridley[RIDLEY].base.y_pos) as i32
                + ridley_data::feet_y(
                    game,
                    (session_slot!(game, Ridley[RIDLEY].feet_distance_index) >> 1) as isize
                ) as i32)) as u16
    );
}

/// Ridley lunge missed Samus ($A6:BA85): dodge a power bomb, retreat, or after
/// ten lunges at no health, die. Was `Ridley_Func_32`.
pub fn lunge_missed_samus<L: Lease + ?Sized>(game: &mut L) {
    if below(session_slot!(game, Ridley[RIDLEY].death_lunges), 10) {
        if word!(game.POWER_BOMB_FLAG) != 0 {
            session_slot!(game, Ridley[RIDLEY].function = function::DODGE_POWER_BOMB);
            dodge_power_bomb(game);
        } else {
            session_slot!(game, Ridley[RIDLEY].function = function::LUNGE_MISSED);
            word!(game.TILEMAP_STUFF[tail::WHIP_REQUEST] = 1);
        }
    } else {
        start_death_sequence(game);
        session_slot!(game, Ridley[RIDLEY].function = function::DEATH_ROAR);
        death_roar(game);
    }
}

/// Ridley action / function - lunge ($A6:BAB7): while Samus is below and on
/// the lunging side, home in on her and grab her when the feet touch. Was
/// `Ridley_Func_33`.
pub fn lunge<L: Lease + ?Sized>(game: &mut L) {
    if !(samus_not_spinning(game)
        && session_slot!(game, Ridley[RIDLEY].hit_boundary) == 0
        && {
            let side =
                ridley_data::lunge_side(game, session_slot!(game, Ridley[RIDLEY].facing) as isize);
            let dx = session_slot!(game, Ridley[RIDLEY].base.x_pos).wrapping_sub(game.player_x());
            ((side ^ dx) & 0x8000) == 0 || below(abs16(dx), 32)
        }
        && ((session_slot!(game, Ridley[RIDLEY].base.y_pos) as i32 + 35 - game.player_y() as i32)
            as i16)
            < 0)
    {
        lunge_missed_samus(game);
        return;
    }
    let lead: i16 = if session_slot!(game, Ridley[RIDLEY].facing) != 0 {
        16
    } else {
        -16
    };
    accelerate(
        0,
        ridley_data::lunge_acceleration(
            game,
            session_slot!(game, Ridley[RIDLEY].acceleration_index) as isize,
        ),
        game.player_x().wrapping_add(lead as u16),
        game.player_y().wrapping_sub(4),
        game,
    );
    let rect = Rect {
        x: session_slot!(game, Ridley[RIDLEY].base.x_pos).wrapping_add(ridley_data::feet_x(
            game,
            session_slot!(game, Ridley[RIDLEY].facing) as isize,
        )),
        y: session_slot!(game, Ridley[RIDLEY].base.y_pos).wrapping_add(35),
        w: 8,
        h: 12,
    };
    if samus_in_rect(rect, game) {
        session_slot!(
            game,
            Ridley[RIDLEY].y_velocity =
                -(session_slot!(game, Ridley[RIDLEY].y_velocity) as i32) as u16
        );
        if session_slot!(game, Ridley[RIDLEY].base.health) != 0 {
            if word!(game.POWER_BOMB_FLAG) != 0 {
                grab_samus(game);
                session_slot!(game, Ridley[RIDLEY].function = function::DODGE_POWER_BOMB);
                dodge_power_bomb(game);
            } else {
                grabbed_samus(game);
            }
        } else {
            if session_slot!(game, Ridley[RIDLEY].holding_samus) == 0 {
                grab_samus(game);
            }
            start_death_sequence(game);
            session_slot!(
                game,
                Ridley[RIDLEY].function = function::DYING_GRABBED_SAMUS
            );
            dying_grabbed_samus(game);
        }
    }
}

/// Ridley function - grabbed Samus - setup ($A6:BB8F). Was `Ridley_Func_33b`.
pub fn grabbed_samus<L: Lease + ?Sized>(game: &mut L) {
    /// Ridley lunge target X position ($A6:BBEB), by facing direction.
    const TARGET_X: [u16; 3] = [0x40, 0, 0xd0];
    session_slot!(
        game,
        Ridley[RIDLEY].target_x = TARGET_X[session_slot!(game, Ridley[RIDLEY].facing) as usize]
    );
    let mut y = (session_slot!(game, Ridley[RIDLEY].base.y_pos) as i16).wrapping_sub(64);
    if below(session_slot!(game, Ridley[RIDLEY].base.y_pos), 320) {
        y = 256;
    }
    session_slot!(game, Ridley[RIDLEY].target_y = y as u16);
    if session_slot!(game, Ridley[RIDLEY].holding_samus) == 0 {
        grab_samus(game);
    }
    session_slot!(
        game,
        Ridley[RIDLEY].function = function::GRABBED_SAMUS_RISING
    );
    session_slot!(game, Ridley[RIDLEY].function_timer = 32);
    grabbed_samus_rising(game);
}

/// Ridley function - grabbed Samus - rising to up to 40h px ($A6:BBC4). Was
/// `Ridley_Func_34`.
pub fn grabbed_samus_rising<L: Lease + ?Sized>(game: &mut L) {
    let x = session_slot!(game, Ridley[RIDLEY].target_x);
    let y = session_slot!(game, Ridley[RIDLEY].target_y);
    accelerate(0, 0, x, y, game);
    if timer_expired(game) {
        session_slot!(
            game,
            Ridley[RIDLEY].function = function::GRABBED_SAMUS_RISING_TO_MIDDLE
        );
        session_slot!(game, Ridley[RIDLEY].function_timer = 32);
    }
}

/// Ridley function - grabbed Samus - rising to room middle ($A6:BBF1). Was
/// `Ridley_Func_35`.
pub fn grabbed_samus_rising_to_middle<L: Lease + ?Sized>(game: &mut L) {
    if timer_expired(game) {
        word! {game; TILEMAP_STUFF[tail::IDEAL_SEGMENT_ANGLE] = 8; TILEMAP_STUFF[tail::EXTENSION_SPEED] = 240; }
        release_samus(game);
        session_slot!(
            game,
            Ridley[RIDLEY].function = function::GRABBED_SAMUS_RELEASE
        );
        session_slot!(game, Ridley[RIDLEY].function_timer = 64);
    } else {
        accelerate(
            0,
            0,
            session_slot!(game, Ridley[RIDLEY].target_x),
            256,
            game,
        );
    }
}

/// Ridley function - grabbed Samus - release Samus ($A6:BC2E). Was
/// `Ridley_Func_36`.
pub fn grabbed_samus_release<L: Lease + ?Sized>(game: &mut L) {
    /// The X position Ridley flies to after releasing Samus ($A6:BC62), by
    /// facing direction.
    const TARGET_X: [u16; 3] = [0xb0, 0, 0x50];
    if timer_expired(game) {
        word! {game; TILEMAP_STUFF[tail::IDEAL_SEGMENT_ANGLE] = 16; TILEMAP_STUFF[tail::EXTENSION_SPEED] = 240; }
        session_slot!(game, Ridley[RIDLEY].function = function::DECIDE_ACTION);
    } else {
        let x = TARGET_X[session_slot!(game, Ridley[RIDLEY].facing) as usize];
        accelerate(0, 0, x, 224, game);
    }
}

/// Grab Samus ($A6:BC68). Was `Ridley_Func_37`.
pub fn grab_samus<L: Lease + ?Sized>(game: &mut L) {
    set_grabbed_samus_offset(game);
    session_slot!(
        game,
        Ridley[RIDLEY].base.properties =
            session_slot!(game, Ridley[RIDLEY].base.properties) | K_ENEMY_PROPS_TANGIBLE as u16
    );
    session_slot!(game, Ridley[RIDLEY].holding_samus = 1);
    game.foreign(|game| game.command_player(LOCK_SAMUS));
    draw_feet(true, game);
}

/// Release Samus ($A6:BC84). Was `Ridley_Func_38`.
pub fn release_samus<L: Lease + ?Sized>(game: &mut L) {
    word! { game, 1; TILEMAP_STUFF[tail::WHIP_REQUEST], TILEMAP_STUFF[tail::FUNCTION] }
    if (session_slot!(game, Ridley[RIDLEY].fight_mode) & 0x8000) == 0 {
        let timer = if samus_morphed(game) { 6 } else { 10 };
        session_slot!(game, Ridley[RIDLEY].intangibility_timer = timer);
    }
    session_slot!(game, Ridley[RIDLEY].holding_samus = 0);
    game.foreign(|game| game.command_player(UNLOCK_SAMUS));
    draw_feet(false, game);
}

/// Set Ridley tangibility ($A6:BCB4): intangible off-screen while fighting,
/// and tangible again when the intangibility timer runs out. Was
/// `Ridley_Func_39`.
pub fn set_tangibility<S: State + ?Sized>(game: &mut S) {
    let mode = session_slot!(game, Ridley[RIDLEY].fight_mode) as i16;
    if mode < 0 {
        return;
    }
    if mode != 0 {
        if off_screen(game) {
            set_tangible(game, false);
            return;
        }
        set_tangible(game, true);
    }
    let timer = session_slot!(game, Ridley[RIDLEY].intangibility_timer);
    if timer != 0 {
        let timer = timer.wrapping_sub(1);
        session_slot!(game, Ridley[RIDLEY].intangibility_timer = timer);
        if timer == 0 && (session_slot!(game, Ridley[RIDLEY].fight_mode) & 0x8000) == 0 {
            set_tangible(game, true);
        }
    }
}

fn set_tangible<S: State + ?Sized>(game: &mut S, tangible: bool) {
    let properties = session_slot!(game, Ridley[RIDLEY].base.properties);
    let properties = if tangible {
        (properties as i32 & !(K_ENEMY_PROPS_TANGIBLE as i32)) as u16
    } else {
        properties | K_ENEMY_PROPS_TANGIBLE as u16
    };
    session_slot!(game, Ridley[RIDLEY].base.properties = properties);
}

/// Check if Samus is morphed or spin jumping / damage boosting ($A6:BCF1):
/// the carry, Samus is neither spin jumping nor damage boosting. Was
/// `Ridley_Func_40_Carry`.
pub fn samus_not_spinning<L: Lease + ?Sized>(game: &mut L) -> bool {
    MOVEMENT_TYPE_FLAGS[game.slot_byte(SAMUS_MOVEMENT_TYPE) as usize] & 0x80 != 0
}

/// Check if Samus is morphed or spin jumping / damage boosting ($A6:BCF1):
/// the overflow, Samus is morphed. Was `Ridley_Func_40_Sign`.
pub fn samus_morphed<L: Lease + ?Sized>(game: &mut L) -> bool {
    MOVEMENT_TYPE_FLAGS[game.slot_byte(SAMUS_MOVEMENT_TYPE) as usize] & 0x40 != 0
}

/// $A6:BD04, by Samus movement type: bit 7 not spin jumping or damage
/// boosting, bit 6 morphed.
const MOVEMENT_TYPE_FLAGS: [u8; 28] = [
    0x80, 0x80, 0x80, 0, 0xff, 0x80, 0x80, 0xff, 0xff, 0xff, 0x80, 0, 0, 0x80, 0x80, 0x80, 0x80,
    0xff, 0xff, 0xff, 0x80, 0x80, 0, 0x80, 0x80, 0, 0, 0x80,
];

/// Check if Samus is in pogo zone ($A6:BD20): at or below Y 352. Was
/// `Ridley_Func_41`.
pub fn samus_in_pogo_zone<L: Lease + ?Sized>(game: &mut L) -> bool {
    ((game.player_y() as i32 - 352) & 0x8000) == 0
}

/// Lunge if power bomb active and not holding Samus ($A6:BD2C). Was
/// `Ridley_Func_42`.
pub fn lunge_if_power_bomb<S: State + ?Sized>(game: &mut S) {
    let mode = session_slot!(game, Ridley[RIDLEY].fight_mode) as i16;
    if mode > 0
        && mode != 2
        && word!(game.POWER_BOMB_FLAG) != 0
        && session_slot!(game, Ridley[RIDLEY].holding_samus) == 0
    {
        set_neutral_tail(game);
        session_slot!(game, Ridley[RIDLEY].function = function::LUNGE);
    }
}

/// Ridley function - dodging power bomb ($A6:BD4E): to the corner away from
/// the explosion, then back to the fight. Was `Ridley_Func_43`.
pub fn dodge_power_bomb<L: Lease + ?Sized>(game: &mut L) {
    if word!(game.POWER_BOMB_FLAG) != 0 {
        session_slot!(game, Ridley[RIDLEY].fight_mode = 2);
        let x = if below(word!(game.POWER_BOMB_EXPLOSION_X_POS), 128) {
            192
        } else {
            80
        };
        let y = if below(word!(game.POWER_BOMB_EXPLOSION_Y_POS), 256) {
            384
        } else {
            192
        };
        accelerate(0, acceleration_factor(game), x, y, game);
    } else {
        session_slot!(game, Ridley[RIDLEY].fight_mode = 1);
        let next = if session_slot!(game, Ridley[RIDLEY].holding_samus) != 0 {
            function::GRABBED_SAMUS
        } else {
            function::DECIDE_ACTION
        };
        session_slot!(game, Ridley[RIDLEY].function = next);
    }
}

/// Ridley function - fake retreat - move to position ($A6:BD9A). Was
/// `Ridley_Func_44`.
pub fn fake_retreat_move_to_position<L: Lease + ?Sized>(game: &mut L) {
    ceres_accelerate(0, 1, 192, 128, game);
    if !below(session_slot!(game, Ridley[RIDLEY].base.x_pos), 192) {
        session_slot!(
            game,
            Ridley[RIDLEY].function = function::FAKE_RETREAT_RISING
        );
    }
}

/// Ridley function - fake retreat - rising ($A6:BDBC): off the top of the
/// screen, then drop the baby metroid. Was `Ridley_Func_45`.
pub fn fake_retreat_rising<L: Lease + ?Sized>(game: &mut L) {
    session_slot!(game, Ridley[RIDLEY].min_y = -192i32 as u16);
    ceres_accelerate(0, 1, 192, -128i32 as u16, game);
    if below(session_slot!(game, Ridley[RIDLEY].base.y_pos), 32) {
        session_slot!(
            game,
            Ridley[RIDLEY].baby_metroid_function = baby_metroid_function::START_FALLING
        );
        session_slot!(game, Ridley[RIDLEY].function = function::DROP_BABY_METROID);
        session_slot!(game, Ridley[RIDLEY].function_timer = 21);
    }
}

/// Ridley function - fake retreat - drop baby metroid ($A6:BDF2). Was
/// `Ridley_Func_46`.
pub fn drop_baby_metroid<L: Lease + ?Sized>(game: &mut L) {
    if timer_expired(game) {
        set_instruction_list(instruction_list::RETRIEVE_BABY_METROID, game);
        session_slot!(
            game,
            Ridley[RIDLEY].function = function::RETRIEVE_BABY_METROID
        );
        retrieve_baby_metroid(game);
    }
}

/// Ridley function - fake retreat - retrieve baby metroid ($A6:BE03): swoop
/// down and catch it in the feet, then retreat for real. Was
/// `Ridley_Func_47`.
pub fn retrieve_baby_metroid<L: Lease + ?Sized>(game: &mut L) {
    let x = session_slot!(game, Ridley[RIDLEY].baby_metroid_x).wrapping_sub(10);
    let y = session_slot!(game, Ridley[RIDLEY].baby_metroid_y).wrapping_sub(56);
    accelerate(0, 12, x, y, game);
    let rect = Rect {
        x: session_slot!(game, Ridley[RIDLEY].base.x_pos).wrapping_add(14),
        y: session_slot!(game, Ridley[RIDLEY].base.y_pos).wrapping_add(66),
        w: 4,
        h: 4,
    };
    if baby_metroid_in_rect(rect, game) {
        session_slot!(game, Ridley[RIDLEY].holding_samus = 1);
        draw_feet(true, game);
        session_slot!(game, Ridley[RIDLEY].y_velocity = -512i32 as u16);
        session_slot!(
            game,
            Ridley[RIDLEY].baby_metroid_function = baby_metroid_function::HELD_BY_FEET
        );
        session_slot!(
            game,
            Ridley[RIDLEY].function = function::CERES_REAL_RETREAT_RISING
        );
    }
}

/// Check for baby metroid collision with rectangle ($A6:BE61); the baby
/// metroid's radii are 4. Was `Ridley_Func_48`.
pub fn baby_metroid_in_rect<L: Lease + ?Sized>(rect: Rect, game: &mut L) -> bool {
    let dx = abs16(session_slot!(game, Ridley[RIDLEY].baby_metroid_x).wrapping_sub(rect.x));
    if dx < 4 || dx.wrapping_sub(4) < rect.w {
        let dy = abs16(session_slot!(game, Ridley[RIDLEY].baby_metroid_y).wrapping_sub(rect.y));
        if dy < 4 || dy.wrapping_sub(4) < rect.h {
            return true;
        }
    }
    false
}

/// Run the bank $A6 baby metroid instruction `instruction`, whose arguments
/// start at `arguments`; the next instruction's address.
fn run_baby_metroid_instruction<L: Lease + ?Sized>(
    game: &mut L,
    instruction: u16,
    arguments: u16,
) -> u16 {
    match instruction {
        baby_metroid_instruction::GOTO_IF_STILL => baby_metroid_goto_if_still(arguments, game),
        baby_metroid_instruction::LOAD_PALETTE => baby_metroid_load_palette(arguments, game),
        baby_metroid_instruction::GOTO_IF_MOVING => baby_metroid_goto_if_moving(arguments, game),
        baby_metroid_instruction::GOTO => baby_metroid_goto(arguments, game),
        _ => {
            game.foreign(|game| game.unreachable());
            0
        }
    }
}

/// Run Ridley's bank $A6 instruction at `ea` with its arguments at `script`;
/// the script back when it is not one of his.
pub fn instruction<L: Lease + ?Sized>(
    ea: u32,
    script: EnemyScript,
    game: &mut L,
) -> Result<EnemyScript, EnemyScript> {
    if ea >> 16 != 0xa6 {
        return Err(script);
    }
    Ok(match ea as u16 {
        instruction::ROAR => roar(script, game),
        instruction::STOP_ROARING => stop_roaring(script, game),
        instruction::GOTO_IF_SAMUS_HEALTH_LOW => goto_if_samus_health_low(script, game),
        instruction::GOTO_BY_HOLDING_SAMUS => goto_by_holding_samus(script, game),
        instruction::GOTO_IF_NOT_HOLDING_SAMUS => goto_if_not_holding_samus(script, game),
        instruction::SET_FEET_DISTANCE => set_feet_distance(script, game),
        instruction::GOTO_IF_NOT_FACING_LEFT => goto_if_not_facing_left(script, game),
        instruction::MOVE_BY => move_by(script, game),
        instruction::FLIP_LEFT => flip_left(script, game),
        instruction::FACE_FORWARD => face_forward(script, game),
        instruction::FLIP_RIGHT => flip_right(script, game),
        instruction::CALCULATE_FIREBALL_VELOCITIES => calculate_fireball_velocities(script, game),
        instruction::FIREBALL_WITH_AFTERBURN => fireball_with_afterburn(script, game),
        instruction::FIREBALL_WITHOUT_AFTERBURN => fireball_without_afterburn(script, game),
        instruction::START_CERES_LIFTOFF => start_ceres_liftoff(script, game),
        instruction::START_NORFAIR_LIFTOFF => start_norfair_liftoff(script, game),
        _ => return Err(script),
    })
}

/// Execute baby metroid function ($A6:BE93). Was `Ridley_Func_49`.
pub fn run_baby_metroid_function<L: Lease + ?Sized>(game: &mut L) {
    match session_slot!(game, Ridley[RIDLEY].baby_metroid_function) {
        baby_metroid_function::HELD => baby_metroid_held(game),
        baby_metroid_function::HELD_BY_FEET => baby_metroid_held_by_feet(game),
        baby_metroid_function::START_FALLING => baby_metroid_start_falling(game),
        baby_metroid_function::FALLING => baby_metroid_falling(game),
        baby_metroid_function::NOTHING => {}
        _ => game.foreign(|game| game.unreachable()),
    }
}

/// Baby metroid function - held by Ridley ($A6:BE9C). Was `Ridley_Func_50`.
pub fn baby_metroid_held<L: Lease + ?Sized>(game: &mut L) {
    session_slot!(
        game,
        Ridley[RIDLEY].baby_metroid_x =
            session_slot!(game, Ridley[RIDLEY].base.x_pos).wrapping_sub(16)
    );
    session_slot!(
        game,
        Ridley[RIDLEY].baby_metroid_y =
            session_slot!(game, Ridley[RIDLEY].base.y_pos).wrapping_add(22)
    );
}

/// Baby metroid function - held by Ridley's feet ($A6:BEB3). Was
/// `Ridley_Func_51`.
pub fn baby_metroid_held_by_feet<L: Lease + ?Sized>(game: &mut L) {
    session_slot!(
        game,
        Ridley[RIDLEY].baby_metroid_x =
            session_slot!(game, Ridley[RIDLEY].base.x_pos).wrapping_add(14)
    );
    session_slot!(
        game,
        Ridley[RIDLEY].baby_metroid_y =
            session_slot!(game, Ridley[RIDLEY].base.y_pos).wrapping_add(66)
    );
}

/// Baby metroid function - start falling ($A6:BECA). Was `Ridley_Func_52`.
pub fn baby_metroid_start_falling<L: Lease + ?Sized>(game: &mut L) {
    session_slot!(game, Ridley[RIDLEY]; baby_metroid_unread = 0, baby_metroid_y_velocity = 0);
    session_slot!(
        game,
        Ridley[RIDLEY].baby_metroid_function = baby_metroid_function::FALLING
    );
    baby_metroid_falling(game);
}

/// Baby metroid function - falling ($A6:BEDC): accelerate down to the floor
/// at Y 192. Was `Ridley_Func_53`.
pub fn baby_metroid_falling<L: Lease + ?Sized>(game: &mut L) {
    let velocity = session_slot!(game, Ridley[RIDLEY].baby_metroid_y_velocity).wrapping_add(8);
    session_slot!(game, Ridley[RIDLEY].baby_metroid_y_velocity = velocity);
    let subpixel = session_slot!(byte game, Ridley[RIDLEY].baby_metroid_y_subpixel[1]) as i32
        + (velocity as i32 & 0xff);
    session_slot!(byte game, Ridley[RIDLEY].baby_metroid_y_subpixel[1] = subpixel as u8);
    session_slot!(
        game,
        Ridley[RIDLEY].baby_metroid_y = (session_slot!(game, Ridley[RIDLEY].baby_metroid_y) as i32
            + ((velocity >> 8) as i8 as i32 + (subpixel >> 8)))
            as u16
    );
    if !below(session_slot!(game, Ridley[RIDLEY].baby_metroid_y), 192) {
        session_slot!(game, Ridley[RIDLEY].baby_metroid_y = 192);
        session_slot!(
            game,
            Ridley[RIDLEY].baby_metroid_function = baby_metroid_function::NOTHING
        );
    }
}

/// Draw baby metroid ($A6:BF1A), in palette 0. Was `DrawBabyMetroid_0`.
pub fn draw_baby_metroid<L: Lease + ?Sized>(game: &mut L) {
    let Some(spritemap) = baby_metroid_spritemap(game) else {
        return;
    };
    let x = session_slot!(game, Ridley[RIDLEY].baby_metroid_x);
    let y = session_slot!(game, Ridley[RIDLEY].baby_metroid_y);
    add_spritemap(spritemap, x, y, 0, game);
}

/// Baby metroid instruction - go to [[X]] with 50% chance if not moving
/// ($A6:BFC9), else the flapping sound. Was `BabyMetroid_Instr_2`.
pub fn baby_metroid_goto_if_still<L: Lease + ?Sized>(k: u16, game: &mut L) -> u16 {
    if session_slot!(game, Ridley[RIDLEY].baby_metroid_y_velocity) == 0
        && (word!(game.RANDOM_NUMBER) & 1) != 0
    {
        return baby_metroid_goto(k, game);
    }
    game.queue_sound(2, 0x24, 6);
    k.wrapping_add(2)
}

/// Baby metroid instruction - load palette data [[X]] ($A6:BFE1). Was
/// `BabyMetroid_Instr_3`.
pub fn baby_metroid_load_palette<L: Lease + ?Sized>(k: u16, game: &mut L) -> u16 {
    let colours = crate::cartridge::native_record(0xa6, k, game).word(0);
    game.foreign(|game| copy_rom_colors_to_palette(0x162, 0xa6, colours, 0xf, game));
    k.wrapping_add(2)
}

/// Baby metroid instruction - go to [[X]] if moving ($A6:BFF2). Was
/// `BabyMetroid_Instr_1`.
pub fn baby_metroid_goto_if_moving<L: Lease + ?Sized>(k: u16, game: &mut L) -> u16 {
    if session_slot!(game, Ridley[RIDLEY].baby_metroid_y_velocity) != 0 {
        baby_metroid_goto(k, game)
    } else {
        k.wrapping_add(2)
    }
}

/// Baby metroid instruction - go to [[X]] ($A6:BFF8). Was `BabyMetroid_Goto`.
pub fn baby_metroid_goto<L: Lease + ?Sized>(k: u16, game: &mut L) -> u16 {
    crate::cartridge::native_record(0xa6, k, game).word(0)
}

/// Ridley function - dying - grabbed Samus ($A6:C538). Was `Ridley_Func_63`.
pub fn dying_grabbed_samus<L: Lease + ?Sized>(game: &mut L) {
    if move_to_death_spot(game) {
        death_roar(game);
    }
}

/// Ridley function - death sequence - death roar ($A6:C53E). Was
/// `Ridley_C53E`.
pub fn death_roar<L: Lease + ?Sized>(game: &mut L) {
    set_instruction_list(instruction_list::DEATH_ROAR, game);
    session_slot!(
        game,
        Ridley[RIDLEY].function = function::DEATH_MOVE_TO_DEATH_SPOT
    );
    session_slot!(game, Ridley[RIDLEY].function_timer = 32);
}

/// Ridley function - death sequence - move to death spot ($A6:C551): then stop
/// and lower the acid. Was `Ridley_C551`.
pub fn death_move_to_death_spot<L: Lease + ?Sized>(game: &mut L) {
    move_to_death_spot(game);
    if timer_expired(game) {
        session_slot!(game, Ridley[RIDLEY]; x_velocity = 0, y_velocity = 0);
        game.foreign(|game| game.move_liquid(528, 64, 1));
        session_slot!(game, Ridley[RIDLEY]; small_explosion_timer = 0, small_explosion_index = 0);
        session_slot!(
            game,
            Ridley[RIDLEY].function = function::LOWER_ACID_AND_RELEASE_SAMUS
        );
        session_slot!(game, Ridley[RIDLEY].function_timer = 160);
        lower_acid_and_release_samus(game);
    }
}

/// Ridley function - death sequence - lowering acid and release Samus
/// ($A6:C588). Was `Ridley_Func_64`.
pub fn lower_acid_and_release_samus<L: Lease + ?Sized>(game: &mut L) {
    spawn_small_explosion(game);
    if timer_expired(game) {
        if session_slot!(game, Ridley[RIDLEY].holding_samus) != 0 {
            release_samus(game);
        }
        session_slot!(
            game,
            Ridley[RIDLEY].function = function::START_FALLING_APART
        );
        session_slot!(game, Ridley[RIDLEY]; x_velocity = 0, y_velocity = 0);
        spawn_explosions(game);
    }
}

/// Ridley function - death sequence - start falling apart ($A6:C5A8). Was
/// `Ridley_Func_65`.
pub fn start_falling_apart<L: Lease + ?Sized>(game: &mut L) {
    spawn_small_explosion(game);
    session_slot!(game, Ridley[RIDLEY].movement_enabled = 0);
    session_slot!(
        game,
        Ridley[RIDLEY].base.properties =
            session_slot!(game, Ridley[RIDLEY].base.properties) | K_ENEMY_PROPS_INVISIBLE as u16
    );
    session_slot!(game, Ridley[RIDLEY].function = function::FALLING_APART);
    session_slot!(game, Ridley[RIDLEY].function_timer = 32);
}

/// Ridley function - death sequence - falling apart ($A6:C5C8). Was
/// `Ridley_Func_66`.
pub fn falling_apart<L: Lease + ?Sized>(game: &mut L) {
    if timer_expired(game) {
        session_slot!(
            game,
            Ridley[RIDLEY].function = function::STILL_FALLING_APART
        );
        session_slot!(game, Ridley[RIDLEY].function_timer = 256);
    }
}

/// Ridley function - death sequence - still falling apart ($A6:C5DA): the boss
/// is defeated, his drops scatter and the music returns. Was
/// `Ridley_Func_67`.
pub fn still_falling_apart<L: Lease + ?Sized>(game: &mut L) {
    if timer_expired(game) {
        // The original reads an undefined X for the drop's enemy index; it is 0.
        game.foreign(|game| {
            game.warn_x_undefined();
            game.set_boss_bit(1);
            game.scatter_enemy_drop(DropSource::Ridley, 0);
            game.queue_music_delayed(3);
        });
        session_slot!(
            game,
            Ridley[RIDLEY].base.properties =
                session_slot!(game, Ridley[RIDLEY].base.properties) | K_ENEMY_PROPS_DELETED as u16
        );
        session_slot!(game, Ridley[RIDLEY].function = function::DEAD);
    }
}

/// Move Ridley to death spot ($A6:C601); true once he is there. Was
/// `Ridley_Func_68`.
pub fn move_to_death_spot<L: Lease + ?Sized>(game: &mut L) -> bool {
    let rect = Rect {
        x: 128,
        y: 328,
        w: 4,
        h: 4,
    };
    accelerate_with_force(0, 0, 0x10, rect.x, rect.y, game);
    !separated_from_rect(0, rect, game)
}

/// Spawn small explosion near Ridley ($A6:C623): a dust cloud every five
/// frames, cycling through ten offsets. Was `Ridley_Func_69`.
pub fn spawn_small_explosion<L: Lease + ?Sized>(game: &mut L) {
    /// The explosion offsets ($A6:C66E): X and Y pairs.
    const OFFSETS: [i16; 20] = [
        -24, -24, -20, 20, 16, -30, 30, -3, 14, -13, -2, 18, -2, -32, -31, 8, -4, -10, 19, 19,
    ];
    let timer = (session_slot!(game, Ridley[RIDLEY].small_explosion_timer) as i16).wrapping_sub(1);
    if timer >= 0 {
        session_slot!(game, Ridley[RIDLEY].small_explosion_timer = timer as u16);
        return;
    }
    session_slot!(game, Ridley[RIDLEY].small_explosion_timer = 4);
    let mut index = session_slot!(game, Ridley[RIDLEY].small_explosion_index).wrapping_add(1);
    if !below(session_slot!(game, Ridley[RIDLEY].small_explosion_index), 9) {
        index = 0;
    }
    session_slot!(game, Ridley[RIDLEY].small_explosion_index = index);
    let pair = 2 * usize::from(index);
    let x = session_slot!(game, Ridley[RIDLEY].base.x_pos).wrapping_add(OFFSETS[pair] as u16);
    let y = session_slot!(game, Ridley[RIDLEY].base.y_pos).wrapping_add(OFFSETS[pair + 1] as u16);
    game.place_projectile_spawn(x, y);
    game.spawn_room_projectile(DUST_CLOUD, 3);
    game.queue_sound(1, 0x24, 3);
}

/// Initialisation AI - enemy $E1BF (Ridley explosion) ($A6:C696): the part
/// picks the place and instruction list; a random X drift. Was
/// `RidleysExplosion_Init`.
pub fn explosion_init<L: Lease + ?Sized>(game: &mut L) {
    let e = current(game);
    session_slot!(game, Explosion[e].base.instruction_timer = 1);
    session_slot!(game, Explosion[e].base; timer = 0, vram_tiles_index = 0);
    session_slot!(game, Explosion[e].base.palette_index = 3584);
    let part = session_slot!(game, Explosion[e].part);
    session_slot!(
        game,
        Explosion[e].lifetime = ridley_explosion_data::lifetime(game, (part >> 1) as isize)
    );
    let random = game.roll_random();
    let drift = if random & 0x8000 != 0 {
        -(random as i32 & 0x130)
    } else {
        random as i32 & 0x130
    };
    session_slot!(game, Explosion[e].x_velocity = drift as u16);
    session_slot!(game, Explosion[e].y_velocity = 0);
    match part {
        0 | 2 | 4 | 6 | 8 | 10 => {
            // Ridley explosion initialisation - index = 0..Ah (tail segments 0..5).
            let segment = usize::from(part / 2);
            const LISTS: [u16; 3] = [
                explosion_instruction_list::TAIL_SEGMENTS_0_1,
                explosion_instruction_list::TAIL_SEGMENTS_2_3,
                explosion_instruction_list::TAIL_SEGMENTS_4_5,
            ];
            session_slot!(
                game,
                Explosion[e].base.x_pos = word!(game.TILEMAP_STUFF[tail::x(segment)])
            );
            session_slot!(
                game,
                Explosion[e].base.y_pos = word!(game.TILEMAP_STUFF[tail::y(segment)])
            );
            session_slot!(
                game,
                Explosion[e].base.current_instruction = LISTS[segment / 2]
            );
        }
        12 => {
            // Ridley explosion initialisation - index = Ch (tail tip).
            session_slot!(
                game,
                Explosion[e].base.x_pos = word!(game.TILEMAP_STUFF[tail::x(6)])
            );
            session_slot!(
                game,
                Explosion[e].base.y_pos = word!(game.TILEMAP_STUFF[tail::y(6)])
            );
            let list = game.native_image().table_word(
                lorom_offset(TAIL_TIP_EXPLOSION_LISTS) as usize,
                tail_tip_sector(game),
            );
            session_slot!(game, Explosion[e].base.current_instruction = list);
        }
        14 => place_explosion(game, e, WINGS_ADDRESS, 0),
        16 => place_explosion(game, e, LEGS_ADDRESS, 22),
        18 => place_explosion(game, e, MOUTH_ADDRESS, -24),
        20 => place_explosion(game, e, TORSO_ADDRESS, 0),
        22 => place_explosion(game, e, HAND_ADDRESS, 7),
        _ => game.foreign(|game| game.unreachable()),
    }
}

/// Ridley explosion initialisation - index = Eh..16h ($A6:C7DA..C8A2): explosion
/// `e` on Ridley's `part`, its X from the part's placement for his facing and
/// its Y `y_offset` from his. Was `place_ridley_explosion_x`.
fn place_explosion<L: Lease + ?Sized>(game: &mut L, e: i32, part: u32, y_offset: i16) {
    let side: isize = if session_slot!(game, Ridley[RIDLEY].facing) != 0 {
        1
    } else {
        0
    };
    let x = session_slot!(game, Ridley[RIDLEY].base.x_pos)
        .wrapping_add(ridley_explosion_data::placement(game, part, side).x_offset);
    session_slot!(game, Explosion[e].base.x_pos = x);
    session_slot!(
        game,
        Explosion[e].base.y_pos =
            session_slot!(game, Ridley[RIDLEY].base.y_pos).wrapping_add(y_offset as u16)
    );
    session_slot!(
        game,
        Explosion[e].base.current_instruction =
            ridley_explosion_data::placement(game, part, side).instruction_list
    );
}

/// Main AI - enemy $E1BF (Ridley explosion) ($A6:C8D4): flicker, slow the
/// drift, fall, and die when the lifetime runs out. Was
/// `RidleysExplosion_Main`.
pub fn explosion_main<L: Lease + ?Sized>(game: &mut L) {
    explosion_flicker(game.current_enemy(), game);
    let e = current(game);
    let mut speed = (abs16(session_slot!(game, Explosion[e].x_velocity)) as i16).wrapping_sub(4);
    if speed < 0 {
        speed = 0;
    }
    let x_velocity = if session_slot!(game, Explosion[e].x_velocity) & 0x8000 != 0 {
        -(speed as i32)
    } else {
        speed as i32
    };
    session_slot!(game, Explosion[e].x_velocity = x_velocity as u16);
    session_slot!(game, Explosion[e].y_velocity, |old| old.wrapping_add(4));
    move_with_velocity(game);
    session_slot!(game, Explosion[e].lifetime, |old| old.wrapping_sub(1));
    if (session_slot!(game, Explosion[e].lifetime) & 0x8000) != 0 {
        game.foreign(|game| game.normal_death(0));
    }
}

/// Handle flickering ($A6:C913): explosion `k` is invisible on odd frames.
/// Was `RidleysExplosion_Func_1`.
pub fn explosion_flicker<L: Lease + ?Sized>(k: u16, game: &mut L) {
    let e = i32::from(k);
    let odd = session_slot!(game, Explosion[e].base.frame_counter) & 1;
    session_slot!(game, Explosion[e].base.frame_counter = odd);
    let properties = if odd != 0 {
        session_slot!(game, Explosion[e].base.properties) | K_ENEMY_PROPS_INVISIBLE as u16
    } else {
        (session_slot!(game, Explosion[e].base.properties) as i32
            & !(K_ENEMY_PROPS_INVISIBLE as i32)) as u16
    };
    session_slot!(game, Explosion[e].base.properties = properties);
}

/// Spawn Ridley explosion enemies ($A6:C932). Was `RidleysExplosion_Func_2`.
pub fn spawn_explosions<L: Lease + ?Sized>(game: &mut L) {
    /// The explosion enemy populations ($A6:C987..CA37), in spawn order.
    const POPULATIONS: [u16; 12] = [
        0xc9e7, 0xc9d7, 0xc9c7, 0xc9b7, 0xc9a7, 0xc997, 0xc987, 0xc9f7, 0xca07, 0xca27, 0xca17,
        0xca37,
    ];
    for population in POPULATIONS {
        game.foreign(|game| game.spawn_population(0xa6, population));
    }
}

/// Ceres escape sequence "emergency" text glow ($A6:C19C): every fourth frame
/// without a palette change or door transition, the next of 16 glow steps.
/// Was `Ridley_Func_61`.
pub fn emergency_text_glow<L: Lease + ?Sized>(game: &mut L) {
    if word!(game.DOOR_TRANSITION_FLAG_ENEMIES) | word!(game.PALETTE_CHANGE_NUM) == 0
        && (game.frame() & 3) == 0
    {
        let mut step = session_slot!(game, Ridley[RIDLEY].emergency_glow_index).wrapping_add(1);
        if step >= 0x10 {
            step = 0;
        }
        session_slot!(game, Ridley[RIDLEY].emergency_glow_index = step);
        let colour =
            session_slot!(game, Ridley[RIDLEY].emergency_glow_index) as isize + 2 * step as isize;
        word! { game;
            PALETTE_BUFFER[97] = ridley_data::emergency_glow(game, colour);
            PALETTE_BUFFER[98] = ridley_data::emergency_glow(game, colour + 1);
            PALETTE_BUFFER[99] = ridley_data::emergency_glow(game, colour + 2);
        }
    }
}

/// Handle Ridley tail ($A6:CAF5): the whip sound, the tail function, the
/// segment positions and distances, and the tail hitting Samus. Was
/// `Ridley_Func_70`.
pub fn handle_tail<L: Lease + ?Sized>(game: &mut L) {
    tail_whip_sound(game);
    // Ridley tail function pointers ($A6:CB21), by tail function index.
    let functions: [fn(&mut L); 9] = [
        tail_inactive,
        neutral_tail,
        set_up_pogo_tail,
        point_tail_down,
        pogo_tail,
        pogo_stab_setup_tail,
        pogo_descending_stab,
        unused_pogo_descending_stab,
        spinny_tail_whips,
    ];
    functions[word!(game.TILEMAP_STUFF[tail::FUNCTION]) as usize](&mut *game);
    calculate_tail_segment_positions(game);
    update_tail_segment_distances(game);
    if session_slot!(game, Ridley[RIDLEY].holding_samus)
        | (word!(game.SAMUS_INVINCIBILITY_TIMER)
            | session_slot!(game, Ridley[RIDLEY].base.properties) & K_ENEMY_PROPS_TANGIBLE as u16)
        == 0
    {
        tail_samus_collision(game);
    }
}

/// Ridley tail function index 0: nothing ($A6:CB20, an RTS). Was
/// `locret_A6CB20`.
fn tail_inactive<L: Lease + ?Sized>(_game: &mut L) {}

/// Set up Ridley pogo tail ($A6:CB33), tail function index 2. Was
/// `Ridley_Func_71`.
pub fn set_up_pogo_tail<L: Lease + ?Sized>(game: &mut L) {
    word!(game.TILEMAP_STUFF[tail::ANGLE_DELTA] = 8);
    pogo_tail_pointed_down(game);
    word!(game.TILEMAP_STUFF[tail::FUNCTION] = 3);
}

/// Ridley tail function index 3 - pogo - point tail down ($A6:CB45). Was
/// `Ridley_Func_72`.
pub fn point_tail_down<L: Lease + ?Sized>(game: &mut L) {
    word!(game.TILEMAP_STUFF[tail::ANGLE_DELTA] = 8);
    pogo_tail_pointed_down(game);
}

/// Ridley tail function index 6 - pogo - descending - stab ($A6:CB4E). Was
/// `Ridley_Func_73`.
pub fn pogo_descending_stab<L: Lease + ?Sized>(game: &mut L) {
    word!(game.TILEMAP_STUFF[tail::ANGLE_DELTA] = 3);
    pogo_tail_pointed_down(game);
    word!(game.TILEMAP_STUFF[tail::FUNCTION] = 6);
}

/// Ridley tail function index 7 - unused - pogo descending - stab ($A6:CB60).
/// Was `Ridley_Func_74`.
pub fn unused_pogo_descending_stab<L: Lease + ?Sized>(game: &mut L) {
    word!(game.TILEMAP_STUFF[tail::ANGLE_DELTA] = 2);
    pogo_tail_pointed_down(game);
    word!(game.TILEMAP_STUFF[tail::FUNCTION] = 7);
}

/// Ridley pogo tail movement with tail pointed down ($A6:CB72): whip towards
/// straight down, then the normal pogo tail once no segment moves. Was
/// `Ridley_Func_75`.
pub fn pogo_tail_pointed_down<L: Lease + ?Sized>(game: &mut L) {
    if session_slot!(game, Ridley[RIDLEY].facing) != 0 {
        word! {game; TILEMAP_STUFF[tail::WHIP_TARGET_CLOCKWISE] = 0x4000; TILEMAP_STUFF[tail::WHIP_TARGET_ANTICLOCKWISE] = NO_WHIP_TARGET; }
    } else {
        word! {game; TILEMAP_STUFF[tail::WHIP_TARGET_ANTICLOCKWISE] = 0x4000; TILEMAP_STUFF[tail::WHIP_TARGET_CLOCKWISE] = NO_WHIP_TARGET; }
    }
    pogo_tail(game);
    if !any_tail_segment_active(game) {
        word!(game.TILEMAP_STUFF[tail::FUNCTION] = 4);
    }
}

/// Ridley tail function index 1 - neutral (natural tail movement + tail
/// whips) ($A6:CBC0). Was `Ridley_CBC0`.
pub fn neutral_tail<L: Lease + ?Sized>(game: &mut L) {
    handle_tail_whip(game);
    update_tail_segment_angles(game);
}

/// Ridley pogo tail movement / Ridley tail function index 4 - pogo - normal
/// ($A6:CBC7). Was `Ridley_CBC7`.
pub fn pogo_tail<L: Lease + ?Sized>(game: &mut L) {
    handle_normal_pogo_tail(game);
    update_tail_segment_angles(game);
}

/// Ridley tail function index 5 - pogo - descending - stab setup ($A6:CBCE).
/// Was `Ridley_CBCE`.
pub fn pogo_stab_setup_tail<L: Lease + ?Sized>(game: &mut L) {
    handle_pogo_tail_stab(game);
    update_tail_segment_angles(game);
}

/// Ridley tail function index 8 - unused - neutral with spinny tail whips
/// ($A6:CBD5). Was `Ridley_CBD5`.
pub fn spinny_tail_whips<L: Lease + ?Sized>(game: &mut L) {
    handle_spinning_tail_whip(game);
    update_tail_segment_angles(game);
}

/// Update Ridley tail segment angles ($A6:CBDC). Was `Ridley_CBDC`.
pub fn update_tail_segment_angles<L: Lease + ?Sized>(game: &mut L) {
    for segment in 0..7 {
        update_tail_segment_angle(segment, game);
    }
}

/// Set Ridley tail angle extrema ($A6:CBFE), by facing direction. Was
/// `Ridley_Func_77`.
pub fn set_tail_angle_extrema<L: Lease + ?Sized>(game: &mut L) {
    let limits =
        ridley_data::tail_angles(game, session_slot!(game, Ridley[RIDLEY].facing) as isize);
    word!(game.TILEMAP_STUFF[tail::MIN_CLOCKWISE_ANGLE] = limits.min_clockwise);
    word!(game.TILEMAP_STUFF[tail::MAX_ANTICLOCKWISE_ANGLE] = limits.max_anticlockwise);
}

/// Check if all Ridley tail segments are active ($A6:CC1E), tip first. Was
/// `Ridley_Func_78`.
pub fn all_tail_segments_active<L: Lease + ?Sized>(game: &mut L) -> bool {
    let mut flags = word!(game.TILEMAP_STUFF[tail::active(6)]);
    for segment in (0..6).rev() {
        flags &= word!(game.TILEMAP_STUFF[tail::active(segment)]);
    }
    flags != 0
}

/// Whether any tail segment is active, tip first.
fn any_tail_segment_active<L: Lease + ?Sized>(game: &mut L) -> bool {
    let mut flags = word!(game.TILEMAP_STUFF[tail::active(6)]);
    for segment in (0..6).rev() {
        flags |= word!(game.TILEMAP_STUFF[tail::active(segment)]);
    }
    flags != 0
}

/// Handle Ridley tail whip ($A6:CC39). Was `Ridley_Func_79`.
pub fn handle_tail_whip<L: Lease + ?Sized>(game: &mut L) {
    set_tail_angle_extrema(game);
    if all_tail_segments_active(game) && configure_tail_whip(game) {
        return;
    }
    swing_idle_tail(game);
}

/// The end of the tail whip handlers ($A6:CC39, CCBD): once every segment
/// rests, swing again from the base, faster when Samus is right of 70h. Was
/// `start_ridley_tile_effect_when_idle`.
fn swing_idle_tail<L: Lease + ?Sized>(game: &mut L) {
    if !any_tail_segment_active(game) {
        word!(game.TILEMAP_STUFF[tail::active(0)] = 0x8000);
        word!(game.TILEMAP_STUFF[tail::WHIP_TARGET_CLOCKWISE] = NO_WHIP_TARGET);
        word!(game.TILEMAP_STUFF[tail::WHIP_TARGET_ANTICLOCKWISE] = NO_WHIP_TARGET);
        let delta = if game.player_x() < 0x70 { 1 } else { 2 };
        word!(game.TILEMAP_STUFF[tail::ANGLE_DELTA] = delta);
    }
}

/// Configure Ridley tail whip ($A6:CC7D): on a whip request, at random while
/// idle whips are enabled, or when Samus is near; true when a whip started.
/// Was `Ridley_Func_80_DoubleRet`.
pub fn configure_tail_whip<L: Lease + ?Sized>(game: &mut L) -> bool {
    let attack = if word!(game.TILEMAP_STUFF[tail::WHIP_REQUEST]) != 0
        && (word!(game.TILEMAP_STUFF[tail::WHIP_TARGET_CLOCKWISE])
            & word!(game.TILEMAP_STUFF[tail::WHIP_TARGET_ANTICLOCKWISE])
            & 0x8000)
            != 0
    {
        Some(word!(game.TILEMAP_STUFF[tail::WHIP_REQUEST]).wrapping_sub(1))
    } else if word!(game.TILEMAP_STUFF[tail::IDLE_WHIP]) == 0 {
        None
    } else if (word!(game.RANDOM_NUMBER) as u8) >= 0xf0 {
        Some(word!(game.TILEMAP_STUFF[tail::WHIP_REQUEST]).wrapping_sub(1))
    } else if abs16(
        game.player_x()
            .wrapping_sub(session_slot!(game, Ridley[RIDLEY].base.x_pos)),
    ) < 0x80
    {
        Some(0)
    } else {
        None
    };
    let started = if let Some(attack) = attack {
        set_tail_whip_target(attack, game);
        true
    } else {
        false
    };
    word!(game.TILEMAP_STUFF[tail::WHIP_REQUEST] = 0);
    started
}

/// Unused. Handle Ridley tail whip with extra spinning ($A6:CCBD). Was
/// `Ridley_Func_81`.
pub fn handle_spinning_tail_whip<L: Lease + ?Sized>(game: &mut L) {
    set_tail_angle_extrema(game);
    if all_tail_segments_active(game) && configure_spinning_tail_whip(game) {
        return;
    }
    swing_idle_tail(game);
}

/// Unused. Configure Ridley tail whip with extra spinning ($A6:CD0C); true
/// when a whip started. Was `Ridley_Func_82_DoubleRet`.
pub fn configure_spinning_tail_whip<L: Lease + ?Sized>(game: &mut L) -> bool {
    if word!(game.TILEMAP_STUFF[tail::WHIP_REQUEST]) != 0
        && (word!(game.TILEMAP_STUFF[tail::WHIP_TARGET_CLOCKWISE])
            & word!(game.TILEMAP_STUFF[tail::WHIP_TARGET_ANTICLOCKWISE])
            & 0x8000)
            != 0
    {
        set_tail_whip_target(1, game);
        word!(game.TILEMAP_STUFF[tail::WHIP_REQUEST] = 0);
        return true;
    }
    false
}

/// The pogo tail's whip when Samus is near or at random, shared by
/// `handle_normal_pogo_tail` and `handle_pogo_tail_stab`.
fn pogo_tail_whips<L: Lease + ?Sized>(game: &mut L) -> bool {
    set_tail_angle_extrema(game);
    if all_tail_segments_active(game)
        && ((word!(game.RANDOM_NUMBER) as u8) >= 0xf0
            || abs16(
                game.player_x()
                    .wrapping_sub(session_slot!(game, Ridley[RIDLEY].base.x_pos)),
            ) < 0x80)
        && (word!(game.TILEMAP_STUFF[tail::WHIP_TARGET_CLOCKWISE])
            & word!(game.TILEMAP_STUFF[tail::WHIP_TARGET_ANTICLOCKWISE])
            & 0x8000)
            != 0
    {
        word! {game; TILEMAP_STUFF[tail::WHIP_TARGET_CLOCKWISE] = 16128; TILEMAP_STUFF[tail::ANGLE_DELTA] = 8; }
        return true;
    }
    false
}

/// Handle normal Ridley pogo tail ($A6:CD24): whip, or once the tail rests
/// count down to its next swing; set up the stab while rising. Was
/// `Ridley_Func_83`.
pub fn handle_normal_pogo_tail<L: Lease + ?Sized>(game: &mut L) {
    if pogo_tail_whips(game) || any_tail_segment_active(game) {
        return;
    }
    if (session_slot!(game, Ridley[RIDLEY].y_velocity) & 0x8000) == 0 {
        word!(game.TILEMAP_STUFF[tail::FUNCTION] = 5);
    }
    word! {game; TILEMAP_STUFF[tail::WHIP_TARGET_CLOCKWISE] = NO_WHIP_TARGET; TILEMAP_STUFF[tail::WHIP_TARGET_ANTICLOCKWISE] = NO_WHIP_TARGET; }
    let mut timer = word!(game.TILEMAP_STUFF[tail::POGO_TAIL_TIMER]);
    if word!(game.TILEMAP_STUFF[tail::POGO_TAIL_TIMER]) == 0 || {
        timer = word!(game.TILEMAP_STUFF[tail::POGO_TAIL_TIMER]).wrapping_sub(1);
        word!(game.TILEMAP_STUFF[tail::POGO_TAIL_TIMER] = timer);
        word!(game.TILEMAP_STUFF[tail::POGO_TAIL_TIMER]) != 0
    } {
        word!(game.TILEMAP_STUFF[tail::ANGLE_DELTA] = timer);
    } else {
        word!(game.TILEMAP_STUFF[tail::active(0)] = 0x8000);
        set_tail_angle_high_bytes(game);
        word!(game.TILEMAP_STUFF[tail::ANGLE_DELTA] = 8);
    }
}

/// Handle Ridley pogo tail stab ($A6:CDAA): as the normal pogo tail, but
/// falling it points the tail straight down at full length. Was
/// `Ridley_Func_84`.
pub fn handle_pogo_tail_stab<L: Lease + ?Sized>(game: &mut L) {
    if pogo_tail_whips(game) || any_tail_segment_active(game) {
        return;
    }
    if (session_slot!(game, Ridley[RIDLEY].y_velocity) & 0x8000) == 0 {
        word!(game.TILEMAP_STUFF[tail::FUNCTION] = 6);
        for segment in 0..7 {
            word!(game.TILEMAP_STUFF[tail::target_distance(segment)] = 2560);
        }
        for segment in 0..7 {
            word!(game.TILEMAP_STUFF[tail::angle(segment)] = 0x4000);
        }
        word!(game.TILEMAP_STUFF[tail::active(0)] = 0x8000);
    }
    word! {game; TILEMAP_STUFF[tail::WHIP_TARGET_CLOCKWISE] = NO_WHIP_TARGET; TILEMAP_STUFF[tail::WHIP_TARGET_ANTICLOCKWISE] = NO_WHIP_TARGET; }
    let mut timer = word!(game.TILEMAP_STUFF[tail::POGO_TAIL_TIMER]);
    if word!(game.TILEMAP_STUFF[tail::POGO_TAIL_TIMER]) != 0 {
        timer = word!(game.TILEMAP_STUFF[tail::POGO_TAIL_TIMER]).wrapping_sub(1);
        word!(game.TILEMAP_STUFF[tail::POGO_TAIL_TIMER] = timer);
        if timer == 0 {
            word!(game.TILEMAP_STUFF[tail::active(0)] = 0x8000);
            set_tail_angle_high_bytes(game);
            timer = 8;
        }
    }
    word!(game.TILEMAP_STUFF[tail::ANGLE_DELTA] = timer);
}

/// Set tail angle high bytes to 40h ($A6:CE65), through raw work RAM. Was
/// `Ridley_Func_85`.
pub fn set_tail_angle_high_bytes<L: Lease + ?Sized>(game: &mut L) {
    for segment in 0..4 {
        let value = tail_low_byte(game, tail::angle(segment)) | 0x4000;
        set_tail_word(game, tail::angle(segment), value);
    }
    // Segment 4 deliberately takes segment 0's word, including its bit 10.
    let value = tail_word(game, tail::angle(0)) & 0x4ff | 0x4000;
    set_tail_word(game, tail::angle(4), value);
    for segment in 5..7 {
        let value = tail_low_byte(game, tail::angle(segment)) | 0x4000;
        set_tail_word(game, tail::angle(segment), value);
    }
}

/// Calculate Ridley tail segment positions ($A6:CEBA): each segment is the
/// previous one plus its offset; facing forward the tail hangs straight
/// down. Was `Ridley_Func_86`.
pub fn calculate_tail_segment_positions<L: Lease + ?Sized>(game: &mut L) {
    /// The tail base X offsets ($A6:CF54), by facing direction.
    const BASE_X_OFFSETS: [i16; 3] = [0x20, 0, -0x20];
    word!(
        game.TILEMAP_STUFF[tail::y(0)] = (word!(game.TILEMAP_STUFF[tail::y_offset(0)]) as i32
            + session_slot!(game, Ridley[RIDLEY].base.y_pos) as i32
            + 16) as u16
    );
    chain_tail_positions(game, tail::y(0));
    let mut x_offset = (session_slot!(game, Ridley[RIDLEY].facing) as i16).wrapping_sub(1);
    if session_slot!(game, Ridley[RIDLEY].facing) != FACING_FORWARD {
        x_offset = (word!(game.TILEMAP_STUFF[tail::x_offset(0)]) as i16)
            .wrapping_add(BASE_X_OFFSETS[session_slot!(game, Ridley[RIDLEY].facing) as usize]);
    }
    word!(
        game.TILEMAP_STUFF[tail::x(0)] =
            session_slot!(game, Ridley[RIDLEY].base.x_pos).wrapping_add(x_offset as u16)
    );
    if session_slot!(game, Ridley[RIDLEY].facing) == FACING_FORWARD {
        word!(game.TILEMAP_STUFF[tail::x(1)] = session_slot!(game, Ridley[RIDLEY].base.x_pos));
        for segment in 2..7 {
            let x = tail_word(game, tail::x(1));
            set_tail_word(game, tail::x(segment), x);
        }
    } else {
        chain_tail_positions(game, tail::x(0));
    }
}

/// Segments 1 to 6 of one axis from segment 0's `first` word: two segments at
/// a time, the second from both offsets rather than the first's new value.
fn chain_tail_positions<L: Lease + ?Sized>(game: &mut L, first: usize) {
    for index in (first + tail::SEGMENT..=first + 5 * tail::SEGMENT).step_by(2 * tail::SEGMENT) {
        let value = tail_word(game, index + 2).wrapping_add(tail_word(game, index - tail::SEGMENT));
        set_tail_word(game, index, value);
        let value = tail_word(game, index + tail::SEGMENT + 2)
            .wrapping_add(tail_word(game, index + 2))
            .wrapping_add(tail_word(game, index - tail::SEGMENT));
        set_tail_word(game, index + tail::SEGMENT, value);
    }
}

/// Update Ridley tail segment distances ($A6:CF5A): segments 1 to 6 extend
/// towards their target distances or relax to their resting ones. Was
/// `Ridley_Func_87`.
pub fn update_tail_segment_distances<L: Lease + ?Sized>(game: &mut L) {
    const MAXIMA: [u16; 6] = [6144, 6144, 5632, 5632, 4608, 1280];
    const RESTING: [u16; 6] = [2048, 2048, 2048, 2048, 2048, 1280];
    for segment in 1..7 {
        update_tail_segment_distance(game, segment, MAXIMA[segment - 1], RESTING[segment - 1]);
    }
}

fn update_tail_segment_distance<L: Lease + ?Sized>(
    game: &mut L,
    segment: usize,
    maximum: u16,
    resting: u16,
) {
    let (distance, target) = (tail::distance(segment), tail::target_distance(segment));
    if tail_word(game, target) != 0 {
        if (tail_word(game, target) as i16).wrapping_sub(tail_word(game, distance) as i16) < 0 {
            set_tail_word(game, target, 0);
        }
        let speed = tail_word(game, tail::EXTENSION_SPEED);
        let mut value = speed.wrapping_add(tail_word(game, distance));
        if (tail_word(game, tail::EXTENSION_SPEED) as i32 + tail_word(game, distance) as i32
            - maximum as i32)
            & 0x8000
            == 0
        {
            value = maximum;
        }
        set_tail_word(game, distance, value);
    } else if (resting as i16).wrapping_sub(tail_word(game, distance) as i16) < 0 {
        let value = tail_word(game, distance).wrapping_sub(128);
        set_tail_word(game, distance, value);
    }
}

/// Update Ridley tail segment angle ($A6:D09F): an active segment staggers,
/// then swings between the angle extrema or towards a whip target, and stops
/// at a target once the segment before it has; every segment then takes its
/// offset from its angle and distance. Was `Ridley_Func_88`.
pub fn update_tail_segment_angle<L: Lease + ?Sized>(segment: usize, game: &mut L) {
    let angle = if (word!(game.TILEMAP_STUFF[tail::active(segment)]) & 0x8000) != 0 {
        let stagger = word!(game.TILEMAP_STUFF[tail::stagger_angle(segment)]);
        if stagger < word!(game.TILEMAP_STUFF[tail::IDEAL_SEGMENT_ANGLE]) {
            word!(
                game.TILEMAP_STUFF[tail::stagger_angle(segment)] +=
                    word!(game.TILEMAP_STUFF[tail::ANGLE_DELTA])
            );
            return;
        }
        if stagger != 0xffff {
            // The next segment starts moving the same way.
            word!(game.TILEMAP_STUFF[tail::stagger_angle(segment)] = 0xffff);
            word!(game.TILEMAP_STUFF[tail::active(segment + 1)] = 0x8000);
            word!(
                game.TILEMAP_STUFF[tail::direction(segment + 1)] =
                    word!(game.TILEMAP_STUFF[tail::direction(segment)])
            );
        }
        let next = 'advance: {
            if (word!(game.TILEMAP_STUFF[tail::direction(segment)]) & 0x8000) == 0 {
                let (turn, angle) =
                    if (word!(game.TILEMAP_STUFF[tail::WHIP_TARGET_ANTICLOCKWISE]) & 0x8000) != 0 {
                        let angle = word!(game.TILEMAP_STUFF[tail::ANGLE_DELTA])
                            .wrapping_add(word!(game.TILEMAP_STUFF[tail::angle(segment)]));
                        let limit = word!(game.TILEMAP_STUFF[tail::MAX_ANTICLOCKWISE_ANGLE]);
                        ((angle as i16).wrapping_sub(limit as i16) >= 0, angle)
                    } else {
                        word!(game.TILEMAP_STUFF[tail::target_distance(segment)] = 3072);
                        let angle = word!(game.TILEMAP_STUFF[tail::ANGLE_DELTA])
                            .wrapping_add(word!(game.TILEMAP_STUFF[tail::angle(segment)]));
                        let target = word!(game.TILEMAP_STUFF[tail::WHIP_TARGET_ANTICLOCKWISE]);
                        if (angle as i16).wrapping_sub(target as i16) >= 0 {
                            if stops_at_whip_target(game, segment) {
                                word!(
                                    game.TILEMAP_STUFF[tail::angle(segment)] =
                                        word!(game.TILEMAP_STUFF[tail::WHIP_TARGET_ANTICLOCKWISE])
                                );
                                break 'advance None;
                            }
                            (true, angle)
                        } else {
                            (false, angle)
                        }
                    };
                if turn {
                    word!(game.TILEMAP_STUFF[tail::direction(segment)] = 0x8000);
                    Some(word!(game.TILEMAP_STUFF[tail::MAX_ANTICLOCKWISE_ANGLE]))
                } else {
                    Some(angle)
                }
            } else {
                let (turn, decreased) =
                    if (word!(game.TILEMAP_STUFF[tail::WHIP_TARGET_CLOCKWISE]) & 0x8000) != 0 {
                        let decreased = (word!(game.TILEMAP_STUFF[tail::angle(segment)]) as i32
                            - word!(game.TILEMAP_STUFF[tail::ANGLE_DELTA]) as i32
                            - 1) as i16;
                        let limit = word!(game.TILEMAP_STUFF[tail::MIN_CLOCKWISE_ANGLE]);
                        (decreased.wrapping_sub(limit as i16) < 0, decreased)
                    } else {
                        word!(game.TILEMAP_STUFF[tail::target_distance(segment)] = 3072);
                        let decreased = (word!(game.TILEMAP_STUFF[tail::angle(segment)]) as i32
                            - word!(game.TILEMAP_STUFF[tail::ANGLE_DELTA]) as i32
                            - 1) as i16;
                        let target = word!(game.TILEMAP_STUFF[tail::WHIP_TARGET_CLOCKWISE]);
                        if decreased.wrapping_sub(target as i16) < 0 {
                            if stops_at_whip_target(game, segment) {
                                word!(
                                    game.TILEMAP_STUFF[tail::angle(segment)] =
                                        word!(game.TILEMAP_STUFF[tail::WHIP_TARGET_CLOCKWISE])
                                );
                                break 'advance None;
                            }
                            (true, decreased)
                        } else {
                            (false, decreased)
                        }
                    };
                if turn {
                    word!(game.TILEMAP_STUFF[tail::direction(segment)] = 0);
                    Some(word!(game.TILEMAP_STUFF[tail::MIN_CLOCKWISE_ANGLE]))
                } else {
                    Some((decreased as u16).wrapping_add(1))
                }
            }
        };
        let Some(angle) = next else {
            word! { game, 0; TILEMAP_STUFF[tail::active(segment)], TILEMAP_STUFF[tail::stagger_angle(segment)], }
            word!(game.TILEMAP_STUFF[tail::direction(segment)] ^= 0x8000u16);
            return;
        };
        word!(game.TILEMAP_STUFF[tail::angle(segment)] = angle);
        angle
    } else {
        word!(game.TILEMAP_STUFF[tail::angle(segment)])
    };
    // Segment angles add up along the tail, in their low bytes.
    let mut total = angle as u8 as u16;
    if segment != 0 {
        total = game
            .slot_byte(TAIL_RAM + 2 * tail::angle(segment - 1))
            .wrapping_add(total as u8) as u16;
    }
    let length_at = TAIL_RAM + 2 * tail::distance(segment) + 1;
    word! { game;
        TILEMAP_STUFF[tail::x_offset(segment)] =
            game.foreign(|game| game.hardware_sine(game.slot_byte(length_at) as u16, total));
        TILEMAP_STUFF[tail::y_offset(segment)] =
            game.foreign(|game| game.hardware_cosine(game.slot_byte(length_at) as u16, total));
    }
}

/// At a whip target a segment stops once the one before it has.
fn stops_at_whip_target<L: Lease + ?Sized>(game: &L, segment: usize) -> bool {
    segment == 0 || word!(game.TILEMAP_STUFF[tail::active(segment - 1)]) == 0
}

/// Set Ridley tail whip target ($A6:D19D): aim the whip at a missile near the
/// tip, or else at Samus, within reach of the facing side. Was
/// `Ridley_Func_89`.
pub fn set_tail_whip_target<L: Lease + ?Sized>(attack: u16, game: &mut L) {
    // The attack's low byte becomes the target angle's high byte.
    word!(game.TILEMAP_STUFF[tail::WHIP_ANGLE_OFFSET] = u16::from(attack as u8) << 8);
    if session_slot!(game, Ridley[RIDLEY].facing) != FACING_FORWARD
        && session_slot!(game, Ridley[RIDLEY].fight_mode) != 0
    {
        let (dx, dy) = if let Some(missile) = missile_near_tail(game) {
            (
                missile
                    .x
                    .wrapping_sub(word!(game.TILEMAP_STUFF[tail::x(0)])),
                missile
                    .y
                    .wrapping_sub(word!(game.TILEMAP_STUFF[tail::y(0)])),
            )
        } else {
            (
                game.player_x()
                    .wrapping_sub(word!(game.TILEMAP_STUFF[tail::x(0)])),
                (game.player_y() as i32 + 24 - word!(game.TILEMAP_STUFF[tail::y(0)]) as i32) as u16,
            )
        };
        let direction = -(angle_from_delta(dx, dy) as i32 + 0x80) as u8 as u16;
        if session_slot!(game, Ridley[RIDLEY].facing) != 0 {
            let mut reach = direction;
            if (0x18..0xe8).contains(&direction) {
                reach = 24;
            }
            let target = (word!(game.TILEMAP_STUFF[tail::WHIP_ANGLE_OFFSET]) as i32
                + reach as i32
                + 0x4000) as u16;
            if target >= word!(game.TILEMAP_STUFF[tail::angle(0)]) {
                word! {game; TILEMAP_STUFF[tail::WHIP_TARGET_ANTICLOCKWISE] = target; TILEMAP_STUFF[tail::ANGLE_DELTA] = 8; }
            }
        } else {
            let mut reach = direction;
            if (0x18..0xe8).contains(&direction) {
                reach = 232;
            }
            let target = (reach as i32 + 16128
                - word!(game.TILEMAP_STUFF[tail::WHIP_ANGLE_OFFSET]) as i32)
                as u16;
            if target < word!(game.TILEMAP_STUFF[tail::angle(0)]) {
                word! {game; TILEMAP_STUFF[tail::WHIP_TARGET_CLOCKWISE] = target; TILEMAP_STUFF[tail::ANGLE_DELTA] = 8; }
            }
        }
    }
}

/// Detect (super) missile near Ridley tail ($A6:D242): the first missile or
/// super missile within 64px of the tip. Was `Ridley_Func_90`.
pub fn missile_near_tail<L: Lease + ?Sized>(game: &mut L) -> Option<Point> {
    let rect = Rect {
        x: word!(game.TILEMAP_STUFF[tail::x(6)]),
        y: word!(game.TILEMAP_STUFF[tail::y(6)]),
        w: 64,
        h: 64,
    };
    if word!(game.PROJECTILE_COUNTER) == 0 {
        return None;
    }
    let found = (0..5).find(|&n| {
        let kind = projectile_kind(game, n);
        (kind == MISSILE || kind == SUPER_MISSILE) && projectile_overlaps(game, n, rect)
    })?;
    Some(Point {
        x: word!(game.PROJECTILE_X_POS[found]),
        y: word!(game.PROJECTILE_Y_POS[found]),
    })
}

/// The low nibble of projectile `n`'s type high byte: its weapon.
fn projectile_kind<L: Lease + ?Sized>(game: &L, n: usize) -> u8 {
    game.slot_byte(PROJECTILE_TYPES + 2 * n + 1) & 0xf
}

/// Whether projectile `n`'s hitbox overlaps `rect`.
fn projectile_overlaps<L: Lease + ?Sized>(game: &L, n: usize, rect: Rect) -> bool {
    let dx = abs16(word!(game.PROJECTILE_X_POS[n]).wrapping_sub(rect.x));
    if dx < word!(game.PROJECTILE_X_RADIUS[n])
        || dx.wrapping_sub(word!(game.PROJECTILE_X_RADIUS[n])) < rect.w
    {
        let dy = abs16(word!(game.PROJECTILE_Y_POS[n]).wrapping_sub(rect.y));
        if dy < word!(game.PROJECTILE_Y_RADIUS[n])
            || dy.wrapping_sub(word!(game.PROJECTILE_Y_RADIUS[n])) < rect.h
        {
            return true;
        }
    }
    false
}

/// Handle Ridley tail whip sound effect ($A6:D2AA): every 16 frames of a fast
/// tail, unless roaring. Was `Ridley_Func_91`.
pub fn tail_whip_sound<L: Lease + ?Sized>(game: &mut L) {
    if !below(word!(game.TILEMAP_STUFF[tail::ANGLE_DELTA]), 8) {
        let counter = session_slot!(game, Ridley[RIDLEY].whip_sound_counter).wrapping_add(1);
        session_slot!(game, Ridley[RIDLEY].whip_sound_counter = counter);
        if below(counter, 16) {
            return;
        }
        if session_slot!(game, Ridley[RIDLEY].roaring) == 0 {
            game.queue_sound(2, 0x21, 6);
        }
    }
    session_slot!(game, Ridley[RIDLEY].whip_sound_counter = 0);
}

/// Initialise Ridley tail RAM ($A6:D2D6). Was `Ridley_Func_92`.
pub fn initialise_tail<L: Lease + ?Sized>(game: &mut L) {
    word! {game; TILEMAP_STUFF[tail::ANGLE_DELTA] = 1; TILEMAP_STUFF[tail::MIN_CLOCKWISE_ANGLE] = 16368; TILEMAP_STUFF[tail::MAX_ANTICLOCKWISE_ANGLE] = 16448;
    TILEMAP_STUFF[tail::WHIP_TARGET_CLOCKWISE] = NO_WHIP_TARGET; TILEMAP_STUFF[tail::WHIP_TARGET_ANTICLOCKWISE] = NO_WHIP_TARGET;
    TILEMAP_STUFF[tail::EXTENSION_SPEED] = 240; TILEMAP_STUFF[tail::IDEAL_SEGMENT_ANGLE] = 16; }
    for segment in 0..7 {
        word!(game.TILEMAP_STUFF[tail::stagger_angle(segment)] = 17);
    }
    game.foreign(|game| {
        set_tail_segments_from_list(
            tail_address(tail::direction(0)),
            tail_data::DIRECTIONS,
            game,
        )
    });
    game.foreign(|game| {
        set_tail_segments_from_list(tail_address(tail::distance(0)), tail_data::DISTANCES, game);
        set_tail_segments_from_list(tail_address(tail::angle(0)), tail_data::ANGLES, game);
        set_tail_segments_from_list(tail_address(tail::x(0)), tail_data::X_POSITIONS, game);
        set_tail_segments_from_list(tail_address(tail::y(0)), tail_data::Y_POSITIONS, game);
    });
    set_tail_segment_flags(0, game);
}

/// Ridley tail segment active flags = 8000h ($A6:D3D9). Was `Ridley_Func_95`.
pub fn activate_tail_segments<L: Lease + ?Sized>(game: &mut L) {
    set_tail_segment_flags(0x8000, game);
}

/// Ridley tail segment active flags = [A] ($A6:D3DC). Was `Ridley_Func_D3DC`.
pub fn set_tail_segment_flags<L: Lease + ?Sized>(flags: u16, game: &mut L) {
    for segment in 0..7 {
        word!(game.TILEMAP_STUFF[tail::active(segment)] = flags);
    }
}

/// Flip Ridley tail ($A6:D3F9): mirror every segment's angle and turn it
/// anticlockwise. Was `Ridley_Func_96`.
pub fn flip_tail<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| {
        set_tail_segments_from_list(tail_address(tail::distance(0)), tail_data::DISTANCES, game)
    });
    for segment in 0..7 {
        word!(
            game.TILEMAP_STUFF[tail::angle(segment)] =
                0x8000u16.wrapping_sub(word!(game.TILEMAP_STUFF[tail::angle(segment)]))
        );
        word!(game.TILEMAP_STUFF[tail::direction(segment)] |= 0x8000u16);
    }
}

/// Deal suit-adjusted enemy damage to Samus ($A6:D453): the current enemy's
/// contact damage. Was `Ridley_Func_98`.
pub fn deal_contact_damage<L: Lease + ?Sized>(game: &mut L) {
    let definition = session_slot!(game, Ridley[current(game)].base.enemy_ptr);
    let damage = crate::cartridge::native_record(ENEMY_DEFINITION_BANK, definition, game)
        .word(ENEMY_DEFINITION_DAMAGE);
    game.foreign(|game| {
        let amount = game.suit_damage(damage);
        game.damage_player(amount);
    });
}

/// Set Ridley instruction list ($A6:D467). Was `Ridley_Func_99`.
pub fn set_instruction_list<L: Lease + ?Sized>(list: u16, game: &mut L) {
    session_slot!(game, Ridley[RIDLEY].base; current_instruction = list, instruction_timer = 1);
    session_slot!(game, Ridley[RIDLEY].base.timer = 0);
}

/// Norfair Ridley health-based palette handling ($A6:D474): below 9000, 5400
/// and 1800 health Ridley accelerates harder and glows redder. Was
/// `Ridley_Func_100`.
pub fn health_palette<L: Lease + ?Sized>(game: &mut L) {
    let health = session_slot!(game, Ridley[RIDLEY].base.health);
    let index: i16 = if health >= 0x2328 {
        0
    } else if health >= 0x1518 {
        1
    } else if health >= 0x708 {
        2
    } else {
        3
    };
    session_slot!(game, Ridley[RIDLEY].acceleration_index = index as u16);
    if index.wrapping_sub(1) >= 0 {
        load_health_palette((index as u16).wrapping_sub(1), game);
    }
}

/// Load Ridley health-based palette ($A6:D495) `index` into sprite palette 7
/// unless a palette change is pending. Was `Ridley_D495`.
pub fn load_health_palette<L: Lease + ?Sized>(index: u16, game: &mut L) {
    /// Ridley health-based palettes ($A6:E46A), 14 colours each.
    const HEALTH_PALETTES: u16 = 0xe46a;
    if word!(game.PALETTE_CHANGE_NUM) == 0 {
        let colours = HEALTH_PALETTES.wrapping_add(index.wrapping_mul(28));
        game.foreign(|game| copy_rom_colors_to_palette(0x1e2, 0xa6, colours, 0xe, game));
    }
}

/// Ceres Ridley health-based palette handling ($A6:D4B5): by the hit counter.
/// Was `Ridley_Func_101`.
pub fn ceres_health_palette<L: Lease + ?Sized>(game: &mut L) {
    if session_slot!(game, Ridley[RIDLEY].fight_mode) != 0
        && session_slot!(game, Ridley[RIDLEY].hit_counter) >= 0x32
    {
        let index = if session_slot!(game, Ridley[RIDLEY].hit_counter) >= 0x46 {
            2
        } else {
            0
        };
        load_health_palette(index, game);
    }
}

/// Ridley hurt flash handling ($A6:D4DA): the spritemap palette flashes to
/// palette 0 while the flash timer runs. Was `Ridley_Func_102`.
pub fn hurt_flash<S: State + ?Sized>(game: &mut S) {
    let mut palette = 3584;
    if session_slot!(game, Ridley[RIDLEY].base.flash_timer) >= 2
        && ((word!(game.RANDOM_ENEMY_COUNTER) as i32 + 1) & 2) != 0
    {
        palette = 0;
    }
    session_slot!(game, Ridley[RIDLEY].palette = palette);
}

/// Check for collision with non-air block ($A6:D4F9): whether the level data
/// block at (`x`, `y`) has a type. Was `ridley_offscreen`.
pub fn solid_block_at<H: crate::ram::RamHost + ?Sized>(x: u16, y: u16, game: &mut H) -> bool {
    let row = mult8x8((y >> 4) as u8, word!(game.ROOM_WIDTH_IN_BLOCKS) as u8);
    let at = usize::try_from(LEVEL_DATA + 2 * (row as i32 + (x as i32 >> 4)) as isize)
        .expect("RAM address outside allocation");
    let block = u16::from_ne_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)]);
    block & 0xf000 != 0
}

/// Ridley acceleration ($A6:D523): enemy `k` homes in on (`x`, `y`) with
/// inertia `inertia`. Was `Ridley_Func_104_0`.
pub fn accelerate<L: Lease + ?Sized>(k: u16, inertia: u16, x: u16, y: u16, game: &mut L) {
    accelerate_with_force(k, inertia, 0, x, y, game);
}

/// Ridley death spot acceleration ($A6:D526): as `accelerate`, with `force`
/// extra when reversing. Was `Ridley_Func_104`.
pub fn accelerate_with_force<L: Lease + ?Sized>(
    k: u16,
    inertia: u16,
    force: u16,
    x: u16,
    y: u16,
    game: &mut L,
) {
    crate::probe::hit("homing.ridley.extra");
    let divisor = INERTIA[inertia as usize];
    accelerate_x(k, x, u16::from(divisor), force, game);
    let e = i32::from(k);
    let distance = (session_slot!(game, Ridley[e].base.y_pos) as i16).wrapping_sub(y as i16);
    if distance != 0 {
        session_slot!(
            game,
            Ridley[e].y_velocity = homing_velocity(
                distance,
                session_slot!(game, Ridley[e].y_velocity),
                divisor,
                HomingReversal::ExtraForce(force.wrapping_add(8)),
                1280,
            )
        );
    }
}

/// Ridley inertia table ($A6:D61F).
const INERTIA: [u8; 16] = [
    0x10, 0xf, 0xe, 0xd, 0xc, 0xb, 0xa, 9, 8, 7, 6, 5, 4, 3, 2, 1,
];

/// Ridley X acceleration ($A6:D5A9). Was `Ridley_Func_105`.
pub fn accelerate_x<L: Lease + ?Sized>(k: u16, x: u16, divisor: u16, force: u16, game: &mut L) {
    let e = i32::from(k);
    let distance = (session_slot!(game, Ridley[e].base.x_pos) as i16).wrapping_sub(x as i16);
    if distance != 0 {
        session_slot!(
            game,
            Ridley[e].x_velocity = homing_velocity(
                distance,
                session_slot!(game, Ridley[e].x_velocity),
                divisor as u8,
                HomingReversal::ExtraForce(force.wrapping_add(8)),
                1280,
            )
        );
    }
}

/// Ceres Ridley acceleration ($A6:D62F): reversing doubles the step. Was
/// `Ridley_Func_106`.
pub fn ceres_accelerate<L: Lease + ?Sized>(k: u16, inertia: u16, x: u16, y: u16, game: &mut L) {
    /// Ceres Ridley inertia table ($A6:D712).
    const CERES_INERTIA: [u8; 16] = [
        0x10, 0xf, 0xe, 0xd, 0xc, 0xb, 0xa, 9, 8, 7, 6, 5, 4, 3, 2, 1,
    ];
    crate::probe::hit("homing.ridley.double");
    let divisor = CERES_INERTIA[inertia as usize];
    ceres_accelerate_x(k, x, u16::from(divisor), game);
    let e = i32::from(k);
    let distance = (session_slot!(game, Ridley[e].base.y_pos) as i16).wrapping_sub(y as i16);
    if distance != 0 {
        session_slot!(
            game,
            Ridley[e].y_velocity = homing_velocity(
                distance,
                session_slot!(game, Ridley[e].y_velocity),
                divisor,
                HomingReversal::DoubleStep,
                1280,
            )
        );
    }
}

/// Ceres Ridley X acceleration ($A6:D6A6). Was `Ridley_Func_107`.
pub fn ceres_accelerate_x<L: Lease + ?Sized>(k: u16, x: u16, divisor: u16, game: &mut L) {
    let e = i32::from(k);
    let distance = (session_slot!(game, Ridley[e].base.x_pos) as i16).wrapping_sub(x as i16);
    if distance != 0 {
        session_slot!(
            game,
            Ridley[e].x_velocity = homing_velocity(
                distance,
                session_slot!(game, Ridley[e].x_velocity),
                divisor as u8,
                HomingReversal::DoubleStep,
                1280,
            )
        );
    }
}

/// Calculate swoop velocities ($A6:D800): bring the swoop speed 20h a frame
/// towards `target_speed`, turn the swoop angle by `angle_step` up to
/// `target_angle`, and split the speed along it. Was `Ridley_Func_111`.
pub fn calculate_swoop_velocities<L: Lease + ?Sized>(
    angle_step: u16,
    target_angle: u16,
    target_speed: u16,
    game: &mut L,
) {
    if target_speed != session_slot!(game, Ridley[RIDLEY].swoop_speed) {
        if (target_speed as i16)
            .wrapping_sub(session_slot!(game, Ridley[RIDLEY].swoop_speed) as i16)
            >= 0
        {
            let mut speed = session_slot!(game, Ridley[RIDLEY].swoop_speed).wrapping_add(32);
            if !below(speed, target_speed) {
                speed = target_speed;
            }
            session_slot!(game, Ridley[RIDLEY].swoop_speed = speed);
        } else {
            let mut speed = session_slot!(game, Ridley[RIDLEY].swoop_speed).wrapping_sub(32);
            if below(speed, target_speed) {
                speed = target_speed;
            }
            session_slot!(game, Ridley[RIDLEY].swoop_speed = speed);
        }
    }
    let mut angle = session_slot!(game, Ridley[RIDLEY].swoop_angle).wrapping_add(angle_step);
    let past_target = if (angle_step & 0x8000) != 0 {
        below(angle, target_angle)
    } else {
        !below(angle, target_angle)
    };
    if past_target {
        angle = target_angle;
    }
    session_slot!(game, Ridley[RIDLEY].swoop_angle = angle);
    let direction = angle.to_ne_bytes()[1] as u16;
    session_slot!(
        game,
        Ridley[RIDLEY].x_velocity =
            x_velocity_at_angle(session_slot!(game, Ridley[RIDLEY].swoop_speed), direction)
    );
    session_slot!(
        game,
        Ridley[RIDLEY].y_velocity =
            y_velocity_at_angle(session_slot!(game, Ridley[RIDLEY].swoop_speed), direction)
    );
}

/// Move Ridley ($A6:D86B): the current enemy by its velocities, clamped to
/// Ridley's bounds; `hit_boundary` records the side hit (1 left, 2 right, 4
/// top, 8 bottom). Was `Ridley_Func_112`.
pub fn move_ridley<S: State + ?Sized>(game: &mut S) {
    session_slot!(game, Ridley[RIDLEY].hit_boundary = 0);
    let k = word!(game.CUR_ENEMY_INDEX);
    let e = word!(game.CUR_ENEMY_INDEX) as i32;
    let carry = session_slot!(byte game, Ridley[e].base.x_subpos[1]) as i32
        + session_slot!(byte game, Ridley[e].x_velocity[0]) as i32;
    session_slot!(byte game, Ridley[e].base.x_subpos[1] = carry as u8);
    let mut x = (session_slot!(game, Ridley[e].base.x_pos) as i32
        + (session_slot!(game, Ridley[e].x_velocity) >> 8) as i8 as i32
        + (carry >> 8)) as u16;
    if (x as i16).wrapping_sub(session_slot!(game, Ridley[RIDLEY].min_x) as i16) >= 0 {
        if (x as i16).wrapping_sub(session_slot!(game, Ridley[RIDLEY].max_x) as i16) >= 0 {
            session_slot!(game, Ridley[e].x_velocity = 0);
            session_slot!(game, Ridley[RIDLEY].hit_boundary = 2);
            x = session_slot!(game, Ridley[RIDLEY].max_x);
        }
        session_slot!(game, Ridley[e].base.x_pos = x);
    } else {
        ceres_earthquake(k, game);
        let copied = session_slot!(game, Ridley[RIDLEY].min_x);
        session_slot!(game, Ridley[e].base.x_pos = copied);
        session_slot!(game, Ridley[e].x_velocity = 0);
        session_slot!(game, Ridley[RIDLEY].hit_boundary = 1);
    }
    let carry = session_slot!(byte game, Ridley[e].base.y_subpos[1]) as i32
        + session_slot!(byte game, Ridley[e].y_velocity[0]) as i32;
    session_slot!(byte game, Ridley[e].base.y_subpos[1] = carry as u8);
    let mut y = (session_slot!(game, Ridley[e].base.y_pos) as i32
        + (session_slot!(game, Ridley[e].y_velocity) >> 8) as i8 as i32
        + (carry >> 8)) as u16;
    if (y as i16).wrapping_sub(session_slot!(game, Ridley[RIDLEY].min_y) as i16) >= 0 {
        if (y as i16).wrapping_sub(session_slot!(game, Ridley[RIDLEY].max_y) as i16) >= 0 {
            session_slot!(game, Ridley[e].y_velocity = 0);
            session_slot!(game, Ridley[RIDLEY].hit_boundary = 8);
            y = session_slot!(game, Ridley[RIDLEY].max_y);
        }
        session_slot!(game, Ridley[e].base.y_pos = y);
    } else {
        let copied = session_slot!(game, Ridley[RIDLEY].min_y);
        session_slot!(game, Ridley[e].base.y_pos = copied);
        session_slot!(game, Ridley[e].y_velocity = 0);
        session_slot!(game, Ridley[RIDLEY].hit_boundary = 4);
    }
}

/// Trigger Ceres Ridley earthquake if moving fast enough ($A6:D914): enemy `k`
/// hitting the left wall at 280h or more, outside Norfair. Was
/// `Ridley_Func_113`.
pub fn ceres_earthquake<S: State + ?Sized>(k: u16, game: &mut S) {
    if word!(game.AREA_INDEX) != NORFAIR {
        let e = i32::from(k);
        let speed = abs16(session_slot!(game, Ridley[e].x_velocity))
            .max(abs16(session_slot!(game, Ridley[e].y_velocity)));
        if speed >= 0x280 {
            let kind = if word!(game.AREA_INDEX) == NORFAIR {
                24
            } else {
                33
            };
            word! {game; EARTHQUAKE_TYPE = kind; EARTHQUAKE_TIMER = 12; }
        }
    }
}

/// Turn Ridley around if not facing room middle ($A6:D955). Was
/// `Ridley_Func_114`.
pub fn face_room_middle<L: Lease + ?Sized>(game: &mut L) {
    let list = match session_slot!(game, Ridley[RIDLEY].facing) {
        FACING_LEFT if !in_right_half(game) => instruction_list::TURN_TO_FACE_RIGHT,
        FACING_LEFT | FACING_FORWARD => return,
        _ if in_right_half(game) => instruction_list::TURN_TO_FACE_LEFT,
        _ => return,
    };
    session_slot!(game, Ridley[RIDLEY].base; current_instruction = list, instruction_timer = 2);
    session_slot!(game, Ridley[RIDLEY].base.timer = 0);
}

/// Whether Ridley's X position mod 100h is at least 80h. The original reads
/// the word at $0F79, whose sign is that bit.
fn in_right_half<L: Lease + ?Sized>(game: &L) -> bool {
    session_slot!(byte game, Ridley[RIDLEY].base.x_pos[0]) >= 0x80
}

/// Update Ridley wings animation ($A6:D97D): ten frames, faster at speed.
/// Was `Ridley_Func_115`.
pub fn update_wings<S: State + ?Sized>(game: &mut S) {
    wings_animation_timer(game);
    let timer = (session_slot!(game, Ridley[RIDLEY].wings_timer) as i16)
        .wrapping_sub(session_slot!(game, Ridley[RIDLEY].wings_timer_delta) as i16);
    session_slot!(game, Ridley[RIDLEY].wings_timer = timer as u16);
    if timer < 0 {
        session_slot!(game, Ridley[RIDLEY].wings_timer = 32);
        let mut frame = session_slot!(game, Ridley[RIDLEY].wings_frame).wrapping_add(1);
        if frame >= 0xa {
            frame = 0;
        }
        session_slot!(game, Ridley[RIDLEY].wings_frame = frame);
    }
}

/// Calculate Ridley wings animation timer ($A6:D9A8): the timer delta from
/// the larger velocity, halved while rising. Was `Ridley_Func_116`.
pub fn wings_animation_timer<S: State + ?Sized>(game: &mut S) {
    /// The wings timer deltas ($A6:D9ED), by speed.
    const DELTAS: [u16; 8] = [0xc, 0xe, 0x10, 0x12, 0x1c, 0x20, 0x28, 0x30];
    let x_speed = abs16(session_slot!(game, Ridley[RIDLEY].x_velocity));
    let sum = x_speed.wrapping_add(abs16(session_slot!(game, Ridley[RIDLEY].y_velocity)));
    if sum == 0 {
        session_slot!(game, Ridley[RIDLEY].wings_timer_delta = 0);
        return;
    }
    let mut speed = sum.wrapping_sub(x_speed);
    if below(speed, x_speed) {
        speed = x_speed;
    }
    let index = (((4 * speed as i32) & 0xf00) >> 8).min(7);
    let mut delta = DELTAS[index as usize];
    if (session_slot!(game, Ridley[RIDLEY].y_velocity) & 0x8000) == 0 {
        delta >>= 1;
    }
    session_slot!(game, Ridley[RIDLEY].wings_timer_delta = delta);
}

/// Initialise Ridley ribs animation ($A6:D9FD). Was `Ridley_Func_117`.
pub fn initialise_ribs<L: Lease + ?Sized>(game: &mut L) {
    session_slot!(game, Ridley[RIDLEY].ribs_pointer = RIBS_ANIMATION);
    session_slot!(game, Ridley[RIDLEY].ribs_timer = 1);
}

/// Handle Ridley ribs animation ($A6:DA0C): when the timer runs out, follow
/// the table to its next frame and queue the frame's two tile rows. Was
/// `Ridley_Func_118`.
pub fn handle_ribs<L: Lease + ?Sized>(game: &mut L) {
    let timer = session_slot!(game, Ridley[RIDLEY].ribs_timer).wrapping_sub(1);
    session_slot!(game, Ridley[RIDLEY].ribs_timer = timer);
    if timer != 0 {
        return;
    }
    let mut cursor = session_slot!(game, Ridley[RIDLEY].ribs_pointer);
    let frame = loop {
        let frame = crate::cartridge::native_record(0xa6, cursor, game);
        if (frame.word(0) & 0x8000) == 0 {
            break frame;
        }
        // A word with bit 15 set is the table's loop pointer.
        cursor = frame.word(0);
        session_slot!(game, Ridley[RIDLEY].ribs_pointer = cursor);
    };
    session_slot!(game, Ridley[RIDLEY].ribs_timer = frame.word(0));
    queue_tile_pair(game, [frame.word(2), frame.word(4)], [29216, 29472], 64);
    session_slot!(game, Ridley[RIDLEY].ribs_pointer = cursor.wrapping_add(6));
}

/// Draw Ridley's feet ($A6:DA8B): queue the clenched or unclenched tiles. Was
/// `Ridley_Func_119`.
pub fn draw_feet<L: Lease + ?Sized>(clenched: bool, game: &mut L) {
    let tiles = if clenched {
        FEET_CLENCHED_TILES
    } else {
        FEET_UNCLENCHED_TILES
    };
    let tiles = crate::cartridge::native_record(0xa6, tiles, game);
    queue_tile_pair(game, [tiles.word(0), tiles.word(2)], [31424, 31680], 128);
}

/// Draw Ridley wings ($A6:DAD8): none while facing forward. Was
/// `Ridley_Func_120`.
pub fn draw_wings<L: Lease + ?Sized>(game: &mut L) {
    let side = match session_slot!(game, Ridley[RIDLEY].facing) {
        FACING_LEFT => 0,
        FACING_FORWARD => return,
        _ => 10,
    };
    let frame = session_slot!(game, Ridley[RIDLEY].wings_frame) as isize + side;
    let spritemap = ridley_data::wing_spritemap(game, frame);
    let x = session_slot!(game, Ridley[RIDLEY].base.x_pos);
    let y = session_slot!(game, Ridley[RIDLEY].base.y_pos);
    let palette = session_slot!(game, Ridley[RIDLEY].palette);
    add_spritemap(spritemap, x, y, palette, game);
}

/// Draw Ridley tail ($A6:DB2A): the tip by its angle, then the segments
/// towards the base. Was `sub_A6DB2A`.
pub fn draw_tail<L: Lease + ?Sized>(game: &mut L) {
    if (session_slot!(game, Ridley[RIDLEY].base.properties) & K_ENEMY_PROPS_INVISIBLE as u16) != 0 {
        return;
    }
    let tip = ridley_data::tail_tip_spritemap(game, tail_tip_sector(game));
    add_ridley_spritemap(
        tip,
        word!(game.TILEMAP_STUFF[tail::x(6)]),
        word!(game.TILEMAP_STUFF[tail::y(6)]),
        game,
    );
    for segment in (0..6).rev() {
        let spritemap = TAIL_SEGMENT_SPRITEMAPS[segment / 2];
        add_ridley_spritemap(
            spritemap,
            word!(game.TILEMAP_STUFF[tail::x(segment)]),
            word!(game.TILEMAP_STUFF[tail::y(segment)]),
            game,
        );
    }
}

/// The tail tip's direction in 16 sectors: segment 5 and 6 angles' low bytes.
fn tail_tip_sector<L: Lease + ?Sized>(game: &L) -> isize {
    (((game.slot_byte(TAIL_RAM + 2 * tail::angle(5)) as i32
        + game.slot_byte(TAIL_RAM + 2 * tail::angle(6)) as i32
        + 8)
        & 0xf0) as u8
        >> 4) as isize
}

/// Add Ridley spritemap to OAM ($A6:DBC2), in his spritemap palette. Was
/// `sub_A6DBC2`.
pub fn add_ridley_spritemap<L: Lease + ?Sized>(spritemap: u16, x: u16, y: u16, game: &mut L) {
    let palette = session_slot!(game, Ridley[RIDLEY].palette);
    add_spritemap(spritemap, x, y, palette, game);
}

/// Y = baby metroid spritemap pointer ($A6:DBCB): advance the baby metroid's
/// instruction list, running its instructions; its spritemap, or `None` when
/// the list is stopped and nothing is drawn. Was
/// `BabyMetroid_DBCB_DoubleRetEx`.
pub fn baby_metroid_spritemap<L: Lease + ?Sized>(game: &mut L) -> Option<u16> {
    if (session_slot!(game, Ridley[RIDLEY].baby_metroid_instruction) & 0x8000) == 0 {
        return None;
    }
    let mut cursor = session_slot!(game, Ridley[RIDLEY].baby_metroid_instruction);
    let mut entry = crate::cartridge::native_record(0xa6, cursor, game);
    let mut instruction = entry.word(0) & 0x8000 != 0;
    if !instruction {
        if session_slot!(game, Ridley[RIDLEY].baby_metroid_instruction_timer) != entry.word(0) {
            session_slot!(
                game,
                Ridley[RIDLEY].baby_metroid_instruction_timer =
                    session_slot!(game, Ridley[RIDLEY].baby_metroid_instruction_timer)
                        .wrapping_add(1)
            );
            return Some(entry.word(2));
        }
        cursor = cursor.wrapping_add(4);
    }
    loop {
        if instruction {
            cursor = run_baby_metroid_instruction(game, entry.word(0), cursor.wrapping_add(2));
            instruction = false;
        } else {
            entry = crate::cartridge::native_record(0xa6, cursor, game);
            if entry.word(0) & 0x8000 == 0 {
                break;
            }
            instruction = true;
        }
    }
    session_slot!(game, Ridley[RIDLEY].baby_metroid_instruction_timer = 1);
    session_slot!(game, Ridley[RIDLEY].baby_metroid_instruction = cursor);
    Some(entry.word(2))
}

/// Add spritemap to OAM ($A6:DC13): the bank $A6 spritemap `spritemap` at
/// (`x`, `y`) with palette bits `palette`; a count, then 5-byte entries of X
/// word, Y byte and tile word, read natively. Entries off the screen's
/// height are skipped. Was `sub_A6DC13`.
pub fn add_spritemap<L: Lease + ?Sized>(
    spritemap: u16,
    x: u16,
    y: u16,
    palette: u16,
    game: &mut L,
) {
    let map = crate::cartridge::native_record(0xa6, spritemap, game);
    let mut count = map.word(0) as i32;
    let mut entry = 2;
    let mut index = word!(game.OAM_NEXT_PTR) as i32;
    let layer1 = game.camera().layer1;
    loop {
        let screen_y = (y as i32 + map.byte(entry + 2) as i8 as i32 - layer1[1] as i32) as i16;
        if screen_y >= 0 && below(screen_y as u16, 224) {
            let screen_x = (x as i32 + map.word(entry) as i32 - layer1[0] as i32) as u16;
            let at = OAM + index as usize;
            game.set_slot_byte(at, screen_x as u8);
            game.set_slot_byte(at + 1, screen_y as u8);
            let tile = (palette | map.word(entry + 3)).to_ne_bytes();
            game.set_slot_byte(at + 2, tile[0]);
            game.set_slot_byte(at + 3, tile[1]);
            word!(
                bits_i32,
                game.OAM_EXT[(index >> 5)] |= (((screen_x as i32 & 0x100) >> 8)
                    | (((map.word(entry) as i16) < 0) as i32 * 2))
                    << (2 * (index >> 2 & 7))
            );
            index = (index + 4) & 0x1ff;
        }
        entry += 5;
        count -= 1;
        if count == 0 {
            break;
        }
    }
    word!(game.OAM_NEXT_PTR = index as u16);
}

/// Check if Ridley is on screen ($A6:DE7A): true when he is off it, with a
/// 20h px margin. Was `Ridley_Func_121`.
pub fn off_screen<S: State + ?Sized>(game: &mut S) -> bool {
    let y = session_slot!(game, Ridley[RIDLEY].base.y_pos) as i16;
    if y >= 0 {
        let layer1 = crate::camera::current(game).layer1;
        let screen_y = (y as i32 + 32 - layer1[1] as i32) as i16;
        if screen_y >= 0 && below(screen_y as u16, 288) {
            let x = session_slot!(game, Ridley[RIDLEY].base.x_pos) as i16;
            if x >= 0 {
                let screen_x = (x as i32 + 32 - layer1[0] as i32) as i16;
                if screen_x >= 0 && below(screen_x as u16, 320) {
                    return false;
                }
            }
        }
    }
    true
}

/// Check for projectile collision with rectangle ($A6:DEA6): the first of
/// Samus's beams (types below 3) in `rect`, which moves to its centre and
/// is marked deflected. Was `Ridley_Func_122`.
pub fn projectile_in_rect<L: Lease + ?Sized>(rect: Rect, game: &mut L) -> Option<usize> {
    if word!(game.PROJECTILE_COUNTER) == 0 {
        return None;
    }
    let n = (0..5).find(|&n| {
        let kind = word!(game.PROJECTILE_TYPE[n]) as i16;
        kind < 0
            && ((kind.to_ne_bytes()[1] as i32 & 0xf) - 3) & 0x8000 != 0
            && projectile_overlaps(game, n, rect)
    })?;
    word! {game; PROJECTILE_X_POS[n] = rect.x; PROJECTILE_Y_POS[n] = rect.y; }
    word!(game.PROJECTILE_DIR[n] |= 0x10u16);
    Some(n)
}

/// Check for Samus collision with rectangle ($A6:DF29). Was `Ridley_Func_124`.
pub fn samus_in_rect<L: Lease + ?Sized>(rect: Rect, game: &mut L) -> bool {
    let dx = abs16(game.player_x().wrapping_sub(rect.x));
    if dx < word!(game.SAMUS_X_RADIUS) || dx.wrapping_sub(word!(game.SAMUS_X_RADIUS)) < rect.w {
        let dy = abs16(game.player_y().wrapping_sub(rect.y));
        if dy < word!(game.SAMUS_Y_RADIUS) || dy.wrapping_sub(word!(game.SAMUS_Y_RADIUS)) < rect.h {
            return true;
        }
    }
    false
}

/// Enemy touch - Ridley ($A6:DF59). Was `sub_A6DF59`.
pub fn touch<L: Lease + ?Sized>(game: &mut L) {
    game.contact_reaction(Death::LeavePending);
}

/// Enemy shot - enemy $E13F/$E17F (Ridley) ($A6:DF8A): Norfair Ridley reacts
/// normally; Ceres Ridley flashes and counts the hit. Was `Ridley_Shot`.
pub fn shot<L: Lease + ?Sized>(game: &mut L) {
    if game.area() == NORFAIR {
        game.shot_reaction(Shot::ReactionOnly);
        return;
    }
    let flash_timer = session_slot!(game, Ridley[RIDLEY].base.flash_timer);
    let flash = if flash_timer != 0 && flash_timer & 1 != 0 {
        14
    } else {
        13
    };
    session_slot!(game, Ridley[RIDLEY].base.flash_timer = flash);
    session_slot!(
        game,
        Ridley[RIDLEY].hit_counter =
            session_slot!(game, Ridley[RIDLEY].hit_counter).wrapping_add(1)
    );
}

/// Power bomb reaction - enemy $E13F/$E17F (Ridley) ($A6:DFB2). Was
/// `Ridley_Powerbomb`.
pub fn power_bomb<L: Lease + ?Sized>(game: &mut L) {
    game.power_bomb_reaction(Death::LeavePending);
}

/// Start Norfair Ridley death sequence ($A6:DFB7), once, at no health. Was
/// `Ridley_Func_126`.
pub fn start_death_sequence<L: Lease + ?Sized>(game: &mut L) {
    if session_slot!(game, Ridley[RIDLEY].base.health) == 0
        && (session_slot!(game, Ridley[RIDLEY].fight_mode) & 0x8000) == 0
    {
        session_slot!(game, Ridley[RIDLEY].fight_mode = DEATH_SEQUENCE);
        set_tangible(game, false);
        session_slot!(
            game,
            Ridley[RIDLEY].function = function::DYING_GRABBED_SAMUS
        );
    }
}

/// Ridley tail / Samus collision handling ($A6:DFD9): the tip hurts and knocks
/// Samus back. Was `Ridley_Func_127`.
pub fn tail_samus_collision<L: Lease + ?Sized>(game: &mut L) {
    let rect = Rect {
        x: word!(game.TILEMAP_STUFF[tail::x(6)]),
        y: word!(game.TILEMAP_STUFF[tail::y(6)]),
        w: 14,
        h: 14,
    };
    if samus_in_rect(rect, game) {
        game.foreign(|game| {
            let amount = game.suit_damage(session_slot!(game, Ridley[RIDLEY].tail_damage));
            game.damage_player(amount);
        });
        word!(game.SAMUS_INVINCIBILITY_TIMER = 96);
        word!(game.SAMUS_KNOCKBACK_TIMER = 5);
        word!(
            game.KNOCKBACK_X_DIR = ((word!(game.SAMUS_X_POS) as i16)
                .wrapping_sub(word!(game.TILEMAP_STUFF[tail::x(6)]) as i16)
                >= 0) as u16
        );
    }
}

/// Ridley tail / projectile collision handling ($A6:E088): a beam on the tip
/// or segment 5 deflects in a dust cloud, missiles with a clank. Was
/// `Ridley_Func_129`.
pub fn tail_projectile_collision<L: Lease + ?Sized>(game: &mut L) {
    if (session_slot!(game, Ridley[RIDLEY].base.properties) & K_ENEMY_PROPS_TANGIBLE as u16) != 0 {
        return;
    }
    let tip = Rect {
        x: word!(game.TILEMAP_STUFF[tail::x(6)]),
        y: word!(game.TILEMAP_STUFF[tail::y(6)]),
        w: 14,
        h: 14,
    };
    let Some(n) = projectile_in_rect(tip, game).or_else(|| {
        let segment = Rect {
            x: word!(game.TILEMAP_STUFF[tail::x(5)]),
            y: word!(game.TILEMAP_STUFF[tail::y(5)]),
            w: 10,
            h: 10,
        };
        projectile_in_rect(segment, game)
    }) else {
        return;
    };
    let x = word!(game.PROJECTILE_X_POS[n]);
    let y = word!(game.PROJECTILE_Y_POS[n]);
    game.place_projectile_spawn(x, y);
    let mut kind = 12;
    if projectile_kind(game, n) == MISSILE {
        game.queue_sound(0, 0x3d, 6);
        kind = 6;
    }
    game.spawn_room_projectile(DUST_CLOUD, kind);
}

/// Instruction - roar ($A6:E4BE). Was `Ridley_Instr_5`.
pub fn roar<L: Lease + ?Sized>(script: EnemyScript, game: &mut L) -> EnemyScript {
    session_slot!(game, Ridley[RIDLEY].roaring = 89);
    game.queue_sound(1, 0x59, 6);
    script
}

/// Instruction - clear roaring flag ($A6:E4CA). Was `Ridley_Instr_6`.
pub fn stop_roaring<L: Lease + ?Sized>(script: EnemyScript, game: &mut L) -> EnemyScript {
    session_slot!(game, Ridley[RIDLEY].roaring = 0);
    script
}

/// Instruction - go to [[Y]] if not Norfair and Samus has low health
/// ($A6:E4D2): below 30. Was `Ridley_Instr_10`.
pub fn goto_if_samus_health_low<L: Lease + ?Sized>(
    script: EnemyScript,
    game: &mut L,
) -> EnemyScript {
    if game.area() == NORFAIR || !below(word!(game.SAMUS_HEALTH), 30) {
        return script.offset(1);
    }
    session_slot!(game, Ridley[RIDLEY].timer = 8);
    EnemyScript::jump(script.word(0))
}

/// Unused. Instruction - go to [[Y]] if holding Samus, else go to [[Y] + 2]
/// ($A6:E4EE). Was `Ridley_Instr_4`.
pub fn goto_by_holding_samus<L: Lease + ?Sized>(script: EnemyScript, game: &mut L) -> EnemyScript {
    if session_slot!(game, Ridley[RIDLEY].holding_samus) != 0 {
        EnemyScript::jump(script.word(0))
    } else {
        EnemyScript::jump(script.word(2))
    }
}

/// Unused. Instruction - go to [[Y]] if not holding Samus ($A6:E4F8). Was
/// `Ridley_Instr_3`.
pub fn goto_if_not_holding_samus<L: Lease + ?Sized>(
    script: EnemyScript,
    game: &mut L,
) -> EnemyScript {
    if session_slot!(game, Ridley[RIDLEY].holding_samus) != 0 {
        script.offset(1)
    } else {
        EnemyScript::jump(script.word(0))
    }
}

/// Instruction - Ridley feet distance index = [[Y]] ($A6:E501); Samus's
/// previous position catches up. Was `Ridley_Instr_2`.
pub fn set_feet_distance<L: Lease + ?Sized>(script: EnemyScript, game: &mut L) -> EnemyScript {
    game.set_previous_player_x(word!(game.SAMUS_X_POS));
    game.set_previous_player_y(word!(game.SAMUS_Y_POS));
    session_slot!(game, Ridley[RIDLEY].feet_distance_index = script.word(0));
    script.offset(1)
}

/// Instruction - go to [[Y]] if not facing left ($A6:E517). Was
/// `Ridley_Instr_1`.
pub fn goto_if_not_facing_left<L: Lease + ?Sized>(
    script: EnemyScript,
    game: &mut L,
) -> EnemyScript {
    if session_slot!(game, Ridley[RIDLEY].facing) != FACING_LEFT {
        EnemyScript::jump(script.word(0))
    } else {
        script.offset(1)
    }
}

/// Instruction - move Ridley ([[Y]], [[Y] + 2]) pixels ($A6:E51F). Was
/// `Ridley_Instr_14`.
pub fn move_by<L: Lease + ?Sized>(script: EnemyScript, game: &mut L) -> EnemyScript {
    session_slot!(
        game,
        Ridley[RIDLEY].base.x_pos =
            session_slot!(game, Ridley[RIDLEY].base.x_pos).wrapping_add(script.word(0))
    );
    session_slot!(
        game,
        Ridley[RIDLEY].base.y_pos =
            session_slot!(game, Ridley[RIDLEY].base.y_pos).wrapping_add(script.word(2))
    );
    script.offset(2)
}

/// Instruction - flip Ridley left ($A6:E71C). Was `Ridley_Instr_9`.
pub fn flip_left<L: Lease + ?Sized>(script: EnemyScript, game: &mut L) -> EnemyScript {
    session_slot!(game, Ridley[RIDLEY].facing = FACING_LEFT);
    flip_tail(game);
    script
}

/// Instruction - face Ridley forward ($A6:E727). Was `Ridley_Instr_7`.
pub fn face_forward<L: Lease + ?Sized>(script: EnemyScript, game: &mut L) -> EnemyScript {
    session_slot!(game, Ridley[RIDLEY].facing = FACING_FORWARD);
    script
}

/// Instruction - flip Ridley right ($A6:E72F). Was `Ridley_Instr_8`.
pub fn flip_right<L: Lease + ?Sized>(script: EnemyScript, game: &mut L) -> EnemyScript {
    session_slot!(game, Ridley[RIDLEY].facing = FACING_RIGHT);
    flip_tail(game);
    script
}

/// Instruction - calculate fireball X/Y velocities ($A6:E84D): 500h towards
/// Samus from the mouth, within the facing side's cone. Was
/// `Ridley_Instr_11`.
pub fn calculate_fireball_velocities<L: Lease + ?Sized>(
    script: EnemyScript,
    game: &mut L,
) -> EnemyScript {
    let angle = if session_slot!(game, Ridley[RIDLEY].facing) != 0 {
        let dx = (game.player_x() as i32
            - (session_slot!(game, Ridley[RIDLEY].base.x_pos) as i32 + 25)) as u16;
        let dy = (game.player_y() as i32
            - (session_slot!(game, Ridley[RIDLEY].base.y_pos) as i32 - 43)) as u16;
        let mut angle = -(angle_from_delta(dx, dy) as i32 + 0x80) as u8 as u16;
        if !(0x15..0xc0).contains(&angle) {
            angle = 0x15;
        } else if angle >= 0x50 {
            angle = 0x50;
        }
        angle
    } else {
        let dx = (game.player_x() as i32
            - (session_slot!(game, Ridley[RIDLEY].base.x_pos) as i32 - 25)) as u16;
        let dy = (game.player_y() as i32
            - (session_slot!(game, Ridley[RIDLEY].base.y_pos) as i32 - 43)) as u16;
        let mut angle = -(angle_from_delta(dx, dy) as i32 + 0x80) as u8 as u16;
        if !(0x40..0xeb).contains(&angle) {
            angle = 0xeb;
        } else if angle < 0xb0 {
            angle = 0xb0;
        }
        angle
    };
    session_slot!(
        game,
        Ridley[RIDLEY].fireball_x_velocity = x_velocity_at_angle(0x500, angle)
    );
    session_slot!(
        game,
        Ridley[RIDLEY].fireball_y_velocity = y_velocity_at_angle(0x500, angle)
    );
    script
}

/// Instruction - spawn Ridley's fireball enemy projectile - with afterburn
/// ($A6:E904). Was `Ridley_Instr_12`.
pub fn fireball_with_afterburn<L: Lease + ?Sized>(
    script: EnemyScript,
    game: &mut L,
) -> EnemyScript {
    spawn_fireball(script, 0, game)
}

/// Instruction - spawn Ridley's fireball enemy projectile - without afterburn
/// ($A6:E909). Was `Ridley_Instr_13`.
pub fn fireball_without_afterburn<L: Lease + ?Sized>(
    script: EnemyScript,
    game: &mut L,
) -> EnemyScript {
    spawn_fireball(script, 0xe, game)
}

/// Spawn Ridley's fireball enemy projectile ($A6:E90C): `afterburn` 0 spawns
/// the afterburn. Was `Ridley_E90C`.
pub fn spawn_fireball<L: Lease + ?Sized>(
    script: EnemyScript,
    afterburn: u16,
    game: &mut L,
) -> EnemyScript {
    word!(game.EPROJ_INIT_PARAM_2 = afterburn);
    game.spawn_room_projectile(FIREBALL, session_slot!(game, Ridley[RIDLEY].facing));
    script
}

/// Instruction - start liftoff - Ceres Ridley ($A6:E969). Was `Ridley_Instr_15`.
pub fn start_ceres_liftoff<L: Lease + ?Sized>(script: EnemyScript, game: &mut L) -> EnemyScript {
    session_slot!(
        game,
        Ridley[RIDLEY].function = function::CERES_LIFTOFF_ACCELERATING
    );
    session_slot!(game, Ridley[RIDLEY].y_velocity = -352i32 as u16);
    script
}

/// Instruction - start liftoff - Norfair Ridley ($A6:E976). Was
/// `Ridley_Instr_16`.
pub fn start_norfair_liftoff<L: Lease + ?Sized>(script: EnemyScript, game: &mut L) -> EnemyScript {
    session_slot!(game, Ridley[RIDLEY].function = function::LIFTOFF);
    session_slot!(game, Ridley[RIDLEY].y_velocity = -352i32 as u16);
    script
}
