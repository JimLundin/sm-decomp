//! Spec: `ReadSlot`. Reading one file: accept either matching check pair, or clear the file.

use types::file_select::Slot;
use types::save_files::{CheckPair, EncodedRun, RunSnapshot, Supply, WorkingRun};
use types::{Area, PlayTime, StationId};

use crate::files::{Refusal, SaveFiles};
use crate::format::SaveFormat;

/// Spec: `ReadSlot`. The area a failed read reports: Crateria.
pub const EMPTY_FILE_AREA: Area = Area::Crateria;
/// Spec: `ReadSlot`. The station a failed read reports: station zero.
pub const EMPTY_FILE_STATION: StationId = StationId(0);

/// Spec: `ReadSlot`. The check pair of a payload: `sum = sum(words) mod 65536` and
/// `complement = 65535 - sum`.
#[must_use]
pub fn integrity_pair(payload: &EncodedRun) -> CheckPair {
    let sum = payload
        .as_slice()
        .iter()
        .fold(0, |sum: u16, &word| sum.wrapping_add(word));
    CheckPair {
        sum,
        complement: u16::MAX.wrapping_sub(sum),
    }
}

/// What a read found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReadOutcome {
    /// A check pair matched: the payload's run.
    Decoded(Box<RunSnapshot>),
    /// Neither pair matched: the file is empty, at the default location.
    Empty {
        /// The default area, Crateria.
        area: Area,
        /// The default station, zero.
        station: StationId,
    },
}

/// Spec: `ReadSlot`. A read's receipt: the payload as it was copied, and what it held.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadReceipt {
    /// The slot read.
    pub slot: Slot,
    /// The payload as the read copied it, before any clear.
    pub copied: EncodedRun,
    /// What the read found.
    pub outcome: ReadOutcome,
}

/// Spec: `MenuFiles`. A file's summary on the menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileSummary {
    /// The slot holds no file.
    Empty,
    /// The slot holds a file.
    Present {
        /// Its energy and energy capacity.
        energy: Supply,
        /// Its play time.
        play_time: PlayTime,
    },
}

impl ReadReceipt {
    /// Whether a check pair matched.
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        matches!(self.outcome, ReadOutcome::Decoded(_))
    }

    /// The file's summary: whether the slot held a file, and if so its energy and play time.
    #[must_use]
    pub fn summary(&self) -> FileSummary {
        match &self.outcome {
            ReadOutcome::Decoded(run) => FileSummary::Present {
                energy: run.profile.energy,
                play_time: run.profile.play_time,
            },
            ReadOutcome::Empty { .. } => FileSummary::Empty,
        }
    }
}

impl<F: SaveFormat> SaveFiles<F> {
    /// Spec: `ReadSlot`. Read one file, unless a choice or write is pending.
    ///
    /// # Errors
    ///
    /// `ActionPending` while a choice or write stands between its milestones.
    pub fn read_slot(&mut self, slot: Slot) -> Result<ReadReceipt, Refusal> {
        self.admit()?;
        Ok(self.read(slot))
    }

    /// Spec: `ReadSlot`. Read one file. Either pair matching the payload decodes it, and its
    /// profile and maps become the working ones. Neither matching clears the cartridge's
    /// payload, keeping the copy in the receipt and the working profile and maps as they were.
    /// Both pairs stay as they are either way.
    pub(crate) fn read(&mut self, slot: Slot) -> ReadReceipt {
        let record = &mut self.state.cartridge.slots[slot];
        let expected = integrity_pair(&record.payload);
        let copied = record.payload.clone();
        let outcome = if record.primary == expected || record.mirror == expected {
            let run = self.format.decode(&copied);
            self.state.working = WorkingRun {
                profile: run.profile.clone(),
                maps: run.maps.clone(),
            };
            ReadOutcome::Decoded(Box::new(run))
        } else {
            record.payload = EncodedRun::zeroed();
            ReadOutcome::Empty {
                area: EMPTY_FILE_AREA,
                station: EMPTY_FILE_STATION,
            }
        };
        ReadReceipt {
            slot,
            copied,
            outcome,
        }
    }
}
