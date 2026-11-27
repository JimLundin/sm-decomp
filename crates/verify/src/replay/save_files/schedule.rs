//! Ordered cartridge requests and their coordinates in the recorded movie.

use crate::cache::Cache;
use crate::lock::{RowId, Start};
use crate::replay::{Sources, data_failure, index::Case, readers::record, result::Outcome};
use crate::report::seeds;
use game::exact::save_files::BootProgress;
use serde::{Deserialize, Serialize};
use types::save_files::{BatteryImage, IMAGE_BYTES, PAYLOAD_WORDS};

use super::{Capture, Expected, Inputs, battery, observed};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum Request {
    Hold,
    Diagnostic(#[serde(with = "Frontier")] BootProgress),
    Boot,
    Menu,
    Choose,
    Start,
    Arrival,
    Finish,
    Escape,
}

#[derive(Deserialize, Serialize)]
#[serde(remote = "BootProgress", rename_all = "kebab-case")]
enum Frontier {
    Clearing(u16),
    Testing(u16),
    Restoring(u16),
    ClearingFiles(u16),
}

impl Request {
    fn valid(self) -> bool {
        match self {
            Self::Diagnostic(BootProgress::ClearingFiles(count)) => {
                usize::from(count) <= PAYLOAD_WORDS * 3
            }
            Self::Diagnostic(
                BootProgress::Clearing(word)
                | BootProgress::Testing(word)
                | BootProgress::Restoring(word),
            ) => usize::from(word) <= IMAGE_BYTES / 2,
            Self::Hold
            | Self::Boot
            | Self::Menu
            | Self::Choose
            | Self::Start
            | Self::Arrival
            | Self::Finish
            | Self::Escape => true,
        }
    }

    const fn starts_write(self) -> bool {
        matches!(self, Self::Arrival | Self::Escape)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Step {
    row: String,
    cut: u64,
    request: Request,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
/// The ordered cartridge requests and source cuts for a battery witness.
pub struct SaveSchedule(Vec<Step>);

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(in crate::replay) enum SaveSource {
    DiagnosticFrontier,
    ArrivalCapture,
    EscapeCapture,
}

impl SaveSchedule {
    pub(in crate::replay) fn uses(&self, source: SaveSource) -> bool {
        self.0.iter().any(|step| {
            matches!(
                (source, step.request),
                (SaveSource::DiagnosticFrontier, Request::Diagnostic(_))
                    | (SaveSource::ArrivalCapture, Request::Arrival)
                    | (SaveSource::EscapeCapture, Request::Escape)
            )
        })
    }

    pub(in crate::replay) fn valid(&self, case: &Case) -> bool {
        !self.0.is_empty()
            && u64::try_from(self.0.len()) == Ok(case.end.cut_inclusive)
            && self
                .0
                .iter()
                .all(|step| !step.row.is_empty() && step.request.valid())
    }

    pub(in crate::replay) fn pending_boundaries(&self, cuts: usize) -> usize {
        let mut pending = false;
        self.0
            .iter()
            .take(cuts)
            .filter(|step| {
                if step.request.starts_write() {
                    pending = true;
                } else if matches!(step.request, Request::Finish) {
                    pending = false;
                }
                pending
            })
            .count()
    }

    pub(super) fn read(
        &self,
        case: &Case,
        sources: &Sources,
    ) -> Result<(BatteryImage, Inputs, Expected), Outcome> {
        let lock = crate::replay::read_lock(sources)?;
        let locked = |id: &str| {
            lock.row(&RowId(id.to_owned()))
                .ok_or_else(|| data_failure(None, "battery source row"))
        };
        let open = |row| {
            Cache::open(&sources.cache, row).map_err(|_| data_failure(None, "pinned battery cache"))
        };
        let mut row = locked(&case.row)?;
        if row.start != Start::PowerOn {
            return Err(data_failure(Some(0), "cold battery source"));
        }
        let movie = seeds::row_movie(row);
        let mut cache = open(row)?;
        let mut previous = record(&cache, 0)?;
        let origin = battery(std::mem::take(&mut previous.sram).into_vec())?;
        let mut inputs = Vec::with_capacity(self.0.len());
        let mut expected = Vec::with_capacity(self.0.len());
        let mut first = 0u64;
        let mut prior_frame = 0u64;
        for step in &self.0 {
            if step.row != row.id.0 {
                row = locked(&step.row)?;
                if seeds::row_movie(row) != movie {
                    return Err(data_failure(Some(step.cut), "battery movie"));
                }
                let root = sources
                    .lock
                    .ancestors()
                    .nth(3)
                    .ok_or_else(|| data_failure(None, "battery lock path"))?;
                first = seeds::first_frame(root, row)
                    .map_err(|_| data_failure(Some(step.cut), "battery seed frame"))?;
                cache = open(row)?;
            }
            let frame = first
                .checked_add(step.cut)
                .ok_or_else(|| data_failure(Some(step.cut), "battery movie coordinate"))?;
            if frame <= prior_frame {
                return Err(data_failure(
                    Some(step.cut),
                    "ordered battery movie coordinates",
                ));
            }
            let capture = if step.request.starts_write() {
                if prior_frame.checked_add(1) != Some(frame) {
                    return Err(data_failure(Some(step.cut), "preceding live save frame"));
                }
                Some(Box::new(Capture::from_record(&previous).ok_or_else(
                    || data_failure(Some(prior_frame), "preceding live save inputs"),
                )?))
            } else {
                None
            };
            inputs.push((step.request, capture));
            previous = record(&cache, step.cut)?;
            expected.push(observed(battery(
                std::mem::take(&mut previous.sram).into_vec(),
            )?));
            prior_frame = frame;
        }
        Ok((origin, Inputs(inputs), Expected(expected)))
    }
}
