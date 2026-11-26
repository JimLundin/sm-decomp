use super::{
    Answers, Button, Content, Fault, FileStart, FrameEnd, Pressed, Rules, Screen, State, Work,
};
use crate::exact::save_files::Format;
use crate::production::save_files::{Input, Receipt, advance, image, power_on};
use save_files::{BootReport, ChoiceProgress, ChoiceStage, StartData};
use screens_options::ChosenFile;
use types::file_select::{FileSelectStage, LoadStep, MenuItem, MenuPhase, Slot};
use types::options::FileBegins;
use types::save_files::{BatteryImage, Destination, PendingAction, RunSnapshot, SaveFilesState};
use types::{RandomSequence, Set};

/// Cartridge state carried through the screens and into the run they start.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cartridge {
    /// Complete battery-backed owner, including any interrupted publication.
    pub state: SaveFilesState,
    files: Set<Slot>,
    cursor: Option<Slot>,
    started: Option<Box<StartData>>,
}

impl Cartridge {
    /// The platform supplies one initial battery image at power-on.
    #[must_use]
    pub fn power_on(format: &Format, battery: &BatteryImage) -> Self {
        Self {
            state: power_on(format, battery),
            files: Set::default(),
            cursor: None,
            started: None,
        }
    }

    /// The cartridge diagnostic has finished and startup scans the files.
    ///
    /// # Errors
    /// A pending file action prevents a new scan.
    pub fn boot(&mut self, format: &Format) -> Result<BootReport, Fault> {
        let Receipt::Boot(report) = self.request(format, Input::Boot)? else {
            return Err(Fault::CartridgeReceipt);
        };
        Ok(report)
    }

    /// The complete platform battery at this cut.
    #[must_use]
    pub fn image(&self, format: &Format) -> BatteryImage {
        image(format, &self.state)
    }

    /// The run handed on by file select, including settings as last left by the options.
    #[must_use]
    pub fn started(&self) -> Option<&RunSnapshot> {
        self.started.as_ref().map(|start| &start.run)
    }

    fn request(&mut self, format: &Format, input: Input<'_>) -> Result<Receipt, Fault> {
        advance(&mut self.state, format, input)
            .map(|output| output.receipt)
            .map_err(Fault::Cartridge)
    }

    fn scan(&mut self, format: &Format) -> Result<(), Fault> {
        let Receipt::Menu(report) = self.request(format, Input::Menu)? else {
            return Err(Fault::CartridgeReceipt);
        };
        self.files = Set::default();
        for receipt in report.receipts {
            self.files[receipt.slot] = receipt.is_valid();
        }
        self.cursor = Some(report.cursor);
        Ok(())
    }

    fn start(&mut self, format: &Format, input: Input<'_>) -> Result<(), Fault> {
        self.started = match self.request(format, input)? {
            Receipt::Start(start) | Receipt::Choice(ChoiceProgress::Loaded(start)) => Some(start),
            Receipt::Choice(ChoiceProgress::SelectionWritten) => None,
            Receipt::None | Receipt::Read(_) | Receipt::Boot(_) | Receipt::Menu(_) => {
                return Err(Fault::CartridgeReceipt);
            }
        };
        Ok(())
    }

