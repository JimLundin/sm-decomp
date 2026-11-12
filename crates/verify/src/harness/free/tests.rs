use std::convert::Infallible;

use oracle_record::FrameRecord;

use super::{End, FreeReport, Segment, run};
use crate::harness::Hides;
use crate::harness::game_time::Clock;
use crate::harness::tests::{Copying, record};
use crate::lock::Frame;

/// Gameplay records whose clocks show `frames` in its frames word, one a record.
fn clock(frames: &[u16]) -> Vec<Result<FrameRecord, Infallible>> {
    frames
        .iter()
        .zip(0_u64..)
        .map(|(&count, index)| Ok(record(index, 8, [0, 0, 0, count])))
        .collect()
}

fn free(records: Vec<Result<FrameRecord, Infallible>>) -> FreeReport {
    let Ok(report) = run(&Clock, records);
    report
}

#[test]
fn a_model_that_follows_the_oracle_holds_the_row_in_one_segment() {
    let report = free(clock(&[0, 1, 2, 3, 4]));
    assert_eq!((report.frames, report.held, report.segments()), (4, 4, 1));
    assert_eq!(
        report.longest,
        [Segment {
            seed: Frame(0),
            held: 4,
            end: End::RowEnd,
        }]
    );
    assert!(report.first.iter().all(|(_, first)| first.is_none()));
}

#[test]
fn a_divergence_ends_the_segment_and_the_next_frame_seeds_afresh() {
    let report = free(clock(&[0, 1, 2, 9, 10]));
    assert_eq!((report.frames, report.held), (4, 3));
    assert_eq!((report.diverged, report.row_ends), (1, 1));
    let Some((_, Some(first))) = report.first.iter().find(|(name, _)| *name == "frames") else {
        panic!("frames never diverged: {report}");
    };
    assert_eq!(
        (first.frame, first.model.as_str(), first.oracle.as_str()),
        (Frame(3), "3", "9")
    );
    assert_eq!(report.ended_by.get("frames"), Some(&1));
    assert_eq!(
        report
            .longest
            .first()
            .map(|segment| (segment.seed, segment.held)),
        Some((Frame(0), 2))
    );
}

#[test]
fn the_free_run_steps_its_own_state_not_the_records() {
    // Record 2's clock reads 7 though the frame before ended at 2: the per-frame harness
    // re-seeds from it and matches frame 3, the free run carries 2 and diverges at frame 2.
    let records = [0, 1, 7, 8];
    let Ok(per_frame) = crate::harness::run(&Clock, clock(&records));
    assert_eq!(per_frame.matched, 2);
    let report = free(clock(&records));
    assert_eq!((report.held, report.diverged, report.row_ends), (2, 1, 1));
}

#[test]
fn a_mask_ends_the_segment_once_its_frame_is_held_on_the_other_fields() {
    let records = [record(0, 8, [0, 0, 0, 59]), record(1, 8, [0, 0, 1, 0])];
    let Ok(report) = run(
        &Copying(Hides::AnotherWriter),
        records.map(Ok::<_, Infallible>),
    );
    assert_eq!((report.held, report.masked), (1, 1));
    assert!(matches!(
        report.longest.first().map(|segment| &segment.end),
        Some(End::Masked(Frame(1), fields)) if fields == &["frames"]
    ));
}

#[test]
fn a_mask_no_writer_stands_behind_leaves_the_segment_running() {
    // The frames word never moves as the model would move it, but the mask hides it and names
    // no writer, so the carried State holds across both frames.
    let records = [
        record(0, 8, [0, 0, 0, 59]),
        record(1, 8, [0, 0, 1, 7]),
        record(2, 8, [0, 0, 2, 3]),
    ];
    let Ok(report) = run(&Copying(Hides::NoWriter), records.map(Ok::<_, Infallible>));
    assert_eq!((report.held, report.masked, report.row_ends), (2, 0, 1));
    assert_eq!(report.segments(), 1);
}

#[test]
fn a_frame_the_domain_cannot_read_ends_the_segment_as_skipped() {
    let mut records = clock(&[0, 1, 2, 3]);
    let mut short = record(2, 8, [0; 4]);
    short.wram = Box::default();
    if let Some(slot) = records.get_mut(2) {
        *slot = Ok(short);
    }
    let report = free(records);
    assert_eq!((report.frames, report.held, report.segments()), (3, 1, 1));
    assert_eq!(report.skipped, 1);
}

