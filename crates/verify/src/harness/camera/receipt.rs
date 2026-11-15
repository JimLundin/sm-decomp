//! Mandatory receipt-only proof; it makes no claim to resume a partly completed view move.

use super::execution::{self, Phase};
use super::{CameraDomain, Placement, RECEIPT, Skip, Upstream, quantity, source, upstream, word};
use crate::harness::gameplay_caller::{self, Caller};
use crate::harness::{Domain, Entry, Field, FieldSet, Interval, Mask, Uncompared};
use oracle_record::FrameRecord;
use types::{Pair, Parts, PlacementY};
use world_camera::{Camera, RememberedPosition, Rules};

/// Independent proof of the camera-owned vertical receipt.
pub struct ReceiptDomain {
    camera: CameraDomain,
    caller: Caller,
}

/// Content that the independent receipt proof could not import.
#[derive(Debug)]
pub enum LoadError {
    /// Camera and upstream movement content is invalid.
    Camera(crate::bundle::ContentError),
    /// The gameplay caller's imported source content is invalid.
    Caller(super::ExecutionError),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Compared receipt projection, separate from view and remembered centre.
pub struct Fraction {
    vertical_fraction: u16,
}

/// Upstream values and independently established measurement phase.
pub struct Inputs {
    upstream: Upstream,
    phase: Phase,
    alternative: bool,
}

fn physical_centre(record: &FrameRecord) -> Result<types::Position, Skip> {
    Ok(Pair {
        x: quantity(record, source::SAMUS_X_POS, source::SAMUS_X_SUBPOS)?,
        y: quantity(record, source::SAMUS_Y_POS, source::SAMUS_Y_SUBPOS)?,
    })
}

impl ReceiptDomain {
    /// The receipt-only proof using the camera's ordinary upstream adapter.
    ///
    /// # Errors
    /// The camera or movement content could not be imported.
    pub fn load(image: &[u8]) -> Result<Self, LoadError> {
        Ok(Self {
            camera: CameraDomain::load(image).map_err(LoadError::Camera)?,
            caller: Caller::load(image).map_err(|error| {
                LoadError::Caller(super::ExecutionError::Capability(error.message()))
            })?,
        })
    }

    fn inputs(&self, from: &FrameRecord, next: &FrameRecord) -> Result<Inputs, Skip> {
        let mut upstream = match self.camera.world(from, next) {
            Ok(upstream) => upstream,
            // The selected gameplay call completes after setting HitDoor. A stable main-loop
            // endpoint proves that the remainder, including follow, has completed.
            Err(Skip::GameState(9)) if execution::main_wait(next).map_err(Skip::Execution)? => {
                let frames = word(next, source::NMI_FRAME_COUNTER_WORD)?
                    .wrapping_sub(word(from, source::NMI_FRAME_COUNTER_WORD)?);
                if frames != 1 {
                    return Err(Skip::Frames(frames));
                }
                if word(from, source::ROOM_PTR)? != word(next, source::ROOM_PTR)? {
                    return Err(Skip::RoomChange);
                }
                upstream(next)?
            }
            Err(Skip::GameState(9 | 11)) => {
                return Err(Skip::Execution(super::ExecutionError::Capability(
                    "door loading and actor memory-write capability unsupported",
                )));
            }
            Err(skip) => return Err(skip),
        };
        gameplay_caller::completed(from, next)
            .map_err(|error| Skip::Execution(super::ExecutionError::Capability(error.message())))?;
        if execution::main_wait(from).map_err(Skip::Execution)? {
            self.caller.receipt_start(from).map_err(|error| {
                Skip::Execution(super::ExecutionError::Capability(error.message()))
            })?;
        } else {
            return Err(Skip::Execution(super::ExecutionError::Capability(
                "asynchronous writes in interrupted camera suffix unsupported",
            )));
        }
        execution::centre_tail(from, next).map_err(Skip::Execution)?;
        upstream.samus.position = physical_centre(next)?;
        Ok(Inputs {
            upstream,
            phase: execution::phase_between(from, next).map_err(Skip::Execution)?,
            alternative: word(next, source::SLOW_GRAPPLE_SCROLLING)? != 0,
        })
    }

