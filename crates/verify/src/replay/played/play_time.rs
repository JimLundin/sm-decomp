//! Independently read the complete elapsed play clock.

use crate::replay::{data_failure, result::Outcome};
use types::{PlayFrames, PlayHours, PlayMinutes, PlaySeconds, PlayTime};

pub(super) fn read(wram: &[u8]) -> Result<PlayTime, Outcome> {
    let invalid = || data_failure(None, "elapsed play time");
    let component = |offset: usize| {
        let bytes: [u8; 2] = wram
            .get(offset..offset.saturating_add(2))?
            .try_into()
            .ok()?;
        u8::try_from(u16::from_le_bytes(bytes)).ok()
    };
    Ok(PlayTime {
        hours: component(0x09e0)
            .and_then(PlayHours::new)
            .ok_or_else(invalid)?,
        minutes: component(0x09de)
            .and_then(PlayMinutes::new)
            .ok_or_else(invalid)?,
        seconds: component(0x09dc)
            .and_then(PlaySeconds::new)
            .ok_or_else(invalid)?,
        frames: component(0x09da)
            .and_then(PlayFrames::new)
            .ok_or_else(invalid)?,
    })
}

pub(super) fn observation(time: PlayTime) -> crate::replay::compare::Observation {
    crate::replay::compare::Observation {
        field: "game-time.elapsed".into(),
        bytes: [
            time.hours.get(),
            time.minutes.get(),
            time.seconds.get(),
            time.frames.get(),
        ]
        .to_vec(),
    }
}

#[cfg(test)]
mod continuous {
    use super::super::{Sources, Stop, open_rows, power_on, read_lock, tests};
    use crate::cache::Cache;
    use crate::lock::{Lock, RowId, Sha256};
    use crate::replay::result::Outcome;
    use serde::Deserialize;
    use std::collections::BTreeMap;

    #[derive(Deserialize)]
    struct Fixture {
        indexed_case: String,
        consumer: String,
        fields: [String; 4],
        row: Vec<PinnedRow>,
        case: Vec<Case>,
    }

    #[derive(Deserialize)]
    struct PinnedRow {
        id: String,
        seed_record: String,
        pads: String,
        cuts: [u64; 2],
    }

    #[derive(Deserialize)]
    struct Case {
        id: String,
        spans: Vec<Span>,
    }

    #[derive(Deserialize)]
    struct Span {
        row: String,
        cuts: [u64; 2],
    }

    const BOOT: &str = "any-power-on";
    const LAST_SUPPORTED: u64 = 13_017;

    #[derive(Clone, Copy)]
    enum End {
        ShaftBoundary,
        FirstCarry,
    }

    /// The required clock prefix uses the continuous clock fixture through the hall.
    #[test]
    #[ignore = "pinned record cache and player ROM"]
    fn ceres_clock_prefix() {
        check(End::ShaftBoundary);
    }

    #[test]
    #[ignore = "pinned record cache and player ROM"]
    fn ceres_clock_first_carry() {
        check(End::FirstCarry);
    }

