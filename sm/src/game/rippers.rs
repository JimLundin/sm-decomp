//! Ripper patrols share velocity selection, collision reversal and animation.
#![forbid(unsafe_code)]
use crate::game::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use crate::game::actors::{api, combat};
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;

pub(crate) enum Kind {
    Ripper,
    JetPowered,
}

impl Kind {
    fn instructions(self) -> [u16; 2] {
        match self {
            Self::Ripper => [0xe48b, 0xe477],
            Self::JetPowered => [0xe2e0, 0xe2f4],
        }
    }
}

fn set_velocity(h: &mut Runtime<'_>, key: Key, speed: u16, forward: bool) {
    let index = (speed >> 1) as isize + if forward { 0 } else { 2 };
    // Publish the whole word before reading the fraction, including failure.
    for (offset, variable) in [(0, 3), (1, 2)] {
        let value = h
            .loaded_image()
            .expect("ROM must be loaded")
            .table_word(crate::rom::lorom_offset(0xa28187) as usize, index + offset);
        h.edit_actor(key, Native, |actor| actor.variables[variable] = value);
    }
}

pub(crate) fn initialize(kind: Kind, game: &mut Game) {
    let key = Key::from_legacy(game.cur_enemy_index());
    let mut runtime = Runtime::new(game);
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
    if api::move_horizontal(&mut runtime, key, velocity, api::SlopePolicy::Ignore)
        != api::MovementResult::Blocked
    {
        return;
    }
    // Movement may change the record. Retain its key, then reread its fields.
    let (speed, forward) = {
        let actor = runtime.resolve_actor(key, Native);
        (actor.variables[4], actor.variables[3] & 0x8000 != 0)
    };
    set_velocity(&mut runtime, key, speed, forward);
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction = kind.instructions()[usize::from(forward)];
        actor.instruction_delay = 1;
        actor.timer = 0;
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
