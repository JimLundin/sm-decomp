//! Space pirates in bank $B2: wall-jumping pirates, ninja pirates and walking
//! pirates. They share checked actor state and movement; their extended words
//! are the Session's, through [`Pirate`].
use crate::actors::access::{AccessMode::Native, ActorKey as Key};
use crate::actors::api::{self, Axis, MovementResult};
use crate::actors::combat;
use crate::actors::slot_view::{SlotState, slot_view};
use crate::enemy_script::EnemyScript;
use crate::player_projectiles::access::shot;
use crate::sprite_effect_state::{LegacySpriteDefinition, SpriteSpawn};
use sm_data::cartridge::RomRecord;

/// Byte offset of an enemy definition's vulnerability table pointer.
pub const DEFINITION_VULNERABILITY: usize = 0x3c;
/// Byte offsets in a vulnerability table: power beam, and the plasma/ice/wave
/// row the beam combinations index from.
pub const VULNERABILITY_POWER: usize = 0;
pub const VULNERABILITY_PLASMA_ICE_WAVE: usize = 11;

/// One original `Runtime`: the actor, projectile and mechanism access a
/// pirate routine edits through, the slots on the Session, the session-aware
/// RAM words, and owned steps.
pub trait PirateLease:
    crate::actors::effects::ActorHost
    + crate::enemy_projectiles::host::Host
    + crate::room_mechanisms::host::Host
    + crate::room_mechanisms::host::Effects
    + crate::player_projectiles::host::Host
    + crate::sprite_effect_host::Host
    + crate::records::Records
    + crate::word_transfers::LoadedRom
    + SlotState
{
    /// The compatibility word at `at`, through the session projection.
    fn word(&self, at: usize) -> u16;
    fn write_word_le(&mut self, value: u16, at: usize);
    /// Run `action` on this lease: the external observations are refreshed on entry and on return.
    fn owned_step<T>(&mut self, action: impl FnOnce(&mut Self) -> T) -> T;
    /// The actor content record at `bank:address`, as the original actor read.
    fn actor_legacy_record(&mut self, bank: u8, address: u16) -> RomRecord;
    /// The native record at `bank:address`, with the original diagnostic.
    fn legacy_native_record(&mut self, bank: u8, address: u16) -> RomRecord;
}

/// What the pirates need beyond their lease: the two projectile spawns that set
/// the spawn parameters first.
pub trait PirateHost: PirateLease {
    /// Fire a pirate laser from (`x`, `y`): the spawn point, its facing and
    /// the room-graphics laser projectile, in that order.
    fn fire_laser_projectile(&mut self, x: u16, y: u16, right: bool);
    /// Spawn projectile `parameter` from the current enemy within the
    /// rectangle at (`x`, `y`), after setting the rectangle.
    fn spawn_in_rect(&mut self, x: u16, y: u16, width: u16, height: u16, parameter: u16);
}

/// The ninja pirates' palette ($B2:8727), the target colours of sprite palette 7.
const NINJA_PALETTE: u32 = 0xb2_8727;
const NINJA_PALETTE_TARGETS: core::ops::Range<usize> = 240..256;

/// Copy the ninja pirates' palette into the target colours, first colour
/// first, in native order.
fn load_ninja_palette<R: PirateLease>(runtime: &mut R) {
    let colours: [u16; 16] = crate::word_transfers::rom_words(runtime, NINJA_PALETTE);
    let targets = &mut runtime.session_mut().presentation.visual_mut().targets;
    for (target, colour) in targets[NINJA_PALETTE_TARGETS].iter_mut().zip(colours) {
        *target = colour;
    }
}

/// The high word of `amplitude` times the sine of `phase`.
fn actor_sine<R: PirateLease>(runtime: &mut R, phase: u16, amplitude: u16) -> u16 {
    let angle = u16::from(phase.wrapping_add(0x80) as u8);
    (crate::actors::queries::sine_product(runtime, amplitude, angle) >> 16) as u16
}

