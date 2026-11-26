//! Production owns the cartridge and publishes its full image at each boundary.

use canonical::model::Pinned;
use save_files::{
    BootReport, ChoiceProgress, ChoiceStage, MenuReport, ReadOutcome, ReadReceipt, Refusal,
    SaveCut, SaveFiles, SaveFormat, StartData, WriteStage,
};
use serde::{Deserialize, Serialize};
use types::file_select::Slot;
use types::save_files::{
    BatteryImage, EncodedRun, ExploredCells, RunSnapshot, SaveFilesState, WorkingRun,
};

use crate::exact::save_files::{BootProgress, Format};

/// Initialize the working owners at cold boot from the platform battery.
#[must_use]
pub fn power_on(format: &Format, image: &BatteryImage) -> SaveFilesState {
    let run = format.decode(&EncodedRun::zeroed());
    SaveFilesState::power_on(
        format.cartridge(image),
        WorkingRun {
            profile: run.profile,
            maps: run.maps,
        },
    )
}

/// The cartridge owner's input at a production cut.
#[derive(Clone, Copy)]
pub enum Input<'a> {
    /// Carry the image with no cartridge request.
    Hold,
    /// A physical diagnostic or initial erase is still in progress.
    InspectBoot(BootProgress),
    /// Inspect a single slot.
    Read(Slot),
    /// Boot's diagnostic has restored the cartridge and its file scan is complete.
    Boot,
    /// The menu inspects the three slots.
    Menu,
    /// The menu confirms a slot.
    Choose(Slot, ChoiceStage),
    /// The selected slot's load finishes.
    FinishChoice,
    /// A new save captures the current run and active grid.
    Write {
        /// The scene handoff making the request.
        cut: SaveCut,
        /// The run from its current owners.
        run: &'a RunSnapshot,
        /// The active explored grid.
        live_map: &'a ExploredCells,
        /// How far the save progressed before the cut.
        stage: WriteStage,
    },
    /// The previous save publishes both checks.
    FinishWrite,
}

/// The report of a completed request, alongside the complete image.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Receipt {
    /// No report beyond the image.
    None,
    /// One file read.
    Read(Box<ReadReceipt>),
    /// Boot's file scan and demo count.
    Boot(BootReport),
    /// The menu's file scan and cursor.
    Menu(MenuReport),
    /// Progress of a selection.
    Choice(ChoiceProgress),
    /// The selected file's starting run.
    Start(Box<StartData>),
}

/// Full battery output and the current request's result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    /// The image to publish to the platform.
    pub image: BatteryImage,
    /// The requested behaviour's output.
    pub receipt: Receipt,
}

/// Run the selected cartridge operation and publish its complete image.
///
/// # Errors
/// Refused operations leave the input state intact.
pub fn advance(
    state: &mut SaveFilesState,
    format: &Format,
    input: Input<'_>,
) -> Result<Output, Refusal> {
    let mut owner = SaveFiles::new(format.clone(), state.clone());
    let receipt = match input {
        Input::Hold => Receipt::None,
        Input::InspectBoot(progress) => {
            return Ok(Output {
                image: progress.image(format.image(&state.cartridge)),
                receipt: Receipt::None,
            });
        }
        Input::Read(slot) => Receipt::Read(Box::new(owner.read_slot(slot)?)),
        Input::Boot => Receipt::Boot(owner.boot_files()?),
        Input::Menu => Receipt::Menu(owner.menu_files()?),
        Input::Choose(slot, milestone) => {
            let mut progress = owner.choose_file(slot, milestone)?;
            if let ChoiceProgress::Loaded(start) = &mut progress {
                prepare_start(&mut owner.state, start);
            }
            Receipt::Choice(progress)
        }
        Input::FinishChoice => {
            let mut start = owner.continue_choice()?;
            prepare_start(&mut owner.state, &mut start);
            Receipt::Start(Box::new(start))
        }
        Input::Write {
            cut,
            run,
            live_map,
            stage,
        } => {
            owner.write_file(cut, run, live_map, stage)?;
            Receipt::None
        }
        Input::FinishWrite => {
            owner.continue_write()?;
            Receipt::None
        }
    };
    let image = format.image(&owner.state.cartridge);
    *state = owner.state;
    Ok(Output { image, receipt })
}

