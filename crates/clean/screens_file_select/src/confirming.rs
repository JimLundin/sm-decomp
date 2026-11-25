//! Spec: `HelmetTurns`. Once a file is chosen its helmet turns for the helmet turn's frames of
//! work; A or Start, or the turn's end, starts the fade-out.

use types::Set;
use types::file_select::{MenuPhase, MenuState, Slot, TurnLeft};

use crate::input::MenuAction;
use crate::report::FrameReport;

/// Spec: `HelmetTurns`. One finished frame of the turn of `chosen`'s helmet: a frame less left,
/// then the menu leaves if Choose was pressed or the turn has ended.
pub fn turn(
    menu: &mut MenuState,
    chosen: Slot,
    turn_left: TurnLeft,
    pressed: Set<MenuAction>,
) -> FrameReport {
    menu.phase = match turn_left.counted_down() {
        Some(left) if !pressed[MenuAction::Choose] => MenuPhase::Confirming {
            chosen,
            turn_left: left,
        },
        Some(_) | None => MenuPhase::Leaving,
    };
    FrameReport::NOTHING
}