/// The high word of `amplitude` times the cosine of `phase`.
fn actor_cosine<R: PirateLease>(runtime: &mut R, phase: u16, amplitude: u16) -> u16 {
    let angle = u16::from(phase.wrapping_add(64) as u8);
    (crate::actors::queries::sine_product(runtime, amplitude, angle) >> 16) as u16
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

slot_view! {
    /// A pirate's extended words. Ninja: its speed, the high byte the per-frame
    /// movement (walking: an X backup), the divekick's midpoint X (never read)
    /// and its spawn Y, restored after a divekick. Wall: the arc's right and
    /// left target angles and its per-frame angle step.
    Pirate, PirateEdit {
        record: [],
        extended: [
            speed = var_00, divekick_midpoint_x = var_03, spawn_y = var_08,
            arc_right_target = var_20, arc_left_target = var_21, arc_angle_delta = var_22,
        ],
    }
}

/// A pirate's extended words, as [`Pirate`] names them.
#[derive(Clone, Copy)]
enum Extra {
    Speed,
    DivekickMidpointX,
    SpawnY,
    ArcRightTarget,
    ArcLeftTarget,
    ArcAngleDelta,
}
/// Pirate `index`'s extended word `word`.
fn extra(runtime: &impl SlotState, index: u16, word: Extra) -> u16 {
    let pirate = Pirate::of(runtime, index);
    match word {
        Extra::Speed => pirate.speed(),
        Extra::DivekickMidpointX => pirate.divekick_midpoint_x(),
        Extra::SpawnY => pirate.spawn_y(),
        Extra::ArcRightTarget => pirate.arc_right_target(),
        Extra::ArcLeftTarget => pirate.arc_left_target(),
        Extra::ArcAngleDelta => pirate.arc_angle_delta(),
    }
}
/// Store `value` to pirate `index`'s extended word `word`, then refresh the
/// lease's observations after each byte.
fn set_extra(runtime: &mut impl SlotState, index: u16, word: Extra, value: u16) {
    let mut pirate = PirateEdit::of(runtime, index);
    *match word {
        Extra::Speed => pirate.speed(),
        Extra::DivekickMidpointX => pirate.divekick_midpoint_x(),
        Extra::SpawnY => pirate.spawn_y(),
        Extra::ArcRightTarget => pirate.arc_right_target(),
        Extra::ArcLeftTarget => pirate.arc_left_target(),
        Extra::ArcAngleDelta => pirate.arc_angle_delta(),
    } = value;
    drop(pirate);
    runtime.record_stored();
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
    Key::from_legacy(runtime.session().actors.controls.selected)
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

fn initialize_wall(runtime: &mut impl PirateHost) {
    let index = runtime.session().actors.controls.selected;
    let key = Key::from_legacy(index);
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction = if actor.parameters[0] & 1 == 0 {
            list::MOVING_DOWN_LEFT_WALL
        } else {
            list::MOVING_DOWN_RIGHT_WALL
        };
    });
    for (word, value) in [
        (Extra::ArcRightTarget, 190),
        (Extra::ArcLeftTarget, 66),
        (Extra::ArcAngleDelta, 2),
    ] {
        set_extra(runtime, index, word, value);
    }
    if runtime.resolve_actor(key, Native).parameters[0] & PARAMETER_ALTERNATE == 0 {
        for (word, delta) in [
            (Extra::ArcRightTarget, 2u16),
            (Extra::ArcLeftTarget, u16::MAX - 1),
            (Extra::ArcAngleDelta, 2),
        ] {
            let value = extra(&*runtime, index, word).wrapping_add(delta);
            set_extra(runtime, index, word, value);
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

fn climb_wall<R: PirateLease>(right: bool, runtime: &mut R) {
    let key = current(runtime);
    if api::player_within(runtime, key, Axis::Y, 32) {
        let key = current(runtime);
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

fn wall_jump<R: PirateLease>(right: bool, runtime: &mut R) {
    let index = runtime.session().actors.controls.selected;
    let key = Key::from_legacy(index);
    let actor = runtime.resolve_actor(key, Native);
    let (origin, phase, radius) = (
        actor.variables[WALL_ARC_CENTRE_X],
        actor.pre_instruction,
        actor.parameters[1] >> 1,
    );
    let dx = runtime.owned_step(|runtime| actor_sine(runtime, phase, radius));
    runtime.edit_actor(key, Native, |actor| actor.x.whole = origin.wrapping_add(dx));
    let actor = runtime.resolve_actor(key, Native);
    let (origin, phase, radius) = (
        actor.variables[WALL_ARC_CENTRE_Y],
        actor.pre_instruction,
        actor.parameters[1] >> 2,
    );
    let dy = runtime.owned_step(|runtime| actor_cosine(runtime, phase, radius));
    runtime.edit_actor(key, Native, |actor| actor.y.whole = origin.wrapping_sub(dy));
    let phase = runtime
        .resolve_actor(key, Native)
        .pre_instruction
        .to_ne_bytes()[0];
    let step = Pirate::of(runtime, index).arc_angle_delta().to_le_bytes()[0];
    let phase = u16::from(if right {
        phase.wrapping_sub(step)
    } else {
        phase.wrapping_add(step)
    });
    runtime.edit_actor(key, Native, |actor| actor.pre_instruction = phase);
    let end = extra(
        runtime,
        index,
        if right {
            Extra::ArcRightTarget
        } else {
            Extra::ArcLeftTarget
        },
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

fn choose_wall_direction(
    right: bool,
    random: bool,
    script: EnemyScript,
    runtime: &mut impl PirateHost,
) -> EnemyScript {
    let direction = if random {
        let value = runtime.next_random() & 1;
        let key = current(&*runtime);
        runtime.edit_actor(key, Native, |actor| actor.variables[WALL_DIRECTION] = value);
        value
    } else {
        let key = current(&*runtime);
        let distance = (script.word(0) as i16 as i32) << 16;
        if api::move_vertical(runtime, key, distance) != MovementResult::Blocked {
            return script.offset(1);
        }
        let key = current(&*runtime);
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

fn prepare_wall_jump(
    right: bool,
    script: EnemyScript,
    runtime: &mut impl PirateHost,
) -> EnemyScript {
    let key = current(&*runtime);
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

fn ninja_close_attack<R: PirateLease>(check_center: bool, runtime: &mut R) -> bool {
    let key = current(runtime);
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

/// Flinch from the last of Samus's first five shots within 32 pixels of the
/// current pirate: the flinch list facing her, on `runtime`.
fn flinch<R: PirateLease>(walking: bool, runtime: &mut R) -> bool {
    let Some(shot) = runtime.session().projectiles.shots[..5]
        .iter()
        .rev()
        .find(|shot| shot.kind != 0)
    else {
        return false;
    };
    let key = current(runtime);
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

fn fire_laser(right: bool, height: u16, runtime: &mut impl PirateHost) {
    let (x, y) = {
        let actor = runtime.resolve_actor(current(&*runtime), Native);
        (
            offset_x(actor.x.whole, 24, right),
            actor.y.whole.wrapping_sub(height),
        )
    };
    runtime.fire_laser_projectile(x, y, right);
}

fn landing_dust<R: PirateLease>(runtime: &mut R, key: Key) {
    for dx in [0xfff8, 8] {
        let actor = runtime.resolve_actor(key, Native);
        let (x, y) = (
            actor.x.whole.wrapping_add(dx),
            actor.y.whole.wrapping_add(28),
        );
        runtime.owned_step(|runtime| {
            let spawn = SpriteSpawn {
                position: crate::geometry::Point { x, y },
                definition: LegacySpriteDefinition(10),
                appearance: 0,
            };
            crate::sprite_objects::spawn(runtime, spawn);
        });
    }
}

fn spin_jump<R: PirateLease>(index: u16, right: bool, descending: bool, runtime: &mut R) {
    let key = Key::from_legacy(index);
    let speed = Pirate::of(runtime, index).speed() >> 8;
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
    let velocity = Pirate::of(runtime, index)
        .speed()
        .wrapping_add(if descending { 0xffe0 } else { 32 });
    set_extra(runtime, index, Extra::Speed, velocity);
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
            landing_dust(runtime, key);
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

fn divekick<R: PirateLease>(index: u16, right: bool, phase: DivekickPhase, runtime: &mut R) {
    let key = Key::from_legacy(index);
    match phase {
        DivekickPhase::Jump => {
            let speed = Pirate::of(runtime, index).speed() >> 8;
            runtime.edit_actor(key, Native, |actor| {
                actor.y.whole = actor.y.whole.wrapping_sub(speed)
            });
            let velocity = (Pirate::of(runtime, index).speed() as i16).wrapping_sub(64);
            set_extra(runtime, index, Extra::Speed, velocity as u16);
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
                set_extra(runtime, index, Extra::Speed, 1536);
            }
        }
        DivekickPhase::Divekick => {
            runtime.edit_actor(key, Native, |actor| {
                actor.x.whole = offset_x(actor.x.whole, 5, right);
            });
            let [fraction, whole] = Pirate::of(runtime, index).speed().to_le_bytes();
            let amount = (i32::from(whole) << 16) | i32::from(fraction);
            let finished = api::move_vertical(runtime, key, amount) == MovementResult::Blocked || {
                let velocity = (Pirate::of(runtime, index).speed() as i16).wrapping_sub(64);
                set_extra(runtime, index, Extra::Speed, velocity as u16);
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
                let y = Pirate::of(runtime, index).spawn_y();
                runtime.edit_actor(key, Native, |actor| actor.y.whole = y);
                landing_dust(runtime, key);
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

fn stop_ninja(index: u16, runtime: &mut impl PirateHost) {
    set_extra(runtime, index, Extra::Speed, 0);
}

fn set_divekick_speed(index: u16, right: bool, runtime: &mut impl PirateHost) {
    set_extra(runtime, index, Extra::Speed, 1536);
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
    set_extra(runtime, index, Extra::DivekickMidpointX, midpoint);
}

fn initialize_ninja(runtime: &mut impl PirateHost) {
    let index = runtime.session().actors.controls.selected;
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
    set_extra(runtime, index, Extra::SpawnY, y);
    runtime.owned_step(load_ninja_palette);
}

fn ninja_initial<R: PirateLease>(runtime: &mut R) {
    let key = current(runtime);
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
        runtime.owned_step(|runtime| flinch(false, runtime));
    }
}

fn ninja_active<R: PirateLease>(index: u16, runtime: &mut R) {
    if flinch(false, runtime) || ninja_close_attack(true, runtime) {
        return;
    }
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

fn ready_to_divekick<R: PirateLease>(runtime: &mut R) {
    if flinch(false, runtime) || ninja_close_attack(false, runtime) {
        return;
    }
    let key = current(runtime);
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
    let instruction = runtime.owned_step(|runtime| {
        crate::gameplay_tables::pirate_instruction_list(runtime, choice as isize)
    });
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction = instruction;
        actor.instruction_delay = 1;
    });
}

fn damage<R: PirateLease>(runtime: &mut R) {
    let key = current(runtime);
    let x = runtime.resolve_actor(key, Native).x.whole;
    runtime.session_mut().enemy_projectiles.drop_origin.x = x;
    runtime.words_stored();
    let y = runtime.resolve_actor(key, Native).y.whole;
    runtime.session_mut().enemy_projectiles.drop_origin.y = y;
    runtime.words_stored();
    combat::shot_reaction(runtime, api::ShotHandling::ReactionOnly);
    if runtime.resolve_actor(key, Native).health == 0 {
        let index = runtime.session().actors.controls.selected;
        let drop_items = runtime.resolve_actor(key, Native).definition == GOLD_NINJA;
        runtime.edit_actor(key, Native, |actor| {
            actor.variables[NINJA_INITIAL_SPEED] = 0
        });
        combat::die(runtime, api::DeathKind::Normal { animation: 4 });
        if drop_items {
            runtime.owned_step(|runtime| {
                crate::enemy_projectiles::drops::scatter(
                    crate::enemy_projectiles::drops::DropSource::LowerNorfairPirate,
                    index,
                    runtime,
                );
            });
        }
    }
}

/// Enemy touch - space pirate ($B2:876C), on the actor's lease.
pub fn touch_on<R: PirateLease>(runtime: &mut R) {
    if runtime.resolve_actor(current(runtime), Native).frozen == 0 {
        combat::contact_reaction(runtime, api::DeathHandling::Finalize);
    }
}

/// Enemy shot - space pirate ($B2:8779), on the actor's lease.
pub fn shot_on<R: PirateLease>(runtime: &mut R) {
    if runtime.resolve_actor(current(runtime), Native).definition == GOLD_NINJA {
        damage(runtime);
    } else {
        combat::shot_reaction(runtime, api::ShotHandling::Finalize);
    }
}

/// Enemy shot - space pirate, reflecting what the gold ninja pirate is immune
/// to ($B2:87C8), on the actor's lease.
pub fn check_shot_on<R: PirateLease>(runtime: &mut R) {
    if runtime.resolve_actor(current(runtime), Native).definition == GOLD_NINJA {
        let raw = runtime
            .session()
            .room
            .collision_detection_index
            .wrapping_mul(2);
        let kind = shot!(runtime, raw, kind);
        let class = kind & 0xf00;
        if class >= 0x300 {
            return;
        }
        let definition = runtime.resolve_actor(current(runtime), Native).definition;
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
            reflect_shot(runtime);
            return;
        }
    }
    damage(runtime);
}

/// Enemy shot - space pirate, the gold ninja's reflection ($B2:883E), on the
/// actor's lease.
pub fn reflect_on<R: PirateLease>(runtime: &mut R) {
    reflect_shot(runtime);
}

fn reflect_shot<R: PirateLease>(runtime: &mut R) {
    if runtime.resolve_actor(current(runtime), Native).definition != GOLD_NINJA {
        damage(runtime);
        return;
    }
    let reflection_index = runtime
        .session()
        .room
        .collision_detection_index
        .wrapping_mul(2);
    let class = shot!(runtime, reflection_index, kind) & 0xf00;
    if class == 0x200 {
        if shot!(runtime, reflection_index, variable) == 0 {
            return;
        }
    } else if class >= 0x300 {
        return;
    }
    let key = current(runtime);
    runtime.edit_actor(key, Native, |actor| actor.invincibility = 10);
    let direction = match shot!(runtime, reflection_index, direction) & 15 {
        7 => 1,
        2 => 8,
        _ => 5,
    };
    shot!(runtime, reflection_index, direction = direction; PrefixLe);
    runtime.owned_step(|runtime| {
        crate::player_projectiles::motion::reflect(runtime, reflection_index)
    });
    runtime.queue_sound(2, 6, 0x66);
}

fn initialize_walking(runtime: &mut impl PirateHost) {
    let key = current(&*runtime);
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

fn walk<R: PirateLease>(right: bool, runtime: &mut R) {
    let index = runtime.session().actors.controls.selected;
    let key = Key::from_legacy(index);
    if api::player_within(runtime, key, Axis::Y, 16) {
        let instruction = walking_instruction(runtime, key, true);
        runtime.edit_actor(key, Native, |actor| {
            actor.instruction = instruction;
            actor.instruction_delay = 1;
        });
        return;
    }
    let mover = current(runtime);
    if api::move_vertical(runtime, mover, 1 << 16) != MovementResult::Blocked {
        return;
    }
    let x = runtime.resolve_actor(key, Native).x.whole;
    set_extra(runtime, index, Extra::Speed, x);
    runtime.edit_actor(key, Native, |actor| {
        actor.x.whole = offset_x(x, if right { 16 } else { 17 }, right)
    });
    let mover = current(runtime);
    let floor = api::move_vertical(runtime, mover, 1 << 16) == MovementResult::Blocked;
    let x = Pirate::of(runtime, index).speed();
    runtime.edit_actor(key, Native, |actor| actor.x.whole = x);
    let turn = !floor || {
        if !right {
            let mover = current(runtime);
            api::probe_horizontal(runtime, mover, -9 << 16);
        }
        let mover = current(runtime);
        api::move_horizontal(
            runtime,
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

fn advance(walking: bool, runtime: &mut impl PirateHost) {
    let key = current(&*runtime);
    let state = u32::from(runtime.resolve_actor(key, Native).variables[FUNCTION]) | 0xb20000;
    let index = runtime.session().actors.controls.selected;
    runtime.owned_step(|runtime| dispatch_state(state, index, runtime));
    if walking && runtime.resolve_actor(key, Native).parameters[0] & PARAMETER_ALTERNATE != 0 {
        runtime.owned_step(|runtime| flinch(true, runtime));
    }
}

pub fn dispatch_ai(ea: u32, runtime: &mut impl PirateHost) {
    match ea {
        routine::TOUCH => touch_on(runtime),
        routine::SHOT => shot_on(runtime),
        routine::WALKING_INITIALIZATION => initialize_walking(runtime),
        routine::WALKING_MAIN_AI => advance(true, runtime),
        routine::FROZEN_AI => combat::select_frozen_ai(runtime),
        routine::UPDATE_FROZEN => crate::actors::scheduling::update_frozen_enemy(runtime),
        routine::POWER_BOMB_REACTION => {
            combat::power_bomb_reaction(runtime, api::DeathHandling::Finalize)
        }
        routine::WALL_MAIN_AI | routine::NINJA_MAIN_AI => advance(false, runtime),
        routine::WALL_INITIALIZATION => initialize_wall(runtime),
        routine::NINJA_INITIALIZATION => initialize_ninja(runtime),
        _ => runtime.unreachable(),
    }
}

pub fn dispatch_instruction(
    ea: u32,
    k: u16,
    script: EnemyScript,
    runtime: &mut impl PirateHost,
) -> EnemyScript {
    match ea {
        instruction::FIRE_LASER_LEFT => {
            let index = runtime.session().actors.controls.selected;
            let definition = runtime
                .resolve_actor(Key::from_legacy(index), Native)
                .definition;
            let value = PirateLease::legacy_native_record(runtime, 0xa0, definition).word(6);
            runtime.session_mut().enemy_projectiles.spawn.primary = value;
            fire_laser(false, 16, runtime);
            script
        }
        instruction::FIRE_LASER_RIGHT => {
            fire_laser(true, 16, runtime);
            script
        }
        instruction::WALL_SET_FUNCTION
        | instruction::WALKING_SET_FUNCTION
        | instruction::SET_PALETTE => {
            let value = script.word(0);
            let key = current(&*runtime);
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
            runtime.queue_sound(2, 6, 0x66);
            script
        }
        instruction::QUEUE_SOUND => {
            let sound = script.word(0);
            runtime.queue_sound(2, 6, sound);
            script.offset(1)
        }
        instruction::SPAWN_CLAW => {
            let (x, y) = {
                let actor = runtime.resolve_actor(current(&*runtime), Native);
                (actor.x.whole, actor.y.whole)
            };
            let (width, height) = (script.word(2), script.word(4));
            runtime.spawn_in_rect(x, y, width, height, script.word(0));
            script.offset(3)
        }
        instruction::GO_TO_ACTIVE => {
            let key = current(&*runtime);
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
                runtime,
            );
            script.offset(1)
        }
        instruction::CHOOSE_MOVEMENT => {
            let key = current(&*runtime);
            let attack = api::player_within(runtime, key, Axis::Y, 16);
            EnemyScript::jump(walking_instruction(&*runtime, current(&*runtime), attack))
        }
        instruction::STOP => {
            stop_ninja(k, runtime);
            script
        }
        instruction::SET_LEFT_DIVEKICK_SPEED | instruction::SET_RIGHT_DIVEKICK_SPEED => {
            set_divekick_speed(k, ea == instruction::SET_RIGHT_DIVEKICK_SPEED, runtime);
            script
        }
        instruction::MOVE_DOWN_LEFT_WALL | instruction::MOVE_DOWN_RIGHT_WALL => {
            choose_wall_direction(
                ea == instruction::MOVE_DOWN_LEFT_WALL,
                false,
                script,
                runtime,
            )
        }
        instruction::RANDOM_DIRECTION_LEFT_WALL | instruction::RANDOM_DIRECTION_RIGHT_WALL => {
            choose_wall_direction(
                ea == instruction::RANDOM_DIRECTION_LEFT_WALL,
                true,
                script,
                runtime,
            )
        }
        instruction::PREPARE_WALL_JUMP_RIGHT | instruction::PREPARE_WALL_JUMP_LEFT => {
            prepare_wall_jump(ea == instruction::PREPARE_WALL_JUMP_RIGHT, script, runtime)
        }
        _ => runtime.unreachable(),
    }
}

fn dispatch_state<R: PirateLease>(ea: u32, k: u16, runtime: &mut R) {
    match ea {
        state::NINJA_INITIAL => ninja_initial(runtime),
        state::NINJA_ACTIVE => ninja_active(k, runtime),
        state::NINJA_READY_TO_DIVEKICK => ready_to_divekick(runtime),
        state::NINJA_SPIN_JUMP_LEFT_RISING => spin_jump(k, false, false, runtime),
        state::NINJA_SPIN_JUMP_LEFT_FALLING => spin_jump(k, false, true, runtime),
        state::NINJA_SPIN_JUMP_RIGHT_RISING => spin_jump(k, true, false, runtime),
        state::NINJA_SPIN_JUMP_RIGHT_FALLING => spin_jump(k, true, true, runtime),
        state::NINJA_DIVEKICK_LEFT_JUMP => divekick(k, false, DivekickPhase::Jump, runtime),
        state::NINJA_DIVEKICK_LEFT => divekick(k, false, DivekickPhase::Divekick, runtime),
        state::NINJA_DIVEKICK_LEFT_WALK_TO_POST => {
            divekick(k, false, DivekickPhase::WalkToPost, runtime)
        }
        state::NINJA_DIVEKICK_RIGHT_JUMP => divekick(k, true, DivekickPhase::Jump, runtime),
        state::NINJA_DIVEKICK_RIGHT => divekick(k, true, DivekickPhase::Divekick, runtime),
        state::NINJA_DIVEKICK_RIGHT_WALK_TO_POST => {
            divekick(k, true, DivekickPhase::WalkToPost, runtime)
        }
        state::WALL_CLIMBING_LEFT => climb_wall(true, runtime),
        state::WALL_JUMPING_RIGHT => wall_jump(true, runtime),
        state::WALL_CLIMBING_RIGHT => climb_wall(false, runtime),
        state::WALL_JUMPING_LEFT => wall_jump(false, runtime),
        state::FROZEN
        | state::WALL_LANDED_LEFT
        | state::WALL_LANDED_RIGHT
        | state::WALKING_IDLE => (),
        state::WALKING_LEFT => walk(false, runtime),
        state::WALKING_RIGHT => walk(true, runtime),
        _ => runtime.unreachable(),
    }
}
