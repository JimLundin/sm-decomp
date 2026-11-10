use std::collections::BTreeMap;
use std::convert::Infallible;
use std::path::{Path, PathBuf};
use std::time::Instant;

use oracle_record::{CpuRegisters, Cut, FrameRecord, Nmi, NmiKind, Pad, PpuRegisters};

use super::game_time::Clock;
use game::state::GameTime;

use super::{
    Behaviours, Coverage, Domain, Entry, Field, FieldSet, Interval, Mask, Mismatch, Report, Run,
    run, run_cached,
};
use crate::lock::Frame;
use crate::source::{self, WramWord};

/// A record holding only work RAM: the game state and the clock.
pub(super) fn record(index: u64, game_state: u16, time: [u16; 4]) -> FrameRecord {
    let mut wram = vec![0_u8; 0x2_0000];
    let mut put = |word: WramWord, value: u16| {
        let at = usize::from(word.offset);
        wram[at..at + 2].copy_from_slice(&value.to_le_bytes());
    };
    put(source::GAME_STATE, game_state);
    let [hours, minutes, seconds, frames] = time;
    put(source::GAME_TIME_HOURS, hours);
    put(source::GAME_TIME_MINUTES, minutes);
    put(source::GAME_TIME_SECONDS, seconds);
    put(source::GAME_TIME_FRAMES, frames);
    FrameRecord {
        index,
        video_frame: 0,
        pad: Pad(0),
        cut: Cut::NmiReturn(Nmi {
            kind: NmiKind::Requested,
            interrupted_pc: 0,
            polled: true,
        }),
        joypad_reads: 0,
        wait_entry: None,
        overscan: false,
        cpu: CpuRegisters::default(),
        wram: wram.into_boxed_slice(),
        vram: Box::default(),
        cgram: Box::default(),
        oam: Box::default(),
        sram: Box::default(),
        ppu: PpuRegisters::default(),
        apu_log: Vec::new(),
        picture: None,
        spc: None,
        bank_instructions: None,
        writes: Vec::new(),
        hits: Vec::new(),
        legacy_ppu_digest: 0,
    }
}

fn two_frames(next: [u16; 4]) -> [Result<FrameRecord, Infallible>; 2] {
    [Ok(record(0, 8, [0, 0, 0, 59])), Ok(record(1, 8, next))]
}

#[test]
fn a_frame_the_model_steps_to_the_oracle_matches_on_every_field() {
    let Ok(report) = run(&Clock, two_frames([0, 0, 1, 0]));
    assert_eq!((report.frames, report.compared, report.matched), (1, 1, 1));
    assert!(report.fields.iter().all(|field| field.matched == 1));
}

#[test]
fn a_mismatch_names_its_field_frame_and_both_values() {
    let Ok(report) = run(&Clock, two_frames([0, 0, 0, 59]));
    assert_eq!((report.frames, report.compared, report.matched), (1, 1, 0));
    let mismatched: Vec<_> = report
        .fields
        .iter()
        .filter(|field| field.mismatched == 1)
        .map(|field| field.name)
        .collect();
    assert_eq!(mismatched, ["seconds", "frames"]);
    assert_eq!(
        report.fields[3].first,
        [Mismatch {
            frame: Frame(1),
            model: "0".to_owned(),
            oracle: "59".to_owned(),
        }]
    );
    assert_eq!(
        report.fields[3].runs,
        [Run {
            first: Frame(1),
            last: Frame(1),
        }]
    );
}

#[test]
fn a_record_the_domain_cannot_read_is_a_harness_defect() {
    let mut short = record(1, 8, [0; 4]);
    short.wram = Box::default();
    let Ok(report) = run(
        &Clock,
        [Ok::<_, Infallible>(record(0, 8, [0; 4])), Ok(short)],
    );
    assert_eq!((report.frames, report.compared), (1, 0));
    assert_eq!((report.defect_frames(), report.skipped_frames()), (1, 0));
}

