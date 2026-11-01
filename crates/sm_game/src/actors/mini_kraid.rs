//! Mini Kraid: the small Kraid that paces back and forth, fires belly spikes
//! on three independent timers and spits twin rocks toward the way it faces.
//!
//! Was `FakeKraid` in the original decompilation (bank $A6:9A58-$A6:9C50). Its
//! view names the slot words over the same RAM bytes as `EnemyLayout40`.
use crate::actors::boss_lease::BossLease;
use crate::actors::deaths::Deaths;
use crate::actors::host::{BossEffects, Death, Shot, Slopes};
use crate::actors::layouts::{EnemyBase, EnemyLayout40};
use crate::constants::enemy_properties::K_ENEMY_PROPS_DISABLE_SAMUS_COLL;
use crate::enemy_projectiles::drops::DropSource;
use crate::enemy_script::EnemyScript;
use crate::ram::word;
use crate::session_slot;

/// A lease on a game with Mini Kraid's effects: the shared boss effects,
/// dying and its drop.
pub trait Lease: BossLease<Game: BossEffects + Deaths> {}
impl<T: BossLease<Game: BossEffects + Deaths> + ?Sized> Lease for T {}

/// Mini Kraid's view of its `EnemyLayout40` slot.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct MiniKraid {
    pub base: EnemyBase,
    pub var_a: u16,
    /// Pixels per step, signed: negative walks left.
    pub walk_speed: u16,
    /// [`FACING_LEFT`] or [`FACING_RIGHT`]: toward Samus at each step.
    pub facing: u16,
    /// Steps left before turning around.
    pub steps_until_turn: u16,
    /// Walk cycles left before spitting.
    pub cycles_until_spit: u16,
    pub var_f: u16,
    pub parameter_1: u16,
    pub parameter_2: u16,
    pub pad: [u8; 26696],
    /// The spit rocks read their velocity from here when they spawn.
    pub spit_x_velocity: u16,
    pub spit_y_velocity: u16,
    /// Horizontal offset of the spit from Mini Kraid's mouth.
    pub spit_x_offset: u16,
    /// The first of three belly spike timers; a byte offset of 0, 2 or 4
    /// from this slot selects the other two.
    pub spike_timer: u16,
    pub spike_timer_1: u16,
    pub spike_timer_2: u16,
    /// The byte offset of the spike that last fired.
    pub last_spike: u16,
    /// The spike timer checked this frame, as a byte offset: 0, 2 or 4.
    pub spike_cycle: u16,
    /// The slot's remaining words, unused by the Mini Kraid.
    pub unused: [u16; 24],
}

const _: () = {
    use core::mem::{offset_of, size_of};
    assert!(offset_of!(MiniKraid, walk_speed) == offset_of!(EnemyLayout40, var_b));
    assert!(offset_of!(MiniKraid, cycles_until_spit) == offset_of!(EnemyLayout40, var_e));
    assert!(offset_of!(MiniKraid, pad) == offset_of!(EnemyLayout40, pad1));
    assert!(offset_of!(MiniKraid, spit_x_velocity) == offset_of!(EnemyLayout40, var_00));
    assert!(offset_of!(MiniKraid, spike_timer) == offset_of!(EnemyLayout40, var_03));
    assert!(offset_of!(MiniKraid, spike_cycle) == offset_of!(EnemyLayout40, var_07));
    assert!(size_of::<MiniKraid>() == size_of::<EnemyLayout40>());
};

/// The `facing` and `walk_speed` values.
pub const FACING_LEFT: u16 = -4i16 as u16;
pub const FACING_RIGHT: u16 = 4;

/// Bank $A6 instruction lists.
mod animation {
    pub const WALK_FORWARD_LEFT: u16 = 0x99ae;
    pub const WALK_BACKWARD_LEFT: u16 = 0x99c6;
    pub const SPIT_LEFT: u16 = 0x99dc;
    pub const WALK_FORWARD_RIGHT: u16 = 0x99fc;
    pub const WALK_BACKWARD_RIGHT: u16 = 0x9a14;
    pub const SPIT_RIGHT: u16 = 0x9a2a;
}

/// Bank $86 enemy projectiles.
mod projectile {
    pub const SPIT: u16 = 0x9db0;
    pub const SPIKES_LEFT: u16 = 0x9dbe;
    pub const SPIKES_RIGHT: u16 = 0x9dcc;
}

/// The spit rocks' (x, y) velocities: two thrown left, then two thrown right.
const SPIT_VELOCITIES: [u16; 8] = [0xfe00, 0xfb00, 0xfc00, 0xfb00, 0x200, 0xfb00, 0x400, 0xfb00];
/// The sounds of firing spikes and of roaring.
const SPIKE_SOUND: u8 = 0x3f;
const ROAR_SOUND: u8 = 0x16;
/// The death animation in Mini Kraid's spawn parameter, for all four drops.
const DEATH_ANIMATION: u16 = 3;

fn current<L: Lease + ?Sized>(game: &L) -> i32 {
    i32::from(game.current_enemy())
}

