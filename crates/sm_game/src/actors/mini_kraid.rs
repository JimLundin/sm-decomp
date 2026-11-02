//! Mini Kraid: the small Kraid that paces back and forth, fires belly spikes
//! on three independent timers and spits twin rocks toward the way it faces.
//!
//! Was `FakeKraid` in the original decompilation (bank $A6:9A58-$A6:9C50). Its
//! view names the slot words over the same RAM bytes as `EnemyLayout40`.
use crate::actors::boss_lease::BossLease;
use crate::actors::deaths::Deaths;
use crate::actors::host::BossEffects;
use crate::actors::layouts::{EnemyBase, EnemyLayout40};
use crate::enemy_script::EnemyScript;

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

/// Sets up Mini Kraid with random timers, facing Samus. Was `FakeKraid_Init`.
pub fn initialize<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| owned::initialize(&mut crate::runtime_access::RuntimeAccess::runtime(game)))
}

/// Checks the next belly spike timer. Was `FakeKraid_Main`.
pub fn main_ai<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| owned::main_ai(&mut crate::runtime_access::RuntimeAccess::runtime(game)))
}

/// Instruction: one step. Turns around when the walk runs out or a wall
/// blocks it, then faces Samus. Was `FakeKraid_Instr_2`.
pub fn step<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| owned::step(&mut crate::runtime_access::RuntimeAccess::runtime(game)))
}

/// Instruction: the next walk animation for the way Mini Kraid faces and
/// walks, or a spit once enough walk cycles have passed. Was `FakeKraid_Instr_1`.
pub fn choose_animation<L: Lease + ?Sized>(game: &mut L) -> EnemyScript {
    game.foreign(|game| {
        owned::choose_animation(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    })
}

/// Instruction: roar, when on screen. Was `FakeKraid_Instr_3`.
pub fn roar<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| owned::roar(&mut crate::runtime_access::RuntimeAccess::runtime(game)))
}

/// Instruction: spit left. Was `FakeKraid_Instr_4`.
pub fn spit_left<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| owned::spit_left(&mut crate::runtime_access::RuntimeAccess::runtime(game)))
}

/// Instruction: spit right. Was `FakeKraid_Instr_5`.
pub fn spit_right<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| owned::spit_right(&mut crate::runtime_access::RuntimeAccess::runtime(game)))
}

/// Touched by Samus. Was `FakeKraid_Touch`.
pub fn touch<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| owned::touch(&mut crate::runtime_access::RuntimeAccess::runtime(game)))
}

/// Hit by a shot. Was `FakeKraid_Shot`.
pub fn shot<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| owned::shot(&mut crate::runtime_access::RuntimeAccess::runtime(game)))
}

#[path = "mini_kraid_owned.rs"]
pub mod owned;
