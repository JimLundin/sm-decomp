use std::collections::{BTreeMap, BTreeSet};
use std::convert::Infallible;

use oracle_record::FrameRecord;

use super::{Accounting, End, Floors, FreeReport, Segment, UNCOMPARED, run};
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
    // The frame is compared, on the fields no other writer moved.
    assert_eq!((report.compared(), report.masked_frames), (1, 1));
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
fn a_rows_records_are_each_compared_skipped_or_folded_once() {
    // Record 2 can't be read, so neither the frame ending at it nor the one starting there is
    // compared: of the four records after the seed, two are compared and two skipped.
    let mut records = clock(&[0, 1, 2, 3, 4]);
    let mut short = record(2, 8, [0; 4]);
    short.wram = Box::default();
    if let Some(slot) = records.get_mut(2) {
        *slot = Ok(short);
    }
    let report = free(records);
    assert_eq!(
        (report.compared(), report.skipped_frames(), report.folded),
        (2, 2, 0)
    );
    assert_eq!(report.compared_at, BTreeSet::from([1, 4]));
    let accounting = Accounting::of(&report);
    assert_eq!((accounting.held, accounting.frames()), (2, 4));
}

#[test]
fn the_floors_parse_each_domains_accounting_and_each_rows_gap() {
    let parsed = Floors::parse("# floors\ngame-time r1 5 4 6 1 2 7 3\nuncompared r1 4\n");
    let Ok(floors) = parsed else {
        panic!("the floors don't parse: {parsed:?}");
    };
    let accounting = floors
        .domains
        .get(&("game-time".to_owned(), "r1".to_owned()));
    assert_eq!(
        accounting.map(|accounting| (accounting.held, accounting.in_scope(), accounting.frames())),
        Some((5, 13, 18))
    );
    assert_eq!(accounting.map(|accounting| accounting.null_held), Some(4));
    assert_eq!(floors.uncompared.get("r1"), Some(&4));
    assert!(Floors::parse("game-time r1 5\n").is_err());
    assert!(Floors::parse("game-time r1 5 6 1 2 7 3\n").is_err());
    assert!(Floors::parse("uncompared r1\n").is_err());
}

#[test]
fn the_null_models_free_run_holds_the_frames_on_which_nothing_moves() {
    // The clock stalls for two frames: the null model, which carries its seed's count, holds
    // them; the model, which ticks, diverges on them and on the jump after.
    let report = free(clock(&[0, 1, 1, 1, 5]));
    assert_eq!((report.held, report.null_held), (1, 2));
    // A clock that ticks every frame leaves the null model nothing to hold.
    assert_eq!(free(clock(&[0, 1, 2, 3])).null_held, 0);
    assert_eq!(Accounting::of(&report).null_held, 2);
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

/// The committed free-run floors.
const FLOORS: &str = include_str!("../free_floors.txt");

/// The clock, which compares every frame: the frames no domain compares count the model domains
/// only, or the clock would hide every gap.
const CLOCK: &str = "game-time";

/// Per row, its frames after the seed and the records that end a frame some model domain
/// compares.
type Reach = BTreeMap<String, (u64, BTreeSet<u64>)>;

/// The domains' free runs over the rows cached under `cache`, held to `floors`.
struct Held<'a> {
    cache: &'a std::path::Path,
    floors: &'a Floors,
    /// The frames the domains run so far compare, by row.
    reach: Reach,
    failures: Vec<String>,
}

