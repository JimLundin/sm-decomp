//! The front end: which part of the game runs ahead of room play, one frame end at a time.
//!
//! From the console's power-on the boot runs (`power_on`, screens-boot's power-on) until the
//! game settles. On the first frame that finishes then, the title begins (`title`), and each
//! screen hands on to the next at its way out: file select (`file_select`), then the options
//! menu (`options`), until the options hand on to the intro, a later piece. From the title on,
//! the frame owner's [`Counts`] carry the count of finished frames and the random number, and
//! owners outside the rungs answer through [`Answers`]. Room play joins this State once the
//! intro and the parts after it hand on.

mod file_select;
mod options;
mod power_on;
mod title;

use super::frame::Counts;
use screens_boot::{DriverUpload, PowerOnState};
use screens_file_select::{FrameOfWork, SaveMemory};
use screens_options::ChosenFile;
use types::RandomSequence;
use types::file_select::{
    FileSelectStage, FileSelectState, MenuPhase, MenuState, Slot, WayOut as FileSelectWayOut,
};
use types::options::{OptionsState, Settings, WayOut};
use types::title::{TitleStage, TitleState};

pub use screens_title::{Button, Pressed};

/// Which part of the game runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    /// The console's power-on, until the game settles.
    PowerOn(PowerOnState),
    /// The screens from the title on.
    Screens(Screens),
}

/// The screens from the title on, with the frame owner's counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Screens {
    /// The screen that runs.
    pub screen: Screen,
    /// The count of finished frames and the random number.
    pub counts: Counts,
}

/// A screen from the title on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    /// The title, from the game's settling until file select begins.
    Title(TitleState),
    /// File select, until the options menu begins.
    FileSelect(FileSelect),
    /// The options menu, until it hands on.
    Options(Options),
}

/// File select, with what it reported that its State doesn't keep.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FileSelect {
    /// The menu.
    pub menu: FileSelectState,
    /// The file chosen, which save memory's owner starts; none until one is.
    pub chosen: Option<Slot>,
    /// Whether the frame of work went on through the last finished frame end, which the frame
    /// owner reads: then no frame of work begins, and none draws, at the next one.
    pub frame: FrameOfWork,
}

/// The options menu, with how the chosen file begins.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Options {
    /// The menu.
    pub menu: OptionsState,
    /// How the chosen file begins, which start game reads.
    pub begins: ChosenFile,
}

/// What a frame end brings in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameEnd {
    /// The frames of work finished since the last frame end.
    pub work: Work,
    /// Whether the boot's sound driver upload ended in the frame.
    pub upload: DriverUpload,
}

/// The frames of work finished since the last frame end, each with the buttons just pressed as
/// it began.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Work {
    /// None: the work runs on through the frame end.
    Unfinished,
    /// One.
    Finished(Pressed),
    /// Two, the first before the second.
    FinishedTwice([Pressed; 2]),
}

impl Work {
    /// How many frames of work finished.
    const fn finished(self) -> u16 {
        match self {
            Self::Unfinished => 0,
            Self::Finished(_) => 1,
            Self::FinishedTwice(_) => 2,
        }
    }
}

/// The rules each screen plays.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rules {
    /// The power-on's.
    pub power_on: screens_boot::Rules,
    /// The title's.
    pub title: screens_title::Rules,
    /// File select's.
    pub file_select: screens_file_select::Rules,
}

impl Rules {
    /// The game as it ships.
    pub const REFERENCE: Self = Self {
        power_on: screens_boot::Rules::REFERENCE,
        title: screens_title::Rules::REFERENCE,
        file_select: screens_file_select::Rules::REFERENCE,
    };
}

/// The answers of owners outside the rungs: save memory's (file select's questions about the
/// slots, and the chosen file's start) and the random number's seeding as the game settles.
pub trait Answers: SaveMemory {
    /// The random number as the game settles.
    fn settled_random(&self) -> RandomSequence;
    /// How the file in `slot` starts once chosen: its settings and how it begins. None where
    /// save memory's owner can't start it.
    fn file(&self, slot: Slot) -> Option<FileStart>;
}