    pub(super) fn before(
        &mut self,
        format: &Format,
        state: &State,
        work: Work,
        in_flight: Pressed,
    ) -> Result<(), Fault> {
        let State::Screens(screens) = state else {
            return Ok(());
        };
        let Screen::FileSelect(file) = &screens.screen else {
            return Ok(());
        };
        match (file.menu.stage, work, self.state.pending) {
            (FileSelectStage::Loading(LoadStep::Step3), Work::Finished(_), _) => self.scan(format),
            (FileSelectStage::Menu(menu), Work::Unfinished, PendingAction::Idle)
                if menu.phase == MenuPhase::Choosing
                    && (in_flight[Button::A] || in_flight[Button::Start]) =>
            {
                if let MenuItem::File(slot) = menu.cursor {
                    self.start(format, Input::Choose(slot, ChoiceStage::SelectionWritten))?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub(super) fn after(
        &mut self,
        format: &Format,
        state: &State,
        work: Work,
    ) -> Result<(), Fault> {
        let State::Screens(screens) = state else {
            return Ok(());
        };
        match &screens.screen {
            Screen::FileSelect(file) if matches!(work, Work::Finished(_)) => {
                if let Some(slot) = file.chosen {
                    match self.state.pending {
                        PendingAction::Choice(_) => self.start(format, Input::FinishChoice)?,
                        PendingAction::Idle if self.started.is_none() => {
                            self.start(format, Input::Choose(slot, ChoiceStage::ChoiceLoaded))?;
                        }
                        PendingAction::Idle | PendingAction::Write { .. } => {}
                    }
                }
            }
            Screen::Options(options) => {
                if let Some(start) = self.started.as_mut() {
                    start.run.profile.settings = options.menu.settings;
                    self.state.working.profile.settings = options.menu.settings;
                }
            }
            Screen::Title(_) | Screen::FileSelect(_) | Screen::Intro(_) => {}
        }
        Ok(())
    }
}

/// Front-end replies that are independent of the cartridge.
pub trait Platform {
    /// The initial random word at title entry.
    fn settled_random(&self) -> RandomSequence;
    /// Whether an intro track waits to load.
    fn track(&self) -> screens_intro::TrackReply;
}

struct LiveAnswers<'a, P> {
    cartridge: &'a Cartridge,
    platform: &'a P,
}

impl<P> screens_file_select::SaveMemory for LiveAnswers<'_, P> {
    fn files(&self) -> Set<Slot> {
        self.cartridge.files
    }

    fn last_chosen(&self) -> Option<Slot> {
        self.cartridge.cursor
    }
}

impl<P: Platform> Answers for LiveAnswers<'_, P> {
    fn settled_random(&self) -> RandomSequence {
        self.platform.settled_random()
    }

    fn track(&self) -> screens_intro::TrackReply {
        self.platform.track()
    }

    fn file(&self, slot: Slot) -> Option<FileStart> {
        let cartridge = self.cartridge;
        let run = cartridge
            .started()
            .filter(|_| cartridge.state.active == Some(slot))?;
        let begins = match run.restart.destination {
            Destination::Intro => ChosenFile::Begins(FileBegins::Intro),
            Destination::ZebesFlight => ChosenFile::Begins(FileBegins::Escape),
            Destination::CeresArrival => ChosenFile::InPlay,
            Destination::Unrecognized(_) => return None,
        };
        Some(FileStart {
            settings: run.profile.settings,
            begins,
        })
    }
}

/// One cut's independent non-save premises.
pub struct CartridgeFrame<'a, P> {
    /// Screen workload and audio upload progress.
    pub end: FrameEnd,
    /// Buttons examined by a frame still in progress.
    pub in_flight: Pressed,
    /// Front-end rules.
    pub rules: &'a Rules,
    /// Screen content.
    pub content: &'a Content,
    /// Non-cartridge replies.
    pub platform: &'a P,
}

impl State {
    /// Step the front end with the live battery as its save memory.
    ///
    /// # Errors
    /// An unsupported screen or cartridge action commits neither owner.
    pub fn step_with_cartridge(
        &mut self,
        cartridge: &mut Cartridge,
        format: &Format,
        frame: &CartridgeFrame<'_, impl Platform>,
    ) -> Result<Vec<super::Sound>, Fault> {
        let mut next = self.clone();
        let mut save = cartridge.clone();
        save.before(format, self, frame.end.work, frame.in_flight)?;
        let answers = LiveAnswers {
            cartridge: &save,
            platform: frame.platform,
        };
        let sounds = next.step(frame.end, frame.rules, frame.content, &answers)?;
        save.after(format, &next, frame.end.work)?;
        *self = next;
        *cartridge = save;
        Ok(sounds)
    }
}
