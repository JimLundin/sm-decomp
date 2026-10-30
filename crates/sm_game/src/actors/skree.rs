//! The skree and the metaree, the ceiling divers that drop on Samus and burst
//! into particles on the ground (bank $A3:88F0-$A3:8B65, $A3:C63E-$A3:C82F).
//! Was `diving_enemies`.
use crate::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key, SpawnAccess};
use crate::actors::api::{self, DeathKind, MovementResult, ShotHandling, VerticalProbeEdge};
use crate::actors::combat;
use crate::actors::host::{ActorLease, LeaseActors, abs16};
use crate::enemy_projectiles::host::{Effects as _, ImportedAliases as _, ShotMember};
use crate::enemy_projectiles::lifecycle::spawn_from_enemy;
use crate::enemy_script::EnemyScript;
use crate::session_access::SessionAccess as _;

/// Which diver.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Metaree ($D67F), which dives at a speed set by Samus's height.
    Metaree,
    /// Skree ($DB7F).
    Skree,
}

/// The divers' actor words, by `variables` lane; the metaree's Y velocity is
/// the pre-instruction word.
mod lane {
    pub const BURROW_TIMER: usize = 0;
    pub const FUNCTION: usize = 1;
    pub const NEW_INSTRUCTION_LIST_INDEX: usize = 2;
    pub const INSTRUCTION_LIST_INDEX: usize = 3;
    pub const ATTACK_READY: usize = 4;
}

/// A diver's functions, instruction lists and particles.
struct Profile {
    /// Instruction list - idling.
    idling: u16,
    /// The functions: idling, prepare to launch attack, launched attack, burrowing.
    functions: [u16; 4],
    /// The instruction list pointers.
    instruction_lists: u32,
    /// Samus is near within this many px columns.
    range: u16,
    /// The X step towards Samus while diving.
    pursuit: u16,
    /// The particle projectiles: down-right, up-right, down-left, up-left.
    particles: [u16; 4],
}
impl Kind {
    fn profile(self) -> Profile {
        match self {
            Self::Metaree => Profile {
                idling: 0x8910,
                functions: [0x8987, 0x89d4, 0x89f3, 0x8a5c],
                instruction_lists: 0xa3_894e,
                range: 0x48,
                pursuit: 2,
                particles: [0x8bfa, 0x8c08, 0x8c16, 0x8c24],
            },
            Self::Skree => Profile {
                idling: 0xc65e,
                functions: [0xc6d5, 0xc6f7, 0xc716, 0xc77f],
                instruction_lists: 0xa3_c69c,
                range: 0x30,
                pursuit: 1,
                particles: [0x8bc2, 0x8bd0, 0x8bde, 0x8bec],
            },
        }
    }
    fn speed(self, runtime: &impl ActorLease, key: Key) -> u16 {
        match self {
            Self::Metaree => runtime.resolve_actor(key, Native).pre_instruction,
            Self::Skree => SKREE_SPEED,
        }
    }
}

const IDLING: usize = 0;
const PREPARING: usize = 1;
const LAUNCHED: usize = 2;
const BURROWING: usize = 3;
const SKREE_SPEED: u16 = 6;
/// Determine metaree Y velocity ($A3:89AC): one more px a frame per 24 px
/// Samus is below, from 4.
const METAREE_ROWS: u16 = 24;
const METAREE_BASE_SPEED: u16 = 4;
/// The burrow lasts 15h frames; the particles burst 8 frames before the end.
const BURROW_FRAMES: u16 = 21;
const BURST_FRAME: u16 = 8;
/// Diving sets the properties' low bits (3); burrowed, the enemy is deleted
/// and leaves palette 5.
const DIVING_PROPERTIES: u16 = 3;
const DELETE: u16 = 0x200;
const PALETTE_5: u16 = 0xa00;
/// The skree's launch (5Bh) and hitting the ground (5Ch), library 2.
const LAUNCH_SOUND: u16 = 0x5b;
const GROUND_SOUND: u16 = 0x5c;
/// Projectile kind bits 8-11 of 2: a super missile, which kills with a big
/// explosion.
const KIND_BITS: u16 = 0xf00;
const SUPER_MISSILE: u16 = 0x200;

