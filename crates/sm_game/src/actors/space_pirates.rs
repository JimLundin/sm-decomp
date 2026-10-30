//! Space pirates in bank $B2: wall-jumping pirates, ninja pirates and walking
//! pirates. They share checked actor state and movement; the extended species
//! words stay authoritative in RAM.
use crate::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use crate::actors::api::{self, Axis, MovementResult};
use crate::actors::combat;
use crate::enemy_projectiles::host::Effects;
use crate::enemy_script::EnemyScript;
use crate::ram::{CompatibilityRam, word_arrays as arrays, words as fields};
use crate::session_access::SessionAccess;
use sm_data::cartridge::RomRecord;

/// Byte offset of an enemy definition's vulnerability table pointer.
pub const DEFINITION_VULNERABILITY: usize = 0x3c;
/// Byte offsets in a vulnerability table: power beam, and the plasma/ice/wave
/// row the beam combinations index from.
pub const VULNERABILITY_POWER: usize = 0;
pub const VULNERABILITY_PLASMA_ICE_WAVE: usize = 11;

/// One original `Runtime`: the actor, projectile and mechanism access a
/// pirate routine edits through, the session-aware RAM words, and `foreign`.
pub trait PirateLease:
    crate::actors::effects::ActorHost
    + crate::enemy_projectiles::host::Host
    + crate::room_mechanisms::host::Host
    + crate::room_mechanisms::host::Effects
{
    type Game: Pirates + ?Sized;
    /// The compatibility word at `at`, through the session projection.
    fn word(&self, at: usize) -> u16;
    /// The native-order word at `at`, through the session projection.
    fn native_word(&self, at: usize) -> u16;
    fn write_slice_native(&mut self, at: usize, value: u16);
    fn write_word_le(&mut self, value: u16, at: usize);
    fn foreign<T>(&mut self, action: impl FnOnce(&mut Self::Game) -> T) -> T;
    /// The actor content record at `bank:address`, as the original actor read.
    fn actor_legacy_record(&mut self, bank: u8, address: u16) -> RomRecord;
    /// The native record at `bank:address`, with the original diagnostic.
    fn legacy_native_record(&mut self, bank: u8, address: u16) -> RomRecord;
}

/// What the pirates need from the game outside a lease.
pub trait Pirates: CompatibilityRam + crate::records::Records {
    type Lease<'a>: PirateLease<Game = Self>
    where
        Self: 'a;
    /// Open one lease, as one original `Runtime::new`.
    fn lease(&mut self) -> Self::Lease<'_>;
    fn actor_sine(&mut self, phase: u16, amplitude: u16) -> u16;
    fn actor_cosine(&mut self, phase: u16, amplitude: u16) -> u16;
    fn create_sprite(&mut self, x: u16, y: u16, kind: u16, appearance: u16) -> u16;
    /// Fire a pirate laser from (`x`, `y`): the spawn point, its facing and
    /// the room-graphics laser projectile, in that order.
    fn fire_laser(&mut self, x: u16, y: u16, right: bool);
    /// Spawn projectile `parameter` from the current enemy within the
    /// rectangle at (`x`, `y`), after setting the rectangle.
    fn spawn_in_rect(&mut self, x: u16, y: u16, width: u16, height: u16, parameter: u16);
    /// The Lower Norfair pirates' special item drop for enemy `index`.
    fn scatter_pirate_drop(&mut self, index: u16);
    fn reflect_player_projectile(&mut self, index: u16);
    /// Copy the ninja pirates' palette into the sprite palette buffer.
    fn load_ninja_palette(&mut self);
}

/// Instruction lists in bank $B2, named after the bank log.
pub mod list {
    pub const FIRE_LASER_AND_WALL_JUMP_LEFT: u16 = 0xecc0;
    pub const LANDED_ON_LEFT_WALL: u16 = 0xece4;
    pub const MOVING_UP_LEFT_WALL: u16 = 0xecec;
    pub const MOVING_DOWN_LEFT_WALL: u16 = 0xed36;
    pub const FIRE_LASER_AND_WALL_JUMP_RIGHT: u16 = 0xed80;
    pub const LANDED_ON_RIGHT_WALL: u16 = 0xeda4;
    pub const MOVING_DOWN_RIGHT_WALL: u16 = 0xedac;
    pub const MOVING_UP_RIGHT_WALL: u16 = 0xedf6;
    pub const NINJA_CLAW_ATTACK_LEFT: u16 = 0xf15c;
    pub const NINJA_SPIN_JUMP_LEFT: u16 = 0xf1c4;
    pub const NINJA_ACTIVE_FACING_LEFT: u16 = 0xf22e;
    pub const NINJA_FLINCH_FACING_LEFT: u16 = 0xf270;
    pub const NINJA_DIVEKICK_LEFT: u16 = 0xf2a0;
    pub const NINJA_DIVEKICK_LEFT_WALK_TO_POST: u16 = 0xf2b2;
    pub const NINJA_INITIAL_FACING_LEFT: u16 = 0xf2da;
    pub const NINJA_LAND_FACING_LEFT: u16 = 0xf2f8;
    pub const NINJA_STANDING_KICK_FACING_LEFT: u16 = 0xf32e;
    pub const NINJA_CLAW_ATTACK_RIGHT: u16 = 0xf34a;
    pub const NINJA_SPIN_JUMP_RIGHT: u16 = 0xf3b2;
    pub const NINJA_ACTIVE_FACING_RIGHT: u16 = 0xf420;
    pub const NINJA_FLINCH_FACING_RIGHT: u16 = 0xf462;
    pub const NINJA_DIVEKICK_RIGHT: u16 = 0xf492;
    pub const NINJA_DIVEKICK_RIGHT_WALK_TO_POST: u16 = 0xf4a4;
    pub const NINJA_INITIAL_FACING_RIGHT: u16 = 0xf4cc;
    pub const NINJA_LAND_FACING_RIGHT: u16 = 0xf4ea;
    pub const NINJA_STANDING_KICK_FACING_RIGHT: u16 = 0xf51a;
    pub const WALKING_FLINCH_FACING_LEFT: u16 = 0xfb4c;
    pub const WALKING_FLINCH_FACING_RIGHT: u16 = 0xfb58;
    pub const WALKING_LEFT: u16 = 0xfb64;
    pub const WALKING_FIRE_LASERS_LEFT: u16 = 0xfb8c;
    pub const WALKING_LOOK_AROUND_FACING_LEFT: u16 = 0xfbc6;
    pub const WALKING_RIGHT: u16 = 0xfbe6;
    pub const WALKING_FIRE_LASERS_RIGHT: u16 = 0xfc0e;
    pub const WALKING_LOOK_AROUND_FACING_RIGHT: u16 = 0xfc48;
}

