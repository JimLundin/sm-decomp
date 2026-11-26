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
    use super::super::{Stop, open_rows, power_on, read_lock, tests};
    use crate::cache::Cache;
    use crate::lock::{RowId, Sha256};
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Fixture {
        indexed_case: String,
        consumer: String,
        opportunities: u64,
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

    #[test]
    #[ignore = "pinned record cache and player ROM"]
    fn ceres_continuous_clock() {
        const BOOT: &str = "any-power-on";
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
        let root = sources.lock.ancestors().nth(3).unwrap();
        let mut rows = open_rows(&sources, &lock, BOOT).unwrap();
        let mut bounds = std::collections::BTreeMap::new();
        let mut previous_end = None;
        for pinned in &fixture.row {
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
            let last = first + pinned.cuts[1];
            if let Some(previous_end) = previous_end {
                assert_eq!(first, previous_end + 1);
            }
            previous_end = Some(last);
            assert!(bounds.insert(pinned.id.as_str(), (first, last)).is_none());
        }

        for pinned in &fixture.row {
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
        let first = bounds[fixture.row[0].id.as_str()].0;
        let end = previous_end.unwrap();
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
                assert!(low <= high && high <= last - first, "{}", case.id);
            }
        }
        let carry = &carry.spans[0];
        let carry_cut = bounds[carry.row.as_str()].0 + carry.cuts[1];

        let control = std::env::var_os("SM_CERES_CLOCK_CARRY_ONLY").is_some();
        let end = if control { carry_cut } else { end };
        let mut rows = open_rows(&sources, &lock, BOOT).unwrap();
        let report = power_on::play(&mut rows, &sources.rom, BOOT, 0, end).unwrap();
        let compared = rows.clock_cuts.as_slice();
        tests::diagnostic(format_args!(
            "{} held={} clock_comparisons={} first={:?} last={:?} stop={:?}",
            fixture.indexed_case,
            report.held,
            compared.len(),
            compared.first(),
            compared.last(),
            report.stop
        ));
        assert!(compared.iter().copied().eq(first..=report.held));
        assert_eq!(report.stop, Stop::End);
        assert_eq!(report.held, end);
        if !control {
            let elapsed = super::read(&rows.record(end).unwrap().wram).unwrap();
            let frames = u64::from(elapsed.frames.get())
                + 60 * u64::from(elapsed.seconds.get())
                + 3_600 * u64::from(elapsed.minutes.get())
                + 216_000 * u64::from(elapsed.hours.get());
            assert_eq!(frames, fixture.opportunities);
        }
    }
}
