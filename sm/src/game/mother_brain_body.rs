//! Mother Brain's body script steps: moving the BG2-drawn body and setting its pose.
//! The body lives in actor slot 0; its pose word stays RAM-authoritative.
#![forbid(unsafe_code)]
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use crate::ram_map::word;
use core::mem::offset_of;
use sm_game::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use sm_game::actors::mother_brain::MotherBrainBody;
use sm_game::enemy_script::EnemyScript;

const BODY: Key = Key::from_legacy(0);
const POSE: usize = 0xf78 + offset_of!(MotherBrainBody, pose);
const PHASE: usize = 0xf78 + offset_of!(MotherBrainBody, form);

/// Body poses written by the walking, crouching and beam scripts.
#[derive(Clone, Copy)]
enum Pose {
    Standing = 0,
    Walking = 1,
    CrouchedTransition = 2,
    Crouched = 3,
    DeathBeamMode = 4,
    LeaningDown = 6,
}

/// A script step that moves the body by `dx`, `dy` pixels.
/// `scroll_x` offsets the BG2 horizontal scroll from the body's position.
struct Step {
    dx: u16,
    dy: u16,
    scroll_x: u16,
    footstep: bool,
}

const fn step(dx: i16, dy: i16, scroll_x: i16, footstep: bool) -> Step {
    Step {
        dx: dx as u16,
        dy: dy as u16,
        scroll_x: scroll_x as u16,
        footstep,
    }
}

fn movement(ea: u32) -> Option<Step> {
    Some(match ea {
        11113910 => step(0, -10, 4, false),
        11113920 => step(0, -16, 4, false),
        11113930 => step(0, -12, -2, false),
        11113950 => step(0, 12, 4, false),
        11113960 => step(0, 16, -2, false),
        11113970 => step(0, 10, -2, false),
        11113980 | 11114224 => step(1, -2, 0, false),
        11113996 => step(2, 0, 0, false),
        11114012 => step(0, 1, 0, false),
        11114018 => step(3, 1, 0, true),
        11114040 => step(15, -2, 0, false),
        11114056 => step(6, -4, 0, false),
        11114072 => step(-2, 4, 0, false),
        11114088 | 11114110 => step(-1, 2, 0, true),
        11114132 => step(-2, 0, 0, false),
        11114148 => step(0, -1, 0, false),
        11114154 => step(-3, -1, 0, false),
        11114170 => step(-15, 2, 0, true),
        11114192 => step(-6, 4, 0, false),
        11114208 => step(2, -4, 0, false),
        _ => return None,
    })
}

fn pose(ea: u32) -> Option<Pose> {
    Some(match ea {
        11114240 => Pose::Standing,
        11114248 => Pose::Walking,
        11114256 => Pose::Crouched,
        11114264 => Pose::CrouchedTransition,
        11114272 => Pose::DeathBeamMode,
        11114280 => Pose::LeaningDown,
        _ => return None,
    })
}

fn footstep(game: &mut Game) {
    word! {game; EARTHQUAKE_TYPE = 1; EARTHQUAKE_TIMER = 4; }
    if Runtime::new(game).native_word(PHASE) == 3 {
        // The original routine stores into ROM here; the port only reports it.
        print!("Write to rom!\n");
    }
}

fn move_body(step: Step, game: &mut Game) {
    if step.footstep {
        footstep(game);
    }
    let x = Runtime::new(game).edit_actor(BODY, Native, |body| {
        body.x.whole = body.x.whole.wrapping_add(step.dx);
        body.y.whole = body.y.whole.wrapping_add(step.dy);
        body.x.whole
    });
    word!(game.REG_BG2VOFS -= step.dy);
    word!(game.REG_BG2HOFS = step.scroll_x.wrapping_add(34).wrapping_sub(x));
}

/// Runs a body script step, or returns `None` for an address this module does not own.
pub(crate) fn dispatch_instruction(
    ea: u32,
    script: EnemyScript,
    game: &mut Game,
) -> Option<EnemyScript> {
    if let Some(step) = movement(ea) {
        move_body(step, game);
    } else if let Some(pose) = pose(ea) {
        Runtime::new(game).write_slice_native(POSE, pose as u16);
    } else {
        return None;
    }
    Some(script)
}