/// How a chosen file starts: its settings, and how it begins after the options menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FileStart {
    /// The language, weapon-reset and moonwalk.
    pub settings: Settings,
    /// With the intro, the escape, or straight on in play.
    pub begins: ChosenFile,
}

/// A sound a screen asks the audio side for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sound {
    /// The title's tunes.
    Title(screens_title::TuneRequest),
    /// File select's sounds.
    FileSelect(screens_file_select::MenuSound),
    /// The options menu's sounds.
    Options(screens_options::MenuSound),
}

/// Why a frame end was not played. State is unchanged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Two frames of work finished by one frame end where only the title states what follows.
    TwoFinished,
    /// The title went out to a later piece: the opening playing on, or the demos.
    TitleLeft,
    /// File select went out to a later piece.
    FileSelectLeft(FileSelectWayOut),
    /// Save memory's owner can't start the chosen file.
    FileUnstarted,
    /// The options menu handed on to a later piece.
    HandedOn(WayOut),
}

impl State {
    /// The console just switched on.
    pub const AT_POWER_ON: Self = Self::PowerOn(PowerOnState::AT_POWER_ON);

    /// Play one frame end of the part that runs, giving the sounds asked for in it.
    ///
    /// # Errors
    /// A [`Fault`] where the frame end belongs to a part that isn't played.
    pub fn step(
        &mut self,
        end: FrameEnd,
        rules: &Rules,
        answers: &impl Answers,
    ) -> Result<Vec<Sound>, Fault> {
        let (next, sounds) = match *self {
            Self::PowerOn(power_on) => power_on::step(power_on, end, rules, answers)?,
            Self::Screens(screens) => {
                let (screens, sounds) = screens.step(end.work, rules, answers)?;
                (Self::Screens(screens), sounds)
            }
        };
        *self = next;
        Ok(sounds)
    }
}

/// One screen's frame end: the screen after it, the frames of work it finished and how many of
/// them drew, and the sounds asked for.
struct Stepped {
    screen: Screen,
    finished: u16,
    drawn: u16,
    sounds: Vec<Sound>,
}

impl Screens {
    /// The game settled: the title begun on the power-on's screen, the count going on from the
    /// power-on's, the random number as the game settles.
    fn settled(power_on: PowerOnState, answers: &impl Answers) -> Self {
        Self {
            screen: Screen::Title(TitleState::begun(power_on.screen)),
            counts: Counts {
                finished_frames: power_on.frames.0,
                random: answers.settled_random(),
            },
        }
    }

    fn step(
        self,
        work: Work,
        rules: &Rules,
        answers: &impl Answers,
    ) -> Result<(Self, Vec<Sound>), Fault> {
        let stepped = match self.screen.handed_on(answers)? {
            Screen::Title(state) => title::step(state, work, rules.title)?,
            Screen::FileSelect(state) => {
                file_select::step(state, work, rules.file_select, answers)?
            }
            Screen::Options(state) => options::step(state, work)?,
        };
        let screens = Self {
            screen: stepped.screen,
            counts: self.counts.after(stepped.finished, stepped.drawn),
        };
        Ok((screens, stepped.sounds))
    }
}

impl Screen {
    /// The screen that plays the next frame end: the next one where this one handed on to it.
    fn handed_on(self, answers: &impl Answers) -> Result<Self, Fault> {
        Ok(match self {
            Self::Title(TitleState {
                stage: TitleStage::FileSelectBegins,
                screen,
            }) => Self::FileSelect(FileSelect::begin(screen)),
            Self::FileSelect(FileSelect {
                menu:
                    FileSelectState {
                        stage:
                            FileSelectStage::Menu(MenuState {
                                phase: MenuPhase::Out(FileSelectWayOut::OptionsBegin),
                                ..
                            }),
                        ..
                    },
                chosen,
                ..
            }) => {
                let file = chosen
                    .and_then(|slot| answers.file(slot))
                    .ok_or(Fault::FileUnstarted)?;
                Self::Options(Options {
                    menu: OptionsState::begin(file.settings),
                    begins: file.begins,
                })
            }
            unchanged @ (Self::Title(_) | Self::FileSelect(_) | Self::Options(_)) => unchanged,
        })
    }
}

#[cfg(test)]
mod tests;