#[test]
fn the_null_model_scores_the_frames_on_which_nothing_changes() {
    let records = [
        record(0, 8, [0, 0, 0, 1]),
        record(1, 9, [0, 0, 0, 1]),
        record(2, 8, [0, 0, 0, 1]),
    ];
    let Ok(report) = run(&Clock, records.map(Ok::<_, Infallible>));
    // Frame 1: the model ticks, the oracle holds, the null model holds. Frame 2: both hold.
    assert_eq!(
        (report.compared, report.matched, report.null_matched),
        (2, 1, 2)
    );
    assert_eq!(report.fields[3].null_matched, 2);
}

/// The clock, with the frames field masked out of scope and the seconds field copied from
/// record N+1 by its stand-in: a domain that exercises the harness's masks and its unverified
/// count.
struct Copying;

impl Domain for Copying {
    const NAME: &'static str = "copying clock";
    type State = GameTime;
    type Inputs = ();
    type World = Option<u16>;
    type Fields = GameTime;
    type Skip = String;
    const FIELDS: &'static [Field<GameTime>] = <Clock as Domain>::FIELDS;

    fn decode(&self, from: &FrameRecord, _: Interval) -> Result<(GameTime, ()), String> {
        let time = Clock.expected(from).map_err(|error| error.to_string())?;
        Ok((time, ()))
    }

    fn world(&self, _: &FrameRecord, next: &FrameRecord) -> Result<Option<u16>, String> {
        let time = Clock.expected(next).map_err(|error| error.to_string())?;
        Ok(Some(time.seconds))
    }

    fn masks(&self, _: &FrameRecord, _: &FrameRecord, (): &()) -> Result<Vec<Mask>, String> {
        Ok(vec![Mask {
            reason: "frames out of scope",
            fields: FieldSet::single(3),
        }])
    }

    fn neutral(&self, _: &Option<u16>) -> Option<Option<u16>> {
        Some(None)
    }

    fn behaviours(&self) -> Vec<Entry> {
        [
            ("Hours", FieldSet::single(0)),
            ("Seconds", FieldSet::single(2)),
            ("Frames", FieldSet::single(3)),
            ("Event", FieldSet::EMPTY),
        ]
        .map(|(name, fields)| Entry {
            name: name.to_owned(),
            fields,
        })
        .into()
    }

    fn exercised(&self, _: &GameTime, (): &(), _: &Option<u16>) -> Vec<usize> {
        vec![0, 1, 2, 3]
    }

    fn step(&self, state: &mut GameTime, (): &(), world: &mut Option<u16>) {
        if let Some(seconds) = *world {
            state.seconds = seconds;
        }
    }

    fn fields(&self, state: &GameTime) -> GameTime {
        *state
    }

    fn expected(&self, next: &FrameRecord) -> Result<GameTime, String> {
        Clock.expected(next).map_err(|error| error.to_string())
    }
}

#[test]
fn a_masked_field_is_not_compared_and_a_copied_one_is_unverified() {
    let Ok(report) = run(&Copying, two_frames([0, 0, 1, 0]));
    assert_eq!(
        (report.compared, report.matched, report.unverified),
        (1, 0, 1)
    );
    assert_eq!(report.masked.get("frames out of scope"), Some(&1));
    let seconds = &report.fields[2];
    assert_eq!((seconds.matched, seconds.unverified), (0, 1));
    let frames = &report.fields[3];
    assert_eq!((frames.masked, frames.compared()), (1, 0));
}

#[test]
fn an_entry_is_credited_only_where_its_fields_are_compared() {
    let Ok(report) = run(&Copying, two_frames([0, 0, 1, 0]));
    let counts: Vec<(&str, u64, u64, u64, u64)> = report
        .behaviours
        .iter()
        .map(|entry| {
            (
                entry.name.as_str(),
                entry.exercised,
                entry.matched,
                entry.null_matched,
                entry.unchecked,
            )
        })
        .collect();
    // Hours and seconds are compared, but the frame is unverified; the null model holds the
    // hours as the oracle does, not the seconds; frames is masked; the event isn't a field.
    assert_eq!(
        counts,
        [
            ("Hours", 1, 0, 1, 0),
            ("Seconds", 1, 0, 0, 0),
            ("Frames", 0, 0, 0, 1),
            ("Event", 0, 0, 0, 1)
        ]
    );
}

