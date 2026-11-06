use std::convert::Infallible;
use std::path::Path;
use std::time::Instant;

use oracle_record::{CpuRegisters, Cut, FrameRecord, Nmi, NmiKind, Pad, PpuRegisters};

use super::game_time::Clock;
use super::{Mismatch, Run, run, run_cached};
use crate::lock::Frame;
use crate::source::{self, WramWord};

/// A record holding only work RAM: the game state and the clock.
fn record(index: u64, game_state: u16, time: [u16; 4]) -> FrameRecord {
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
fn a_record_the_domain_cannot_read_is_skipped_with_its_reason() {
    let mut short = record(1, 8, [0; 4]);
    short.wram = Box::default();
    let Ok(report) = run(
        &Clock,
        [Ok::<_, Infallible>(record(0, 8, [0; 4])), Ok(short)],
    );
    assert_eq!((report.frames, report.compared), (1, 0));
    assert_eq!(report.skipped.values().sum::<u64>(), 1);
}

/// The frames on which the clock domain matches the oracle, per cached row: a floor, so a
/// change to the model or the harness that loses a frame fails. `item-morph-ball`'s 399 others
/// are frames 817 to 1215, where the clock holds though the game state stays gameplay.
const CLOCK_MATCHED: [(&str, u64); 2] = [("area-brinstar", 476), ("item-morph-ball", 1028)];

/// The clock domain over every cached row, against the record cache in `SM_CACHE`, held to
/// [`CLOCK_MATCHED`]. Skipped only where `SM_CACHE` and `SM_ROM` are both unset, as the ratchet
/// test is; with one set, or a path that isn't there, it fails. Run with `--nocapture` for the
/// reports and the rate.
#[test]
fn the_clock_holds_its_matched_frames_against_the_cache() {
    use std::io::Write as _;
    let cache = match (std::env::var_os("SM_CACHE"), std::env::var_os("SM_ROM")) {
        (Some(cache), Some(_)) => cache,
        (None, None) => {
            let _ = writeln!(std::io::stdout(), "skipped: SM_CACHE/SM_ROM unset");
            return;
        }
        (cache, rom) => panic!("SM_CACHE and SM_ROM go together: {cache:?}, {rom:?}"),
    };
    let cache = Path::new(&cache);
    assert!(
        cache.is_dir(),
        "SM_CACHE {}: not a directory",
        cache.display()
    );
    let lock = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenarios/locks/lsnes.toml");
    let started = Instant::now();
    let mut frames = 0_u64;
    for (row, floor) in CLOCK_MATCHED {
        let report = match run_cached(&Clock, &lock, cache, row) {
            Ok(report) => report,
            Err(error) => panic!("{row}: {error}"),
        };
        let _ = write!(std::io::stdout(), "{row}: {report}");
        assert!(report.matched >= floor, "{row}: {report}");
        frames = frames.saturating_add(report.frames);
    }
    let seconds = started.elapsed().as_secs_f64();
    let rate = f64::from(u32::try_from(frames).unwrap_or(u32::MAX)) / seconds.max(f64::EPSILON);
    let _ = writeln!(
        std::io::stdout(),
        "{frames} frames in {seconds:.2} s ({rate:.0} frames/s)"
    );
}