/// Pirate functions in bank $B2, run each frame by the main AIs; `NONE` is an RTS.
pub mod function {
    pub const NONE: u16 = 0x804b;
    pub const CLIMBING_LEFT_WALL: u16 = 0xf034;
    pub const CLIMBING_RIGHT_WALL: u16 = 0xf0c8;
    pub const SPIN_JUMP_LEFT_FALLING: u16 = 0xf84c;
    pub const SPIN_JUMP_RIGHT_FALLING: u16 = 0xf8c5;
    pub const DIVEKICK_LEFT: u16 = 0xf9c1;
    pub const DIVEKICK_LEFT_WALK_TO_POST: u16 = 0xfa15;
    pub const DIVEKICK_RIGHT: u16 = 0xfa95;
    pub const DIVEKICK_RIGHT_WALK_TO_POST: u16 = 0xfae9;
}

/// AI routines, as the enemy definitions address them.
pub mod routine {
    pub const TOUCH: u32 = 0xb2876c;
    pub const SHOT: u32 = 0xb28779;
    pub const WALKING_INITIALIZATION: u32 = 0xb2fd02;
    pub const WALKING_MAIN_AI: u32 = 0xb2fd32;
    pub const FROZEN_AI: u32 = 0xb2800f;
    pub const UPDATE_FROZEN: u32 = 0xb28041;
    pub const POWER_BOMB_REACTION: u32 = 0xb28767;
    pub const WALL_MAIN_AI: u32 = 0xb2f02d;
    pub const NINJA_MAIN_AI: u32 = 0xb2f6a2;
    pub const WALL_INITIALIZATION: u32 = 0xb2ef9f;
    pub const NINJA_INITIALIZATION: u32 = 0xb2f5de;
}

/// Instructions, as the instruction lists address them.
pub mod instruction {
    pub const FIRE_LASER_LEFT: u32 = 0xb2ef2a;
    pub const FIRE_LASER_RIGHT: u32 = 0xb2ef5d;
    pub const WALL_SET_FUNCTION: u32 = 0xb2ef83;
    pub const WALKING_SET_FUNCTION: u32 = 0xb2fcb8;
    pub const SET_PALETTE: u32 = 0xb2f536;
    pub const QUEUE_ATTACK_SOUND: u32 = 0xb2ef93;
    pub const QUEUE_SOUND: u32 = 0xb2f546;
    pub const SPAWN_CLAW: u32 = 0xb2f564;
    pub const GO_TO_ACTIVE: u32 = 0xb2f590;
    pub const FIRE_LASER_LEFT_WITH_Y_OFFSET: u32 = 0xb2fc68;
    pub const FIRE_LASER_RIGHT_WITH_Y_OFFSET: u32 = 0xb2fc90;
    pub const CHOOSE_MOVEMENT: u32 = 0xb2fcc8;
    pub const STOP: u32 = 0xb2f5d6;
    pub const SET_LEFT_DIVEKICK_SPEED: u32 = 0xb2f969;
    pub const SET_RIGHT_DIVEKICK_SPEED: u32 = 0xb2fa3d;
    pub const MOVE_DOWN_LEFT_WALL: u32 = 0xb2ee40;
    pub const MOVE_DOWN_RIGHT_WALL: u32 = 0xb2ee72;
    pub const RANDOM_DIRECTION_LEFT_WALL: u32 = 0xb2eea4;
    pub const RANDOM_DIRECTION_RIGHT_WALL: u32 = 0xb2eebc;
    pub const PREPARE_WALL_JUMP_RIGHT: u32 = 0xb2eed4;
    pub const PREPARE_WALL_JUMP_LEFT: u32 = 0xb2eefd;
}

/// Pirate function addresses, as the main AIs dispatch them.
pub mod state {
    pub const NINJA_INITIAL: u32 = 0xb2f6a9;
    pub const NINJA_ACTIVE: u32 = 0xb2f6e4;
    pub const NINJA_READY_TO_DIVEKICK: u32 = 0xb2f909;
    pub const NINJA_SPIN_JUMP_LEFT_RISING: u32 = 0xb2f817;
    pub const NINJA_SPIN_JUMP_LEFT_FALLING: u32 = 0xb2f84c;
    pub const NINJA_SPIN_JUMP_RIGHT_RISING: u32 = 0xb2f890;
    pub const NINJA_SPIN_JUMP_RIGHT_FALLING: u32 = 0xb2f8c5;
    pub const NINJA_DIVEKICK_LEFT_JUMP: u32 = 0xb2f985;
    pub const NINJA_DIVEKICK_LEFT: u32 = 0xb2f9c1;
    pub const NINJA_DIVEKICK_LEFT_WALK_TO_POST: u32 = 0xb2fa15;
    pub const NINJA_DIVEKICK_RIGHT_JUMP: u32 = 0xb2fa59;
    pub const NINJA_DIVEKICK_RIGHT: u32 = 0xb2fa95;
    pub const NINJA_DIVEKICK_RIGHT_WALK_TO_POST: u32 = 0xb2fae9;
    pub const WALL_CLIMBING_LEFT: u32 = 0xb2f034;
    pub const WALL_JUMPING_RIGHT: u32 = 0xb2f050;
    pub const WALL_CLIMBING_RIGHT: u32 = 0xb2f0c8;
    pub const WALL_JUMPING_LEFT: u32 = 0xb2f0e4;
    pub const WALKING_LEFT: u32 = 0xb2fd44;
    pub const WALKING_RIGHT: u32 = 0xb2fdce;
    pub const FROZEN: u32 = 0xb2804b;
    pub const WALL_LANDED_LEFT: u32 = 0xb2f04f;
    pub const WALL_LANDED_RIGHT: u32 = 0xb2f0e3;
    pub const WALKING_IDLE: u32 = 0xb2fe4a;
}

