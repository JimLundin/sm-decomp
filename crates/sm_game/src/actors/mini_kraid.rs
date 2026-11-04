//! Mini Kraid: the small Kraid that paces back and forth, fires belly spikes
//! on three independent timers and spits twin rocks toward the way it faces.
//!
//! Bank $A6:9A58-$A6:9C50. Its view names the slot words over the same RAM
//! bytes as `EnemyLayout40`.
use crate::actors::boss_lease::BossLease;
use crate::actors::host::{Death, Shot, Slopes};
use crate::actors::layouts::EnemyLayout104;
use crate::actors::slot_view::{SlotState, extended_var, slot_view};
use crate::constants::enemy_properties::K_ENEMY_PROPS_DISABLE_SAMUS_COLL;
use crate::enemy_projectiles::drops::DropSource;
use crate::enemy_script::EnemyScript;
use core::mem::offset_of;

/// A lease on a game with Mini Kraid's effects: the shared boss effects,
/// dying and its drop.
pub trait Lease: BossLease + SlotState + crate::enemy_projectiles::host::Host {}
impl<T> Lease for T where T: BossLease + SlotState + crate::enemy_projectiles::host::Host + ?Sized {}

slot_view! {
    /// Mini Kraid's slot. `walk_speed` is pixels per step, signed: negative
    /// walks left; `facing` is [`FACING_LEFT`] or [`FACING_RIGHT`], toward
    /// Samus at each step; `steps_until_turn` counts the steps left before
    /// turning around and `cycles_until_spit` the walk cycles left before
    /// spitting. The spit rocks read their velocity from `spit_x_velocity`
    /// and `spit_y_velocity` when they spawn; `spit_x_offset` is the spit's
    /// horizontal offset from the mouth. `spike_timer` is the first of three
    /// belly spike timers ([`MiniKraid::spike_timer_at`]); `last_spike` is the
    /// byte offset (0, 2 or 4) of the spike that last fired and `spike_cycle`
    /// the one checked this frame.
    MiniKraid, MiniKraidEdit {
        record: [
            walk_speed = var_b, facing = var_c, steps_until_turn = var_d,
            cycles_until_spit = var_e,
        ],
        extended: [
            spit_x_velocity = var_00, spit_y_velocity = var_01, spit_x_offset = var_02,
            spike_timer = var_03, spike_timer_1 = var_04, spike_timer_2 = var_05,
            last_spike = var_06, spike_cycle = var_07,
        ],
    }
}

/// The extended word of the belly spike timer at byte offset `spike` (0, 2 or
/// 4) from `spike_timer`.
fn spike_timer_var(spike: u16) -> usize {
    debug_assert!(spike.is_multiple_of(2));
    extended_var(offset_of!(EnemyLayout104, var_03)) + usize::from(spike / 2)
}

impl MiniKraid<'_> {
    /// The belly spike timer at byte offset `spike` from `spike_timer`.
    fn spike_timer_at(&self, spike: u16) -> u16 {
        self.0.extended(spike_timer_var(spike))
    }
}

impl<H: SlotState + ?Sized> MiniKraidEdit<'_, H> {
    /// The belly spike timer at byte offset `spike` from `spike_timer`, to
    /// store to.
    fn spike_timer_at(&mut self, spike: u16) -> &mut u16 {
        self.0.extended(spike_timer_var(spike))
    }
}

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

/// Face Samus.
fn face_samus<L: Lease>(game: &mut L, k: u16) {
    *MiniKraidEdit::of(game, k).facing() = FACING_LEFT;
    if ((MiniKraid::of(game, k).actor().x.whole) as i16).wrapping_sub((game.player_x()) as i16) < 0
    {
        *MiniKraidEdit::of(game, k).facing() = FACING_RIGHT;
    }
}

