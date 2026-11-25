//! Spec: `SpecialSettings`. The special settings: the cursor moves first, then B or end goes back
//! to the options, or A, Left, Right or Start switches the setting the cursor is now on.

use types::options::{DissolveCursor, OptionsStage, Settings, SpecialItem};

use crate::cursor;
use crate::dissolve;
use crate::frame::{MenuAction, MenuSound, Pressed};

/// Spec: `SpecialSettings`. One finished frame of the special settings. B wins over a switch;
/// going back and a switch each ask for `Selected`.
pub fn special_settings(
    cursor: SpecialItem,
    pressed: Pressed,
    settings: &mut Settings,
    sounds: &mut Vec<MenuSound>,
) -> OptionsStage {
    let cursor = cursor::moved(cursor, pressed, sounds);
    let switches = pressed[MenuAction::Choose] || pressed[MenuAction::Sideways];
    if !pressed[MenuAction::Back] && !switches {
        return OptionsStage::Special(cursor);
    }
    sounds.push(MenuSound::Selected);
    if pressed[MenuAction::Back] {
        return back_to_options();
    }
    match cursor {
        SpecialItem::WeaponReset => settings.weapon_reset = !settings.weapon_reset,
        SpecialItem::Moonwalk => settings.moonwalk = !settings.moonwalk,
        SpecialItem::End => return back_to_options(),
    }
    OptionsStage::Special(cursor)
}

/// Spec: `SpecialSettings`. Back to the options by a dissolve, the cursor on start game.
const fn back_to_options() -> OptionsStage {
    dissolve::start(DissolveCursor::StartGame)
}