/// Face Samus.
fn face_samus<L: Lease + ?Sized>(game: &mut L, me: i32) {
    session_slot!(game, MiniKraid[me].facing = FACING_LEFT);
    if ((session_slot!(game, MiniKraid[me].base.x_pos)) as i16)
        .wrapping_sub((game.player_x()) as i16)
        < 0
    {
        session_slot!(game, MiniKraid[me].facing = FACING_RIGHT);
    }
}

/// Sets up Mini Kraid with random timers, facing Samus. Was `FakeKraid_Init`.
pub fn initialize<L: Lease + ?Sized>(game: &mut L) {
    let mut steps: u16 = ((game.current_random() as i32 & 3) + 2) as u16;
    let me = current(game);
    session_slot!(game, MiniKraid[me]; steps_until_turn = steps, cycles_until_spit = steps);
    steps = steps.wrapping_add(64);
    session_slot!(game, MiniKraid[me].spike_timer = steps);
    steps = steps.wrapping_add(32);
    session_slot!(game, MiniKraid[me]; spike_timer_1 = steps, spike_timer_2 = steps.wrapping_sub(48),
        spike_cycle = 0);
    session_slot!(
        game,
        MiniKraid[me].base.properties = (session_slot!(game, MiniKraid[me].base.properties))
            | (K_ENEMY_PROPS_DISABLE_SAMUS_COLL as u16)
    );
    session_slot!(game, MiniKraid[me].base; instruction_timer = 1, timer = 0);
    session_slot!(game, MiniKraid[me]; walk_speed = FACING_LEFT, facing = FACING_LEFT);
    let mut walk = animation::WALK_FORWARD_LEFT;
    if ((session_slot!(game, MiniKraid[me].base.x_pos)) as i16)
        .wrapping_sub((game.player_x()) as i16)
        < 0
    {
        session_slot!(game, MiniKraid[me]; walk_speed = FACING_RIGHT, facing = FACING_RIGHT);
        walk = animation::WALK_FORWARD_RIGHT;
    }
    session_slot!(game, MiniKraid[me].base.current_instruction = walk);
}

/// Checks the next belly spike timer. Was `FakeKraid_Main`.
pub fn main_ai<L: Lease + ?Sized>(game: &mut L) {
    let me = current(game);
    let spike = session_slot!(game, MiniKraid[me].spike_cycle);
    let mut next = spike.wrapping_add(2);
    if (spike as i32 - 4i32) & 0x8000 == 0 {
        next = 0;
    }
    session_slot!(game, MiniKraid[me].spike_cycle = next);
    fire_spikes_when_ready(game, game.current_enemy(), spike);
}

/// Counts down belly spike timer `spike` (a byte offset) of Mini Kraid `k`;
/// at zero it fires spikes the way Mini Kraid faces and rearms the timer.
/// Was `FakeKraid_Func_1`.
fn fire_spikes_when_ready<L: Lease + ?Sized>(game: &mut L, k: u16, spike: u16) {
    let timer: i32 = k as i32 + spike as i32;
    let left = session_slot!(game, MiniKraid[timer].spike_timer);
    if left != 0 {
        session_slot!(game, MiniKraid[timer].spike_timer = left.wrapping_sub(1));
    } else {
        session_slot!(
            game,
            MiniKraid[timer].spike_timer = ((game.current_random() as i32 & 0x3f) + 16) as u16
        );
        let me = k as i32;
        session_slot!(game, MiniKraid[me].last_spike = spike);
        let mut spikes = projectile::SPIKES_LEFT;
        let facing: i16 = session_slot!(game, MiniKraid[me].facing) as i16;
        if facing >= 0 {
            spikes = projectile::SPIKES_RIGHT;
        }
        game.spawn_actor_eproj(facing as u16, k, spikes);
        if u16::from(game.current_outside_screen()) == 0 {
            game.queue_sound(1, SPIKE_SOUND, 6);
        }
    }
}

/// Spits two rocks: velocity pairs from row `row` (a byte offset into
/// [`SPIT_VELOCITIES`]), `offset` pixels from the mouth. Was
/// `FakeKraid_InstrHelper_45`.
fn spit<L: Lease + ?Sized>(game: &mut L, row: u16, offset: u16) {
    let k: u16 = game.current_enemy();
    let me = current(game);
    session_slot!(game, MiniKraid[me].spit_x_offset = offset);
    let index: i32 = row as i32 >> 1;
    session_slot!(
        game,
        MiniKraid[me].spit_x_velocity = SPIT_VELOCITIES[index as usize]
    );
    let y_velocity: u16 = SPIT_VELOCITIES[(index + 1) as usize];
    session_slot!(game, MiniKraid[me].spit_y_velocity = y_velocity);
    game.spawn_actor_eproj(y_velocity, game.current_enemy(), projectile::SPIT);
    session_slot!(
        game,
        MiniKraid[me].spit_x_velocity = SPIT_VELOCITIES[(index + 2) as usize]
    );
    let y_velocity: u16 = SPIT_VELOCITIES[(index + 3) as usize];
    session_slot!(game, MiniKraid[me].spit_y_velocity = y_velocity);
    game.spawn_actor_eproj(y_velocity, k, projectile::SPIT);
}