/// Sets up Mini Kraid with random timers, facing Samus.
pub fn initialize<L: Lease>(game: &mut L) {
    let mut steps: u16 = ((game.current_random() as i32 & 3) + 2) as u16;
    let k = game.current_enemy();
    *MiniKraidEdit::of(game, k).steps_until_turn() = steps;
    *MiniKraidEdit::of(game, k).cycles_until_spit() = steps;
    steps = steps.wrapping_add(64);
    *MiniKraidEdit::of(game, k).spike_timer() = steps;
    steps = steps.wrapping_add(32);
    *MiniKraidEdit::of(game, k).spike_timer_1() = steps;
    *MiniKraidEdit::of(game, k).spike_timer_2() = steps.wrapping_sub(48);
    *MiniKraidEdit::of(game, k).spike_cycle() = 0;
    MiniKraidEdit::of(game, k).actor().properties =
        (MiniKraid::of(game, k).actor().properties) | (K_ENEMY_PROPS_DISABLE_SAMUS_COLL as u16);
    MiniKraidEdit::of(game, k).actor().instruction_delay = 1;
    MiniKraidEdit::of(game, k).actor().timer = 0;
    *MiniKraidEdit::of(game, k).walk_speed() = FACING_LEFT;
    *MiniKraidEdit::of(game, k).facing() = FACING_LEFT;
    let mut walk = animation::WALK_FORWARD_LEFT;
    if ((MiniKraid::of(game, k).actor().x.whole) as i16).wrapping_sub((game.player_x()) as i16) < 0
    {
        *MiniKraidEdit::of(game, k).walk_speed() = FACING_RIGHT;
        *MiniKraidEdit::of(game, k).facing() = FACING_RIGHT;
        walk = animation::WALK_FORWARD_RIGHT;
    }
    MiniKraidEdit::of(game, k).actor().instruction = walk;
}

/// Checks the next belly spike timer.
pub fn main_ai<L: Lease>(game: &mut L) {
    let k = game.current_enemy();
    let spike = MiniKraid::of(game, k).spike_cycle();
    let mut next = spike.wrapping_add(2);
    if (spike as i32 - 4i32) & 0x8000 == 0 {
        next = 0;
    }
    *MiniKraidEdit::of(game, k).spike_cycle() = next;
    fire_spikes_when_ready(game, game.current_enemy(), spike);
}

/// Counts down belly spike timer `spike` (a byte offset) of Mini Kraid `k`;
/// at zero it fires spikes the way Mini Kraid faces and rearms the timer.
fn fire_spikes_when_ready<L: Lease>(game: &mut L, k: u16, spike: u16) {
    let left = MiniKraid::of(game, k).spike_timer_at(spike);
    if left != 0 {
        *MiniKraidEdit::of(game, k).spike_timer_at(spike) = left.wrapping_sub(1);
    } else {
        *MiniKraidEdit::of(game, k).spike_timer_at(spike) =
            ((game.current_random() as i32 & 0x3f) + 16) as u16;
        *MiniKraidEdit::of(game, k).last_spike() = spike;
        let mut spikes = projectile::SPIKES_LEFT;
        let facing: i16 = MiniKraid::of(game, k).facing() as i16;
        if facing >= 0 {
            spikes = projectile::SPIKES_RIGHT;
        }
        game.spawn_actor_eproj(facing as u16, k, spikes);
        if u16::from(game.current_outside_screen()) == 0 {
            crate::effects::SoundEffects::queue_sound(game, 1, SPIKE_SOUND, 6);
        }
    }
}

/// Spits two rocks: velocity pairs from row `row` (a byte offset into
/// [`SPIT_VELOCITIES`]), `offset` pixels from the mouth.
fn spit<L: Lease>(game: &mut L, row: u16, offset: u16) {
    let k: u16 = game.current_enemy();
    *MiniKraidEdit::of(game, k).spit_x_offset() = offset;
    let index: i32 = row as i32 >> 1;
    *MiniKraidEdit::of(game, k).spit_x_velocity() = SPIT_VELOCITIES[index as usize];
    let y_velocity: u16 = SPIT_VELOCITIES[(index + 1) as usize];
    *MiniKraidEdit::of(game, k).spit_y_velocity() = y_velocity;
    game.spawn_actor_eproj(y_velocity, game.current_enemy(), projectile::SPIT);
    *MiniKraidEdit::of(game, k).spit_x_velocity() = SPIT_VELOCITIES[(index + 2) as usize];
    let y_velocity: u16 = SPIT_VELOCITIES[(index + 3) as usize];
    *MiniKraidEdit::of(game, k).spit_y_velocity() = y_velocity;
    game.spawn_actor_eproj(y_velocity, k, projectile::SPIT);
}

