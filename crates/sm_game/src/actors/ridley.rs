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
//! and its escape sequence are in `ceres_ridley`, whose `run_function`
//! dispatches both through the bank's function table.
use crate::actors::boss_lease::bg2_staging;
use crate::actors::boss_lease::{BossLease, BossState};
use crate::actors::layouts::{EnemyBase, same_words};
use crate::actors::session_slots::{SlotAccess, session_slot};
use crate::constants::enemy_properties::K_ENEMY_PROPS_INVISIBLE;
use crate::display::transfer::{Source, TilemapUpload, queue_tilemap};
use crate::menus::prelude::battle;
use crate::sprite_drawing::Clip;
use crate::{ridley_data, sprite_output};

/// A lease with Ridley's effects, Norfair's and Ceres's, that types the self
/// destruct text, on a game with the shared boss effects, deaths and sprites.
pub trait Lease:
    crate::actors::boss_interaction::Access
    + BossLease<Game: GameEffects>
    + crate::actors::ceres_ridley::Host
    + crate::escape_typewriter::Host
{
}
impl<T> Lease for T where
    T: crate::actors::boss_interaction::Access
        + BossLease<Game: GameEffects>
        + crate::actors::ceres_ridley::Host
        + crate::escape_typewriter::Host
        + ?Sized
{
}

/// What Ridley's lease still reaches in the game: the shared boss effects,
/// deaths and sprites.
pub trait GameEffects:
    crate::actors::host::BossEffects + crate::actors::deaths::Deaths + crate::sprite_output::SpriteHost
{
}
impl<T> GameEffects for T where
    T: crate::actors::host::BossEffects
        + crate::actors::deaths::Deaths
        + crate::sprite_output::SpriteHost
        + ?Sized
{
}