/// Per-enemy words outside the enemy slots, at the slot offset from these bases.
pub mod extra {
    /// Ninja: speed, its high byte the per-frame movement. Walking: X backup.
    pub const SPEED: usize = 0x7800;
    /// Ninja: the divekick's midpoint X (never read).
    pub const DIVEKICK_MIDPOINT_X: usize = 0x7806;
    /// Ninja: spawn Y, restored after a divekick.
    pub const SPAWN_Y: usize = 0x7810;
    /// Wall: the arc's right and left target angles and its per-frame angle step.
    pub const ARC_RIGHT_TARGET: usize = 0x8000;
    pub const ARC_LEFT_TARGET: usize = 0x8002;
    pub const ARC_ANGLE_DELTA: usize = 0x8004;
}

// Enemy variable slots ($0FA8 + 2n). `pre_instruction` ($0FB2) is the right
// post X for ninja and walking pirates and the arc angle for wall pirates.
/// $0FA8: the pirate function the main AI runs each frame.
const FUNCTION: usize = 0;
/// Wall: the far jump target X (never read).
const WALL_UNUSED_TARGET_X: usize = 1;
/// Wall: 0 while climbing down, nonzero while climbing up.
const WALL_DIRECTION: usize = 2;
/// Wall: the wall-jump arc's centre.
const WALL_ARC_CENTRE_X: usize = 3;
const WALL_ARC_CENTRE_Y: usize = 4;
/// Ninja: the initial speed, cleared when a gold ninja dies (never read).
const NINJA_INITIAL_SPEED: usize = 1;
/// Ninja: the active instruction list (never read).
const NINJA_ACTIVE_LIST: usize = 2;
/// Ninja and walking: the posts' midpoint and the left post.
const POSTS_MIDPOINT_X: usize = 3;
const LEFT_POST_X: usize = 4;

/// Parameter 1 bit 15: wall pirates jump slower; walking pirates flinch from
/// nearby projectiles.
const PARAMETER_ALTERNATE: u16 = 0x8000;
/// The gold ninja pirate's enemy definition, the one that reflects shots.
const GOLD_NINJA: u16 = 0xf593;
/// The vulnerability table used when a definition has none.
const DEFAULT_VULNERABILITY: u16 = 0xec1c;

fn current<R: PirateLease>(runtime: &R) -> Key {
    Key::from_legacy(runtime.word(0xe54))
}

fn offset_x(x: u16, amount: u16, right: bool) -> u16 {
    x.wrapping_add(if right { amount } else { amount.wrapping_neg() })
}

fn snap_to_wall(x: u16) -> u16 {
    if x & 15 < 11 {
        x & 0xfff8
    } else {
        (x & 0xfff0).wrapping_add(16)
    }
}

fn initialize_wall<G: Pirates + ?Sized>(game: &mut G) {
    let mut runtime = game.lease();
    let index = runtime.word(0xe54);
    let key = Key::from_legacy(index);
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction = if actor.parameters[0] & 1 == 0 {
            list::MOVING_DOWN_LEFT_WALL
        } else {
            list::MOVING_DOWN_RIGHT_WALL
        };
    });
    for (at, value) in [
        (extra::ARC_RIGHT_TARGET, 190),
        (extra::ARC_LEFT_TARGET, 66),
        (extra::ARC_ANGLE_DELTA, 2),
    ] {
        runtime.write_slice_native(at + usize::from(index), value);
    }
    if runtime.resolve_actor(key, Native).parameters[0] & PARAMETER_ALTERNATE == 0 {
        for (at, delta) in [
            (extra::ARC_RIGHT_TARGET, 2u16),
            (extra::ARC_LEFT_TARGET, u16::MAX - 1),
            (extra::ARC_ANGLE_DELTA, 2),
        ] {
            let at = at + usize::from(index);
            runtime.write_slice_native(at, runtime.native_word(at).wrapping_add(delta));
        }
    }
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[FUNCTION] = if actor.parameters[0] & 1 == 0 {
            function::CLIMBING_LEFT_WALL
        } else {
            function::CLIMBING_RIGHT_WALL
        };
        actor.x.whole = snap_to_wall(actor.x.whole);
    });
}

fn climb_wall<G: Pirates + ?Sized>(right: bool, game: &mut G) {
    let mut runtime = game.lease();
    let key = current(&runtime);
    if api::player_within(&mut runtime, key, Axis::Y, 32) {
        let key = current(&runtime);
        runtime.edit_actor(key, Native, |actor| {
            actor.instruction = if right {
                list::FIRE_LASER_AND_WALL_JUMP_RIGHT
            } else {
                list::FIRE_LASER_AND_WALL_JUMP_LEFT
            };
            actor.instruction_delay = 1;
        });
    }
}

fn wall_jump<G: Pirates + ?Sized>(right: bool, game: &mut G) {
    let mut runtime = game.lease();
    let index = runtime.word(0xe54);
    let key = Key::from_legacy(index);
    let actor = runtime.resolve_actor(key, Native);
    let (origin, phase, radius) = (
        actor.variables[WALL_ARC_CENTRE_X],
        actor.pre_instruction,
        actor.parameters[1] >> 1,
    );
    let dx = runtime.foreign(|game| game.actor_sine(phase, radius));
    runtime.edit_actor(key, Native, |actor| actor.x.whole = origin.wrapping_add(dx));
    let actor = runtime.resolve_actor(key, Native);
    let (origin, phase, radius) = (
        actor.variables[WALL_ARC_CENTRE_Y],
        actor.pre_instruction,
        actor.parameters[1] >> 2,
    );
    let dy = runtime.foreign(|game| game.actor_cosine(phase, radius));
    runtime.edit_actor(key, Native, |actor| actor.y.whole = origin.wrapping_sub(dy));
    let phase = runtime
        .resolve_actor(key, Native)
        .pre_instruction
        .to_ne_bytes()[0];
    let step = runtime
        .native_word(extra::ARC_ANGLE_DELTA + usize::from(index))
        .to_ne_bytes()[0];
    let phase = u16::from(if right {
        phase.wrapping_sub(step)
    } else {
        phase.wrapping_add(step)
    });
    runtime.edit_actor(key, Native, |actor| actor.pre_instruction = phase);
    let end = runtime.native_word(
        (if right {
            extra::ARC_RIGHT_TARGET
        } else {
            extra::ARC_LEFT_TARGET
        }) + usize::from(index),
    );
    if phase == end {
        runtime.edit_actor(key, Native, |actor| {
            actor.instruction = if right {
                list::LANDED_ON_RIGHT_WALL
            } else {
                list::LANDED_ON_LEFT_WALL
            };
            actor.instruction_delay = 1;
            actor.x.whole = snap_to_wall(actor.x.whole);
        });
    }
}

