//! The fire pillar: the graphics part waits, erupts on its script, and the hitbox part in the
//! next slot follows the column's height (bank $A6:8CFB-$A6:946F). Was `fire_geysers`.
#![forbid(unsafe_code)]
use crate::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use crate::actors::host::{ActorLease, BossEffects, LeaseActors};
use crate::enemy_projectiles::host::Effects as _;
use crate::enemy_script::EnemyScript;
use crate::ram::words as W;

const INVISIBLE: u16 = 0x0100;
const INTANGIBLE: u16 = 0x0400;
const GRAPHICS_PART: u16 = 0x8d1b; // instruction lists
const HITBOX_PART: u16 = 0x8da9;
const WAIT: u16 = 0x902f; // functions, in lane 0
const ERUPT: u16 = 0x9062;
/// The column's Y radii by activity frame ($A6:8DE7); frame n raises it by 5 * (n + 1).
const Y_RADII: [u16; 22] = [
    0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
    0x18, 0x18, 0x14, 0x10, 0xc, 8,
];

/// The graphics part and the native slot of the hitbox part after it.
fn parts(runtime: &impl ActorLease) -> (Key, usize) {
    let index = runtime.word(W::CUR_ENEMY_INDEX);
    // The legacy child offset adds 64 after widening the selected u16 index.
    (Key::from_legacy(index), 0xf78 + usize::from(index) + 64)
}

/// Initialisation AI - enemy $E07F (fire pillar) ($A6:8FFC): parameter 2 zero for the graphics.
pub fn initialize<H: LeaseActors>(game: &mut H) {
    let mut runtime = game.lease();
    let (graphics, _) = parts(&runtime);
    runtime.edit_actor(graphics, Native, |actor| {
        actor.instruction = HITBOX_PART;
        if actor.parameters[1] == 0 {
            (actor.instruction, actor.variables[0]) = (GRAPHICS_PART, WAIT);
            (actor.variables[3], actor.radius.x) = (actor.y.whole, 0);
        }
    });
}

/// Main AI - enemy $E07F (fire pillar) ($A6:9023): wait (parameter 1 frames), then erupt.
pub fn main_ai<H: LeaseActors>(game: &mut H) {
    let mut runtime = game.lease();
    let (graphics, column) = parts(&runtime);
    let actor = runtime.resolve_actor(graphics, Native);
    let (active, function) = (actor.parameters[1] == 0, actor.variables[0]);
    if !active {
        return;
    }
    match function {
        WAIT => {
            let erupt = runtime.edit_actor(graphics, Native, |actor| {
                actor.variables[1] = actor.variables[1].wrapping_sub(1);
                let erupt = (actor.variables[1] & 0x8000) != 0;
                if erupt {
                    (actor.variables[0], actor.variables[2]) = (ERUPT, 0);
                    (actor.instruction_delay, actor.timer, actor.instruction) =
                        (1, 0, GRAPHICS_PART);
                    actor.properties &= !INVISIBLE;
                }
                erupt
            });
            if erupt {
                let properties = runtime.lease_native_word(column + 0x0e) & !INTANGIBLE;
                runtime.write_slice_native(column + 0x0e, properties);
            }
        }
        ERUPT => runtime.edit_actor(graphics, Native, |actor| {
            if actor.variables[2] != 0 {
                (actor.variables[1], actor.variables[0]) = (actor.parameters[0], WAIT);
                actor.properties |= INVISIBLE;
            }
        }),
        _ => runtime.unreachable(),
    }
}

/// Fire pillar instructions: queue the sound ($A6:8DAF), activity frames 0-21 setting the
/// column's height ($A6:8E13-$A6:8FBD) and the finish ($A6:8FD1).
pub fn instruction<H: LeaseActors + BossEffects>(
    ea: u32,
    script: EnemyScript,
    game: &mut H,
) -> EnemyScript {
    let frame = match ea {
        0xa68daf => {
            game.queue_sound(1, 0x61, 6);
            return script;
        }
        0xa68fd1 => {
            finish(game);
            return script;
        }
        0xa68e13 => 0,
        0xa68e2d..=0xa68fbd if (ea - 0xa68e2d).is_multiple_of(20) => {
            (ea - 0xa68e2d) as usize / 20 + 1
        }
        _ => {
            game.lease().unreachable();
            return EnemyScript::Stop;
        }
    };
    let mut runtime = game.lease();
    let (graphics, column) = parts(&runtime);
    let base = runtime.resolve_actor(graphics, Native).variables[3];
    runtime.write_slice_native(column + 0x06, base.wrapping_sub(5 * (frame as u16 + 1)));
    runtime.write_slice_native(column + 0x0c, Y_RADII[frame]);
    if frame == 0 {
        runtime.write_slice_native(column + 0x0a, 8);
    }
    script
}

/// The eruption ends: the column collapses and both parts hide.
fn finish<H: LeaseActors>(game: &mut H) {
    let mut runtime = game.lease();
    let (graphics, column) = parts(&runtime);
    runtime.edit_actor(graphics, Native, |actor| actor.variables[2] = 1);
    runtime.write_slice_native(column + 0x0a, 0);
    runtime.write_slice_native(column + 0x0c, 0);
    runtime.edit_actor(graphics, Native, |actor| {
        actor.y.whole = actor.variables[3];
        actor.properties |= INVISIBLE;
    });
    let properties = runtime.lease_native_word(column + 0x0e) | INTANGIBLE;
    runtime.write_slice_native(column + 0x0e, properties);
}