/// Project a held or restored cartridge's battery image.
#[must_use]
pub fn image(format: &Format, state: &SaveFilesState) -> BatteryImage {
    format.image(&state.cartridge)
}

fn prepare_start(state: &mut SaveFilesState, start: &mut StartData) {
    if matches!(start.receipt.outcome, ReadOutcome::Empty { .. }) {
        crate::exact::save_files::fresh_words(&mut start.run.profile);
        state.working.profile = start.run.profile.clone();
    }
}

/// Retained production state for a cartridge and the selected run's handoff.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    /// The clean cartridge state.
    pub cartridge: SaveFilesState,
    /// Run handed to the live owners by the most recent completed choice.
    pub started: Option<Box<RunSnapshot>>,
    /// File-select's initial cursor after the last completed inspection.
    pub menu_cursor: Option<Slot>,
}

impl State {
    /// At power-on a cartridge is present and no run is selected.
    #[must_use]
    pub const fn new(cartridge: SaveFilesState) -> Self {
        Self {
            cartridge,
            started: None,
            menu_cursor: None,
        }
    }

    pub(super) fn advance(&mut self, format: &Format, input: Input<'_>) -> Result<Output, Refusal> {
        let output = advance(&mut self.cartridge, format, input)?;
        match &output.receipt {
            Receipt::Start(start) | Receipt::Choice(ChoiceProgress::Loaded(start)) => {
                self.started = Some(Box::new(start.run.clone()));
            }
            Receipt::Choice(ChoiceProgress::SelectionWritten) => self.started = None,
            Receipt::Menu(report) => self.menu_cursor = Some(report.cursor),
            Receipt::Boot(_) => self.menu_cursor = None,
            Receipt::None | Receipt::Read(_) => {}
        }
        Ok(output)
    }
}

pub(super) struct SaveFilesPin;

impl Pinned for SaveFilesPin {
    type State = Box<State>;
    const NAME: &'static str = "save-files-checkpoint-v1";
    // The declared-length vectors reject the shape tracer's one-element probe. The checkpoint
    // wire test pins the hash of representative encoded states, including both pending actions.
    const SHAPE: u64 = 0xBF1D_8D8D_D2EC_7B39;
}

#[cfg(test)]
mod tests {
    use super::{SaveFilesPin, State, power_on};
    use crate::exact::save_files::Format;
    use canonical::model::{Model, Pinned};
    use save_files::SaveFormat;
    use types::file_select::Slot;
    use types::save_files::{
        BatteryImage, CheckPair, Destination, EncodedRun, IMAGE_BYTES, MenuInspection, OpaqueValue,
        PendingAction, StoredArea,
    };

    #[test]
    fn checkpoint_wire_preserves_the_save_state_and_holds_its_pin() {
        let format = Format::test_format();
        let image = BatteryImage::new(
            (0..IMAGE_BYTES)
                .map(|index| (index.wrapping_mul(53) ^ (index >> 4)).to_le_bytes()[0])
                .collect(),
        )
        .unwrap();
        let mut state = State::new(power_on(&format, &image));
        state.cartridge.menu = MenuInspection::Inspected;
        state.cartridge.active = Some(Slot::Second);
        state.menu_cursor = Some(Slot::Third);
        let mut run = format.decode(&EncodedRun::zeroed());
        run.restart.destination = Destination::Unrecognized(OpaqueValue::Word(0x9234));
        run.restart.area = StoredArea::Unrecognized(OpaqueValue::Byte(0xcd));
        state.started = Some(Box::new(run));
        let mut fixture = Vec::new();
        for pending in [
            PendingAction::Idle,
            PendingAction::Choice(Slot::Second),
            PendingAction::Write {
                slot: Slot::Second,
                pair: CheckPair {
                    sum: 0x9876,
                    complement: 0x6789,
                },
            },
        ] {
            state.cartridge.pending = pending;
            let model = Model::<SaveFilesPin>(Box::new(state.clone()));
            let bytes = canonical::to_bytes(&model);
            assert_eq!(
                canonical::from_bytes::<Model<SaveFilesPin>>(&bytes).unwrap(),
                model
            );
            fixture.extend(bytes);
        }
        let digest = data::rom::sha256(&fixture);
        assert_eq!(
            u64::from_le_bytes(*digest.first_chunk().unwrap()),
            SaveFilesPin::SHAPE
        );
    }
}