/// The committed floors: `<domain> <row> <frames matched and verified>` a line, `#` comments.
const FLOORS: &str = include_str!("floors.txt");

/// The floor of `domain` on each row, from [`FLOORS`].
fn floors(domain: &str) -> BTreeMap<&'static str, u64> {
    FLOORS
        .lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .filter_map(|line| {
            let mut words = line.split_whitespace();
            let (Some(name), Some(row), Some(floor)) = (words.next(), words.next(), words.next())
            else {
                panic!("floors.txt: {line:?}");
            };
            let floor = floor
                .parse()
                .unwrap_or_else(|error| panic!("floors.txt: {line:?}: {error}"));
            (name == domain).then_some((row, floor))
        })
        .collect()
}

/// The committed oracle coverage over every row: [`Coverage`]'s lines, `#` comments.
const COVERAGE: &str = include_str!("coverage.txt");

/// The committed coverage lines of `domain`.
fn coverage(domain: &str) -> String {
    let mut lines = String::new();
    for line in COVERAGE
        .lines()
        .filter(|line| line.split_whitespace().next() == Some(domain))
    {
        lines.push_str(line);
        lines.push('\n');
    }
    lines
}

/// The rows cached under `cache` (a directory with an `index.txt`), sorted.
fn cached_rows(cache: &Path) -> Vec<String> {
    let entries = match std::fs::read_dir(cache) {
        Ok(entries) => entries,
        Err(error) => panic!("SM_CACHE {}: {error}", cache.display()),
    };
    let mut rows: Vec<String> = entries
        .filter_map(Result::ok)
        .filter(|entry| entry.path().join("index.txt").is_file())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect();
    rows.sort();
    rows
}

/// `SM_CACHE` and `SM_ROM`, or `None` (with a line saying so) when both are unset; one without
/// the other, or a cache that isn't a directory, fails.
fn environment() -> Option<(PathBuf, PathBuf)> {
    use std::io::Write as _;
    let (cache, rom) = match (std::env::var_os("SM_CACHE"), std::env::var_os("SM_ROM")) {
        (Some(cache), Some(rom)) => (PathBuf::from(cache), PathBuf::from(rom)),
        (None, None) => {
            let _ = writeln!(std::io::stdout(), "skipped: SM_CACHE/SM_ROM unset");
            return None;
        }
        (cache, rom) => panic!("SM_CACHE and SM_ROM go together: {cache:?}, {rom:?}"),
    };
    assert!(
        cache.is_dir(),
        "SM_CACHE {}: not a directory",
        cache.display()
    );
    Some((cache, rom))
}

