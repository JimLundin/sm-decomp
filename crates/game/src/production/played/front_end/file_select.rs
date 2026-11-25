//! File select (screens-file-select), until the options menu begins.

use super::{Button, Fault, FileSelect, Pressed, Screen, Sound, Stepped, Work};
use screens_file_select::{FrameEnd, FrameOfWork, MenuAction, Rules, SaveMemory, Unsupported};
use types::Set;
use types::file_select::{FileSelectStage, FileSelectState, MenuPhase, MenuState};
use types::power_on::ScreenLight;

impl FileSelect {
    /// File select as it begins, on the title's last screen.
    pub(super) const fn begin(screen: ScreenLight) -> Self {
        Self {
            menu: FileSelectState::begin(screen),
            chosen: None,
            frame: FrameOfWork::Ends,
        }
    }
}

/// One frame end of file select, its slots asked of save memory's owner.
pub(super) fn step(
    file_select: FileSelect,
    work: Work,
    rules: Rules,
    save: &impl SaveMemory,
) -> Result<Stepped, Fault> {
    let end = match work {
        Work::Unfinished => FrameEnd::Busy,
        Work::Finished(pressed) => FrameEnd::Finished(actions(&pressed)),
        Work::FinishedTwice(_) => return Err(Fault::TwoFinished),
    };
    let mut menu = file_select.menu;
    let report = screens_file_select::frame_end(&mut menu, end, save, &rules)
        .map_err(|Unsupported(way)| Fault::FileSelectLeft(way))?;
    let finished = work.finished();
    // A frame of work that went on through the last finished frame end began none at this one,
    // and a frame end with none finished leaves that as it was.
    let (drawn, frame) = match (finished, file_select.frame) {
        (0, frame) => (0, frame),
        (_, FrameOfWork::GoesOn) => (0, report.frame),
        (_, FrameOfWork::Ends) => (finished, report.frame),
    };
    let chosen = match menu.stage {
        FileSelectStage::Menu(MenuState {
            phase: MenuPhase::Confirming { chosen, .. },
            ..
        }) => Some(chosen),
        FileSelectStage::Menu(_) | FileSelectStage::Loading(_) => file_select.chosen,
    };
    Ok(Stepped {
        screen: Screen::FileSelect(FileSelect {
            menu,
            chosen,
            frame,
        }),
        finished,
        drawn,
        sounds: report.sounds.into_iter().map(Sound::FileSelect).collect(),
    })
}

/// What the buttons just pressed ask of the menu.
fn actions(pressed: &Pressed) -> Set<MenuAction> {
    pressed
        .iter()
        .filter(|&(_, &down)| down)
        .filter_map(|(button, _)| action(button))
        .map(|action| (action, true))
        .collect()
}

/// The menu's binding: A and Start choose, B goes back.
const fn action(button: Button) -> Option<MenuAction> {
    match button {
        Button::A | Button::Start => Some(MenuAction::Choose),
        Button::Up => Some(MenuAction::Up),
        Button::Down => Some(MenuAction::Down),
        Button::B => Some(MenuAction::Back),
        Button::X
        | Button::Y
        | Button::L
        | Button::R
        | Button::Select
        | Button::Left
        | Button::Right => None,
    }
}