impl Held<'_> {
    /// `domain` run free over every row: each row's report and the total, and a failure for a
    /// row whose held frames fall below its floor or whose records it accounts for otherwise
    /// than its line says. A row without a line prints the line that would hold it, so a first
    /// run over a new cache sets them.
    fn hold<D: crate::harness::Domain>(&mut self, domain: &D, name: &str) {
        use std::io::Write as _;
        let lock = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../scenarios/locks/lsnes.toml");
        let mut total = FreeReport::new::<D>();
        for row in crate::harness::tests::cached_rows(self.cache) {
            let report = super::run_cached(domain, &lock, self.cache, &row)
                .unwrap_or_else(|error| panic!("{row}: {error}"));
            let _ = write!(std::io::stdout(), "{row}: {report}");
            let ran = Accounting::of(&report);
            if let Some(floor) = self.floors.domains.get(&(name.to_owned(), row.clone())) {
                let status = if ran.held >= floor.held {
                    "ok"
                } else {
                    "FAILED"
                };
                let _ = writeln!(
                    std::io::stdout(),
                    "FLOOR free {name} {row} {} {} {status}",
                    ran.held,
                    floor.held
                );
            }
            match self.floors.domains.get(&(name.to_owned(), row.clone())) {
                Some(floor) if ran.held < floor.held => self.failures.push(format!(
                    "{name} {row}: {} held < floor {}",
                    ran.held, floor.held
                )),
                Some(floor) if !ran.same_records(floor) => {
                    let line = Accounting {
                        held: floor.held,
                        ..ran
                    }
                    .line(name, &row);
                    self.failures.push(format!(
                        "{name} {row}: its records are accounted as `{line}`"
                    ));
                }
                Some(_) => {}
                None => {
                    let _ = writeln!(
                        std::io::stdout(),
                        "no floor; add `{}`",
                        ran.line(name, &row)
                    );
                }
            }
            let (frames, compared) = self.reach.entry(row).or_default();
            *frames = ran.frames();
            if name != CLOCK {
                compared.extend(&report.compared_at);
            }
            total.add(&report);
        }
        let _ = write!(std::io::stdout(), "all rows: {total}");
    }
}

/// The ROM image `SM_ROM` names, with the cache, or `None` where both are unset.
fn cache_and_image() -> Option<(std::path::PathBuf, Vec<u8>)> {
    let (cache, rom) = crate::harness::tests::environment()?;
    let image =
        std::fs::read(&rom).unwrap_or_else(|error| panic!("SM_ROM {}: {error}", rom.display()));
    Some((cache, image))
}

/// A single domain's free-run floors, independent of failures in its neighbours.
fn hold_domain<D: crate::harness::Domain>(domain: &D, name: &str, cache: &std::path::Path) {
    let floors = Floors::parse(FLOORS).unwrap_or_else(|error| panic!("free_floors.txt: {error}"));
    let mut held = Held {
        cache,
        floors: &floors,
        reach: Reach::new(),
        failures: Vec::new(),
    };
    held.hold(domain, name);
    assert!(held.failures.is_empty(), "{}", held.failures.join("\n"));
}

#[test]
fn movement_runs_free_against_the_cache() {
    let Some((cache, image)) = cache_and_image() else {
        return;
    };
    let domain = crate::harness::movement::Movement::load(&image)
        .unwrap_or_else(|error| panic!("content: {error:?}"));
    hold_domain(&domain, "samus-movement", &cache);
}

#[test]
fn the_camera_runs_free_against_the_cache() {
    let Some((cache, image)) = cache_and_image() else {
        return;
    };
    let domain = crate::harness::camera::CameraDomain::load(&image)
        .unwrap_or_else(|error| panic!("content: {error:?}"));
    hold_domain(&domain, "world-camera", &cache);
}

#[test]
fn the_ceres_doors_run_free_against_the_cache() {
    let Some((cache, image)) = cache_and_image() else {
        return;
    };
    let domain = crate::harness::ceres::CeresDomain::load(&image)
        .unwrap_or_else(|error| panic!("content: {error:?}"));
    hold_domain(&domain, "boss-ceres-ridley", &cache);
}

#[test]
fn appearance_runs_free_against_the_cache() {
    let Some((cache, image)) = cache_and_image() else {
        return;
    };
    let domain = crate::harness::appearance::Appearance::load(&image)
        .unwrap_or_else(|error| panic!("content: {error:?}"));
    hold_domain(&domain, "samus-appearance", &cache);
}