fn choose_wall_direction<G: Pirates + ?Sized>(
    right: bool,
    random: bool,
    script: EnemyScript,
    game: &mut G,
) -> EnemyScript {
    let mut runtime = game.lease();
    let direction = if random {
        let value = runtime.next_random() & 1;
        let key = current(&runtime);
        runtime.edit_actor(key, Native, |actor| actor.variables[WALL_DIRECTION] = value);
        value
    } else {
        let key = current(&runtime);
        let distance = (script.word(0) as i16 as i32) << 16;
        if api::move_vertical(&mut runtime, key, distance) != MovementResult::Blocked {
            return script.offset(1);
        }
        let key = current(&runtime);
        runtime.edit_actor(key, Native, |actor| {
            actor.variables[WALL_DIRECTION] ^= 1;
            actor.variables[WALL_DIRECTION]
        })
    };
    EnemyScript::jump(match (right, direction != 0) {
        (true, false) => list::MOVING_DOWN_LEFT_WALL,
        (true, true) => list::MOVING_UP_LEFT_WALL,
        (false, false) => list::MOVING_DOWN_RIGHT_WALL,
        (false, true) => list::MOVING_UP_RIGHT_WALL,
    })
}

fn prepare_wall_jump<G: Pirates + ?Sized>(
    right: bool,
    script: EnemyScript,
    game: &mut G,
) -> EnemyScript {
    let mut runtime = game.lease();
    let key = current(&runtime);
    runtime.edit_actor(key, Native, |actor| {
        let x = actor.x.whole;
        let distance = actor.parameters[1];
        actor.variables[WALL_UNUSED_TARGET_X] = offset_x(x, distance, right);
        actor.variables[WALL_ARC_CENTRE_X] = offset_x(x, distance >> 1, right);
        actor.variables[WALL_ARC_CENTRE_Y] = actor.y.whole;
        actor.pre_instruction = if right { 64 } else { 192 };
    });
    script
}

fn within(delta: u16, distance: i16) -> bool {
    (delta as i16).wrapping_abs().wrapping_sub(distance) < 0
}

fn ninja_close_attack<G: Pirates + ?Sized>(check_center: bool, game: &mut G) -> bool {
    let mut runtime = game.lease();
    let key = current(&runtime);
    let actor = runtime.resolve_actor(key, Native);
    let player = &runtime.session().player;
    let instruction = if within(player.x.wrapping_sub(actor.x.whole), 40)
        && within(player.y.wrapping_sub(actor.y.whole), 40)
    {
        if (actor.x.whole.wrapping_sub(player.x) as i16) < 0 {
            list::NINJA_STANDING_KICK_FACING_RIGHT
        } else {
            list::NINJA_STANDING_KICK_FACING_LEFT
        }
    } else if check_center && within(actor.variables[POSTS_MIDPOINT_X].wrapping_sub(player.x), 32) {
        if actor.x.whole == actor.variables[LEFT_POST_X] {
            list::NINJA_SPIN_JUMP_RIGHT
        } else {
            list::NINJA_SPIN_JUMP_LEFT
        }
    } else {
        return false;
    };
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction = instruction;
        actor.instruction_delay = 1;
    });
    true
}

fn flinch_from_projectile<G: Pirates + ?Sized>(walking: bool, game: &mut G) -> bool {
    let mut runtime = game.lease();
    let Some(shot) = runtime.session().projectiles.shots[..5]
        .iter()
        .rev()
        .find(|shot| shot.kind != 0)
    else {
        return false;
    };
    let key = current(&runtime);
    let actor = runtime.resolve_actor(key, Native);
    if !within(shot.x.wrapping_sub(actor.x.whole), 32)
        || !within(shot.y.wrapping_sub(actor.y.whole), 32)
    {
        return false;
    }
    let left = (actor.x.whole.wrapping_sub(runtime.session().player.x) as i16) < 0;
    let [right_instruction, left_instruction] = if walking {
        [
            list::WALKING_FLINCH_FACING_LEFT,
            list::WALKING_FLINCH_FACING_RIGHT,
        ]
    } else {
        [
            list::NINJA_FLINCH_FACING_LEFT,
            list::NINJA_FLINCH_FACING_RIGHT,
        ]
    };
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction = if left {
            left_instruction
        } else {
            right_instruction
        };
        actor.instruction_delay = 1;
    });
    true
}

fn fire_laser<G: Pirates + ?Sized>(right: bool, height: u16, game: &mut G) {
    let (x, y) = {
        let runtime = game.lease();
        let actor = runtime.resolve_actor(current(&runtime), Native);
        (
            offset_x(actor.x.whole, 24, right),
            actor.y.whole.wrapping_sub(height),
        )
    };
    game.fire_laser(x, y, right);
}

fn landing_dust<R: PirateLease>(runtime: &mut R, key: Key) {
    for dx in [0xfff8, 8] {
        let actor = runtime.resolve_actor(key, Native);
        let (x, y) = (
            actor.x.whole.wrapping_add(dx),
            actor.y.whole.wrapping_add(28),
        );
        runtime.foreign(|game| game.create_sprite(x, y, 10, 0));
    }
}

fn spin_jump<G: Pirates + ?Sized>(index: u16, right: bool, descending: bool, game: &mut G) {
    let mut runtime = game.lease();
    let key = Key::from_legacy(index);
    let velocity_at = extra::SPEED + usize::from(index);
    let speed = u16::from(runtime.native_word(velocity_at).to_ne_bytes()[1]);
    runtime.edit_actor(key, Native, |actor| {
        actor.x.whole = offset_x(actor.x.whole, speed, right);
    });
    for _ in 0..2 {
        runtime.edit_actor(key, Native, |actor| {
            actor.y.whole = actor
                .y
                .whole
                .wrapping_add(if descending { 1 } else { u16::MAX });
        });
    }
    let velocity = runtime
        .native_word(velocity_at)
        .wrapping_add(if descending { 0xffe0 } else { 32 });
    runtime.write_slice_native(velocity_at, velocity);
    if descending {
        if velocity == 0 {
            runtime.edit_actor(key, Native, |actor| {
                actor.variables[FUNCTION] = function::NONE;
                actor.instruction = if right {
                    list::NINJA_LAND_FACING_RIGHT
                } else {
                    list::NINJA_LAND_FACING_LEFT
                };
                actor.instruction_delay = 1;
                actor.x.whole = if right {
                    actor.pre_instruction
                } else {
                    actor.variables[LEFT_POST_X]
                };
            });
            landing_dust(&mut runtime, key);
        }
    } else {
        runtime.edit_actor(key, Native, |actor| {
            if ((actor
                .x
                .whole
                .wrapping_sub(actor.variables[POSTS_MIDPOINT_X]) as i16)
                >= 0)
                == right
            {
                actor.variables[FUNCTION] = if right {
                    function::SPIN_JUMP_RIGHT_FALLING
                } else {
                    function::SPIN_JUMP_LEFT_FALLING
                };
            }
        });
    }
}

