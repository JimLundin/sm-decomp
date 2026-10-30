//! Ripper patrols share velocity selection, collision reversal and animation.
#![forbid(unsafe_code)]
use crate::game::actors::{api, combat};
use crate::game::enemy_speeds::SpeedTable;
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use sm_game::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};

pub(crate) enum Kind {
    Ripper,
    JetPowered,
    Gripper,
}

impl Kind {
    fn instructions(&self) -> [u16; 2] {
        match self {
            Self::Ripper => [0xe48b, 0xe477],
            Self::JetPowered => [0xe2e0, 0xe2f4],
            Self::Gripper => [0xe19b, 0xe1af],
        }
    }
}

fn set_velocity(h: &mut Runtime<'_>, key: Key, speed: u16, forward: bool) {
    let index = (speed >> 1) as isize + if forward { 0 } else { 2 };
    // Publish the whole word before reading the fraction, including failure.
    for (offset, variable) in [(0, 3), (1, 2)] {
        let value = h.enemy_speed(SpeedTable::Linear, index + offset);
        h.edit_actor(key, Native, |actor| actor.variables[variable] = value);
    }
}

pub(crate) fn initialize(kind: Kind, game: &mut Game) {
    let key = Key::from_legacy(game.cur_enemy_index());
    let mut runtime = Runtime::new(game);
    if matches!(kind, Kind::Gripper) {
        let speed = u16::from(runtime.resolve_actor(key, Native).instruction.to_ne_bytes()[0]) * 8;
        runtime.edit_actor(key, Native, |actor| actor.variables[4] = speed);
        let forward = runtime.resolve_actor(key, Native).instruction & 0xfeff != 0;
        set_velocity(&mut runtime, key, speed, forward);
        let forward = runtime.resolve_actor(key, Native).variables[3] & 0x8000 == 0;
        runtime.edit_actor(key, Native, |actor| {
            actor.instruction = kind.instructions()[usize::from(forward)];
            actor.variables[0] = actor.parameters[0];
            actor.variables[1] = actor.parameters[1];
        });
        return;
    }
    let forward = runtime.resolve_actor(key, Native).parameters[1] != 0;
    let instruction = kind.instructions()[usize::from(forward)];
    runtime.edit_actor(key, Native, |actor| actor.instruction = instruction);
    let speed = runtime.resolve_actor(key, Native).parameters[0].wrapping_mul(8);
    runtime.edit_actor(key, Native, |actor| actor.variables[4] = speed);
    let forward = runtime.resolve_actor(key, Native).parameters[1] != 0;
    set_velocity(&mut runtime, key, speed, forward);
}

pub(crate) fn advance(kind: Kind, game: &mut Game) {
    let key = Key::from_legacy(game.cur_enemy_index());
    let mut runtime = Runtime::new(game);
    let velocity = {
        let actor = runtime.resolve_actor(key, Native);
        ((u32::from(actor.variables[3]) << 16) | u32::from(actor.variables[2])) as i32
    };
    let blocked = api::move_horizontal(&mut runtime, key, velocity, api::SlopePolicy::Ignore)
        == api::MovementResult::Blocked;
    if !blocked {
        if !matches!(kind, Kind::Gripper) {
            return;
        }
        // Bounds use the live current index; reversal retains the original actor.
        let current = Key::from_legacy(runtime.word(0xe54));
        let actor = runtime.resolve_actor(current, Native);
        let left = actor.variables[3] & 0x8000 != 0;
        let bound = actor.variables[usize::from(!left)];
        if ((actor.x.whole as i16).wrapping_sub(bound as i16) < 0) != left {
            return;
        }
    }
    // Movement may change the record. Retain its key, then reread its fields.
    let (speed, forward) = {
        let actor = runtime.resolve_actor(key, Native);
        (actor.variables[4], actor.variables[3] & 0x8000 != 0)
    };
    set_velocity(&mut runtime, key, speed, forward);
    runtime.edit_actor(key, Native, |actor| {
        actor.restart_animation(kind.instructions()[usize::from(forward)]);
    });
}

pub(crate) fn jet_shot(game: &mut Game) {
    combat::shot_reaction(&mut Runtime::new(game), api::ShotHandling::Finalize);
    // The reaction may select another actor; acquire the live key afterwards.
    let key = Key::from_legacy(game.cur_enemy_index());
    let mut runtime = Runtime::new(game);
    if runtime.resolve_actor(key, Native).frozen != 0 {
        let spritemap = if runtime.resolve_actor(key, Native).variables[3] & 0x8000 == 0 {
            (-7093i16) as u16
        } else {
            (-7105i16) as u16
        };
        runtime.edit_actor(key, Native, |actor| actor.spritemap = spritemap);
    }
}