#[test]
fn a_total_adds_rows_counts() {
    let mut total = FreeReport::new::<Clock>();
    total.add(&free(clock(&[0, 1, 2])));
    total.add(&free(clock(&[0, 1, 5])));
    assert_eq!(
        (total.frames, total.held, total.diverged, total.row_ends),
        (4, 3, 1, 1)
    );
}

/// The committed free-run floors: `<domain> <row> <frames held>` a line, `#` comments.
const FLOORS: &str = include_str!("../free_floors.txt");

/// The floor of `domain` on `row`, from [`FLOORS`].
fn floor(domain: &str, row: &str) -> Option<u64> {
    FLOORS
        .lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .find_map(|line| {
            let mut words = line.split_whitespace();
            let (Some(name), Some(at), Some(held)) = (words.next(), words.next(), words.next())
            else {
                panic!("free_floors.txt: {line:?}");
            };
            (name == domain && at == row).then(|| {
                held.parse()
                    .unwrap_or_else(|error| panic!("free_floors.txt: {line:?}: {error}"))
            })
        })
}

/// `domain` run free over every row cached under `cache`: each row's report, the total, and
/// for a row with a floor in `free_floors.txt` a failure when its held frames fall below it. A
/// row without a floor prints the line that would hold it, so a first run over a new cache
/// sets the floors.
fn hold_free_floors<D: crate::harness::Domain>(domain: &D, name: &str, cache: &std::path::Path) {
    use std::io::Write as _;
    let lock =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenarios/locks/lsnes.toml");
    let mut total = FreeReport::new::<D>();
    let mut failures = Vec::new();
    for row in crate::harness::tests::cached_rows(cache) {
        let report = super::run_cached(domain, &lock, cache, &row)
            .unwrap_or_else(|error| panic!("{row}: {error}"));
        let _ = write!(std::io::stdout(), "{row}: {report}");
        match floor(name, &row) {
            Some(floor) if report.held < floor => {
                failures.push(format!("{row}: {} held < floor {floor}", report.held));
            }
            Some(_) => {}
            None => {
                let _ = writeln!(
                    std::io::stdout(),
                    "no floor; add `{name} {row} {}`",
                    report.held
                );
            }
        }
        total.add(&report);
    }
    let _ = write!(std::io::stdout(), "all rows: {total}");
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The ROM image `SM_ROM` names, with the cache, or `None` where both are unset.
fn cache_and_image() -> Option<(std::path::PathBuf, Vec<u8>)> {
    let (cache, rom) = crate::harness::tests::environment()?;
    let image =
        std::fs::read(&rom).unwrap_or_else(|error| panic!("SM_ROM {}: {error}", rom.display()));
    Some((cache, image))
}

/// Every domain's free run over every cached row, held to `free_floors.txt`; `SM_CACHE` and
/// `SM_ROM` as the per-frame harness's tests read them. Run with `--nocapture` to see each
/// row's segments and first divergences.
#[test]
fn every_domain_runs_free_against_the_cache() {
    use crate::harness::{appearance, blocks, camera, ceres, hud, movement, progress};
    let Some((cache, image)) = cache_and_image() else {
        return;
    };
    hold_free_floors(&Clock, "game-time", &cache);
    let samus =
        movement::Movement::load(&image).unwrap_or_else(|error| panic!("content: {error:?}"));
    hold_free_floors(&samus, "samus-movement", &cache);
    let looks =
        appearance::Appearance::load(&image).unwrap_or_else(|error| panic!("content: {error:?}"));
    hold_free_floors(&looks, "samus-appearance", &cache);
    let view =
        camera::CameraDomain::load(&image).unwrap_or_else(|error| panic!("content: {error:?}"));
    hold_free_floors(&view, "world-camera", &cache);
    let hud = hud::HudDomain::load(&image).unwrap_or_else(|error| panic!("content: {error:?}"));
    hold_free_floors(&hud, "interface-hud", &cache);
    let progress = progress::ProgressDomain::new(image.clone());
    hold_free_floors(&progress, "run-progress", &cache);
    let doors =
        ceres::CeresDomain::load(&image).unwrap_or_else(|error| panic!("content: {error:?}"));
    let blocks =
        blocks::BlocksDomain::load(image).unwrap_or_else(|error| panic!("content: {error}"));
    hold_free_floors(&blocks, "world-blocks", &cache);
    hold_free_floors(&doors, "boss-ceres-ridley", &cache);
}