    fn bind_rows<'a>(
        pinned_rows: &'a [PinnedRow],
        sources: &Sources,
        lock: &Lock,
    ) -> BTreeMap<&'a str, (u64, u64)> {
        let root = sources.lock.ancestors().nth(3).unwrap();
        let mut rows = open_rows(sources, lock, BOOT).unwrap();
        let mut bounds = BTreeMap::new();
        let mut previous_end: Option<u64> = None;
        for pinned in pinned_rows {
            let row = lock.row(&RowId(pinned.id.clone())).unwrap();
            assert_eq!(
                row.seed_record,
                Some(Sha256::try_from(pinned.seed_record.as_str()).unwrap())
            );
            assert_eq!(row.pads, Sha256::try_from(pinned.pads.as_str()).unwrap());
            assert_eq!(pinned.cuts[0], 0);
            assert!(pinned.cuts[1] < row.frames.0);
            Cache::open(&sources.cache, row).unwrap();
            let first = crate::report::seeds::first_frame(root, row).unwrap();
            let last = first.checked_add(pinned.cuts[1]).unwrap();
            if let Some(previous_end) = previous_end {
                assert_eq!(first, previous_end.checked_add(1).unwrap());
            }
            previous_end = Some(last);
            assert!(bounds.insert(pinned.id.as_str(), (first, last)).is_none());
        }

        for pinned in pinned_rows {
            let (first, last) = bounds[pinned.id.as_str()];
            assert_eq!(
                rows.place(first).unwrap(),
                (pinned.id.clone(), pinned.cuts[0])
            );
            assert_eq!(
                rows.place(last).unwrap(),
                (pinned.id.clone(), pinned.cuts[1])
            );
        }
        bounds
    }

    fn check(end: End) {
        let fixture: Fixture = toml::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../docs/behaviours/game-time-ceres.sources.toml"
        )))
        .unwrap();
        assert_eq!(fixture.indexed_case, "game-time/ceres-continuous-clock");
        assert_eq!(fixture.consumer, "played::play_time");
        assert_eq!(fixture.fields, ["hours", "minutes", "seconds", "frames"]);

        let sources = tests::sources();
        let lock = read_lock(&sources).unwrap();
        let bounds = bind_rows(&fixture.row, &sources, &lock);
        let first = bounds[fixture.row[0].id.as_str()].0;
        let carry = fixture
            .case
            .iter()
            .find(|case| case.id == "R-ceres-first-carry")
            .unwrap();
        for case in &fixture.case {
            assert!(!case.spans.is_empty(), "{}", case.id);
            for span in &case.spans {
                let (first, last) = bounds[span.row.as_str()];
                let [low, high] = span.cuts;
                assert!(
                    low <= high && high <= last.checked_sub(first).unwrap(),
                    "{}",
                    case.id
                );
            }
        }
        let carry = &carry.spans[0];
        let carry_cut = bounds[carry.row.as_str()]
            .0
            .checked_add(carry.cuts[1])
            .unwrap();

        let (last, stop) = match end {
            End::ShaftBoundary => (
                LAST_SUPPORTED,
                Stop::At(Outcome::Unsupported {
                    cut: Some(LAST_SUPPORTED + 1),
                    dependency: "played frame Load(TurningShaftVent)".into(),
                }),
            ),
            End::FirstCarry => (carry_cut, Stop::End),
        };
        let attempted = last
            .checked_add(u64::from(matches!(end, End::ShaftBoundary)))
            .unwrap();
        let mut rows = open_rows(&sources, &lock, BOOT).unwrap();
        let report = power_on::play(&mut rows, &sources.rom, BOOT, 0, attempted).unwrap();
        let compared = rows.clock_cuts.as_slice();
        tests::diagnostic(format_args!(
            "{} prefix held={} clock_comparisons={} first={:?} last={:?} stop={:?}",
            fixture.indexed_case,
            report.held,
            compared.len(),
            compared.first(),
            compared.last(),
            report.stop
        ));
        assert_eq!(report.stop, stop);
        assert_eq!(report.held, last);
        assert!(compared.iter().copied().eq(first..=last));
        match end {
            End::ShaftBoundary => assert_eq!(compared.len(), 4_703),
            End::FirstCarry => assert_eq!(compared.len(), 253),
        }
        for case in fixture.case {
            for span in case.spans {
                let row_start = bounds[span.row.as_str()].0;
                let first_cut = row_start.checked_add(span.cuts[0]).unwrap();
                if first_cut <= last {
                    let held_end = span.cuts[1].min(last.checked_sub(row_start).unwrap());
                    tests::diagnostic(format_args!(
                        "{} {} cuts {}..={}{}",
                        case.id,
                        span.row,
                        span.cuts[0],
                        held_end,
                        if held_end == span.cuts[1] {
                            ""
                        } else {
                            " (partial)"
                        },
                    ));
                }
            }
        }
    }
}