enum DivekickPhase {
    Jump,
    Divekick,
    WalkToPost,
}

fn divekick<G: Pirates + ?Sized>(index: u16, right: bool, phase: DivekickPhase, game: &mut G) {
    let mut runtime = game.lease();
    let key = Key::from_legacy(index);
    let velocity_at = extra::SPEED + usize::from(index);
    match phase {
        DivekickPhase::Jump => {
            let speed = u16::from(runtime.native_word(velocity_at).to_ne_bytes()[1]);
            runtime.edit_actor(key, Native, |actor| {
                actor.y.whole = actor.y.whole.wrapping_sub(speed)
            });
            let velocity = (runtime.native_word(velocity_at) as i16).wrapping_sub(64);
            runtime.write_slice_native(velocity_at, velocity as u16);
            if velocity < 0 {
                runtime.edit_actor(key, Native, |actor| {
                    actor.variables[FUNCTION] = if right {
                        function::DIVEKICK_RIGHT
                    } else {
                        function::DIVEKICK_LEFT
                    };
                    actor.instruction = if right {
                        list::NINJA_DIVEKICK_RIGHT
                    } else {
                        list::NINJA_DIVEKICK_LEFT
                    };
                    actor.instruction_delay = 1;
                });
                runtime.write_slice_native(velocity_at, 1536);
            }
        }
        DivekickPhase::Divekick => {
            runtime.edit_actor(key, Native, |actor| {
                actor.x.whole = offset_x(actor.x.whole, 5, right);
            });
            let [fraction, whole] = runtime.native_word(velocity_at).to_ne_bytes();
            let amount = (i32::from(whole) << 16) | i32::from(fraction);
            let finished = api::move_vertical(&mut runtime, key, amount) == MovementResult::Blocked
                || {
                    let velocity = (runtime.native_word(velocity_at) as i16).wrapping_sub(64);
                    runtime.write_slice_native(velocity_at, velocity as u16);
                    velocity < 0 || velocity as u16 & 0xff00 == 0
                };
            if finished {
                runtime.edit_actor(key, Native, |actor| {
                    actor.variables[FUNCTION] = if right {
                        function::DIVEKICK_RIGHT_WALK_TO_POST
                    } else {
                        function::DIVEKICK_LEFT_WALK_TO_POST
                    };
                    actor.instruction = if right {
                        list::NINJA_DIVEKICK_RIGHT_WALK_TO_POST
                    } else {
                        list::NINJA_DIVEKICK_LEFT_WALK_TO_POST
                    };
                    actor.instruction_delay = 1;
                });
                let y = runtime.native_word(extra::SPAWN_Y + usize::from(index));
                runtime.edit_actor(key, Native, |actor| actor.y.whole = y);
                landing_dust(&mut runtime, key);
            }
        }
        DivekickPhase::WalkToPost => {
            runtime.edit_actor(key, Native, |actor| {
                actor.x.whole = offset_x(actor.x.whole, 2, right);
                let target = if right {
                    actor.pre_instruction
                } else {
                    actor.variables[LEFT_POST_X]
                };
                if ((actor.x.whole.wrapping_sub(target) as i16) >= 0) == right {
                    actor.x.whole = target;
                    actor.instruction = if right {
                        list::NINJA_LAND_FACING_RIGHT
                    } else {
                        list::NINJA_LAND_FACING_LEFT
                    };
                    actor.instruction_delay = 1;
                    actor.variables[FUNCTION] = function::NONE;
                }
            });
        }
    }
}

fn stop_ninja<G: Pirates + ?Sized>(index: u16, game: &mut G) {
    game.lease()
        .write_slice_native(extra::SPEED + usize::from(index), 0);
}

fn set_divekick_speed<G: Pirates + ?Sized>(index: u16, right: bool, game: &mut G) {
    let mut runtime = game.lease();
    runtime.write_slice_native(extra::SPEED + usize::from(index), 1536);
    let actor = runtime.resolve_actor(Key::from_legacy(index), Native);
    let (start, end) = if right {
        (
            actor.variables[LEFT_POST_X],
            actor.variables[POSTS_MIDPOINT_X],
        )
    } else {
        (actor.variables[POSTS_MIDPOINT_X], actor.pre_instruction)
    };
    let midpoint = start.wrapping_add(end.wrapping_sub(start) >> 1);
    runtime.write_slice_native(extra::DIVEKICK_MIDPOINT_X + usize::from(index), midpoint);
}

fn initialize_ninja<G: Pirates + ?Sized>(game: &mut G) {
    let mut runtime = game.lease();
    let index = runtime.word(0xe54);
    let y = runtime.edit_actor(Key::from_legacy(index), Native, |actor| {
        let right = actor.parameters[0] & 1 != 0;
        actor.instruction = if right {
            list::NINJA_INITIAL_FACING_RIGHT
        } else {
            list::NINJA_INITIAL_FACING_LEFT
        };
        actor.variables[NINJA_ACTIVE_LIST] = actor.instruction;
        let span = actor.parameters[1];
        let left = if right {
            actor.x.whole
        } else {
            actor.x.whole.wrapping_sub(span)
        };
        actor.variables[POSTS_MIDPOINT_X] = left.wrapping_add(span >> 1);
        let threshold = u16::from((span >> 1) as u8) << 8;
        let (mut velocity, mut distance) = (0u16, 0u16);
        loop {
            velocity = velocity.wrapping_add(32);
            distance = distance.wrapping_add(velocity);
            if (distance.wrapping_sub(threshold) as i16) >= 0 {
                break;
            }
        }
        actor.variables[NINJA_INITIAL_SPEED] = velocity;
        actor.pre_instruction = actor.variables[POSTS_MIDPOINT_X].wrapping_add(distance >> 8);
        actor.variables[LEFT_POST_X] =
            actor.variables[POSTS_MIDPOINT_X].wrapping_sub(distance >> 8);
        actor.x.whole = if right {
            actor.variables[LEFT_POST_X]
        } else {
            actor.pre_instruction
        };
        actor.variables[FUNCTION] = function::NONE;
        actor.y.whole
    });
    runtime.write_slice_native(extra::SPAWN_Y + usize::from(index), y);
    runtime.foreign(|game| game.load_ninja_palette());
}

