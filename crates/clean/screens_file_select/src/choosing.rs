//! The menu at full, in the player's hands: A or Start acts on the item under the cursor
//! (`ItemPressed`), Up and Down move the cursor and B goes back to the title (`CursorMoves`).

use types::Set;
use types::file_select::{HelmetTurn, MenuItem, MenuPhase, MenuState, Slot, TurnLeft, WayOut};

use crate::cursor::{self, CursorStep};
use crate::input::MenuAction;
use crate::report::{FrameReport, MenuSound};

/// Spec: `CursorMoves`. B asks for `CursorMoved` twice as it goes back to the title.
const BACK_SOUNDS: &[MenuSound] = &[MenuSound::CursorMoved, MenuSound::CursorMoved];

/// One finished frame while choosing: the first action pressed, in `MenuAction`'s order, acts
/// alone; with none of them pressed nothing changes.
pub fn choose(menu: &mut MenuState, pressed: Set<MenuAction>, turn: HelmetTurn) -> FrameReport {
    let first = pressed
        .iter()
        .find_map(|(action, &is_in)| is_in.then_some(action));
    match first {
        Some(MenuAction::Choose) => press_item(menu, turn),
        Some(MenuAction::Up) => move_cursor(menu, CursorStep::Up),
        Some(MenuAction::Down) => move_cursor(menu, CursorStep::Down),
        Some(MenuAction::Back) => go_out(menu, WayOut::TitleReturns, BACK_SOUNDS),
        None => FrameReport::NOTHING,
    }
}

/// Spec: `ItemPressed`. A file is chosen; copy and clear begin with `CursorMoved`; exit goes back
/// to the title in silence.
fn press_item(menu: &mut MenuState, turn: HelmetTurn) -> FrameReport {
    match menu.cursor {
        MenuItem::File(slot) => choose_file(menu, slot, turn),
        MenuItem::Copy => go_out(menu, WayOut::CopyBegins, &[MenuSound::CursorMoved]),
        MenuItem::Clear => go_out(menu, WayOut::ClearBegins, &[MenuSound::CursorMoved]),
        MenuItem::Exit => go_out(menu, WayOut::TitleReturns, &[]),
    }
}

/// Spec: `ItemPressed`. The file in `slot` chosen, for save memory's owner to record: its helmet
/// begins its whole turn, and `ChoiceMade` is asked for.
fn choose_file(menu: &mut MenuState, slot: Slot, turn: HelmetTurn) -> FrameReport {
    menu.phase = MenuPhase::Confirming {
        chosen: slot,
        turn_left: TurnLeft::whole(turn),
    };
    FrameReport::asking(&[MenuSound::ChoiceMade])
}

/// Spec: `CursorMoves`. The cursor one step, asking for `CursorMoved`.
fn move_cursor(menu: &mut MenuState, step: CursorStep) -> FrameReport {
    menu.cursor = cursor::stepped(menu.cursor, step, menu.copy_and_clear);
    FrameReport::asking(&[MenuSound::CursorMoved])
}

/// File select goes out this way, asking for these sounds.
fn go_out(menu: &mut MenuState, way: WayOut, sounds: &[MenuSound]) -> FrameReport {
    menu.phase = MenuPhase::Out(way);
    FrameReport::asking(sounds)
}