/// Instruction: one step. Turns around when the walk runs out or a wall
/// blocks it, then faces Samus.
pub fn step<L: Lease>(game: &mut L) {
    let k = game.current_enemy();
    if MiniKraid::of(game, k).cycles_until_spit() != 0 {
        *MiniKraidEdit::of(game, k).cycles_until_spit() =
            MiniKraid::of(game, k).cycles_until_spit().wrapping_sub(1);
    }
    let steps = MiniKraid::of(game, k).steps_until_turn();
    *MiniKraidEdit::of(game, k).steps_until_turn() =
        MiniKraid::of(game, k).steps_until_turn().wrapping_sub(1);
    let turn_around: bool = if steps == 1 {
        *MiniKraidEdit::of(game, k).steps_until_turn() =
            ((game.current_random() as i32 & 3) + 7) as u16;
        true
    } else {
        let actor = game.current_enemy();
        let amount = (MiniKraid::of(game, k).walk_speed() as i16 as i32) << 16;
        game.move_horizontal(actor, amount, Slopes::Ignore)
    };
    if turn_around {
        *MiniKraidEdit::of(game, k).walk_speed() =
            -(MiniKraid::of(game, k).walk_speed() as i32) as u16;
    }
    face_samus(game, k);
}

/// Instruction: the next walk animation for the way Mini Kraid faces and walks,
/// or a spit once enough walk cycles have passed.
pub fn choose_animation<L: Lease>(game: &mut L) -> EnemyScript {
    let k = game.current_enemy();
    if MiniKraid::of(game, k).cycles_until_spit() != 0 {
        if (MiniKraid::of(game, k).facing() & 0x8000) != 0 {
            if (MiniKraid::of(game, game.current_enemy()).walk_speed() & 0x8000) == 0 {
                return EnemyScript::jump(animation::WALK_BACKWARD_LEFT);
            }
            EnemyScript::jump(animation::WALK_FORWARD_LEFT)
        } else {
            if (MiniKraid::of(game, k).walk_speed() & 0x8000) != 0 {
                return EnemyScript::jump(animation::WALK_BACKWARD_RIGHT);
            }
            EnemyScript::jump(animation::WALK_FORWARD_RIGHT)
        }
    } else {
        *MiniKraidEdit::of(game, k).cycles_until_spit() =
            ((game.current_random() as i32 & 3) + 3) as u16;
        if (MiniKraid::of(game, k).facing() & 0x8000) != 0 {
            return EnemyScript::jump(animation::SPIT_LEFT);
        }
        EnemyScript::jump(animation::SPIT_RIGHT)
    }
}

/// Instruction: roar, when on screen.
pub fn roar<L: Lease>(game: &mut L) {
    if u16::from(game.current_outside_screen()) == 0 {
        crate::effects::SoundEffects::queue_sound(game, 1, ROAR_SOUND, 6);
    }
}

/// Instruction: spit left.
pub fn spit_left<L: Lease>(game: &mut L) {
    spit(game, 0, FACING_LEFT);
}

/// Instruction: spit right.
pub fn spit_right<L: Lease>(game: &mut L) {
    spit(game, 8, FACING_RIGHT);
}

/// Where a death's drops start.
fn mark_drop_origin<L: Lease>(game: &mut L) {
    let k = game.current_enemy();
    let x = MiniKraid::of(game, k).actor().x.whole;
    game.session_mut().enemy_projectiles.drop_origin.x = x;
    game.words_stored();
    let y = MiniKraid::of(game, k).actor().y.whole;
    game.session_mut().enemy_projectiles.drop_origin.y = y;
    game.words_stored();
}

/// Touched by Samus.
pub fn touch<L: Lease>(game: &mut L) {
    mark_drop_origin(game);
    game.contact_reaction(Death::LeavePending);
    die_when_out_of_health(game);
}

/// Hit by a shot.
pub fn shot<L: Lease>(game: &mut L) {
    mark_drop_origin(game);
    game.shot_reaction(Shot::ReactionOnly);
    die_when_out_of_health(game);
}

/// Dies with its drop once out of health.
fn die_when_out_of_health<L: Lease>(game: &mut L) {
    if MiniKraid::of(game, game.current_enemy()).actor().health == 0 {
        crate::actors::boss_lease::steps::normal_death(game, DEATH_ANIMATION);
        let index = game.current_enemy();
        game.owned_step(|game| {
            crate::enemy_projectiles::drops::scatter(DropSource::MiniKraid, index, game);
        });
    }
}