fn ninja_initial<G: Pirates + ?Sized>(game: &mut G) {
    let mut runtime = game.lease();
    let key = current(&runtime);
    let player_x = runtime.session().player.x;
    let x = runtime.resolve_actor(key, Native).x.whole;
    if within(x.wrapping_sub(player_x), 128) {
        runtime.edit_actor(key, Native, |actor| {
            actor.instruction = if (x.wrapping_sub(player_x) as i16) < 0 {
                list::NINJA_ACTIVE_FACING_RIGHT
            } else {
                list::NINJA_ACTIVE_FACING_LEFT
            };
            actor.variables[NINJA_ACTIVE_LIST] = actor.instruction;
            actor.instruction_delay = 1;
        });
    } else {
        runtime.foreign(|game| flinch_from_projectile(false, game));
    }
}

fn ninja_active<G: Pirates + ?Sized>(index: u16, game: &mut G) {
    if flinch_from_projectile(false, game) || ninja_close_attack(true, game) {
        return;
    }
    let mut runtime = game.lease();
    let key = Key::from_legacy(index);
    let actor = runtime.resolve_actor(key, Native);
    let left = actor.x.whole == actor.variables[LEFT_POST_X];
    if actor.frame & 0x3f != 0
        || ((actor.x.whole.wrapping_sub(runtime.session().player.x) as i16) >= 0) != left
    {
        return;
    }
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction = if left {
            list::NINJA_CLAW_ATTACK_LEFT
        } else {
            list::NINJA_CLAW_ATTACK_RIGHT
        };
        actor.instruction_delay = 1;
    });
}

fn ready_to_divekick<G: Pirates + ?Sized>(game: &mut G) {
    if flinch_from_projectile(false, game) || ninja_close_attack(false, game) {
        return;
    }
    let mut runtime = game.lease();
    let key = current(&runtime);
    if !within(
        runtime.resolve_actor(key, Native).variables[POSTS_MIDPOINT_X]
            .wrapping_sub(runtime.session().player.x),
        32,
    ) {
        return;
    }
    let choice = loop {
        let choice = runtime.next_random() & 3;
        if choice != 0 {
            break choice;
        }
    };
    let actor = runtime.resolve_actor(key, Native);
    let choice = choice
        + if actor.x.whole == actor.variables[LEFT_POST_X] {
            4
        } else {
            0
        };
    let instruction = runtime
        .foreign(|game| crate::gameplay_tables::pirate_instruction_list(game, choice as isize));
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction = instruction;
        actor.instruction_delay = 1;
    });
}

fn damage<R: PirateLease>(runtime: &mut R) {
    let key = current(runtime);
    let x = runtime.resolve_actor(key, Native).x.whole;
    runtime.write_word_le(x, fields::SPECIAL_DEATH_ITEM_DROP_X_ORIGIN_POS);
    let y = runtime.resolve_actor(key, Native).y.whole;
    runtime.write_word_le(y, fields::SPECIAL_DEATH_ITEM_DROP_Y_ORIGIN_POS);
    combat::shot_reaction(runtime, api::ShotHandling::ReactionOnly);
    if runtime.resolve_actor(key, Native).health == 0 {
        let index = runtime.word(fields::CUR_ENEMY_INDEX);
        let drop_items = runtime.resolve_actor(key, Native).definition == GOLD_NINJA;
        runtime.edit_actor(key, Native, |actor| {
            actor.variables[NINJA_INITIAL_SPEED] = 0
        });
        combat::die(runtime, api::DeathKind::Normal { animation: 4 });
        if drop_items {
            runtime.foreign(|game| game.scatter_pirate_drop(index));
        }
    }
}

pub fn touch<G: Pirates + ?Sized>(game: &mut G) {
    let mut runtime = game.lease();
    if runtime.resolve_actor(current(&runtime), Native).frozen == 0 {
        combat::contact_reaction(&mut runtime, api::DeathHandling::Finalize);
    }
}

pub fn shot<G: Pirates + ?Sized>(game: &mut G) {
    let mut runtime = game.lease();
    if runtime.resolve_actor(current(&runtime), Native).definition == GOLD_NINJA {
        damage(&mut runtime);
    } else {
        combat::shot_reaction(&mut runtime, api::ShotHandling::Finalize);
    }
}

pub fn check_shot<G: Pirates + ?Sized>(game: &mut G) {
    let mut runtime = game.lease();
    if runtime.resolve_actor(current(&runtime), Native).definition == GOLD_NINJA {
        let index = usize::from(runtime.word(fields::COLLISION_DETECTION_INDEX));
        let kind = runtime.word(arrays::PROJECTILE_TYPE + 2 * index);
        let class = kind & 0xf00;
        if class >= 0x300 {
            return;
        }
        let definition = runtime.resolve_actor(current(&runtime), Native).definition;
        let vulnerability = runtime
            .actor_legacy_record(0xa0, definition)
            .word(DEFINITION_VULNERABILITY);
        let vulnerability = if vulnerability == 0 {
            DEFAULT_VULNERABILITY
        } else {
            vulnerability
        };
        let (index, field) = if class == 0 {
            (kind & 15, VULNERABILITY_POWER)
        } else {
            (class >> 8, VULNERABILITY_PLASMA_ICE_WAVE)
        };
        let coefficient = runtime
            .actor_legacy_record(0xb4, vulnerability.wrapping_add(index))
            .byte(field)
            & 15;
        if matches!(coefficient, 0 | 15) {
            reflect_shot(&mut runtime);
            return;
        }
    }
    damage(&mut runtime);
}

pub fn reflect<G: Pirates + ?Sized>(game: &mut G) {
    reflect_shot(&mut game.lease());
}

