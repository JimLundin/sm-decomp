//! One frame end of file select: what acts, by where file select stands.

use types::file_select::{FileSelectStage, FileSelectState, MenuPhase, MenuState};
use types::power_on::ScreenLight;

use crate::input::{FrameEnd, SaveMemory};
use crate::report::{FrameReport, Unsupported};
use crate::rules::Rules;
use crate::{choosing, confirming, fade, load};

/// File select at one frame end. A frame end with no frame finished changes nothing; one after
/// file select went out belongs to the piece that follows, and is `Unsupported` here.
///
/// # Errors
///
/// `Unsupported`, with the way out, once file select has gone out.
pub fn frame_end(
    state: &mut FileSelectState,
    end: FrameEnd,
    save: &impl SaveMemory,
    rules: &Rules,
) -> Result<FrameReport, Unsupported> {
    match (&mut state.stage, end) {
        (FileSelectStage::Menu(menu), end) => menu_frame(menu, &mut state.screen, end, *rules),
        (FileSelectStage::Loading(_), FrameEnd::Busy) => Ok(FrameReport::NOTHING),
        (FileSelectStage::Loading(step), FrameEnd::Finished(_)) => {
            let step = *step;
            Ok(load::load_frame(state, step, save, rules.menu_pace))
        }
    }
}

/// The menu at one frame end, by its phase.
fn menu_frame(
    menu: &mut MenuState,
    screen: &mut ScreenLight,
    end: FrameEnd,
    rules: Rules,
) -> Result<FrameReport, Unsupported> {
    Ok(match (menu.phase, end) {
        (MenuPhase::Out(way), _) => return Err(Unsupported(way)),
        (_, FrameEnd::Busy) => FrameReport::NOTHING,
        (MenuPhase::LoadEnding, FrameEnd::Finished(_)) => load::end_load(menu),
        (MenuPhase::Brightening, FrameEnd::Finished(_)) => {
            fade::brighten(menu, screen, rules.menu_pace)
        }
        (MenuPhase::Choosing, FrameEnd::Finished(pressed)) => {
            choosing::choose(menu, pressed, rules.helmet_turn)
        }
        (MenuPhase::Confirming { chosen, turn_left }, FrameEnd::Finished(pressed)) => {
            confirming::turn(menu, chosen, turn_left, pressed)
        }
        (MenuPhase::Leaving, FrameEnd::Finished(_)) => fade::dim(menu, screen, rules.menu_pace),
    })
}
