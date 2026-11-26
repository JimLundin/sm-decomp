//! Spec: `WriteFile`. The two automatic saves, each published payload first, then checks.

use types::save_files::{
    Destination, ExploredCells, PendingAction, RestartPoint, RunSnapshot, StoredArea,
};
use types::{Area, StationId};

use crate::files::{Refusal, SaveFiles};
use crate::format::SaveFormat;
use crate::read::integrity_pair;

/// Spec: `WriteFile`. The station the arrival save records: station zero.
const ARRIVAL_STATION: StationId = StationId(0);

/// Spec: `WriteFile`. The cut that saves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveCut {
    /// The trip to Ceres ends: restart at the Ceres arrival, in Ceres, at station zero.
    CeresArrival,
    /// The Ceres escape fade ends: restart with the flight to Zebes, where she is now.
    CeresEscape {
        /// Her current area.
        area: Area,
        /// Her current station.
        station: StationId,
    },
}

impl SaveCut {
    /// The area she is in as the cut saves, whose live map replaces the remembered one.
    const fn current_area(self) -> Area {
        match self {
            Self::CeresArrival => Area::Ceres,
            Self::CeresEscape { area, .. } => area,
        }
    }

    /// The restart point the cut records.
    const fn restart_point(self) -> RestartPoint {
        match self {
            Self::CeresArrival => RestartPoint {
                destination: Destination::CeresArrival,
                area: StoredArea::Known(Area::Ceres),
                station: ARRIVAL_STATION,
            },
            Self::CeresEscape { area, station } => RestartPoint {
                destination: Destination::ZebesFlight,
                area: StoredArea::Known(area),
                station,
            },
        }
    }
}

/// Spec: `WriteFile`. How far a save has finished, as the caller reports it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WriteStage {
    /// The new payload is on the cartridge, beside the preceding checks.
    PayloadWritten,
    /// Both new check pairs are on the cartridge too.
    ChecksWritten,
}

impl<F: SaveFormat> SaveFiles<F> {
    /// Spec: `WriteFile`. Save the live run to the active slot as `cut` ends. The run is
    /// captured once, with the cut's restart point and her current area's live map in place
    /// of its remembered map, and encoded once; its payload is published at once and its two
    /// check pairs at `ChecksWritten`, now if the request reached it. No other slot, spare word
    /// or retained byte changes.
    ///
    /// # Errors
    ///
    /// `ActionPending` while a choice or write stands between its milestones, and
    /// `NoActiveSlot` when no file is chosen.
    pub fn write_file(
        &mut self,
        cut: SaveCut,
        run: &RunSnapshot,
        live_map: &ExploredCells,
        reached: WriteStage,
    ) -> Result<(), Refusal> {
        self.admit()?;
        let slot = self.state.active.ok_or(Refusal::NoActiveSlot)?;
        let mut captured = run.clone();
        captured.maps[cut.current_area()] = live_map.clone();
        captured.restart = cut.restart_point();
        let payload = self.format.encode(&captured);
        let pair = integrity_pair(&payload);
        self.state.cartridge.slots[slot].payload = payload;
        self.state.pending = PendingAction::Write { slot, pair };
        match reached {
            WriteStage::PayloadWritten => Ok(()),
            WriteStage::ChecksWritten => self.continue_write(),
        }
    }

    /// Spec: `WriteFile`. The pending save reaches `ChecksWritten`: both check pairs of its slot
    /// become the pair of the payload it published.
    ///
    /// # Errors
    ///
    /// `NothingToComplete` when no write is pending.
    pub fn continue_write(&mut self) -> Result<(), Refusal> {
        let PendingAction::Write { slot, pair } = self.state.pending else {
            return Err(Refusal::NothingToComplete);
        };
        let record = &mut self.state.cartridge.slots[slot];
        record.primary = pair;
        record.mirror = pair;
        self.state.pending = PendingAction::Idle;
        Ok(())
    }
}
