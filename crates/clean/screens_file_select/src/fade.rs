//! The menu's fades: in to full as it opens (`MenuBrightens`), out to dark as it is left
//! (`MenuDims`), one level every menu pace of finished frames.

use types::file_select::{FadeWait, MenuPace, MenuPhase, MenuState, WayOut};
use types::power_on::{Level, ScreenLight};

use crate::report::FrameReport;

/// Spec: `MenuBrightens`, `MenuDims`. One finished frame of a fade: while the wait is above 0 it
/// counts down and no level is due; at 0 a level is due, and the wait starts again at the menu
/// pace less one.
const fn level_due(menu: &mut MenuState, pace: MenuPace) -> bool {
    if let Some(left) = menu.fade_wait.0.checked_sub(1) {
        menu.fade_wait = FadeWait(left);
        false
    } else {
        menu.fade_wait = pace.wait();
        true
    }
}

/// Spec: `MenuBrightens`. One finished frame of the fade in: one level brighter when its wait is over (never past
/// full), and at full the cursor is the player's.
pub fn brighten(menu: &mut MenuState, screen: &mut ScreenLight, pace: MenuPace) -> FrameReport {
    if level_due(menu, pace) {
        let level = Level::new(screen.level().get().saturating_add(1)).unwrap_or(Level::FULL);
        *screen = ScreenLight::Lit(level);
        if level == Level::FULL {
            menu.phase = MenuPhase::Choosing;
        }
    }
    FrameReport::NOTHING
}

/// Spec: `MenuDims`. One finished frame of the fade out: one level darker when its wait is over; the level that
/// would reach dark blanks the screen instead, and the options menu begins.
pub fn dim(menu: &mut MenuState, screen: &mut ScreenLight, pace: MenuPace) -> FrameReport {
    if level_due(menu, pace) {
        match screen.level().get().checked_sub(1).and_then(Level::new) {
            Some(level) if level != Level::DARK => *screen = ScreenLight::Lit(level),
            Some(_) | None => {
                *screen = ScreenLight::Blank(Level::DARK);
                menu.phase = MenuPhase::Out(WayOut::OptionsBegin);
            }
        }
    }
    FrameReport::NOTHING
}