fn reflect_shot<R: PirateLease>(runtime: &mut R) {
    if runtime.resolve_actor(current(runtime), Native).definition != GOLD_NINJA {
        damage(runtime);
        return;
    }
    let reflection_index = runtime
        .word(fields::COLLISION_DETECTION_INDEX)
        .wrapping_mul(2);
    // Array reads keep the unwrapped element index; the reflection argument wraps.
    let index = 2 * usize::from(runtime.word(fields::COLLISION_DETECTION_INDEX));
    let class = runtime.word(arrays::PROJECTILE_TYPE + index) & 0xf00;
    if class == 0x200 {
        if runtime.word(arrays::PROJECTILE_VARIABLES + index) == 0 {
            return;
        }
    } else if class >= 0x300 {
        return;
    }
    let key = current(runtime);
    runtime.edit_actor(key, Native, |actor| actor.invincibility = 10);
    let direction_at = arrays::PROJECTILE_DIR + index;
    let direction = match runtime.word(direction_at) & 15 {
        7 => 1,
        2 => 8,
        _ => 5,
    };
    runtime.write_word_le(direction, direction_at);
    runtime.foreign(|game| game.reflect_player_projectile(reflection_index));
    runtime.queue_sound(2, 6, 0x66);
}

fn initialize_walking<G: Pirates + ?Sized>(game: &mut G) {
    let mut runtime = game.lease();
    let key = current(&runtime);
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction = if actor.parameters[0] & 1 != 0 {
            list::WALKING_RIGHT
        } else {
            list::WALKING_LEFT
        };
        actor.variables[FUNCTION] = function::NONE;
        actor.pre_instruction = actor.parameters[1].wrapping_add(actor.x.whole);
        actor.variables[LEFT_POST_X] = actor.x.whole.wrapping_sub(actor.parameters[1]);
    });
}

fn walking_instruction<R: PirateLease>(runtime: &R, key: Key, attack: bool) -> u16 {
    let x = runtime.resolve_actor(key, Native).x.whole;
    let right = (runtime.session().player.x.wrapping_sub(x) as i16) >= 0;
    match (attack, right) {
        (true, true) => list::WALKING_FIRE_LASERS_RIGHT,
        (true, false) => list::WALKING_FIRE_LASERS_LEFT,
        (false, true) => list::WALKING_LEFT,
        (false, false) => list::WALKING_RIGHT,
    }
}

fn walk<G: Pirates + ?Sized>(right: bool, game: &mut G) {
    let mut runtime = game.lease();
    let index = runtime.word(fields::CUR_ENEMY_INDEX);
    let key = Key::from_legacy(index);
    if api::player_within(&mut runtime, key, Axis::Y, 16) {
        let instruction = walking_instruction(&runtime, key, true);
        runtime.edit_actor(key, Native, |actor| {
            actor.instruction = instruction;
            actor.instruction_delay = 1;
        });
        return;
    }
    let mover = current(&runtime);
    if api::move_vertical(&mut runtime, mover, 1 << 16) != MovementResult::Blocked {
        return;
    }
    let x = runtime.resolve_actor(key, Native).x.whole;
    runtime.write_slice_native(extra::SPEED + usize::from(index), x);
    runtime.edit_actor(key, Native, |actor| {
        actor.x.whole = offset_x(x, if right { 16 } else { 17 }, right)
    });
    let mover = current(&runtime);
    let floor = api::move_vertical(&mut runtime, mover, 1 << 16) == MovementResult::Blocked;
    let x = runtime.native_word(extra::SPEED + usize::from(index));
    runtime.edit_actor(key, Native, |actor| actor.x.whole = x);
    let turn = !floor || {
        if !right {
            let mover = current(&runtime);
            api::probe_horizontal(&mut runtime, mover, -9 << 16);
        }
        let mover = current(&runtime);
        api::move_horizontal(
            &mut runtime,
            mover,
            if right { 14336 } else { -14337 },
            api::SlopePolicy::Ignore,
        ) == MovementResult::Blocked
            || {
                let actor = runtime.resolve_actor(key, Native);
                let bound = if right {
                    actor.pre_instruction
                } else {
                    actor.variables[LEFT_POST_X]
                };
                ((actor.x.whole.wrapping_sub(bound) as i16) >= 0) == right
            }
    };
    if turn {
        runtime.edit_actor(key, Native, |actor| {
            actor.instruction = if right {
                list::WALKING_LOOK_AROUND_FACING_RIGHT
            } else {
                list::WALKING_LOOK_AROUND_FACING_LEFT
            };
            actor.instruction_delay = 1;
        });
    }
}

fn advance<G: Pirates + ?Sized>(walking: bool, game: &mut G) {
    let mut runtime = game.lease();
    let key = current(&runtime);
    let state = u32::from(runtime.resolve_actor(key, Native).variables[FUNCTION]) | 0xb20000;
    let index = runtime.word(fields::CUR_ENEMY_INDEX);
    runtime.foreign(|game| dispatch_state(state, index, game));
    if walking && runtime.resolve_actor(key, Native).parameters[0] & PARAMETER_ALTERNATE != 0 {
        runtime.foreign(|game| flinch_from_projectile(true, game));
    }
}

pub fn dispatch_ai<G: Pirates + ?Sized>(ea: u32, game: &mut G) {
    match ea {
        routine::TOUCH => touch(game),
        routine::SHOT => shot(game),
        routine::WALKING_INITIALIZATION => initialize_walking(game),
        routine::WALKING_MAIN_AI => advance(true, game),
        routine::FROZEN_AI => combat::select_frozen_ai(&mut game.lease()),
        routine::UPDATE_FROZEN => crate::actors::scheduling::update_frozen_enemy(&mut game.lease()),
        routine::POWER_BOMB_REACTION => {
            combat::power_bomb_reaction(&mut game.lease(), api::DeathHandling::Finalize)
        }
        routine::WALL_MAIN_AI | routine::NINJA_MAIN_AI => advance(false, game),
        routine::WALL_INITIALIZATION => initialize_wall(game),
        routine::NINJA_INITIALIZATION => initialize_ninja(game),
        _ => game.lease().unreachable(),
    }
}