/// `domain` over every row cached under `cache`, each held to its floor in [`FLOORS`] under
/// `name`. A cached row without a floor fails, so a new row gets one; a floor whose row isn't
/// cached (a partial local cache) is reported and not checked. With every row cached, the
/// total's coverage must equal [`COVERAGE`]'s, so the report never shows a stale one. Prints
/// each row's report, the total and the per-behaviour table; run with `--nocapture` to see them.
fn hold_floors<D: Domain>(domain: &D, name: &str, cache: &Path) {
    use std::io::Write as _;
    let lock = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenarios/locks/lsnes.toml");
    let floors = floors(name);
    let started = Instant::now();
    let mut total = Report::new(domain);
    let mut failures = Vec::new();
    let rows = cached_rows(cache);
    for row in &rows {
        let report = match run_cached(domain, &lock, cache, row) {
            Ok(report) => report,
            Err(error) => panic!("{row}: {error}"),
        };
        let _ = write!(std::io::stdout(), "{row}: {report}{}", domain.row_notes());
        match floors.get(row.as_str()) {
            Some(&floor) if report.matched >= floor => {}
            Some(&floor) => failures.push(format!("{row}: {} < floor {floor}", report.matched)),
            None => failures.push(format!(
                "{row}: no floor; add `{name} {row} {}`",
                report.matched
            )),
        }
        total.add(&report);
    }
    let uncached: Vec<&&str> = floors
        .keys()
        .filter(|row| !rows.iter().any(|cached| cached == *row))
        .collect();
    for row in &uncached {
        let _ = writeln!(std::io::stdout(), "not cached, floor not checked: {row}");
    }
    let fresh = Coverage {
        name,
        report: &total,
    }
    .to_string();
    if !uncached.is_empty() {
        let _ = writeln!(
            std::io::stdout(),
            "not every row cached, coverage not checked"
        );
    } else if fresh != coverage(name) {
        failures.push(format!(
            "coverage.txt is stale; its {name} lines are:\n{fresh}"
        ));
    }
    let seconds = started.elapsed().as_secs_f64();
    let rate =
        f64::from(u32::try_from(total.frames).unwrap_or(u32::MAX)) / seconds.max(f64::EPSILON);
    let _ = write!(
        std::io::stdout(),
        "all {} rows: {total}behaviours:\n{}{} frames in {seconds:.2} s ({rate:.0} frames/s)\n",
        rows.len(),
        Behaviours(&total),
        total.frames,
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The clock domain over every cached row, held to its floors. Skipped only where `SM_CACHE`
/// and `SM_ROM` are both unset, as the ratchet test is.
#[test]
fn the_clock_holds_its_floors_against_the_cache() {
    if let Some((cache, _)) = environment() {
        hold_floors(&Clock, "game-time", &cache);
    }
}

/// Samus's motion over every cached row, held to its floors; `SM_CACHE` and `SM_ROM` as the
/// clock's test reads them.
#[test]
fn samus_motion_holds_its_floors_against_the_cache() {
    let Some((cache, rom)) = environment() else {
        return;
    };
    let image = match std::fs::read(&rom) {
        Ok(image) => image,
        Err(error) => panic!("SM_ROM {}: {error}", rom.display()),
    };
    let domain = match super::samus_motion::SamusMotion::load(&image) {
        Ok(domain) => domain,
        Err(error) => panic!("content: {error:?}"),
    };
    hold_floors(&domain, "samus-motion", &cache);
}

/// The samus-motion harness cases (the decode notes' "Room content: harness cases"), each over
/// its row's records; a case whose row isn't cached is reported and not checked.
#[test]
fn samus_motions_harness_cases_hold_against_the_cache() {
    use std::io::Write as _;
    let Some((cache, rom)) = environment() else {
        return;
    };
    let image = match std::fs::read(&rom) {
        Ok(image) => image,
        Err(error) => panic!("SM_ROM {}: {error}", rom.display()),
    };
    let domain = match super::samus_motion::SamusMotion::load(&image) {
        Ok(domain) => domain,
        Err(error) => panic!("content: {error:?}"),
    };
    let lock = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenarios/locks/lsnes.toml");
    let text = std::fs::read_to_string(&lock).unwrap_or_else(|error| panic!("{error}"));
    let parsed = crate::lock::Lock::parse(&text).unwrap_or_else(|error| panic!("{error:?}"));
    let rows = cached_rows(&cache);
    let mut failures = Vec::new();
    for case in &super::samus_motion::cases::CASES {
        if !rows.iter().any(|row| row == case.row) {
            let _ = writeln!(std::io::stdout(), "{}: {} not cached", case.id, case.row);
            continue;
        }
        let locked = parsed
            .row(&crate::lock::RowId(case.row.to_owned()))
            .unwrap_or_else(|| panic!("{}: not in the lock", case.row));
        let opened =
            crate::cache::Cache::open(&cache, locked).unwrap_or_else(|error| panic!("{error}"));
        let records: Vec<_> = opened
            .entries()
            .iter()
            .filter(|entry| {
                (case.checked.saturating_sub(4)..=case.checked).contains(&entry.frame.0)
            })
            .map(|entry| {
                opened
                    .record(entry)
                    .unwrap_or_else(|error| panic!("{}: {error}", case.row))
            })
            .collect();
        match domain.case(case, &records) {
            Ok(()) => {
                let _ = writeln!(std::io::stdout(), "{}: holds", case.id);
            }
            Err(differences) => failures.push(format!("{}: {}", case.id, differences.join("; "))),
        }
    }
    assert!(
        failures.is_empty(),
        "{}",
        failures.join(
            "
"
        )
    );
}

/// The camera over every cached row, held to its floors; `SM_CACHE` and `SM_ROM` as the clock's
/// test reads them (the ROM for samus-motion's model, which supplies the placement).
#[test]
fn the_camera_holds_its_floors_against_the_cache() {
    let Some((cache, rom)) = environment() else {
        return;
    };
    let image = match std::fs::read(&rom) {
        Ok(image) => image,
        Err(error) => panic!("SM_ROM {}: {error}", rom.display()),
    };
    let domain = match super::camera::CameraDomain::load(&image) {
        Ok(domain) => domain,
        Err(error) => panic!("content: {error:?}"),
    };
    hold_floors(&domain, "camera", &cache);
}

/// The blocks over every cached row, held to its floors; `SM_CACHE` and `SM_ROM` as the clock's
/// test reads them (the ROM for the reactions and the scroll lists). The cache's rows carry the
/// blocks' watched instructions' hits (`docs/behaviours/blocks.decode.md`, "The harness").
#[test]
fn the_blocks_hold_their_floors_against_the_cache() {
    let Some((cache, rom)) = environment() else {
        return;
    };
    let image = match std::fs::read(&rom) {
        Ok(image) => image,
        Err(error) => panic!("SM_ROM {}: {error}", rom.display()),
    };
    let domain = match super::blocks::BlocksDomain::load(image) {
        Ok(domain) => domain,
        Err(error) => panic!("content: {error}"),
    };
    hold_floors(&domain, "blocks", &cache);
}

/// Progress over every cached row, held to its floors; `SM_CACHE` and `SM_ROM` as the clock's
/// test reads them (the ROM for the rooms' states and the item PLMs' pickups).
#[test]
fn progress_holds_its_floors_against_the_cache() {
    let Some((cache, rom)) = environment() else {
        return;
    };
    let image = match std::fs::read(&rom) {
        Ok(image) => image,
        Err(error) => panic!("SM_ROM {}: {error}", rom.display()),
    };
    hold_floors(
        &super::progress::ProgressDomain::new(image),
        "progress",
        &cache,
    );
}

/// The room state each cached row's first record holds against the model's choice over that
/// record's progress: the loads before the rows, which hold the states other than the default
/// the rows' own loads never reach. Beside it the null model, always the default state. A room
/// case the decode notes name at a row's first record is checked here by its name.
#[test]
fn progress_chooses_each_rows_standing_room_state() {
    use super::progress::cases::{Chosen, ROOMS};
    use std::io::Write as _;
    let Some((cache, rom)) = environment() else {
        return;
    };
    let image = match std::fs::read(&rom) {
        Ok(image) => image,
        Err(error) => panic!("SM_ROM {}: {error}", rom.display()),
    };
    let domain = super::progress::ProgressDomain::new(image);
    let lock = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenarios/locks/lsnes.toml");
    let text = std::fs::read_to_string(&lock).unwrap_or_else(|error| panic!("{error}"));
    let parsed = crate::lock::Lock::parse(&text).unwrap_or_else(|error| panic!("{error:?}"));
    let (mut matched, mut defaults, mut differing) = (0, 0, Vec::new());
    let mut cases_checked = Vec::new();
    let rows = cached_rows(&cache);
    for row in &rows {
        let locked = parsed
            .row(&crate::lock::RowId(row.clone()))
            .unwrap_or_else(|| panic!("{row}: not in the lock"));
        let opened =
            crate::cache::Cache::open(&cache, locked).unwrap_or_else(|error| panic!("{error}"));
        let Some(entry) = opened.entries().first() else {
            continue;
        };
        let record = opened
            .record(entry)
            .unwrap_or_else(|error| panic!("{row}: {error}"));
        let standing = domain
            .standing_state(&record)
            .unwrap_or_else(|error| panic!("{row}: {error}"));
        if standing.held == standing.states.saturating_sub(1) {
            defaults += 1;
        }
        if standing.chosen == standing.held {
            matched += 1;
        } else {
            differing.push(format!(
                "{row}: holds state {}, the model chose {}",
                standing.held, standing.chosen
            ));
        }
        let named = ROOMS.iter().filter(|case| {
            case.chosen == Chosen::BeforeTheRow && case.row == row && case.record == entry.frame.0
        });
        for case in named {
            match super::progress::cases::standing_case(case, &record, standing) {
                Ok(()) => {
                    let _ = writeln!(std::io::stdout(), "{}: holds", case.id);
                }
                Err(failures) => differing.push(format!("{}: {}", case.id, failures.join("; "))),
            }
            cases_checked.push(case.id);
        }
    }
    let _ = writeln!(
        std::io::stdout(),
        "standing room states: {matched} of {} matched; null model (the default) {defaults}",
        rows.len()
    );
    let unchecked = ROOMS
        .iter()
        .filter(|case| case.chosen == Chosen::BeforeTheRow && !cases_checked.contains(&case.id));
    for case in unchecked {
        if rows.iter().any(|row| row == case.row) {
            differing.push(format!(
                "{}: record {} isn't its row's first",
                case.id, case.record
            ));
        } else {
            let _ = writeln!(std::io::stdout(), "{}: {} not cached", case.id, case.row);
        }
    }
    assert!(differing.is_empty(), "{}", differing.join("\n"));
}

/// The progress harness cases (the decode notes' "The harness cases") a frame checks: the report
/// cases and the room loaded in a frame, each over its row's records; a case whose row isn't
/// cached is reported and not checked.
#[test]
fn progress_harness_cases_hold_against_the_cache() {
    use super::progress::cases::{Chosen, Counted, REPORTS, ROOMS};
    use std::io::Write as _;
    let Some((cache, rom)) = environment() else {
        return;
    };
    let image = match std::fs::read(&rom) {
        Ok(image) => image,
        Err(error) => panic!("SM_ROM {}: {error}", rom.display()),
    };
    let domain = super::progress::ProgressDomain::new(image);
    let lock = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenarios/locks/lsnes.toml");
    let text = std::fs::read_to_string(&lock).unwrap_or_else(|error| panic!("{error}"));
    let parsed = crate::lock::Lock::parse(&text).unwrap_or_else(|error| panic!("{error:?}"));
    let rows = cached_rows(&cache);
    let frame = |row: &str, record: u64| {
        let locked = parsed
            .row(&crate::lock::RowId(row.to_owned()))
            .unwrap_or_else(|| panic!("{row}: not in the lock"));
        let opened =
            crate::cache::Cache::open(&cache, locked).unwrap_or_else(|error| panic!("{error}"));
        let mut pair = opened
            .entries()
            .iter()
            .filter(|entry| (record.saturating_sub(1)..=record).contains(&entry.frame.0))
            .map(|entry| {
                opened
                    .record(entry)
                    .unwrap_or_else(|error| panic!("{row}: {error}"))
            });
        match (pair.next(), pair.next()) {
            (Some(from), Some(next)) => (from, next),
            _ => panic!("{row}: record {record} and the one before aren't both cached"),
        }
    };
    let cached = |id: &str, row: &str| {
        let is = rows.iter().any(|cached| cached == row);
        if !is {
            let _ = writeln!(std::io::stdout(), "{id}: {row} not cached");
        }
        is
    };
    let mut results = Vec::new();
    for case in REPORTS.iter().filter(|case| cached(case.id, case.row)) {
        let (from, next) = frame(case.row, case.record);
        let verdict = match case.counted {
            Counted::Heard => "holds",
            Counted::StoodIn => "stood in, not counted",
        };
        results.push((case.id, verdict, domain.report_case(case, &from, &next)));
    }
    let loaded = ROOMS
        .iter()
        .filter(|case| case.chosen == Chosen::InTheFrame && cached(case.id, case.row));
    for case in loaded {
        let (from, next) = frame(case.row, case.record);
        results.push((case.id, "holds", domain.loaded_case(case, &from, &next)));
    }
    let mut failures = Vec::new();
    for (id, verdict, result) in results {
        match result {
            Ok(()) => {
                let _ = writeln!(std::io::stdout(), "{id}: {verdict}");
            }
            Err(differences) => failures.push(format!("{id}: {}", differences.join("; "))),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Every level the game holds loads through the importer, refused where a chain of stand-ins
/// loops: each state's of the rooms the doors reach, and of the two no door reaches, the unused
/// room and the debug room. It needs only `SM_ROM`, and is skipped where it is unset.
#[test]
fn every_level_the_game_holds_loads() {
    use data::rom::Lorom;
    use data::room::{RoomKey, levels};
    use std::io::Write as _;
    /// The rooms no door reaches (bank $8F).
    const UNREACHED: [u16; 2] = [0xB3E1, 0xE82C];
    let Some(rom) = std::env::var_os("SM_ROM").map(PathBuf::from) else {
        let _ = writeln!(std::io::stdout(), "skipped: SM_ROM unset");
        return;
    };
    let image = match std::fs::read(&rom) {
        Ok(image) => image,
        Err(error) => panic!("SM_ROM {}: {error}", rom.display()),
    };
    let reached = super::progress::ProgressDomain::new(image.clone())
        .population()
        .unwrap_or_else(|error| panic!("the walk: {error}"))
        .rooms;
    let states = |keys: &[RoomKey]| -> usize {
        keys.iter()
            .map(|&key| match levels(Lorom::new(&image), key) {
                Ok(levels) => levels.len(),
                Err(error) => panic!("room {key:?}: {error:?}"),
            })
            .sum()
    };
    assert_eq!(reached.len(), 261, "rooms the doors reach");
    assert_eq!(states(&reached), 322, "their states");
    assert_eq!(
        states(&UNREACHED.map(RoomKey::from_room_pointer)),
        2,
        "the unreached rooms' states"
    );
}

/// Every item the game places, taken once from a new file, counts 100 (the decode notes' "No
/// record: the full percentage"). The walk is checked against the population the notes give.
/// It needs only `SM_ROM`, and is skipped where it is unset.
#[test]
fn every_item_the_game_places_counts_a_hundred_percent() {
    use clean_model_progress::{Pickup, Supply};
    use core::num::Wrapping;
    use std::collections::BTreeSet;
    use std::io::Write as _;
    let Some(rom) = std::env::var_os("SM_ROM").map(PathBuf::from) else {
        let _ = writeln!(std::io::stdout(), "skipped: SM_ROM unset");
        return;
    };
    let image = match std::fs::read(&rom) {
        Ok(image) => image,
        Err(error) => panic!("SM_ROM {}: {error}", rom.display()),
    };
    let domain = super::progress::ProgressDomain::new(image);
    let population = domain
        .population()
        .unwrap_or_else(|error| panic!("the walk: {error}"));
    let mut tanks: BTreeMap<String, (usize, BTreeSet<u16>)> = BTreeMap::new();
    let mut one_off = 0;
    for pickup in population.items.values() {
        match *pickup {
            Pickup::Tank { supply, amount } => {
                let (count, amounts) = tanks.entry(format!("{supply:?}")).or_default();
                *count += 1;
                amounts.insert(amount.0);
            }
            Pickup::Item(_) | Pickup::Beam(_) => one_off += 1,
        }
    }
    let tank = |supply: Supply, count: usize, amount: u16| {
        (format!("{supply:?}"), (count, BTreeSet::from([amount])))
    };
    let expected = BTreeMap::from([
        tank(Supply::Energy, 14, 100),
        tank(Supply::Reserve, 4, 100),
        tank(Supply::Missiles, 46, 5),
        tank(Supply::SuperMissiles, 10, 5),
        tank(Supply::PowerBombs, 10, 5),
    ]);
    assert_eq!(population.rooms.len(), 261, "rooms the doors reach");
    assert_eq!(population.items.len(), 100, "locations");
    assert_eq!(tanks, expected, "tanks by kind: count and amounts");
    assert_eq!(one_off, 16, "one-off items and beams");
    assert_eq!(
        super::progress::percentage::of_everything(&population),
        Wrapping(100),
        "the item percentage"
    );
}

/// The HUD over every cached row, held to its floors; `SM_CACHE` and `SM_ROM` as the clock's test
/// reads them (the ROM for the area maps).
#[test]
fn the_hud_holds_its_floors_against_the_cache() {
    let Some((cache, rom)) = environment() else {
        return;
    };
    let image = match std::fs::read(&rom) {
        Ok(image) => image,
        Err(error) => panic!("SM_ROM {}: {error}", rom.display()),
    };
    let domain = match super::hud::HudDomain::load(&image) {
        Ok(domain) => domain,
        Err(error) => panic!("content: {error:?}"),
    };
    hold_floors(&domain, "hud", &cache);
}

/// The HUD's harness cases (the decode notes' "The harness cases"), each over its row's records;
/// a case whose row isn't cached is reported and not checked, and one skipped, masked or
/// differing fails.
#[test]
fn the_huds_harness_cases_hold_against_the_cache() {
    use std::io::Write as _;
    let Some((cache, rom)) = environment() else {
        return;
    };
    let image =
        std::fs::read(&rom).unwrap_or_else(|error| panic!("SM_ROM {}: {error}", rom.display()));
    let domain =
        super::hud::HudDomain::load(&image).unwrap_or_else(|error| panic!("content: {error:?}"));
    let lock = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenarios/locks/lsnes.toml");
    let text = std::fs::read_to_string(&lock).unwrap_or_else(|error| panic!("{error}"));
    let parsed = crate::lock::Lock::parse(&text).unwrap_or_else(|error| panic!("{error:?}"));
    let rows = cached_rows(&cache);
    let mut failures = Vec::new();
    for case in &super::hud::cases::CASES {
        if !rows.iter().any(|row| row == case.row) {
            let _ = writeln!(std::io::stdout(), "{}: {} not cached", case.id, case.row);
            continue;
        }
        let locked = parsed
            .row(&crate::lock::RowId(case.row.to_owned()))
            .unwrap_or_else(|| panic!("{}: not in the lock", case.row));
        let opened =
            crate::cache::Cache::open(&cache, locked).unwrap_or_else(|error| panic!("{error}"));
        let records: Vec<_> = opened
            .entries()
            .iter()
            .filter(|entry| {
                (case.checked.saturating_sub(1)..=case.checked).contains(&entry.frame.0)
            })
            .map(|entry| {
                opened
                    .record(entry)
                    .unwrap_or_else(|error| panic!("{}: {error}", case.row))
            })
            .collect();
        match domain.case(case, &records) {
            Ok(()) => {
                let _ = writeln!(std::io::stdout(), "{}: holds", case.id);
            }
            Err(differences) => failures.push(format!("{}: {}", case.id, differences.join("; "))),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
