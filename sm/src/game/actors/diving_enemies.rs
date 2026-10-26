//! Metalee and Skree share their watch, dive, landing and debris sequence.
#![forbid(unsafe_code)]
use super::access::{AccessMode::Native, ActorAccess, ActorKey as Key, SpawnAccess};
use super::{api, combat};
use crate::game::enemy_projectiles::{
    host::{Effects, ImportedAliases, ShotMember},
    lifecycle::spawn_from_enemy,
};
use crate::game::enemy_script::EnemyScript;
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    Metalee,
    Skree,
}
struct Profile {
    initial: u16,
    phases: [u16; 4],
    animations: u32,
    range: u16,
    pursuit: u16,
    debris: [u16; 4],
}
impl Kind {
    fn profile(self) -> Profile {
        match self {
            Self::Metalee => Profile {
                initial: 0x8910,
                phases: [0x8987, 0x89d4, 0x89f3, 0x8a5c],
                animations: 0xa3894e,
                range: 0x48,
                pursuit: 2,
                debris: [0x8bfa, 0x8c08, 0x8c16, 0x8c24],
            },
            Self::Skree => Profile {
                initial: 0xc65e,
                phases: [0xc6d5, 0xc6f7, 0xc716, 0xc77f],
                animations: 0xa3c69c,
                range: 0x30,
                pursuit: 1,
                debris: [0x8bc2, 0x8bd0, 0x8bde, 0x8bec],
            },
        }
    }
    fn speed(self, runtime: &Runtime<'_>, key: Key) -> u16 {
        match self {
            Self::Metalee => runtime.resolve_actor(key, Native).pre_instruction,
            Self::Skree => 6,
        }
    }
}
pub(crate) fn initialize(kind: Kind, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = combat::current(&runtime);
    let profile = kind.profile();
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[2..5].fill(0);
        actor.instruction = profile.initial;
        actor.variables[1] = profile.phases[0];
    });
}
fn animate(runtime: &mut Runtime<'_>, profile: &Profile) {
    let key = combat::current(runtime);
    let frame = runtime.resolve_actor(key, Native).variables[2];
    if frame != runtime.resolve_actor(key, Native).variables[3] {
        runtime.edit_actor(key, Native, |actor| actor.variables[3] = frame);
        let instruction = runtime
            .loaded_image()
            .expect("ROM must be loaded")
            .table_word(
                crate::rtl::lorom_offset(profile.animations) as usize,
                frame as isize,
            );
        runtime.edit_actor(key, Native, |actor| {
            actor.instruction = instruction;
            actor.instruction_delay = 1;
            actor.timer = 0;
        });
    }
}
fn burst(runtime: &mut Runtime<'_>, profile: &Profile, mut parameter: u16, chain: bool) {
    let enemy = combat::current(runtime).byte_index();
    for definition in profile.debris {
        let result = spawn_from_enemy(runtime, parameter, enemy, definition);
        parameter = if chain { result } else { 0 };
    }
}
pub(crate) fn advance(kind: Kind, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = combat::current(&runtime);
    let profile = kind.profile();
    let phase = runtime.resolve_actor(key, Native).variables[1];
    if phase == profile.phases[0] {
        let distance = runtime
            .resolve_actor(key, Native)
            .x
            .whole
            .wrapping_sub(runtime.session().player.x);
        if crate::rtl::abs16(distance) < profile.range {
            if kind == Kind::Metalee {
                let live = combat::current(&runtime);
                let distance = runtime
                    .session()
                    .player
                    .y
                    .wrapping_sub(runtime.resolve_actor(live, Native).y.whole);
                runtime.edit_actor(live, Native, |actor| {
                    actor.pre_instruction = u16::from((distance / 24) as u8) + 4;
                });
            }
            runtime.edit_actor(key, Native, |actor| {
                actor.variables[2] = actor.variables[2].wrapping_add(1);
            });
            animate(&mut runtime, &profile);
            runtime.edit_actor(key, Native, |actor| actor.variables[1] = profile.phases[1]);
        }
    } else if phase == profile.phases[1] {
        if runtime.resolve_actor(key, Native).variables[4] != 0 {
            runtime.edit_actor(key, Native, |actor| {
                actor.variables[4] = 0;
                actor.variables[2] = actor.variables[2].wrapping_add(1);
            });
            animate(&mut runtime, &profile);
            runtime.edit_actor(key, Native, |actor| actor.variables[1] = profile.phases[2]);
            runtime.queue_sound(2, 6, 0x5b);
        }
    } else if phase == profile.phases[2] {
        runtime.edit_actor(key, Native, |actor| {
            actor.variables[0] = 21;
            actor.properties |= 3;
        });
        let live = combat::current(&runtime);
        let amount = i32::from(kind.speed(&runtime, key) as i16) << 16;
        if api::probe_vertical_offset(&mut runtime, live, api::VerticalProbeEdge::Positive, amount)
            == api::MovementResult::Blocked
        {
            runtime.edit_actor(key, Native, |actor| {
                actor.instruction_delay = 1;
                actor.timer = 0;
                actor.variables[1] = profile.phases[3];
            });
            runtime.queue_sound(2, 6, 0x5c);
        } else {
            let speed = kind.speed(&runtime, key);
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
    } else if phase == profile.phases[3] {
        let previous = runtime.edit_actor(key, Native, |actor| {
            let previous = actor.variables[0];
            actor.variables[0] = previous.wrapping_sub(1);
            previous
        });
        if previous == 1 {
            let live = combat::current(&runtime);
            let graphics = runtime.resolve_actor(key, Native).tile
                | runtime.resolve_actor(key, Native).palette;
            runtime.edit_spawn(live, Native, |spawn| spawn.tile = graphics);
            runtime.edit_actor(key, Native, |actor| {
                actor.palette = 2560;
                actor.tile = 0;
                actor.properties |= 0x200;
            });
        } else {
            if runtime.resolve_actor(key, Native).variables[0] == 8 {
                burst(&mut runtime, &profile, 8, true);
            }
            runtime.edit_actor(key, Native, |actor| {
                actor.y.whole = actor.y.whole.wrapping_add(1)
            });
        }
    } else {
        runtime.unreachable();
    }
}
pub(crate) fn ready(script: EnemyScript, game: &mut Game) -> EnemyScript {
    let mut runtime = Runtime::new(game);
    let key = combat::current(&runtime);
    runtime.edit_actor(key, Native, |actor| actor.variables[4] = 1);
    script
}
pub(crate) fn shot(kind: Kind, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let profile = kind.profile();
    if kind == Kind::Metalee {
        let key = combat::current(&runtime);
        let tile = runtime.resolve_actor(key, Native).tile;
        let palette = runtime.resolve_actor(key, Native).palette;
        combat::shot_reaction(&mut runtime, api::ShotHandling::Finalize);
        if runtime
            .resolve_actor(combat::current(&runtime), Native)
            .health
            == 0
        {
            runtime.edit_actor(key, Native, |actor| {
                actor.tile = tile;
                actor.palette = palette;
            });
            let parameter = runtime.resolve_actor(key, Native).variables[0];
            burst(&mut runtime, &profile, parameter, false);
            runtime.edit_actor(key, Native, |actor| {
                actor.tile = 0;
                actor.palette = 0;
            });
        }
    } else {
        combat::shot_reaction(&mut runtime, api::ShotHandling::ReactionOnly);
        let key = combat::current(&runtime);
        if runtime.resolve_actor(key, Native).health == 0 {
            let parameter = runtime.resolve_actor(key, Native).variables[0];
            burst(&mut runtime, &profile, parameter, true);
            let index = runtime.session().room.collision_detection_index;
            let kind = runtime.shot_word(ShotMember::Kind, isize::try_from(index).unwrap());
            let animation = if kind & 0xf00 == 0x200 { 2 } else { 0 };
            combat::die(&mut runtime, api::DeathKind::Normal { animation });
        }
    }
}
