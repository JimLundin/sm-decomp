//! Mother Brain's body script steps: moving the BG2-drawn body and setting its pose.
//! The body lives in actor slot 0; its pose word stays RAM-authoritative.
#![forbid(unsafe_code)]
use crate::game::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use crate::game::enemy_script::EnemyScript;
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use crate::ram_map::word;
use crate::types::Enemy_MotherBrain;
use core::mem::offset_of;

const BODY: Key = Key::from_legacy(0);
const POSE: usize = 0xf78 + offset_of!(Enemy_MotherBrain, var_02);
const PHASE: usize = 0xf78 + offset_of!(Enemy_MotherBrain, var_00);

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

const fn step(dx: i16, dy: i16, footstep: bool) -> Step {
    Step {
        dx: dx as u16,
        dy: dy as u16,
        scroll_x: 0,
        footstep,
    }
}

const fn scrolled(scroll_x: i16, dy: i16) -> Step {
    Step {
        dx: 0,
        dy: dy as u16,
        scroll_x: scroll_x as u16,
        footstep: false,
    }
}

fn movement(ea: u32) -> Option<Step> {
    Some(match ea {
        11113910 => scrolled(4, -10),
        11113920 => scrolled(4, -16),
        11113930 => scrolled(-2, -12),
        11113950 => scrolled(4, 12),
        11113960 => scrolled(-2, 16),
        11113970 => scrolled(-2, 10),
        11113980 | 11114224 => step(1, -2, false),
        11113996 => step(2, 0, false),
        11114012 => step(0, 1, false),
        11114018 => step(3, 1, true),
        11114040 => step(15, -2, false),
        11114056 => step(6, -4, false),
        11114072 => step(-2, 4, false),
        11114088 | 11114110 => step(-1, 2, true),
        11114132 => step(-2, 0, false),
        11114148 => step(0, -1, false),
        11114154 => step(-3, -1, false),
        11114170 => step(-15, 2, true),
        11114192 => step(-6, 4, false),
        11114208 => step(2, -4, false),
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
    word! {game; earthquake_type = 1; earthquake_timer = 4; }
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
    word!(game.reg_BG2VOFS -= step.dy);
    word!(game.reg_BG2HOFS = step.scroll_x.wrapping_add(34).wrapping_sub(x));
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