pub fn dispatch_instruction<G: Pirates + ?Sized>(
    ea: u32,
    k: u16,
    script: EnemyScript,
    game: &mut G,
) -> EnemyScript {
    match ea {
        instruction::FIRE_LASER_LEFT => {
            let mut runtime = game.lease();
            let index = runtime.word(fields::CUR_ENEMY_INDEX);
            let definition = runtime
                .resolve_actor(Key::from_legacy(index), Native)
                .definition;
            let value = runtime.legacy_native_record(0xa0, definition).word(6);
            runtime.write_word_le(value, fields::EPROJ_INIT_PARAM_1);
            drop(runtime);
            fire_laser(false, 16, game);
            script
        }
        instruction::FIRE_LASER_RIGHT => {
            fire_laser(true, 16, game);
            script
        }
        instruction::WALL_SET_FUNCTION
        | instruction::WALKING_SET_FUNCTION
        | instruction::SET_PALETTE => {
            let value = script.word(0);
            let mut runtime = game.lease();
            let key = current(&runtime);
            runtime.edit_actor(key, Native, |actor| {
                if ea == instruction::SET_PALETTE {
                    actor.palette = value;
                } else {
                    actor.variables[FUNCTION] = value;
                }
            });
            script.offset(1)
        }
        instruction::QUEUE_ATTACK_SOUND => {
            game.lease().queue_sound(2, 6, 0x66);
            script
        }
        instruction::QUEUE_SOUND => {
            let sound = script.word(0);
            game.lease().queue_sound(2, 6, sound);
            script.offset(1)
        }
        instruction::SPAWN_CLAW => {
            let (x, y) = {
                let runtime = game.lease();
                let actor = runtime.resolve_actor(current(&runtime), Native);
                (actor.x.whole, actor.y.whole)
            };
            let (width, height) = (script.word(2), script.word(4));
            game.spawn_in_rect(x, y, width, height, script.word(0));
            script.offset(3)
        }
        instruction::GO_TO_ACTIVE => {
            let mut runtime = game.lease();
            let key = current(&runtime);
            runtime.edit_actor(key, Native, |actor| actor.instruction_delay = 1);
            let x = runtime.resolve_actor(key, Native).x.whole;
            let instruction = if (x.wrapping_sub(runtime.session().player.x) as i16) < 0 {
                list::NINJA_ACTIVE_FACING_RIGHT
            } else {
                list::NINJA_ACTIVE_FACING_LEFT
            };
            runtime.edit_actor(key, Native, |actor| {
                actor.variables[NINJA_ACTIVE_LIST] = instruction
            });
            EnemyScript::jump(instruction)
        }
        instruction::FIRE_LASER_LEFT_WITH_Y_OFFSET
        | instruction::FIRE_LASER_RIGHT_WITH_Y_OFFSET => {
            fire_laser(
                ea == instruction::FIRE_LASER_RIGHT_WITH_Y_OFFSET,
                script.word(0),
                game,
            );
            script.offset(1)
        }
        instruction::CHOOSE_MOVEMENT => {
            let mut runtime = game.lease();
            let key = current(&runtime);
            let attack = api::player_within(&mut runtime, key, Axis::Y, 16);
            EnemyScript::jump(walking_instruction(&runtime, current(&runtime), attack))
        }
        instruction::STOP => {
            stop_ninja(k, game);
            script
        }
        instruction::SET_LEFT_DIVEKICK_SPEED | instruction::SET_RIGHT_DIVEKICK_SPEED => {
            set_divekick_speed(k, ea == instruction::SET_RIGHT_DIVEKICK_SPEED, game);
            script
        }
        instruction::MOVE_DOWN_LEFT_WALL | instruction::MOVE_DOWN_RIGHT_WALL => {
            choose_wall_direction(ea == instruction::MOVE_DOWN_LEFT_WALL, false, script, game)
        }
        instruction::RANDOM_DIRECTION_LEFT_WALL | instruction::RANDOM_DIRECTION_RIGHT_WALL => {
            choose_wall_direction(
                ea == instruction::RANDOM_DIRECTION_LEFT_WALL,
                true,
                script,
                game,
            )
        }
        instruction::PREPARE_WALL_JUMP_RIGHT | instruction::PREPARE_WALL_JUMP_LEFT => {
            prepare_wall_jump(ea == instruction::PREPARE_WALL_JUMP_RIGHT, script, game)
        }
        _ => {
            game.lease().unreachable();
            EnemyScript::Stop
        }
    }
}

fn dispatch_state<G: Pirates + ?Sized>(ea: u32, k: u16, game: &mut G) {
    match ea {
        state::NINJA_INITIAL => ninja_initial(game),
        state::NINJA_ACTIVE => ninja_active(k, game),
        state::NINJA_READY_TO_DIVEKICK => ready_to_divekick(game),
        state::NINJA_SPIN_JUMP_LEFT_RISING => spin_jump(k, false, false, game),
        state::NINJA_SPIN_JUMP_LEFT_FALLING => spin_jump(k, false, true, game),
        state::NINJA_SPIN_JUMP_RIGHT_RISING => spin_jump(k, true, false, game),
        state::NINJA_SPIN_JUMP_RIGHT_FALLING => spin_jump(k, true, true, game),
        state::NINJA_DIVEKICK_LEFT_JUMP => divekick(k, false, DivekickPhase::Jump, game),
        state::NINJA_DIVEKICK_LEFT => divekick(k, false, DivekickPhase::Divekick, game),
        state::NINJA_DIVEKICK_LEFT_WALK_TO_POST => {
            divekick(k, false, DivekickPhase::WalkToPost, game)
        }
        state::NINJA_DIVEKICK_RIGHT_JUMP => divekick(k, true, DivekickPhase::Jump, game),
        state::NINJA_DIVEKICK_RIGHT => divekick(k, true, DivekickPhase::Divekick, game),
        state::NINJA_DIVEKICK_RIGHT_WALK_TO_POST => {
            divekick(k, true, DivekickPhase::WalkToPost, game)
        }
        state::WALL_CLIMBING_LEFT => climb_wall(true, game),
        state::WALL_JUMPING_RIGHT => wall_jump(true, game),
        state::WALL_CLIMBING_RIGHT => climb_wall(false, game),
        state::WALL_JUMPING_LEFT => wall_jump(false, game),
        state::FROZEN
        | state::WALL_LANDED_LEFT
        | state::WALL_LANDED_RIGHT
        | state::WALKING_IDLE => (),
        state::WALKING_LEFT => walk(false, game),
        state::WALKING_RIGHT => walk(true, game),
        _ => game.lease().unreachable(),
    }
}