#[test]
fn the_door_passage_runs_free_against_the_cache() {
    let Some((cache, image)) = cache_and_image() else {
        return;
    };
    let domain = crate::harness::doors::DoorsDomain::new(image);
    hold_domain(&domain, "world-doors", &cache);
}

/// Every domain's free run over every cached row, held to `free_floors.txt`, and each row's
/// frames no model domain compares to its `uncompared` line; `SM_CACHE` and `SM_ROM` as the
/// per-frame harness's tests read them. Run with `--nocapture` to see each row's segments and
/// first divergences.
#[test]
fn every_domain_runs_free_against_the_cache() {
    use std::io::Write as _;

    use crate::harness::{appearance, blocks, camera, ceres, doors, hud, movement, progress};
    let Some((cache, image)) = cache_and_image() else {
        return;
    };
    let floors = Floors::parse(FLOORS).unwrap_or_else(|error| panic!("free_floors.txt: {error}"));
    let mut held = Held {
        cache: &cache,
        floors: &floors,
        reach: Reach::new(),
        failures: Vec::new(),
    };
    held.hold(&Clock, "game-time");
    let samus =
        movement::Movement::load(&image).unwrap_or_else(|error| panic!("content: {error:?}"));
    held.hold(&samus, "samus-movement");
    let looks =
        appearance::Appearance::load(&image).unwrap_or_else(|error| panic!("content: {error:?}"));
    held.hold(&looks, "samus-appearance");
    let view =
        camera::CameraDomain::load(&image).unwrap_or_else(|error| panic!("content: {error:?}"));
    held.hold(&view, "world-camera");
    let hud = hud::HudDomain::load(&image).unwrap_or_else(|error| panic!("content: {error:?}"));
    held.hold(&hud, "interface-hud");
    let progress = progress::ProgressDomain::new(image.clone());
    held.hold(&progress, "run-progress");
    let passage = doors::DoorsDomain::new(image.clone());
    held.hold(&passage, "world-doors");
    let doors =
        ceres::CeresDomain::load(&image).unwrap_or_else(|error| panic!("content: {error:?}"));
    let blocks =
        blocks::BlocksDomain::load(image).unwrap_or_else(|error| panic!("content: {error}"));
    held.hold(&blocks, "world-blocks");
    held.hold(&doors, "boss-ceres-ridley");
    let Held {
        reach,
        mut failures,
        ..
    } = held;
    for (row, (frames, compared)) in &reach {
        let gap = frames.saturating_sub(u64::try_from(compared.len()).unwrap_or(u64::MAX));
        let line = format!("{UNCOMPARED} {row} {gap}");
        match floors.uncompared.get(row) {
            Some(&floor) if floor == gap => {}
            Some(_) => failures.push(format!("{row}: no model domain compares `{line}`")),
            None => {
                let _ = writeln!(std::io::stdout(), "no gap; add `{line}`");
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn a_skip_counts_in_scope_unless_the_domain_names_it_out_of_scope() {
    // The same unreadable record: the clock calls it a defect, a frame it answers for and
    // doesn't reach; the copying clock names every skip out of its scope.
    let records = || {
        let mut records = clock(&[0, 1, 2, 3]);
        let mut short = record(2, 8, [0; 4]);
        short.wram = Box::default();
        if let Some(slot) = records.get_mut(2) {
            *slot = Ok(short);
        }
        records
    };
    let defect = Accounting::of(&free(records()));
    assert_eq!(
        (defect.compared, defect.unreached, defect.out_of_scope),
        (1, 2, 0)
    );
    assert_eq!(defect.in_scope(), 3);
    let Ok(report) = run(&Copying(Hides::NoWriter), records());
    let elsewhere = Accounting::of(&report);
    assert_eq!(
        (
            elsewhere.compared,
            elsewhere.unreached,
            elsewhere.out_of_scope
        ),
        (1, 0, 2)
    );
    assert_eq!(elsewhere.in_scope(), 1);
}