/// What Ridley needs beyond the shared boss state steps.
pub trait Host {
    /// Spawn the enemy population at `bank:population`.
    fn spawn_population(&mut self, bank: u8, population: u16) -> u16;
    /// Mark the current area's boss bits `mask` as defeated.
    fn set_boss_bit(&mut self, mask: u16);
    /// `amplitude` times the sine and the cosine of `angle`, in that order,
    /// through the PPU multiplier.
    fn hardware_sine_cosine(&mut self, amplitude: u16, angle: u16) -> (u16, u16);
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
/// Ridley's feet tiles in bank $B0 ($A6:DAD0): top and bottom.
const FEET_UNCLENCHED_TILES: u16 = 0xdad0;
const FEET_CLENCHED_TILES: u16 = 0xdad4;
/// The ribs animation table ($A6:DA71).
const RIBS_ANIMATION: u16 = 0xda71;
/// Ridley tail segment spritemaps ($A6:DC90).
const TAIL_SEGMENT_SPRITEMAPS: [u16; 3] = [0xdc90, 0xdc97, 0xdc9e];
/// Bank $86 enemy projectiles.
const DUST_CLOUD: u16 = 0xe509;
const FIREBALL: u16 = 0x9642;
const SPIN_JUMPING: u8 = 3;
/// The VRAM write queue.
const VRAM_WRITE_QUEUE: usize = 0xd0;
/// One VRAM write queue entry: size, long source, destination.
/// Ridley's tiles live in bank $B0.
const TILE_BANK: u8 = 0xb0;
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

/// Count down Ridley's function timer; true once it is negative.

/// The work RAM address of tail word `index`.
const fn tail_address(index: usize) -> u16 {
    (TAIL_RAM + 2 * index) as u16
}

/// Queue two VRAM transfers of `size` bytes from bank $B0, then the
/// terminating size the original stores after them.
pub fn queue_tile_pair<L: Lease + ?Sized>(
    game: &mut L,
    sources: [u16; 2],
    destinations: [u16; 2],
    size: u16,
) {
    for (address, vram_address) in sources.into_iter().zip(destinations) {
        let source = Source {
            address,
            bank: TILE_BANK,
        };
        queue_tilemap(
            game,
            TilemapUpload {
                bytes: size,
                source,
                column: false,
                vram_address,
            },
        );
    }
    let end = VRAM_WRITE_QUEUE + usize::from(game.tilemap_queue_tail());
    game.set_tilemap_queue_byte(end, 0);
    game.set_tilemap_queue_byte(end + 1, 0);
}

/// How Ridley's accelerations decelerate: by `force` + 8 more, or by a doubled
/// step at Ceres.
#[derive(Clone, Copy)]
enum Deceleration {
    Force(u16),
    DoubleStep,
}

/// One axis of Ridley's accelerations toward `target`, with the carry the
/// original chains through it: the position comparison's carry feeds the
/// first step add or subtract that no `CLC`/`SEC` precedes, and each one's
/// carry the next. So a step takes or adds one more when `position` and
/// `target` compare differently signed and unsigned. Distance, acceleration
/// and the limit test wrap at sixteen bits; the limit is a signed-difference
/// test, not an ordinary clamp.

/// Move the current enemy by its X and Y velocities: subpixels in the low
/// bytes, pixels in the signed high bytes. Was `MoveEnemyWithVelocity`.

/// Handle baby metroid ($A6:A2DC): run its function, then hook the Ceres
/// door and baby metroid drawing once Ridley moves, or draw them now. Was
/// `Ridley_A2DC`.

/// Enemy graphics drawn hook - Ceres Ridley - draw baby metroid and door
/// ($A6:A2F2). Was `Ridley_A2F2`.
pub fn draw_baby_metroid_and_door<L: Lease + owned::Host>(game: &mut L) {
    /// Ceres Ridley door X offsets during earthquake ($A6:A321).
    const EARTHQUAKE_X_OFFSETS: [i16; 4] = [0, -1024, -4, -1];
    if battle!(game.CeresStatus) == 0 {
        draw_baby_metroid(game);
    }
    if session_slot!(game, CeresDoor[CERES_DOOR].drawn_by_ridley) != 0 {
        let offset = EARTHQUAKE_X_OFFSETS[(battle!(game.EarthquakeTimer) & 3) as usize];
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

/// Run Ridley's bank $A6 function `function` on this lease; false when it
/// is not one of Norfair Ridley's.

/// Time is frozen AI - enemy $E17F (Ridley) ($A6:B28A). Was `Ridley_Func_2`.

/// Hurt AI - enemy $E17F (Ridley) ($A6:B297): on even frames Ridley acts as in
/// `main`; then the grabbed Samus lags until the timer reaches its limit.
/// Was `Ridley_Hurt`.

/// Ridley grabbed Samus movement hurt AI lag time ($A6:B288).
const GRABBED_HURT_LAG: u16 = 8;

/// Ridley function - liftoff ($A6:B2F3). Was `Ridley_Func_3`.

/// Ridley function - decide action ($A6:B321): a random one of the eight
/// choices. Was `Ridley_Func_3b`.

/// Get fight action choices ($A6:B335). Was `Ridley_Func_4`.

/// Ridley function - lunge missed - setup ($A6:B3EC). Was `Ridley_Func_5`.

/// Ridley function - lunge missed - move to position ($A6:B3F8). Was
/// `Ridley_Func_6`.

/// Y = [$B439 + [Ridley acceleration index] * 2] ($A6:B42E): the acceleration
/// factor for lunge missed and dodging power bomb. Was `Ridley_Func_7`.

/// Ridley action - swoop ($A6:B441). Was `Ridley_Func_8`.

/// Ridley function - swoop - move to position ($A6:B455). Was `Ridley_B455`.

/// One swoop stage: steer by `angle_step` towards `target_angle` at
/// `target_speed`, mirrored when facing left, until the timer runs out.

/// Ridley function - swoop - descending - aiming down ($A6:B493). Was
/// `Ridley_Func_9`.

/// Ridley function - swoop - descending - aiming horizontal ($A6:B4D1). Was
/// `Ridley_Func_10`.

/// Ridley function - swoop - ascending - aiming up ($A6:B516). Was
/// `Ridley_Func_11`.

/// Ridley function - swoop - ascending - aiming up faster ($A6:B554). Was
/// `Ridley_Func_12`.

/// Ridley function - swoop - ascending - decelerate ($A6:B594). Was
/// `Ridley_Func_13`.

/// Ridley action - hover ($A6:B5C4). Was `Ridley_Func_14`.

/// Ridley function - hover - move to wall behind Ridley ($A6:B5E5). Was
/// `Ridley_Func_15`.

/// Ridley function - hover - move to wall in front of Ridley ($A6:B613). Was
/// `Ridley_Func_16`.

/// Ridley hover movement ($A6:B641): hover towards `x`; pogo unless Samus
/// spin jumps, and cross to the other wall (`timeout`) when the function
/// timer runs out. Was `advance_ridley_instruction_with_timeout`.

/// Ridley hover movement ($A6:B641): the movement towards (`x`, Samus's Y, at
/// most 352), with a tail whip and sometimes fireballs while Samus spin
/// jumps; true when she does not, so the pogo starts. Was `Ridley_Func_17`.

/// Set up Ridley pogo from hover ($A6:B68B). Was `Ridley_Func_18`.

/// Ridley action / function - pogo - fly to position ($A6:B6A7). Was
/// `Ridley_Func_19`.

/// Ridley function - pogo - wait at position ($A6:B6DD). Was `Ridley_Func_20`.

/// Ridley function - pogo - descending ($A6:B70E): grab Samus, or bounce off
/// the floor with a dust cloud, a quake and every other time fireballs. Was
/// `Ridley_Func_21`.

/// Ridley function - pogo - ascending ($A6:B7B9). Was `Ridley_Func_22`.

/// Check for tail collision with floor ($A6:B7E7): the tip, then segments 5
/// down to 2, just below each. Was `Ridley_Func_23`.

/// Set neutral Ridley tail ($A6:B84D). Was `Ridley_Func_24`.

/// Check if Samus is grabbable ($A6:B859). Was `Ridley_Func_25`.

/// Check for Samus collision with Ridley feet ($A6:B865), with feet radii
/// `x_radius` and `y_radius`. Was `Ridley_Func_26`.

/// Ridley grabs Samus from pogo ($A6:B889): Y velocity = -max(200h, |Y
/// velocity|). Was `Ridley_Func_27`.

/// Set Ridley pogo X movement direction ($A6:B8A9): towards the room inside
/// at the bounds, otherwise mostly towards Samus. Was `Ridley_Func_28`.

/// Move Samus with Ridley's feet ($A6:B9E1): the grabbed offsets settle 4px
/// a frame towards the feet. Was `Ridley_Func_30`.

/// A grabbed offset 4px closer to 0, or `None` when it is already 0. Was
/// `reduce_ridley_axis_speed`.

/// Set grabbed Samus X/Y offset ($A6:BA54). Was `Ridley_Func_31`.

/// Ridley lunge missed Samus ($A6:BA85): dodge a power bomb, retreat, or after
/// ten lunges at no health, die. Was `Ridley_Func_32`.

/// Ridley action / function - lunge ($A6:BAB7): while Samus is below and on
/// the lunging side, home in on her and grab her when the feet touch. Was
/// `Ridley_Func_33`.

/// Ridley function - grabbed Samus - setup ($A6:BB8F). Was `Ridley_Func_33b`.

/// Ridley function - grabbed Samus - rising to up to 40h px ($A6:BBC4). Was
/// `Ridley_Func_34`.

/// Ridley function - grabbed Samus - rising to room middle ($A6:BBF1). Was
/// `Ridley_Func_35`.

/// Ridley function - grabbed Samus - release Samus ($A6:BC2E). Was
/// `Ridley_Func_36`.

/// Grab Samus ($A6:BC68). Was `Ridley_Func_37`.

/// Release Samus ($A6:BC84). Was `Ridley_Func_38`.

/// Set Ridley tangibility ($A6:BCB4): intangible off-screen while fighting,
/// and tangible again when the intangibility timer runs out. Was
/// `Ridley_Func_39`.

/// Check if Samus is morphed or spin jumping / damage boosting ($A6:BCF1):
/// the carry, Samus is neither spin jumping nor damage boosting. Was
/// `Ridley_Func_40_Carry`.

/// Check if Samus is morphed or spin jumping / damage boosting ($A6:BCF1):
/// the overflow, Samus is morphed. Was `Ridley_Func_40_Sign`.

/// $A6:BD04, by Samus movement type: bit 7 not spin jumping or damage
/// boosting, bit 6 morphed.
const MOVEMENT_TYPE_FLAGS: [u8; 28] = [
    0x80, 0x80, 0x80, 0, 0xff, 0x80, 0x80, 0xff, 0xff, 0xff, 0x80, 0, 0, 0x80, 0x80, 0x80, 0x80,
    0xff, 0xff, 0xff, 0x80, 0x80, 0, 0x80, 0x80, 0, 0, 0x80,
];

/// Check if Samus is in pogo zone ($A6:BD20): at or below Y 352. Was
/// `Ridley_Func_41`.

/// Lunge if power bomb active and not holding Samus ($A6:BD2C). Was
/// `Ridley_Func_42`.

/// Ridley function - dodging power bomb ($A6:BD4E): to the corner away from
/// the explosion, then back to the fight. Was `Ridley_Func_43`.

/// Ridley function - fake retreat - move to position ($A6:BD9A). Was
/// `Ridley_Func_44`.

/// Ridley function - fake retreat - rising ($A6:BDBC): off the top of the
/// screen, then drop the baby metroid. Was `Ridley_Func_45`.

/// Ridley function - fake retreat - drop baby metroid ($A6:BDF2). Was
/// `Ridley_Func_46`.

/// Ridley function - fake retreat - retrieve baby metroid ($A6:BE03): swoop
/// down and catch it in the feet, then retreat for real. Was
/// `Ridley_Func_47`.

/// Check for baby metroid collision with rectangle ($A6:BE61); the baby
/// metroid's radii are 4. Was `Ridley_Func_48`.

/// Run the bank $A6 baby metroid instruction `instruction`, whose arguments
/// start at `arguments`; the next instruction's address.

/// Run Ridley's bank $A6 instruction at `ea` with its arguments at `script`;
/// the script back when it is not one of his.

/// Execute baby metroid function ($A6:BE93). Was `Ridley_Func_49`.

/// Baby metroid function - held by Ridley ($A6:BE9C). Was `Ridley_Func_50`.

/// Baby metroid function - held by Ridley's feet ($A6:BEB3). Was
/// `Ridley_Func_51`.

/// Baby metroid function - start falling ($A6:BECA). Was `Ridley_Func_52`.

/// Baby metroid function - falling ($A6:BEDC): accelerate down to the floor
/// at Y 192. Was `Ridley_Func_53`.

/// Draw baby metroid ($A6:BF1A), in palette 0. Was `DrawBabyMetroid_0`.
pub fn draw_baby_metroid<L: Lease + owned::Host>(game: &mut L) {
    let Some(spritemap) = baby_metroid_spritemap(game) else {
        return;
    };
    let x = session_slot!(game, Ridley[RIDLEY].baby_metroid_x);
    let y = session_slot!(game, Ridley[RIDLEY].baby_metroid_y);
    add_spritemap(spritemap, x, y, 0, game);
}

/// Baby metroid instruction - go to [[X]] with 50% chance if not moving
/// ($A6:BFC9), else the flapping sound. Was `BabyMetroid_Instr_2`.

/// Baby metroid instruction - load palette data [[X]] ($A6:BFE1). Was
/// `BabyMetroid_Instr_3`.
pub fn baby_metroid_load_palette<L: Lease + crate::actors::minor_interaction::NativeCapture>(
    k: u16,
    game: &mut L,
) -> u16 {
    let colours = crate::cartridge::native_record(0xa6, k, game).word(0);
    crate::actors::minor_interaction::native_palette(game, false, 0x162, 0xa6, colours, 0xf);
    k.wrapping_add(2)
}

/// Baby metroid instruction - go to [[X]] if moving ($A6:BFF2). Was
/// `BabyMetroid_Instr_1`.

/// Baby metroid instruction - go to [[X]] ($A6:BFF8). Was `BabyMetroid_Goto`.

/// Ridley function - dying - grabbed Samus ($A6:C538). Was `Ridley_Func_63`.

/// Ridley function - death sequence - death roar ($A6:C53E). Was
/// `Ridley_C53E`.

/// Ridley function - death sequence - move to death spot ($A6:C551): then stop
/// and lower the acid. Was `Ridley_C551`.

/// Ridley function - death sequence - lowering acid and release Samus
/// ($A6:C588). Was `Ridley_Func_64`.

/// Ridley function - death sequence - start falling apart ($A6:C5A8). Was
/// `Ridley_Func_65`.

/// Ridley function - death sequence - falling apart ($A6:C5C8). Was
/// `Ridley_Func_66`.

/// Ridley function - death sequence - still falling apart ($A6:C5DA): the boss
/// is defeated, his drops scatter and the music returns. Was
/// `Ridley_Func_67`.

/// Move Ridley to death spot ($A6:C601); true once he is there. Was
/// `Ridley_Func_68`.

/// Spawn small explosion near Ridley ($A6:C623): a dust cloud every five
/// frames, cycling through ten offsets. Was `Ridley_Func_69`.

/// Initialisation AI - enemy $E1BF (Ridley explosion) ($A6:C696): the part
/// picks the place and instruction list; a random X drift. Was
/// `RidleysExplosion_Init`.

/// Ridley explosion initialisation - index = Eh..16h ($A6:C7DA..C8A2): explosion
/// `e` on Ridley's `part`, its X from the part's placement for his facing and
/// its Y `y_offset` from his. Was `place_ridley_explosion_x`.

/// Main AI - enemy $E1BF (Ridley explosion) ($A6:C8D4): flicker, slow the
/// drift, fall, and die when the lifetime runs out. Was
/// `RidleysExplosion_Main`.

/// Handle flickering ($A6:C913): explosion `k` is invisible on odd frames.
/// Was `RidleysExplosion_Func_1`.

/// Spawn Ridley explosion enemies ($A6:C932). Was `RidleysExplosion_Func_2`.

/// Ceres escape sequence "emergency" text glow ($A6:C19C): every fourth frame
/// without a palette change or door transition, the next of 16 glow steps.
/// Was `Ridley_Func_61`.
pub fn emergency_text_glow<L: Lease + ?Sized>(game: &mut L) {
    if battle!(game.DoorTransitionEnemies) | crate::sequence_word!(game.PALETTE_CHANGE_NUM) == 0
        && (game.frame() & 3) == 0
    {
        let mut step = session_slot!(game, Ridley[RIDLEY].emergency_glow_index).wrapping_add(1);
        if step >= 0x10 {
            step = 0;
        }
        session_slot!(game, Ridley[RIDLEY].emergency_glow_index = step);
        let colour =
            session_slot!(game, Ridley[RIDLEY].emergency_glow_index) as isize + 2 * step as isize;
        for offset in 0..3usize {
            let value = ridley_data::emergency_glow(game, colour + offset as isize);
            game.session_mut().presentation.visual_mut().colors[97 + offset] = value;
        }
    }
}

/// Handle Ridley tail ($A6:CAF5): the whip sound, the tail function, the
/// segment positions and distances, and the tail hitting Samus. Was
/// `Ridley_Func_70`.

/// Set up Ridley pogo tail ($A6:CB33), tail function index 2. Was
/// `Ridley_Func_71`.

/// Ridley tail function index 3 - pogo - point tail down ($A6:CB45). Was
/// `Ridley_Func_72`.

/// Ridley tail function index 6 - pogo - descending - stab ($A6:CB4E). Was
/// `Ridley_Func_73`.

/// Ridley tail function index 7 - unused - pogo descending - stab ($A6:CB60).
/// Was `Ridley_Func_74`.

/// Ridley pogo tail movement with tail pointed down ($A6:CB72): whip towards
/// straight down, then the normal pogo tail once no segment moves. Was
/// `Ridley_Func_75`.

/// Ridley tail function index 1 - neutral (natural tail movement + tail
/// whips) ($A6:CBC0). Was `Ridley_CBC0`.

/// Ridley pogo tail movement / Ridley tail function index 4 - pogo - normal
/// ($A6:CBC7). Was `Ridley_CBC7`.

/// Ridley tail function index 5 - pogo - descending - stab setup ($A6:CBCE).
/// Was `Ridley_CBCE`.

/// Ridley tail function index 8 - unused - neutral with spinny tail whips
/// ($A6:CBD5). Was `Ridley_CBD5`.

/// Update Ridley tail segment angles ($A6:CBDC). Was `Ridley_CBDC`.

/// Set Ridley tail angle extrema ($A6:CBFE), by facing direction. Was
/// `Ridley_Func_77`.

/// Check if all Ridley tail segments are active ($A6:CC1E), tip first. Was
/// `Ridley_Func_78`.

/// Whether any tail segment is active, tip first.

/// Handle Ridley tail whip ($A6:CC39). Was `Ridley_Func_79`.

/// The end of the tail whip handlers ($A6:CC39, CCBD): once every segment
/// rests, swing again from the base, faster when Samus is right of 70h. Was
/// `start_ridley_tile_effect_when_idle`.

/// Configure Ridley tail whip ($A6:CC7D): on a whip request, at random while
/// idle whips are enabled, or when Samus is near; true when a whip started.
/// Was `Ridley_Func_80_DoubleRet`.

/// Unused. Handle Ridley tail whip with extra spinning ($A6:CCBD). Was
/// `Ridley_Func_81`.

/// Unused. Configure Ridley tail whip with extra spinning ($A6:CD0C); true
/// when a whip started. Was `Ridley_Func_82_DoubleRet`.

/// The pogo tail's whip when Samus is near or at random, shared by
/// `handle_normal_pogo_tail` and `handle_pogo_tail_stab`.

/// Handle normal Ridley pogo tail ($A6:CD24): whip, or once the tail rests
/// count down to its next swing; set up the stab while rising. Was
/// `Ridley_Func_83`.

/// Handle Ridley pogo tail stab ($A6:CDAA): as the normal pogo tail, but
/// falling it points the tail straight down at full length. Was
/// `Ridley_Func_84`.

/// Set tail angle high bytes to 40h ($A6:CE65), through raw work RAM. Was
/// `Ridley_Func_85`.

/// Calculate Ridley tail segment positions ($A6:CEBA): each segment is the
/// previous one plus its offset; facing forward the tail hangs straight
/// down. Was `Ridley_Func_86`.

/// Segments 1 to 6 of one axis from segment 0's `first` word: two segments at
/// a time, the second from both offsets rather than the first's new value.

/// Update Ridley tail segment distances ($A6:CF5A): segments 1 to 6 extend
/// towards their target distances or relax to their resting ones. Was
/// `Ridley_Func_87`.

/// Update Ridley tail segment angle ($A6:D09F): an active segment staggers,
/// then swings between the angle extrema or towards a whip target, and stops
/// at a target once the segment before it has; every segment then takes its
/// offset from its angle and distance. Was `Ridley_Func_88`.

/// At a whip target a segment stops once the one before it has.

/// Set Ridley tail whip target ($A6:D19D): aim the whip at a missile near the
/// tip, or else at Samus, within reach of the facing side. Was
/// `Ridley_Func_89`.

/// Detect (super) missile near Ridley tail ($A6:D242): the first missile or
/// super missile within 64px of the tip. Was `Ridley_Func_90`.

/// The low nibble of projectile `n`'s type high byte: its weapon.

/// Whether projectile `n`'s hitbox overlaps `rect`.

/// Handle Ridley tail whip sound effect ($A6:D2AA): every 16 frames of a fast
/// tail, unless roaring. Was `Ridley_Func_91`.

/// Initialise Ridley tail RAM ($A6:D2D6). Was `Ridley_Func_92`.

/// Ridley tail segment active flags = 8000h ($A6:D3D9). Was `Ridley_Func_95`.

/// Ridley tail segment active flags = [A] ($A6:D3DC). Was `Ridley_Func_D3DC`.

/// Flip Ridley tail ($A6:D3F9): mirror every segment's angle and turn it
/// anticlockwise. Was `Ridley_Func_96`.

/// The damage deal suit-adjusted enemy damage to Samus ($A6:D453) deals: the
/// current enemy's contact damage, from its definition. Mother Brain shares it.
pub fn contact_damage(game: &mut (impl BossState + ?Sized)) -> u16 {
    let key = i32::from(game.current_enemy());
    let definition = session_slot!(game, Ridley[key].base.enemy_ptr);
    owned::definition_damage(crate::cartridge::native_record(
        ENEMY_DEFINITION_BANK,
        definition,
        game,
    ))
}

/// Set Ridley instruction list ($A6:D467). Was `Ridley_Func_99`.

/// Norfair Ridley health-based palette handling ($A6:D474): below 9000, 5400
/// and 1800 health Ridley accelerates harder and glows redder. Was
/// `Ridley_Func_100`.

/// Load Ridley health-based palette ($A6:D495) `index` into sprite palette 7
/// unless a palette change is pending. Was `Ridley_D495`.
pub fn load_health_palette<L: Lease + crate::actors::minor_interaction::NativeCapture>(
    index: u16,
    game: &mut L,
) {
    /// Ridley health-based palettes ($A6:E46A), 14 colours each.
    const HEALTH_PALETTES: u16 = 0xe46a;
    if crate::sequence_word!(game.PALETTE_CHANGE_NUM) == 0 {
        let colours = HEALTH_PALETTES.wrapping_add(index.wrapping_mul(28));
        crate::actors::minor_interaction::native_palette(game, false, 0x1e2, 0xa6, colours, 0xe);
    }
}

/// Ceres Ridley health-based palette handling ($A6:D4B5): by the hit counter.
/// Was `Ridley_Func_101`.
pub fn ceres_health_palette<L: Lease + crate::actors::minor_interaction::NativeCapture>(
    game: &mut L,
) {
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
pub fn hurt_flash<S: State + crate::session_access::SessionAccess + ?Sized>(game: &mut S) {
    let mut palette = 3584;
    if session_slot!(game, Ridley[RIDLEY].base.flash_timer) >= 2
        && ((game.session().actors.controls.phase_counter as i32 + 1) & 2) != 0
    {
        palette = 0;
    }
    session_slot!(game, Ridley[RIDLEY].palette = palette);
}

/// Check for collision with non-air block ($A6:D4F9): whether the level data
/// block at (`x`, `y`) has a type. Was `ridley_offscreen`.

/// Ridley acceleration ($A6:D523): enemy `k` homes in on (`x`, `y`) with
/// inertia `inertia`. Was `Ridley_Func_104_0`.

/// Ridley death spot acceleration ($A6:D526): as `accelerate`, with `force`
/// extra when reversing. Was `Ridley_Func_104`.

/// Ridley inertia table ($A6:D61F).
const INERTIA: [u8; 16] = [
    0x10, 0xf, 0xe, 0xd, 0xc, 0xb, 0xa, 9, 8, 7, 6, 5, 4, 3, 2, 1,
];

/// Ridley X acceleration ($A6:D5A9). Was `Ridley_Func_105`.

/// Ceres Ridley acceleration ($A6:D62F): reversing doubles the step. Was
/// `Ridley_Func_106`.

/// Ceres Ridley X acceleration ($A6:D6A6). Was `Ridley_Func_107`.

/// Calculate swoop velocities ($A6:D800): bring the swoop speed 20h a frame
/// towards `target_speed`, turn the swoop angle by `angle_step` up to
/// `target_angle`, and split the speed along it. Was `Ridley_Func_111`.

/// Move Ridley ($A6:D86B): the current enemy by its velocities, clamped to
/// Ridley's bounds; `hit_boundary` records the side hit (1 left, 2 right, 4
/// top, 8 bottom). Was `Ridley_Func_112`.

/// Trigger Ceres Ridley earthquake if moving fast enough ($A6:D914): enemy `k`
/// hitting the left wall at 280h or more, outside Norfair. Was
/// `Ridley_Func_113`.

/// Turn Ridley around if not facing room middle ($A6:D955). Was
/// `Ridley_Func_114`.

/// Whether Ridley's X position mod 100h is at least 80h. The original reads
/// the word at $0F79, whose sign is that bit.

/// Update Ridley wings animation ($A6:D97D): ten frames, faster at speed.
/// Was `Ridley_Func_115`.

/// Calculate Ridley wings animation timer ($A6:D9A8): the timer delta from
/// the larger velocity, halved while rising. Was `Ridley_Func_116`.

/// Initialise Ridley ribs animation ($A6:D9FD). Was `Ridley_Func_117`.

/// Handle Ridley ribs animation ($A6:DA0C): when the timer runs out, follow
/// the table to its next frame and queue the frame's two tile rows. Was
/// `Ridley_Func_118`.

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
pub fn draw_tail<L: Lease + owned::Host>(game: &mut L) {
    if (session_slot!(game, Ridley[RIDLEY].base.properties) & K_ENEMY_PROPS_INVISIBLE as u16) != 0 {
        return;
    }
    let tip = ridley_data::tail_tip_spritemap(game, tail_tip_sector(game));
    add_ridley_spritemap(
        tip,
        bg2_staging::get(game, tail::x(6)),
        bg2_staging::get(game, tail::y(6)),
        game,
    );
    for segment in (0..6).rev() {
        let spritemap = TAIL_SEGMENT_SPRITEMAPS[segment / 2];
        add_ridley_spritemap(
            spritemap,
            bg2_staging::get(game, tail::x(segment)),
            bg2_staging::get(game, tail::y(segment)),
            game,
        );
    }
}

/// The tail tip's direction in 16 sectors: segment 5 and 6 angles' low bytes.

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
    let record = crate::cartridge::native_record(0xa6, spritemap, game);
    let map = crate::sprite_drawing::SpriteMap {
        image: record.image,
        offset: record.base,
    };
    let layer1 = game.camera().layer1;
    let camera = (layer1[0], layer1[1]);
    let clip = crate::scene::objects::RoomClip::Rows;
    game.draw_room_spritemap(&map, (x, y), camera, palette, clip);
}

/// Check if Ridley is on screen ($A6:DE7A): true when he is off it, with a
/// 20h px margin. Was `Ridley_Func_121`.

/// Check for projectile collision with rectangle ($A6:DEA6): the first of
/// Samus's beams (types below 3) in `rect`, which moves to its centre and
/// is marked deflected. Was `Ridley_Func_122`.

/// Check for Samus collision with rectangle ($A6:DF29). Was `Ridley_Func_124`.

/// Enemy touch - Ridley ($A6:DF59). Was `sub_A6DF59`.

/// Enemy shot - enemy $E13F/$E17F (Ridley) ($A6:DF8A): Norfair Ridley reacts
/// normally; Ceres Ridley flashes and counts the hit. Was `Ridley_Shot`.

/// Power bomb reaction - enemy $E13F/$E17F (Ridley) ($A6:DFB2). Was
/// `Ridley_Powerbomb`.

/// Start Norfair Ridley death sequence ($A6:DFB7), once, at no health. Was
/// `Ridley_Func_126`.

/// Ridley tail / Samus collision handling ($A6:DFD9): the tip hurts and knocks
/// Samus back. Was `Ridley_Func_127`.

/// Ridley tail / projectile collision handling ($A6:E088): a beam on the tip
/// or segment 5 deflects in a dust cloud, missiles with a clank. Was
/// `Ridley_Func_129`.

/// Instruction - roar ($A6:E4BE). Was `Ridley_Instr_5`.

/// Instruction - clear roaring flag ($A6:E4CA). Was `Ridley_Instr_6`.

/// Instruction - go to [[Y]] if not Norfair and Samus has low health
/// ($A6:E4D2): below 30. Was `Ridley_Instr_10`.

/// Unused. Instruction - go to [[Y]] if holding Samus, else go to [[Y] + 2]
/// ($A6:E4EE). Was `Ridley_Instr_4`.

/// Unused. Instruction - go to [[Y]] if not holding Samus ($A6:E4F8). Was
/// `Ridley_Instr_3`.

/// Instruction - Ridley feet distance index = [[Y]] ($A6:E501); Samus's
/// previous position catches up. Was `Ridley_Instr_2`.

/// Instruction - go to [[Y]] if not facing left ($A6:E517). Was
/// `Ridley_Instr_1`.

/// Instruction - move Ridley ([[Y]], [[Y] + 2]) pixels ($A6:E51F). Was
/// `Ridley_Instr_14`.

/// Instruction - flip Ridley left ($A6:E71C). Was `Ridley_Instr_9`.

/// Instruction - face Ridley forward ($A6:E727). Was `Ridley_Instr_7`.

/// Instruction - flip Ridley right ($A6:E72F). Was `Ridley_Instr_8`.

/// Instruction - calculate fireball X/Y velocities ($A6:E84D): 500h towards
/// Samus from the mouth, within the facing side's cone. Was
/// `Ridley_Instr_11`.

/// Instruction - spawn Ridley's fireball enemy projectile - with afterburn
/// ($A6:E904). Was `Ridley_Instr_12`.

/// Instruction - spawn Ridley's fireball enemy projectile - without afterburn
/// ($A6:E909). Was `Ridley_Instr_13`.

/// Spawn Ridley's fireball enemy projectile ($A6:E90C): `afterburn` 0 spawns
/// the afterburn. Was `Ridley_E90C`.

/// Instruction - start liftoff - Ceres Ridley ($A6:E969). Was `Ridley_Instr_15`.

/// Instruction - start liftoff - Norfair Ridley ($A6:E976). Was
/// `Ridley_Instr_16`.

// The named bank $A6 functions are the decompilation's; the two RTS
// terminators are its locret labels.
const _: () = {
    use crate::actors::ridley::{baby_metroid_function as b, function as f};
    const fn at(word: u16) -> u32 {
        0xa6_0000 | word as u32
    }
    assert!(at(f::LIFTOFF) == 10924787 && at(f::DECIDE_ACTION) == 10924833);
    assert!(at(f::LUNGE_MISSED) == 10925036 && at(f::LUNGE_MISSED_MOVE_TO_POSITION) == 10925048);
    assert!(at(f::SWOOP) == 10925121 && at(f::SWOOP_MOVE_TO_POSITION) == 10925141);
    assert!(at(f::SWOOP_AIMING_DOWN) == 10925203 && at(f::SWOOP_AIMING_HORIZONTAL) == 10925265);
    assert!(at(f::SWOOP_AIMING_UP) == 10925334 && at(f::SWOOP_AIMING_UP_FASTER) == 10925396);
    assert!(at(f::SWOOP_DECELERATE) == 10925460 && at(f::HOVER) == 10925508);
    assert!(at(f::HOVER_TO_WALL_BEHIND) == 10925541 && at(f::HOVER_TO_WALL_IN_FRONT) == 10925587);
    assert!(at(f::POGO_FLY_TO_POSITION) == 10925735 && at(f::POGO_WAIT_AT_POSITION) == 10925789);
    assert!(at(f::POGO_DESCENDING) == 10925838 && at(f::POGO_ASCENDING) == 10926009);
    assert!(at(f::LUNGE) == 10926775 && at(f::GRABBED_SAMUS) == 10926991);
    assert!(at(f::GRABBED_SAMUS_RISING) == 10927044);
    assert!(at(f::GRABBED_SAMUS_RISING_TO_MIDDLE) == 10927089);
    assert!(at(f::GRABBED_SAMUS_RELEASE) == 10927150 && at(f::DODGE_POWER_BOMB) == 10927438);
    assert!(at(f::FAKE_RETREAT_MOVE_TO_POSITION) == 10927514);
    assert!(at(f::FAKE_RETREAT_RISING) == 10927548 && at(f::DROP_BABY_METROID) == 10927602);
    assert!(at(f::RETRIEVE_BABY_METROID) == 10927619 && at(b::HELD) == 10927772);
    assert!(at(b::HELD_BY_FEET) == 10927795 && at(b::START_FALLING) == 10927818);
    assert!(at(b::FALLING) == 10927836 && at(f::DYING_GRABBED_SAMUS) == 10929464);
    assert!(at(f::DEATH_ROAR) == 10929470 && at(f::DEATH_MOVE_TO_DEATH_SPOT) == 10929489);
    assert!(at(f::LOWER_ACID_AND_RELEASE_SAMUS) == 10929544);
    assert!(at(f::START_FALLING_APART) == 10929576 && at(f::FALLING_APART) == 10929608);
    assert!(at(f::STILL_FALLING_APART) == 10929626);
    assert!(
        at(b::NOTHING) == 10927897
            && f::DEAD == crate::constants::bank_a6::ADDR_LOCRET_A6C600 as u16
    );
    assert!(at(f::CERES_LIFTOFF_ACCELERATING) == 10921647);
    assert!(at(f::CERES_REAL_RETREAT_RISING) == 10922353);
};

#[path = "ridley_owned.rs"]
pub mod owned;

pub use owned::handle_baby_metroid;

pub use owned::main;

pub use owned::time_frozen;

pub use owned::hurt;

pub use owned::liftoff;

pub use owned::decide_action;

pub use owned::fight_action_choices;

pub use owned::lunge_missed;

pub use owned::lunge_missed_move_to_position;

pub use owned::acceleration_factor;

pub use owned::swoop;

pub use owned::swoop_move_to_position;

pub use owned::swoop_aiming_down;

pub use owned::swoop_aiming_horizontal;

pub use owned::swoop_aiming_up;

pub use owned::swoop_aiming_up_faster;

pub use owned::swoop_decelerate;

pub use owned::hover;

pub use owned::hover_to_wall_behind;

pub use owned::hover_to_wall_in_front;

pub use owned::hover_towards;

pub use owned::set_up_pogo_from_hover;

pub use owned::pogo_fly_to_position;

pub use owned::pogo_wait_at_position;

pub use owned::pogo_descending;

pub use owned::pogo_ascending;

pub use owned::tail_hits_floor;

pub use owned::set_neutral_tail;

pub use owned::samus_grabbable;

pub use owned::samus_touches_feet;

pub use owned::grab_samus_from_pogo;

pub use owned::set_pogo_x_direction;

pub use owned::move_samus_with_feet;

pub use owned::set_grabbed_samus_offset;

pub use owned::lunge_missed_samus;

pub use owned::lunge;

pub use owned::grabbed_samus;

pub use owned::grabbed_samus_rising;

pub use owned::grabbed_samus_rising_to_middle;

pub use owned::grabbed_samus_release;

pub use owned::grab_samus;

pub use owned::release_samus;

pub use owned::set_tangibility;

pub use owned::samus_not_spinning;

pub use owned::samus_morphed;

pub use owned::samus_in_pogo_zone;

pub use owned::lunge_if_power_bomb;

pub use owned::dodge_power_bomb;

pub use owned::fake_retreat_move_to_position;

pub use owned::fake_retreat_rising;

pub use owned::drop_baby_metroid;

pub use owned::retrieve_baby_metroid;

pub use owned::baby_metroid_in_rect;

pub use owned::instruction;

pub use owned::run_baby_metroid_function;

pub use owned::baby_metroid_held;

pub use owned::baby_metroid_held_by_feet;

pub use owned::baby_metroid_start_falling;

pub use owned::baby_metroid_falling;

pub use owned::baby_metroid_goto_if_still;

pub use owned::baby_metroid_goto_if_moving;

pub use owned::baby_metroid_goto;

pub use owned::dying_grabbed_samus;

pub use owned::death_roar;

pub use owned::death_move_to_death_spot;

pub use owned::lower_acid_and_release_samus;

pub use owned::start_falling_apart;

pub use owned::falling_apart;

pub use owned::still_falling_apart;

pub use owned::move_to_death_spot;

pub use owned::spawn_small_explosion;

pub use owned::explosion_init;

pub use owned::explosion_main;

pub use owned::explosion_flicker;

pub use owned::spawn_explosions;

pub use owned::handle_tail;

pub use owned::set_up_pogo_tail;

pub use owned::point_tail_down;

pub use owned::pogo_descending_stab;

pub use owned::unused_pogo_descending_stab;

pub use owned::pogo_tail_pointed_down;

pub use owned::neutral_tail;

pub use owned::pogo_tail;

pub use owned::pogo_stab_setup_tail;

pub use owned::spinny_tail_whips;

pub use owned::update_tail_segment_angles;

pub use owned::set_tail_angle_extrema;

pub use owned::all_tail_segments_active;

pub use owned::handle_tail_whip;

pub use owned::configure_tail_whip;

pub use owned::handle_spinning_tail_whip;

pub use owned::configure_spinning_tail_whip;

pub use owned::handle_normal_pogo_tail;

pub use owned::handle_pogo_tail_stab;

pub use owned::set_tail_angle_high_bytes;

pub use owned::calculate_tail_segment_positions;

pub use owned::update_tail_segment_distances;

pub use owned::update_tail_segment_angle;

pub use owned::set_tail_whip_target;

pub use owned::missile_near_tail;

pub use owned::tail_whip_sound;

pub use owned::initialise_tail;

pub use owned::activate_tail_segments;

pub use owned::set_tail_segment_flags;

pub use owned::flip_tail;

pub use owned::set_instruction_list;

pub use owned::health_palette;

pub use owned::solid_block_at;

pub use owned::accelerate;

pub use owned::accelerate_with_force;

pub use owned::accelerate_x;

pub use owned::ceres_accelerate;

pub use owned::ceres_accelerate_x;

pub use owned::calculate_swoop_velocities;

pub use owned::move_ridley;

pub use owned::ceres_earthquake;

pub use owned::face_room_middle;

pub use owned::update_wings;

pub use owned::wings_animation_timer;

pub use owned::initialise_ribs;

pub use owned::handle_ribs;

use owned::tail_tip_sector;

pub use owned::baby_metroid_spritemap;

pub use owned::off_screen;

pub use owned::projectile_in_rect;

pub use owned::samus_in_rect;

pub use owned::touch;

pub use owned::shot;

pub use owned::power_bomb;

pub use owned::start_death_sequence;

pub use owned::tail_samus_collision;

pub use owned::tail_projectile_collision;

pub use owned::roar;

pub use owned::stop_roaring;

pub use owned::goto_if_samus_health_low;

pub use owned::goto_by_holding_samus;

pub use owned::goto_if_not_holding_samus;

pub use owned::set_feet_distance;

pub use owned::goto_if_not_facing_left;

pub use owned::move_by;

pub use owned::flip_left;

pub use owned::face_forward;

pub use owned::flip_right;

pub use owned::calculate_fireball_velocities;

pub use owned::fireball_with_afterburn;

pub use owned::fireball_without_afterburn;

pub use owned::spawn_fireball;

pub use owned::start_ceres_liftoff;

pub use owned::start_norfair_liftoff;