/// Instruction: one step. Turns around when the walk runs out or a wall
/// blocks it, then faces Samus. Was `FakeKraid_Instr_2`.
pub fn step<L: Lease + ?Sized>(game: &mut L) {
    let me = current(game);
    if session_slot!(game, MiniKraid[me].cycles_until_spit) != 0 {
        session_slot!(game, MiniKraid[me].cycles_until_spit, |old| old
            .wrapping_sub(1));
    }
    let steps = session_slot!(game, MiniKraid[me].steps_until_turn);
    session_slot!(game, MiniKraid[me].steps_until_turn, |old| old
        .wrapping_sub(1));
    let turn_around: bool = if steps == 1 {
        session_slot!(
            game,
            MiniKraid[me].steps_until_turn = ((game.current_random() as i32 & 3) + 7) as u16
        );
        true
    } else {
        let actor = game.current_enemy();
        let amount = (session_slot!(game, MiniKraid[me].walk_speed) as i16 as i32) << 16;
        game.move_horizontal(actor, amount, Slopes::Ignore)
    };
    if turn_around {
        session_slot!(
            game,
            MiniKraid[me].walk_speed =
                -(session_slot!(game, MiniKraid[me].walk_speed) as i32) as u16
        );
    }
    face_samus(game, me);
}

/// Instruction: the next walk animation for the way Mini Kraid faces and
/// walks, or a spit once enough walk cycles have passed. Was `FakeKraid_Instr_1`.
pub fn choose_animation<L: Lease + ?Sized>(game: &mut L) -> EnemyScript {
    let me = current(game);
    if session_slot!(game, MiniKraid[me].cycles_until_spit) != 0 {
        if (session_slot!(game, MiniKraid[me].facing) & 0x8000) != 0 {
            if (session_slot!(game, MiniKraid[current(game)].walk_speed) & 0x8000) == 0 {
                return EnemyScript::jump(animation::WALK_BACKWARD_LEFT);
            }
            EnemyScript::jump(animation::WALK_FORWARD_LEFT)
        } else {
            if (session_slot!(game, MiniKraid[me].walk_speed) & 0x8000) != 0 {
                return EnemyScript::jump(animation::WALK_BACKWARD_RIGHT);
            }
            EnemyScript::jump(animation::WALK_FORWARD_RIGHT)
        }
    } else {
        session_slot!(
            game,
            MiniKraid[me].cycles_until_spit = ((game.current_random() as i32 & 3) + 3) as u16
        );
        if (session_slot!(game, MiniKraid[me].facing) & 0x8000) != 0 {
            return EnemyScript::jump(animation::SPIT_LEFT);
        }
        EnemyScript::jump(animation::SPIT_RIGHT)
    }
}

/// Instruction: roar, when on screen. Was `FakeKraid_Instr_3`.
pub fn roar<L: Lease + ?Sized>(game: &mut L) {
    if u16::from(game.current_outside_screen()) == 0 {
        game.queue_sound(1, ROAR_SOUND, 6);
    }
}

/// Instruction: spit left. Was `FakeKraid_Instr_4`.
pub fn spit_left<L: Lease + ?Sized>(game: &mut L) {
    spit(game, 0, FACING_LEFT);
}

/// Instruction: spit right. Was `FakeKraid_Instr_5`.
pub fn spit_right<L: Lease + ?Sized>(game: &mut L) {
    spit(game, 8, FACING_RIGHT);
}

/// Where a death's drops start.
fn mark_drop_origin<L: Lease + ?Sized>(game: &mut L) {
    let me = current(game);
    word!(
        game.SPECIAL_DEATH_ITEM_DROP_X_ORIGIN_POS = session_slot!(game, MiniKraid[me].base.x_pos)
    );
    word!(
        game.SPECIAL_DEATH_ITEM_DROP_Y_ORIGIN_POS = session_slot!(game, MiniKraid[me].base.y_pos)
    );
}

/// Touched by Samus. Was `FakeKraid_Touch`.
pub fn touch<L: Lease + ?Sized>(game: &mut L) {
    mark_drop_origin(game);
    game.contact_reaction(Death::LeavePending);
    die_when_out_of_health(game);
}

/// Hit by a shot. Was `FakeKraid_Shot`.
pub fn shot<L: Lease + ?Sized>(game: &mut L) {
    mark_drop_origin(game);
    game.shot_reaction(Shot::ReactionOnly);
    die_when_out_of_health(game);
}

/// Dies with its drop once out of health. Was `FakeKraid_9C50`.
fn die_when_out_of_health<L: Lease + ?Sized>(game: &mut L) {
    if session_slot!(game, MiniKraid[game.current_enemy() as i32].base.health) == 0 {
        game.foreign(|game| game.normal_death(DEATH_ANIMATION));
        let index = game.current_enemy();
        game.foreign(|game| game.scatter_enemy_drop(DropSource::MiniKraid, index));
    }
}
