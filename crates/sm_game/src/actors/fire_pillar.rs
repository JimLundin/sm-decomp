//! The fire pillar: the graphics part waits, erupts on its script, and the hitbox part in the
//! next slot follows the column's height (bank $A6:8CFB-$A6:946F). Was `fire_geysers`.
#![forbid(unsafe_code)]
use crate::actors::access::{AccessMode::Native, ActorKey as Key};
use crate::actors::host::ActorLease;
use crate::enemy_script::EnemyScript;

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

/// The graphics part and the hitbox part in the slot after it.
fn parts(runtime: &impl ActorLease) -> (Key, Key) {
    let index = runtime.session().actors.controls.selected;
    // The selected index stays below the slot table's end, so the next slot's
    // index fits in u16; past the table the access projects the same bytes.
    (Key::from_legacy(index), Key::from_legacy(index + 64))
}

/// Initialisation AI - enemy $E07F (fire pillar) ($A6:8FFC): parameter 2 zero for the graphics.
pub fn initialize(runtime: &mut impl crate::actors::species::TableLease) {
    let (graphics, _) = parts(&*runtime);
    runtime.edit_actor(graphics, Native, |actor| {
        actor.instruction = HITBOX_PART;
        if actor.parameters[1] == 0 {
            (actor.instruction, actor.variables[0]) = (GRAPHICS_PART, WAIT);
            (actor.variables[3], actor.radius.x) = (actor.y.whole, 0);
        }
    });
}

/// Main AI - enemy $E07F (fire pillar) ($A6:9023): wait (parameter 1 frames), then erupt.
pub fn main_ai(runtime: &mut impl crate::actors::species::TableLease) {
    let (graphics, column) = parts(&*runtime);
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
                runtime.edit_actor(column, Native, |actor| actor.properties &= !INTANGIBLE);
            }
        }
        ERUPT => runtime.edit_actor(graphics, Native, |actor| {
            if actor.variables[2] != 0 {
                (actor.variables[1], actor.variables[0]) = (actor.parameters[0], WAIT);
                actor.properties |= INVISIBLE;
            }
        }),
        _ => crate::diagnostics::Fatal::unreachable(runtime),
    }
}

/// Fire pillar instructions: queue the sound ($A6:8DAF), activity frames 0-21 setting the
/// column's height ($A6:8E13-$A6:8FBD) and the finish ($A6:8FD1).
pub fn instruction(
    ea: u32,
    script: EnemyScript,
    runtime: &mut impl crate::actors::species::TableLease,
) -> EnemyScript {
    let frame = match ea {
        0xa68daf => {
            crate::effects::SoundEffects::queue_sound(runtime, 1, 0x61, 6);
            return script;
        }
        0xa68fd1 => {
            finish(runtime);
            return script;
        }
        0xa68e13 => 0,
        0xa68e2d..=0xa68fbd if (ea - 0xa68e2d).is_multiple_of(20) => {
            (ea - 0xa68e2d) as usize / 20 + 1
        }
        _ => crate::diagnostics::Fatal::unreachable(runtime),
    };
    let (graphics, column) = parts(&*runtime);
    let base = runtime.resolve_actor(graphics, Native).variables[3];
    runtime.edit_actor(column, Native, |actor| {
        actor.y.whole = base.wrapping_sub(5 * (frame as u16 + 1));
        actor.radius.y = Y_RADII[frame];
        if frame == 0 {
            actor.radius.x = 8;
        }
    });
    script
}

/// The eruption ends: the column collapses and both parts hide.
fn finish(runtime: &mut impl crate::actors::species::TableLease) {
    let (graphics, column) = parts(&*runtime);
    runtime.edit_actor(graphics, Native, |actor| actor.variables[2] = 1);
    runtime.edit_actor(column, Native, |actor| {
        (actor.radius.x, actor.radius.y) = (0, 0)
    });
    runtime.edit_actor(graphics, Native, |actor| {
        actor.y.whole = actor.variables[3];
        actor.properties |= INVISIBLE;
    });
    runtime.edit_actor(column, Native, |actor| actor.properties |= INTANGIBLE);
}