    fn run_step(camera: &mut Camera, placement: Placement, inputs: &mut Inputs) {
        match inputs.phase {
            Phase::BeforeMeasurement if !inputs.alternative => {
                inputs.upstream.samus.placed = placement.placed().map(|y| PlacementY(y.pixel().0));
                *camera = camera.step(&Rules::REFERENCE, &inputs.upstream.frame());
            }
            Phase::BeforeMeasurement | Phase::AfterMeasurementBeforeRemembering => {
                // Alternative follow and the measured vertical tail both reach Remembering.
                // The centre comes from Samus's own position, never expected camera output.
                *camera =
                    camera.with_remembered(RememberedPosition(inputs.upstream.samus.position));
            }
            Phase::AfterRemembering => {}
        }
    }
}

impl Domain for ReceiptDomain {
    const NAME: &'static str = "world-camera-receipt";
    type State = Camera;
    type Inputs = Placement;
    type World = Inputs;
    type Fields = Fraction;
    type Skip = Skip;
    const FIELDS: &'static [Field<Fraction>] =
        &[crate::domain_field!("vertical_fraction", vertical_fraction)];

    fn decode(&self, from: &FrameRecord, interval: Interval) -> Result<(Camera, Placement), Skip> {
        self.camera
            .decode(from, interval)
            .map_err(|skip| match skip {
                Skip::GameState(9 | 11) => Skip::Execution(super::ExecutionError::Capability(
                    "door loading and actor memory-write capability unsupported",
                )),
                skip @ (Skip::Unreadable(_)
                | Skip::Room(_)
                | Skip::GameState(_)
                | Skip::Frames(_)
                | Skip::RoomChange
                | Skip::MovementCaller(_)
                | Skip::Execution(_)) => skip,
            })
    }
    fn world(&self, from: &FrameRecord, next: &FrameRecord) -> Result<Inputs, Skip> {
        self.inputs(from, next)
    }
    fn masks(
        &self,
        from: &FrameRecord,
        next: &FrameRecord,
        placement: &Placement,
    ) -> Result<Vec<Mask>, Skip> {
        let inputs = self.inputs(from, next)?;
        if inputs.phase != Phase::BeforeMeasurement || inputs.alternative {
            return Ok(Vec::new());
        }
        Ok(CameraDomain::writer_masks(from, next, *placement)?
            .into_iter()
            .filter(|mask| mask.fields.overlaps(RECEIPT))
            .map(|mut mask| {
                mask.fields = FieldSet::single(0);
                mask
            })
            .collect())
    }
    fn step(&self, camera: &mut Camera, placement: &Placement, inputs: &mut Inputs) {
        Self::run_step(camera, *placement, inputs);
    }
    fn fields(&self, camera: &Camera) -> Fraction {
        Fraction {
            vertical_fraction: camera.vertical_fraction().0,
        }
    }
    fn expected(&self, next: &FrameRecord) -> Result<Fraction, Skip> {
        Ok(Fraction {
            vertical_fraction: word(next, source::CAMERA_VERTICAL_FRACTION)?,
        })
    }
    fn uncompared(&self, skip: &Skip) -> Uncompared {
        self.camera.uncompared(skip)
    }
    fn behaviours(&self) -> Vec<Entry> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::recorded_camera;
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::io::Write as _;
    use std::path::Path;