/// Initialisation AI - metaree ($A3:8960) or skree ($A3:C6AE).
pub fn initialize<H: LeaseActors + ?Sized>(kind: Kind, game: &mut H) {
    let runtime = &mut game.lease();
    let key = combat::current(runtime);
    let profile = kind.profile();
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[lane::NEW_INSTRUCTION_LIST_INDEX..=lane::ATTACK_READY].fill(0);
        actor.instruction = profile.idling;
        actor.variables[lane::FUNCTION] = profile.functions[IDLING];
    });
}
/// Set the instruction list for the new instruction list index, if it changed.
fn set_instruction_list(runtime: &mut impl ActorLease, profile: &Profile) {
    let key = combat::current(runtime);
    let index = runtime.resolve_actor(key, Native).variables[lane::NEW_INSTRUCTION_LIST_INDEX];
    if index != runtime.resolve_actor(key, Native).variables[lane::INSTRUCTION_LIST_INDEX] {
        runtime.edit_actor(key, Native, |actor| {
            actor.variables[lane::INSTRUCTION_LIST_INDEX] = index
        });
        let instruction = runtime
            .leased_image()
            .expect("ROM must be loaded")
            .table_word(
                sm_data::cartridge::lorom_offset(profile.instruction_lists) as usize,
                index as isize,
            );
        runtime.edit_actor(key, Native, |actor| actor.restart_animation(instruction));
    }
}
/// Spawn the four particles, each chained to the last's slot or not.
fn burst(runtime: &mut impl ActorLease, profile: &Profile, mut parameter: u16, chain: bool) {
    let enemy = combat::current(runtime).byte_index();
    for definition in profile.particles {
        let result = spawn_from_enemy(runtime, parameter, enemy, definition);
        parameter = if chain { result } else { 0 };
    }
}
/// Main AI - metaree ($A3:8979) or skree ($A3:C6C7): run its function. Was
/// `advance`.
pub fn main_ai<H: LeaseActors + ?Sized>(kind: Kind, game: &mut H) {
    let runtime = &mut game.lease();
    let key = combat::current(runtime);
    let profile = kind.profile();
    let function = runtime.resolve_actor(key, Native).variables[lane::FUNCTION];
    if function == profile.functions[IDLING] {
        // Idling: wait for Samus to pass below.
        let distance = runtime
            .resolve_actor(key, Native)
            .x
            .whole
            .wrapping_sub(runtime.session().player.x);
        if abs16(distance) < profile.range {
            if kind == Kind::Metaree {
                let live = combat::current(runtime);
                let distance = runtime
                    .session()
                    .player
                    .y
                    .wrapping_sub(runtime.resolve_actor(live, Native).y.whole);
                runtime.edit_actor(live, Native, |actor| {
                    actor.pre_instruction =
                        u16::from((distance / METAREE_ROWS) as u8) + METAREE_BASE_SPEED;
                });
            }
            runtime.edit_actor(key, Native, |actor| {
                actor.variables[lane::NEW_INSTRUCTION_LIST_INDEX] =
                    actor.variables[lane::NEW_INSTRUCTION_LIST_INDEX].wrapping_add(1);
            });
            set_instruction_list(runtime, &profile);
            runtime.edit_actor(key, Native, |actor| {
                actor.variables[lane::FUNCTION] = profile.functions[PREPARING]
            });
        }
    } else if function == profile.functions[PREPARING] {
        // Prepare to launch attack: launch once the script is ready.
        if runtime.resolve_actor(key, Native).variables[lane::ATTACK_READY] != 0 {
            runtime.edit_actor(key, Native, |actor| {
                actor.variables[lane::ATTACK_READY] = 0;
                actor.variables[lane::NEW_INSTRUCTION_LIST_INDEX] =
                    actor.variables[lane::NEW_INSTRUCTION_LIST_INDEX].wrapping_add(1);
            });
            set_instruction_list(runtime, &profile);
            runtime.edit_actor(key, Native, |actor| {
                actor.variables[lane::FUNCTION] = profile.functions[LAUNCHED]
            });
            runtime.queue_sound(2, 6, LAUNCH_SOUND);
        }
    } else if function == profile.functions[LAUNCHED] {
        // Launched attack: dive towards Samus until the ground.
        runtime.edit_actor(key, Native, |actor| {
            actor.variables[lane::BURROW_TIMER] = BURROW_FRAMES;
            actor.properties |= DIVING_PROPERTIES;
        });
        let live = combat::current(runtime);
        let amount = i32::from(kind.speed(runtime, key) as i16) << 16;
        if api::probe_vertical_offset(runtime, live, VerticalProbeEdge::Positive, amount)
            == MovementResult::Blocked
        {
            runtime.edit_actor(key, Native, |actor| {
                actor.instruction_delay = 1;
                actor.timer = 0;
                actor.variables[lane::FUNCTION] = profile.functions[BURROWING];
            });
            runtime.queue_sound(2, 6, GROUND_SOUND);
        } else {
            let speed = kind.speed(runtime, key);
            runtime.edit_actor(key, Native, |actor| {
                actor.y.whole = actor.y.whole.wrapping_add(speed)
            });
            let player_x = runtime.session().player.x;
            runtime.edit_actor(key, Native, |actor| {
                actor.x.whole = if (actor.x.whole as i16).wrapping_sub(player_x as i16) >= 0 {
                    actor.x.whole.wrapping_sub(profile.pursuit)
                } else {
                    actor.x.whole.wrapping_add(profile.pursuit)
                };
            });
        }
    } else if function == profile.functions[BURROWING] {
        // Burrowing: sink, burst into particles, then delete.
        let previous = runtime.edit_actor(key, Native, |actor| {
            let previous = actor.variables[lane::BURROW_TIMER];
            actor.variables[lane::BURROW_TIMER] = previous.wrapping_sub(1);
            previous
        });
        if previous == 1 {
            let live = combat::current(runtime);
            let graphics = runtime.resolve_actor(key, Native).tile
                | runtime.resolve_actor(key, Native).palette;
            runtime.edit_spawn(live, Native, |spawn| spawn.tile = graphics);
            runtime.edit_actor(key, Native, |actor| {
                actor.palette = PALETTE_5;
                actor.tile = 0;
                actor.properties |= DELETE;
            });
        } else {
            if runtime.resolve_actor(key, Native).variables[lane::BURROW_TIMER] == BURST_FRAME {
                burst(runtime, &profile, BURST_FRAME, true);
            }
            runtime.edit_actor(key, Native, |actor| {
                actor.y.whole = actor.y.whole.wrapping_add(1)
            });
        }
    } else {
        runtime.unreachable();
    }
}
/// Instruction - set attack ready flag ($A3:8956, $A3:C6A4). Was `ready`.
pub fn set_attack_ready<H: LeaseActors + ?Sized>(script: EnemyScript, game: &mut H) -> EnemyScript {
    let runtime = &mut game.lease();
    let key = combat::current(runtime);
    runtime.edit_actor(key, Native, |actor| actor.variables[lane::ATTACK_READY] = 1);
    script
}
/// Enemy shot - metaree ($A3:8B0F) or skree ($A3:C7F5): burst on death.
pub fn shot<H: LeaseActors + ?Sized>(kind: Kind, game: &mut H) {
    let runtime = &mut game.lease();
    let profile = kind.profile();
    if kind == Kind::Metaree {
        let key = combat::current(runtime);
        let tile = runtime.resolve_actor(key, Native).tile;
        let palette = runtime.resolve_actor(key, Native).palette;
        combat::shot_reaction(runtime, ShotHandling::Finalize);
        if runtime
            .resolve_actor(combat::current(runtime), Native)
            .health
            == 0
        {
            runtime.edit_actor(key, Native, |actor| {
                actor.tile = tile;
                actor.palette = palette;
            });
            let parameter = runtime.resolve_actor(key, Native).variables[lane::BURROW_TIMER];
            burst(runtime, &profile, parameter, false);
            runtime.edit_actor(key, Native, |actor| {
                actor.tile = 0;
                actor.palette = 0;
            });
        }
    } else {
        combat::shot_reaction(runtime, ShotHandling::ReactionOnly);
        let key = combat::current(runtime);
        if runtime.resolve_actor(key, Native).health == 0 {
            let parameter = runtime.resolve_actor(key, Native).variables[lane::BURROW_TIMER];
            burst(runtime, &profile, parameter, true);
            let index = runtime.session().room.collision_detection_index;
            let projectile = runtime.shot_word(ShotMember::Kind, isize::try_from(index).unwrap());
            let animation = if projectile & KIND_BITS == SUPER_MISSILE {
                2
            } else {
                0
            };
            combat::die(runtime, DeathKind::Normal { animation });
        }
    }
}
