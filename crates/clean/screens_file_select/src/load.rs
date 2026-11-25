//! Spec: `MenuLoads`. Black while file select loads over five finished frame ends; the fourth
//! makes the menu ready and unblanks the screen at level 0.

use types::file_select::{
    FileSelectStage, FileSelectState, LoadStep, MenuPace, MenuPhase, MenuState,
};
use types::power_on::{Level, ScreenLight};

use crate::cursor;
use crate::input::SaveMemory;
use crate::report::{FrameOfWork, FrameReport, MenuSound};

/// Where one of the load's finished frame ends leaves it.
#[derive(Clone, Copy)]
enum LoadNext {
    /// At this step, still black.
    Step(LoadStep),
    /// The menu made ready: load step 4.
    MenuReady,
}

/// What one of the load's finished frame ends does.
struct LoadWork {
    /// Where it leaves the load.
    next: LoadNext,
    /// The screen it shows, if it shows a new one.
    screen: Option<ScreenLight>,
    /// The sound it asks for, if any.
    sound: Option<MenuSound>,
    /// Whether its frame of work goes on through the frame end.
    frame: FrameOfWork,
}

/// Spec: `MenuLoads`. The load's finished frame ends, by the step each begins at: the first
/// blanks the screen dark and its frame goes on, the second asks for `MenuOpens`, the third does
/// nothing more, and the fourth makes the menu ready, lit at level 0, its frame going on.
const fn load_work(from: LoadStep) -> LoadWork {
    match from {
        LoadStep::Step0 => LoadWork {
            next: LoadNext::Step(LoadStep::Step1),
            screen: Some(ScreenLight::Blank(Level::DARK)),
            sound: None,
            frame: FrameOfWork::GoesOn,
        },
        LoadStep::Step1 => LoadWork {
            next: LoadNext::Step(LoadStep::Step2),
            screen: None,
            sound: Some(MenuSound::MenuOpens),
            frame: FrameOfWork::Ends,
        },
        LoadStep::Step2 => LoadWork {
            next: LoadNext::Step(LoadStep::Step3),
            screen: None,
            sound: None,
            frame: FrameOfWork::Ends,
        },
        LoadStep::Step3 => LoadWork {
            next: LoadNext::MenuReady,
            screen: Some(ScreenLight::Lit(Level::DARK)),
            sound: None,
            frame: FrameOfWork::GoesOn,
        },
    }
}

/// Spec: `MenuLoads`, `CursorStarts`. One finished frame end of the load at `step`.
pub fn load_frame(
    state: &mut FileSelectState,
    step: LoadStep,
    save: &impl SaveMemory,
    pace: MenuPace,
) -> FrameReport {
    let work = load_work(step);
    if let Some(screen) = work.screen {
        state.screen = screen;
    }
    state.stage = match work.next {
        LoadNext::Step(next) => FileSelectStage::Loading(next),
        LoadNext::MenuReady => FileSelectStage::Menu(MenuState {
            phase: MenuPhase::LoadEnding,
            cursor: cursor::starting_item(save),
            copy_and_clear: cursor::offer(save),
            fade_wait: pace.wait(),
        }),
    };
    FrameReport {
        sounds: work.sound.into_iter().collect(),
        frame: work.frame,
    }
}

/// Spec: `MenuLoads`. The load's fifth finished frame end: the menu begins to brighten.
pub const fn end_load(menu: &mut MenuState) -> FrameReport {
    menu.phase = MenuPhase::Brightening;
    FrameReport::NOTHING
}