    #[test]
    fn m0_receipt_is_carried_through_supported_gameplay_prefix() {
        let Some((cache, rom)) = crate::harness::tests::environment() else {
            return;
        };
        let image = std::fs::read(rom).unwrap();
        let domain = ReceiptDomain::load(&image).unwrap();
        let opened = crate::harness::open_cached(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenarios/locks/lsnes.toml"),
            &cache,
            "chain-any-0003-DFD7",
        )
        .unwrap();
        let records: Vec<_> = opened
            .entries()
            .iter()
            .map(|entry| opened.record(entry).unwrap())
            .collect();
        let mut model = recorded_camera(&records[0]).unwrap();
        let (mut held, mut zero, mut compared, mut null) = (
            1_usize,
            usize::from(model.vertical_fraction().0 == 0),
            0_usize,
            0_usize,
        );
        let mut per_frame_matched = 0;
        let mut first_carried_difference = None;
        let mut first_per_frame_difference = None;
        let mut first_unsupported = None;
        for pair in records.windows(2) {
            let [from, next] = pair else {
                panic!("a window contains two records")
            };
            let mut per_frame = recorded_camera(from).unwrap();
            let (_, placement) = match domain.decode(from, Interval::ending_at(next)) {
                Ok(decoded) => decoded,
                Err(error) => {
                    first_unsupported = Some((next.index, error.to_string()));
                    break;
                }
            };
            let mut inputs = match domain.inputs(from, next) {
                Ok(inputs) => inputs,
                Err(error) => {
                    first_unsupported = Some((next.index, error.to_string()));
                    break;
                }
            };
            assert!(
                domain.masks(from, next, &placement).unwrap().is_empty(),
                "M0 receipt input availability at {}",
                next.index
            );
            ReceiptDomain::run_step(&mut model, placement, &mut inputs);
            ReceiptDomain::run_step(&mut per_frame, placement, &mut inputs);
            let expected = word(next, source::CAMERA_VERTICAL_FRACTION).unwrap();
            let carried_matches = model.vertical_fraction().0 == expected;
            let transition_matches = per_frame.vertical_fraction().0 == expected;
            if !carried_matches {
                first_carried_difference.get_or_insert(next.index);
            }
            if !transition_matches {
                first_per_frame_difference.get_or_insert(next.index);
            }
            held += usize::from(carried_matches);
            per_frame_matched += usize::from(transition_matches);
            zero += usize::from(expected == 0);
            compared += 1;
            null += usize::from(word(from, source::CAMERA_VERTICAL_FRACTION).unwrap() == expected);
        }
        let _ = writeln!(
            std::io::stdout(),
            "M0 receipt-only carry {held}/{} vs constant-zero {zero}; per-frame {per_frame_matched}/{compared} vs null {null}; vertical_fraction first carried difference {first_carried_difference:?}, first per-frame difference {first_per_frame_difference:?}",
            records.len()
        );
        let _ = writeln!(
            std::io::stdout(),
            "M0 first unsupported {first_unsupported:?}; remaining transitions {}",
            records.len().saturating_sub(compared.saturating_add(1))
        );
        assert_eq!(
            (held, zero, per_frame_matched, compared, null),
            (183, 100, 182, 182, 95)
        );
        assert_eq!(
            first_unsupported.as_ref().map(|(frame, _)| *frame),
            Some(183)
        );
        assert_eq!(held, compared.saturating_add(1));
        assert_eq!(per_frame_matched, compared);
        assert!(held > zero);
        assert!(per_frame_matched > null);
        assert!(first_unsupported.is_some());
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
    struct Counts {
        frames: u64,
        compared: u64,
        matched: u64,
        masked: u64,
        skipped: u64,
        null: u64,
        free_held: u64,
        free_null: u64,
        free_masked: u64,
        free_out_of_scope: u64,
        free_unreached: u64,
        unverified: u64,
        defect_frames: u64,
    }

    #[derive(Debug, Deserialize, Serialize)]
    struct Accounting {
        row: String,
        #[serde(flatten)]
        counts: Counts,
        defects: std::collections::BTreeMap<String, u64>,
    }

    #[derive(Deserialize, Serialize)]
    struct Floors {
        row: Vec<Accounting>,
    }

    #[test]
    fn every_supported_camera_receipt_matches_per_frame_and_carried_free() {
        let Some((cache, rom)) = crate::harness::tests::environment() else {
            return;
        };
        let image = std::fs::read(rom).unwrap();
        let domain = ReceiptDomain::load(&image).unwrap();
        let expected: Floors = toml::from_str(include_str!("receipt_floors.toml")).unwrap();
        let lock = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenarios/locks/lsnes.toml");
        let mut fresh = Floors { row: Vec::new() };
        let mut failed_rows = Vec::new();
        for row in crate::harness::tests::cached_rows(&cache) {
            let report = crate::harness::run_cached(&domain, &lock, &cache, &row).unwrap();
            let free = crate::harness::free::run_cached(&domain, &lock, &cache, &row).unwrap();
            let _ = writeln!(
                std::io::stdout(),
                "{row}: {report}
{free}"
            );
            if report.fields[0].mismatched != 0 || free.diverged != 0 {
                failed_rows.push(row.clone());
            }
            fresh.row.push(Accounting {
                row,
                counts: Counts {
                    frames: report.frames,
                    compared: report.compared,
                    matched: report.matched,
                    masked: report.masked_whole,
                    skipped: report.skipped.values().sum(),
                    null: report.null_matched,
                    free_held: free.held,
                    free_null: free.null_held,
                    free_masked: free.masked_frames,
                    free_out_of_scope: free.out_of_scope_frames,
                    free_unreached: free.unreached_frames,
                    unverified: report.fields[0].unverified,
                    defect_frames: report.defects.values().sum(),
                },
                defects: report.defects,
            });
        }
        let _ = writeln!(
            std::io::stdout(),
            "RECEIPT FLOORS
{}",
            toml::to_string(&fresh).unwrap()
        );
        assert!(
            failed_rows.is_empty(),
            "receipt proof failed for {failed_rows:?}"
        );
        for actual in &fresh.row {
            let floor = expected
                .row
                .iter()
                .find(|floor| floor.row == actual.row)
                .unwrap_or_else(|| panic!("{}: missing receipt floor", actual.row));
            assert_eq!(
                actual.counts, floor.counts,
                "{}: receipt accounting",
                actual.row
            );
        }
    }
}
