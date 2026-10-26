//! Lower Norfair fire geysers: the parent waits, erupts on its script, and the
//! next slot's hitbox follows the column height. Actor words stay RAM-authoritative.
#![forbid(unsafe_code)]
use crate::game::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use crate::game::enemy_projectiles::host::Effects;
use crate::game::enemy_script::EnemyScript;
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;

const INVISIBLE: u16 = 0x0100;
const INTANGIBLE: u16 = 0x0400;
const ERUPTION_SCRIPT: u16 = 0x8d1b;
const STILL_SCRIPT: u16 = 0x8da9;
/// Routine words kept in variable A (bank A6 offsets).
const WAIT: u16 = 0x902f;
const ERUPT: u16 = 0x9062;
/// Column heights per eruption step; step n raises the column by 5 * (n + 1).
const HEIGHTS: [u16; 22] = [
    0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
    0x18, 0x18, 0x14, 0x10, 0xc, 8,
];

/// The geyser being run and its hitbox in the following slot.
fn parent_and_column(runtime: &Runtime<'_>) -> (Key, Key) {
    let index = runtime.word(0xe54);
    (
        Key::from_legacy(index),
        Key::from_legacy(index.wrapping_add(64)),
    )
}

pub(crate) fn initialize(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let (parent, _) = parent_and_column(&runtime);
    runtime.edit_actor(parent, Native, |actor| {
        actor.instruction = STILL_SCRIPT;
        if actor.parameters[1] == 0 {
            actor.instruction = ERUPTION_SCRIPT;
            actor.variables[0] = WAIT;
            actor.variables[3] = actor.y.whole;
            actor.radius.x = 0;
        }
    });
}

pub(crate) fn run(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let (parent, column) = parent_and_column(&runtime);
    let (active, routine) = {
        let actor = runtime.resolve_actor(parent, Native);
        (actor.parameters[1] == 0, actor.variables[0])
    };
    if !active {
        return;
    }
    match routine {
        WAIT => {
            let erupt = runtime.edit_actor(parent, Native, |actor| {
                actor.variables[1] = actor.variables[1].wrapping_sub(1);
                let erupt = actor.variables[1] & 0x8000 != 0;
                if erupt {
                    actor.variables[0] = ERUPT;
                    actor.variables[2] = 0;
                    actor.instruction_delay = 1;
                    actor.timer = 0;
                    actor.instruction = ERUPTION_SCRIPT;
                    actor.properties &= !INVISIBLE;
                }
                erupt
            });
            if erupt {
                runtime.edit_actor(column, Native, |actor| actor.properties &= !INTANGIBLE);
            }
        }
        ERUPT => runtime.edit_actor(parent, Native, |actor| {
            if actor.variables[2] != 0 {
                actor.variables[1] = actor.parameters[0];
                actor.properties |= INVISIBLE;
                actor.variables[0] = WAIT;
            }
        }),
        _ => runtime.unreachable(),
    }
}

/// Script steps 0xA68DAF (sound), 0xA68E13..=0xA68FBD (column height) and 0xA68FD1 (finish).
pub(crate) fn instruction(ea: u32, script: EnemyScript, game: &mut Game) -> EnemyScript {
    let step = match ea {
        0xa68daf => {
            crate::game::music::QueueSfx2_Max6(0x61, game);
            return script;
        }
        0xa68fd1 => {
            finish(game);
            return script;
        }
        0xa68e13 => 0,
        0xa68e2d..=0xa68fbd if (ea - 0xa68e2d) % 20 == 0 => (ea - 0xa68e2d) as usize / 20 + 1,
        _ => {
            Runtime::new(game).unreachable();
            return EnemyScript::Stop;
        }
    };
    let mut runtime = Runtime::new(game);
    let (parent, column) = parent_and_column(&runtime);
    let base = runtime.resolve_actor(parent, Native).variables[3];
    let y = base.wrapping_sub(5 * (step as u16 + 1));
    runtime.edit_actor(column, Native, |actor| {
        actor.y.whole = y;
        actor.radius.y = HEIGHTS[step];
        if step == 0 {
            actor.radius.x = 8;
        }
    });
    script
}

/// The eruption ends: the column collapses and both parts hide.
fn finish(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let (parent, column) = parent_and_column(&runtime);
    runtime.edit_actor(parent, Native, |actor| {
        actor.variables[2] = 1;
        actor.y.whole = actor.variables[3];
        actor.properties |= INVISIBLE;
    });
    runtime.edit_actor(column, Native, |actor| {
        actor.radius = Default::default();
        actor.properties |= INTANGIBLE;
    });
}
