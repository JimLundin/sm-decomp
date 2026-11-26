//! Spec: `ChooseFile`. Recording the player's choice, then providing the run to start or resume.

use enum_map::EnumMap;
use types::file_select::Slot;
use types::options::Settings;
use types::save_files::{
    Ammunition, ContentFlags, Control, Destination, Gear, MenuInspection, NO_PLAY_TIME, PadButton,
    PendingAction, PlayerProfile, Progress, RestartPoint, RetainedWords, RunSnapshot,
    SelectedWeapon, StoredArea, Supply, WorkingRun, blank_maps,
};

use crate::files::{Refusal, SaveFiles};
use crate::format::SaveFormat;
use crate::read::{ReadOutcome, ReadReceipt};
use crate::selection::selection_of;

/// Spec: `ChooseFile`. A fresh file's energy: 99 current and maximum.
const FRESH_ENERGY: Supply = Supply {
    current: 99,
    maximum: 99,
};
/// Spec: `ChooseFile`. A fresh file's reserves and ammunition: none, with no capacity.
const NO_SUPPLY: Supply = Supply {
    current: 0,
    maximum: 0,
};
/// Spec: `ChooseFile`. A fresh file's placement-disabled value and spare startup word, unsigned
/// one.
const FRESH_STARTUP_VALUE: u16 = 1;

/// Spec: `ChooseFile`. How far a choice has finished, as the caller reports it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChoiceStage {
    /// The selection words are on the cartridge; the load is still to finish.
    SelectionWritten,
    /// The load has finished too.
    ChoiceLoaded,
}

/// Spec: `ChooseFile`. The run a finished choice hands to the live owners.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StartData {
    /// The chosen slot's read at the load.
    pub receipt: ReadReceipt,
    /// The run to resume, or the fresh one; its destination is the options' start route.
    pub run: RunSnapshot,
}

/// Spec: `ChooseFile`. Where a choice stands once the request returns.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChoiceProgress {
    /// The selection is written; the run is not ready until the choice is loaded.
    SelectionWritten,
    /// The run is ready.
    Loaded(Box<StartData>),
}

/// Spec: `ChooseFile`. A fresh file's binding for each action: jump with A, run with B, shoot
/// with X, cancel items with Y, select items with Select, aim up with R and aim down with L.
const fn fresh_binding(control: Control) -> PadButton {
    match control {
        Control::Jump => PadButton::A,
        Control::Run => PadButton::B,
        Control::Shoot => PadButton::X,
        Control::ItemCancel => PadButton::Y,
        Control::ItemSelect => PadButton::Select,
        Control::AimUp => PadButton::R,
        Control::AimDown => PadButton::L,
    }
}

/// No equipment at all.
fn no_gear() -> Gear {
    Gear {
        items: EnumMap::default(),
        beams: EnumMap::default(),
    }
}

/// Spec: `ChooseFile`. A fresh file's profile: 99 energy, no ammunition, reserves, equipment or
/// weapon, the fresh bindings and settings, no play time, placement disabled and the spare
/// startup word one, and the working profile's other retained words left as they were.
fn fresh_profile(working: &PlayerProfile) -> PlayerProfile {
    PlayerProfile {
        owned: no_gear(),
        in_use: no_gear(),
        controls: EnumMap::from_fn(fresh_binding),
        settings: Settings::default(),
        energy: FRESH_ENERGY,
        reserves: NO_SUPPLY,
        ammunition: EnumMap::from_fn(|_: Ammunition| NO_SUPPLY),
        weapon: SelectedWeapon::Nothing,
        play_time: NO_PLAY_TIME,
        placement_disabled: FRESH_STARTUP_VALUE,
        startup_word: FRESH_STARTUP_VALUE,
        retained: working.retained.clone(),
    }
}

/// Spec: `ChooseFile`. A fresh file's progress: no events, bosses, items, changed locations,
/// doors, stations, elevators or maps collected, with the copied pass-through progress.
fn fresh_progress(pass_through: RetainedWords) -> Progress {
    Progress {
        events: EnumMap::default(),
        bosses: EnumMap::default(),
        item_count: 0,
        items: ContentFlags::default(),
        changed_locations: ContentFlags::default(),
        doors: ContentFlags::default(),
        stations: ContentFlags::default(),
        elevators: ContentFlags::default(),
        maps_collected: EnumMap::default(),
        pass_through,
    }
}

impl<F: SaveFormat> SaveFiles<F> {
    /// Spec: `ChooseFile`. The player confirms `slot` after a finished menu scan: the active
    /// slot and the selection words are set at once, and the run follows at `ChoiceLoaded`,
    /// now if the request reached it.
    ///
    /// # Errors
    ///
    /// `ActionPending` while a choice or write stands between its milestones, and
    /// `MenuUninspected` before the menu scan has finished this session.
    pub fn choose_file(
        &mut self,
        slot: Slot,
        reached: ChoiceStage,
    ) -> Result<ChoiceProgress, Refusal> {
        self.admit()?;
        if self.state.menu == MenuInspection::Uninspected {
            return Err(Refusal::MenuUninspected);
        }
        self.state.cartridge.selection = selection_of(slot);
        self.state.active = Some(slot);
        self.state.pending = PendingAction::Choice(slot);
        match reached {
            ChoiceStage::SelectionWritten => Ok(ChoiceProgress::SelectionWritten),
            ChoiceStage::ChoiceLoaded => self
                .continue_choice()
                .map(|start| ChoiceProgress::Loaded(Box::new(start))),
        }
    }

    /// Spec: `ChooseFile`. The pending choice reaches `ChoiceLoaded`: the chosen slot is read as
    /// the menu scan left it. A valid file resumes its run; a failed one starts a fresh file,
    /// at the intro, from the working profile of the menu session.
    ///
    /// # Errors
    ///
    /// `NothingToComplete` when no choice is pending.
    pub fn continue_choice(&mut self) -> Result<StartData, Refusal> {
        let PendingAction::Choice(slot) = self.state.pending else {
            return Err(Refusal::NothingToComplete);
        };
        let receipt = self.read(slot);
        let run = match &receipt.outcome {
            ReadOutcome::Decoded(run) => (**run).clone(),
            ReadOutcome::Empty { area, station } => {
                let copied = self.format.decode(&receipt.copied);
                let fresh = RunSnapshot {
                    profile: fresh_profile(&self.state.working.profile),
                    progress: fresh_progress(copied.progress.pass_through),
                    maps: blank_maps(),
                    restart: RestartPoint {
                        destination: Destination::Intro,
                        area: StoredArea::Known(*area),
                        station: *station,
                    },
                    tail: copied.tail,
                };
                self.state.working = WorkingRun {
                    profile: fresh.profile.clone(),
                    maps: fresh.maps.clone(),
                };
                fresh
            }
        };
        self.state.pending = PendingAction::Idle;
        Ok(StartData { receipt, run })
    }
}
