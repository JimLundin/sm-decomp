//! The oracle harness's command line (`oracle`).
//!
//! `oracle movie ROM INPUTS [...]` plays a per-video-frame movie from power-on;
//!   `--lag-log` compares it frame by frame with an lsnes lag log and
//!   `--seeds` with lsnes room captures.
//! `oracle corpus ROM LOCK DATA [...]` runs the lock's rows and compares them.
//! `oracle ladder-check ROM INPUTS DIR` checks that each rung of a movie's
//!   ladder (`movie --ladder N DIR`), resumed, reproduces the next.
//! `oracle cross ROM INPUTS [--frames A..B]` steps LakeSnes and lsnes with the
//!   same pads from power-on and reports where their records differ.

use crate::lakesnes::LakeSnes;
use crate::{FrameRecord, NmiKind, Oracle, Start, Tiers};
use std::path::{Path, PathBuf};
use std::time::Instant;

fn fail(message: impl core::fmt::Display) -> ! {
    eprintln!("oracle: {message}");
    std::process::exit(2)
}

/// One row of an lsnes room-seed CSV.
struct RoomSeed {
    index: u32,
    room: u16,
    movie_frame: u64,
}

fn room_seeds(path: &Path) -> Vec<RoomSeed> {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|error| fail(format!("{}: {error}", path.display())));
    let mut lines = text.lines();
    let header: Vec<&str> = lines.next().unwrap_or_default().split(',').collect();
    let column = |name: &str| {
        header
            .iter()
            .position(|&field| field == name)
            .unwrap_or_else(|| fail(format!("{}: no column {name}", path.display())))
    };
    let (index, room, frame) = (column("room_index"), column("room"), column("movie_frame"));
    lines
        .filter(|line| !line.is_empty())
        .map(|line| {
            let fields: Vec<&str> = line.split(',').collect();
            RoomSeed {
                index: fields[index].parse().unwrap_or_else(|_| fail(line)),
                room: u16::from_str_radix(fields[room], 16).unwrap_or_else(|_| fail(line)),
                movie_frame: fields[frame].parse().unwrap_or_else(|_| fail(line)),
            }
        })
        .collect()
}

fn word(wram: &[u8], address: usize) -> u16 {
    u16::from_le_bytes([wram[address], wram[address + 1]])
}

/// The bytes of `left` that differ from `right` in the game's work RAM
/// below $2000 and in $7E:2000-$7F:FFFF, excluding the stack page.
fn differing(left: &[u8], right: &[u8]) -> usize {
    left.iter()
        .zip(right)
        .enumerate()
        .filter(|&(address, (a, b))| a != b && !(0x1f00..0x2000).contains(&address))
        .count()
}

/// A save asked for after one step, written by lsnes once the next step has
/// run: the native seeds and the ladder rung it serves.
struct PendingSave {
    state: PathBuf,
    seeds: Vec<PathBuf>,
    rung: Option<(u64, Vec<u8>)>,
}

/// The ladder a movie run cuts: a rung every `every` frames.
struct Cutting {
    ladder: crate::ladder::Ladder,
    every: u64,
}

impl Cutting {
    /// Whether step `step` (0-based) ends right before a rung that a run of
    /// `frames` steps can still save (its state lands one step later).
    fn rung_after(&self, step: u64, frames: u64) -> bool {
        (step + 1).is_multiple_of(self.every) && step + 1 < frames
    }
}

/// Cut harness-native seeds and ladder rungs on lsnes. The seed of a
/// room-seed row whose capture followed lsnes frame N (1-based) is the state at
/// the boundary after the cut of step N-1 (0-based), with that cut's raw
/// record; rung F likewise. Captures of several lists and a rung can share a
/// frame; one saved state then serves them all.
fn save_after_step(
    lsnes: &mut crate::lsnes::Lsnes,
    lists: &[(Vec<RoomSeed>, PathBuf)],
    mut cutting: Option<&mut Cutting>,
    (step, frames): (u64, u64),
    pending: &mut Option<PendingSave>,
) {
    if let Some(save) = pending.take() {
        for target in save.seeds {
            std::fs::copy(&save.state, &target)
                .unwrap_or_else(|error| fail(format!("{}: {error}", save.state.display())));
            println!("native seed {}: state after step {step}", target.display());
        }
        if let (Some((frame, record)), Some(cutting)) = (save.rung, cutting.as_deref_mut()) {
            cutting
                .ladder
                .add(frame, &record, &save.state)
                .unwrap_or_else(|error| fail(error));
            println!("rung {}", cutting.ladder.state_path(frame).display());
        }
    }
    let mut seeds = Vec::new();
    for (list, directory) in lists {
        for seed in list.iter().filter(|seed| seed.movie_frame == step + 1) {
            std::fs::write(
                directory.join(format!("{:04}.record", seed.index)),
                lsnes.raw(),
            )
            .unwrap_or_else(|error| fail(error));
            seeds.push(directory.join(format!("{:04}.lss", seed.index)));
        }
    }
    let rung = cutting
        .filter(|cutting| cutting.rung_after(step, frames))
        .map(|_| (step + 1, lsnes.raw().to_vec()));
    if !seeds.is_empty() || rung.is_some() {
        *pending = Some(PendingSave {
            state: lsnes.request_save().unwrap_or_else(|error| fail(error)),
            seeds,
            rung,
        });
    }
}

fn movie(arguments: &[String]) {
    let [rom, inputs, rest @ ..] = arguments else {
        fail(
            "movie ROM INPUTS [--frames N] [--seeds CSV DIR] [--ranges] [--lag-log CSV] [--watch FROM EVERY] [--spc] [--no-picture] [--oracle lakesnes|lsnes] [--native-seeds CSV DIR]... [--no-memories] [--from STATE FRAME] [--ladder N DIR] [--from-ladder DIR]",
        )
    };
    let rom = std::fs::read(rom).unwrap_or_else(|error| fail(format!("{rom}: {error}")));
    let pads = crate::inputs::read(Path::new(inputs)).unwrap_or_else(|error| fail(error));
    let mut frames = pads.len();
    let mut seeds: Option<(Vec<RoomSeed>, PathBuf)> = None;
    let mut picture = true;
    let mut ranges = false;
    let mut lag_log: Option<Vec<[i64; 7]>> = None;
    let mut lag_mismatches = 0;
    let mut watch: Option<(usize, usize)> = None;
    let mut spc = false;
    let mut positions_only = false;
    let mut profile: Option<(usize, usize)> = None;
    let mut backend = "lakesnes".to_owned();
    let mut native: Vec<(Vec<RoomSeed>, PathBuf)> = Vec::new();
    let mut memories = true;
    let mut start = Start::PowerOn { wram_fill: 0x55 };
    let mut first = 0;
    let mut cutting: Option<Cutting> = None;
    let mut from_ladder: Option<PathBuf> = None;
    let mut options = rest.iter();
    while let Some(option) = options.next() {
        match option.as_str() {
            "--frames" => {
                frames = options
                    .next()
                    .and_then(|value| value.parse().ok())
                    .unwrap_or_else(|| fail("--frames N"));
            }
            "--seeds" => {
                let (Some(csv), Some(directory)) = (options.next(), options.next()) else {
                    fail("--seeds CSV DIR")
                };
                seeds = Some((room_seeds(Path::new(csv)), PathBuf::from(directory)));
            }
            "--oracle" => {
                backend = options
                    .next()
                    .cloned()
                    .unwrap_or_else(|| fail("--oracle NAME"));
            }
            "--native-seeds" => {
                let (Some(csv), Some(directory)) = (options.next(), options.next()) else {
                    fail("--native-seeds CSV DIR")
                };
                native.push((room_seeds(Path::new(csv)), PathBuf::from(directory)));
            }
            "--from" => {
                let (Some(state), Some(frame)) = (options.next(), options.next()) else {
                    fail("--from STATE FRAME")
                };
                start = Start::State(
                    std::fs::read(state).unwrap_or_else(|error| fail(format!("{state}: {error}"))),
                );
                first = frame.parse().unwrap_or_else(|_| fail("--from STATE FRAME"));
            }
            "--ladder" => {
                let (Some(every), Some(directory)) = (
                    options.next().and_then(|every| every.parse().ok()),
                    options.next(),
                ) else {
                    fail("--ladder N DIR")
                };
                if every == 0 {
                    fail("--ladder N DIR: N is at least 1");
                }
                let ladder = crate::ladder::Ladder::open(Path::new(directory), Path::new(inputs))
                    .unwrap_or_else(|error| fail(error));
                cutting = Some(Cutting { ladder, every });
            }
            "--from-ladder" => {
                from_ladder = Some(PathBuf::from(
                    options.next().unwrap_or_else(|| fail("--from-ladder DIR")),
                ));
            }
            "--no-picture" => picture = false,
            // lsnes only: records without the memories, about twice as fast;
            // for cutting native seeds, not for comparing.
            "--no-memories" => memories = false,
            "--ranges" => ranges = true,
            "--spc" => spc = true,
            "--positions-only" => positions_only = true,
            "--profile" => {
                let mut number = || {
                    options
                        .next()
                        .and_then(|value| value.parse().ok())
                        .unwrap_or_else(|| fail("--profile FROM TO"))
                };
                profile = Some((number(), number()));
            }
            "--watch" => {
                let mut number = || {
                    options
                        .next()
                        .and_then(|value| value.parse().ok())
                        .unwrap_or_else(|| fail("--watch FROM EVERY"))
                };
                watch = Some((number(), number()));
            }
            "--lag-log" => {
                lag_log = Some(read_lag_log(Path::new(
                    options.next().unwrap_or_else(|| fail("--lag-log CSV")),
                )));
            }
            other => fail(format!("unknown option {other}")),
        }
    }
    if let Some(directory) = &from_ladder {
        // The earliest seed is cut after step F-1, so the run starts at or before it.
        let Some(target) = native
            .iter()
            .flat_map(|(list, _)| list.iter().map(|seed| seed.movie_frame.saturating_sub(1)))
            .min()
        else {
            fail(
                "--from-ladder resumes for --native-seeds; for another frame use `oracle ladder-rung`",
            )
        };
        let ladder = crate::ladder::Ladder::open(directory, Path::new(inputs))
            .unwrap_or_else(|error| fail(error));
        match ladder.nearest(target) {
            Some(rung) => {
                let path = ladder.state_path(rung);
                start = Start::State(
                    std::fs::read(&path)
                        .unwrap_or_else(|error| fail(format!("{}: {error}", path.display()))),
                );
                first = usize::try_from(rung).unwrap_or_else(|_| fail("rung past the movie"));
                println!("from rung {} for step {target}", path.display());
            }
            None => println!("no rung at or before step {target}: from power-on"),
        }
    }
    if (cutting.is_some() || !native.is_empty()) && backend != "lsnes" {
        fail("--ladder and --native-seeds need --oracle lsnes");
    }
    let mut oracle =
        crate::Backend::open(&backend, &rom, start).unwrap_or_else(|error| fail(error));
    let mut pending_save: Option<PendingSave> = None;
    oracle.set_tiers(Tiers {
        picture,
        apu_state: spc,
        profile: profile.is_some(),
        ..Tiers::default()
    });
    let mut history: std::collections::VecDeque<Box<[u8]>> = std::collections::VecDeque::new();
    let mut next_seed = 0;
    let (mut lag, mut overrun, mut no_nmi, mut synced, mut checked) = (0u64, 0u64, 0u64, 0, 0);
    let started = Instant::now();
    let run = u64::try_from(frames.min(pads.len())).unwrap_or(u64::MAX);
    for (frame, &pad) in pads.iter().enumerate().take(frames).skip(first) {
        let step = u64::try_from(frame).unwrap_or(u64::MAX);
        if let crate::Backend::Lsnes(lsnes) = &mut oracle {
            // A rung's record always carries the memories, so that resuming
            // the rung before can be checked against all of them.
            lsnes.memories = memories
                || cutting
                    .as_ref()
                    .is_some_and(|cutting| cutting.rung_after(step, run));
        }
        let record: &FrameRecord = oracle.step(pad).unwrap_or_else(|error| fail(error));
        let record = record.clone();
        let record = &record;
        if let crate::Backend::Lsnes(lsnes) = &mut oracle
            && (!native.is_empty() || cutting.is_some())
        {
            save_after_step(
                lsnes,
                &native,
                cutting.as_mut(),
                (step, run),
                &mut pending_save,
            );
        }
        match record.nmi() {
            Some(nmi) => {
                lag += u64::from(!nmi.polled);
                overrun += u64::from(nmi.kind == NmiKind::Overrun);
            }
            None => no_nmi += 1,
        }
        if let Some(log) = &lag_log
            && let Some(row) = log.get(frame)
        {
            let w = |address| i64::from(word(&record.wram, address));
            let lagged = i64::from(record.nmi().is_none_or(|nmi| !nmi.polled));
            let mine = [
                w(0x05b6),
                w(0x05b8),
                lagged,
                w(0x0998),
                w(0x079b),
                w(0x0af6),
                w(0x0afa),
            ];
            let compared = if positions_only { 3 } else { 0 };
            if mine[compared..] != row[compared..] {
                lag_mismatches += 1;
                if lag_mismatches <= 40 {
                    println!(
                        "frame {frame}: lsnes {row:?} emulator {mine:?} wait entry {:?}",
                        record.wait_entry
                    );
                }
            }
        }
        if let Some((from, to)) = profile
            && (from..=to).contains(&frame)
            && let Some(banks) = &record.bank_instructions
        {
            let mut top: Vec<(u32, usize)> = banks
                .iter()
                .enumerate()
                .map(|(bank, &count)| (count, bank))
                .filter(|&(count, _)| count > 0)
                .collect();
            top.sort_unstable_by(|a, b| b.cmp(a));
            let top: Vec<String> = top
                .iter()
                .take(5)
                .map(|(count, bank)| format!("{bank:02x}:{count}"))
                .collect();
            println!(
                "profile {frame}: {} cut {:?} wait {:?} joypad reads {} apu {}",
                top.join(" "),
                record.cut,
                record.wait_entry,
                record.joypad_reads,
                record.apu_log.len()
            );
        }
        if let Some((from, every)) = watch
            && frame >= from
            && frame.is_multiple_of(every)
        {
            let w = |address| word(&record.wram, address);
            if let Some(spc) = &record.spc {
                println!(
                    "spc {frame}: pc {:04x} sp {:02x} a {:02x} x {:02x} y {:02x} psw {:02x} ports in {:02x?} out {:02x?} f1 {:02x} fa {:02x?}",
                    spc.pc,
                    spc.sp,
                    spc.a,
                    spc.x,
                    spc.y,
                    spc.psw,
                    spc.cpu_to_apu,
                    spc.apu_to_cpu,
                    spc.ram[0xf1],
                    &spc.ram[0xfa..0xfd]
                );
            }
            println!(
                "watch {frame}: state {:04x} room {:04x} x {:04x} y {:04x} pose {:04x} hp {:04x} cut {:?} pad {:04x}",
                w(0x0998),
                w(0x079b),
                w(0x0af6),
                w(0x0afa),
                w(0x0a1c),
                w(0x09c2),
                record.cut,
                record.pad.0
            );
        }
        let Some((list, directory)) = &seeds else {
            continue;
        };
        history.push_back(record.wram.clone());
        if history.len() > 7 {
            history.pop_front();
        }
        // Compare once the frames after the seed's movie frame are in.
        while let Some(seed) = list.get(next_seed) {
            let frame = frame as u64 + 1;
            if seed.movie_frame + 3 > frame {
                break;
            }
            next_seed += 1;
            let Ok(capture) = crate::seed::Capture::read(directory, seed.index) else {
                continue;
            };
            checked += 1;
            let counts: Vec<usize> = history
                .iter()
                .map(|wram| differing(wram, &capture.wram))
                .collect();
            if ranges {
                println!("  differing per frame, oldest first: {counts:?}");
            }
            let (best, offset) = history
                .iter()
                .enumerate()
                .map(|(age, wram)| (differing(wram, &capture.wram), age))
                .min()
                .unwrap_or((usize::MAX, 0));
            let newest = history.len() - 1;
            let at = (frame - 1) as i64 - (newest - offset) as i64 - seed.movie_frame as i64;
            let room = word(&history[offset], 0x079b);
            let matched = room == seed.room;
            synced += usize::from(matched);
            if ranges {
                print_ranges(&history[offset], &capture.wram);
            }
            println!(
                "seed {:4} room {:04X} frame {:6}: emulator room {room:04X} {} ; WRAM bytes differing {best} at frame offset {at:+}",
                seed.index,
                seed.room,
                seed.movie_frame,
                if matched { "match" } else { "DIFFERS" }
            );
        }
    }
    let seconds = started.elapsed().as_secs_f64();
    println!(
        "frames {frames} in {seconds:.2}s: {:.0} frames/s; NMIs without a joypad read {lag}, overrun NMIs {overrun}, frames without NMI {no_nmi}",
        frames.saturating_sub(first) as f64 / seconds
    );
    if seeds.is_some() {
        println!("room seeds in the same room: {synced} of {checked}");
    }
}

/// Derive an lsnes native seed: the state of `BASE` with the words written
/// right after it loads (`NativeSeed`), saved as `OUT` and `OUT`'s `.words`.
/// Then prove it: one step from `OUT` as read back must show every word
/// written, and the work-RAM bytes differing from one step of the plain base
/// are reported. `tools/scenario_seeds.py native-args` gives a scenario's
/// arguments.
fn derive_seed(arguments: &[String]) {
    use crate::lsnes::{Lsnes, NativeSeed};
    let [rom, base, out, rest @ ..] = arguments else {
        fail("derive-seed ROM BASE.lss OUT.lss [--word OFFSET=VALUE]...")
    };
    let rom = std::fs::read(rom).unwrap_or_else(|error| fail(format!("{rom}: {error}")));
    let base = NativeSeed::read(Path::new(base)).unwrap_or_else(|error| fail(error));
    let mut seed = base.clone();
    let mut options = rest.iter();
    while let Some(option) = options.next() {
        match option.as_str() {
            "--word" => seed.words.push(
                options
                    .next()
                    .unwrap_or_else(|| fail("--word OFFSET=VALUE"))
                    .parse()
                    .unwrap_or_else(|error| fail(error)),
            ),
            other => fail(format!("unknown option {other}")),
        }
    }
    let out = Path::new(out);
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).unwrap_or_else(|error| fail(error));
    }
    seed.write(out).unwrap_or_else(|error| fail(error));
    let step = |seed: &NativeSeed| {
        let mut lsnes =
            Lsnes::open(&rom, Start::State(seed.state.clone())).unwrap_or_else(|error| fail(error));
        seed.apply(&mut lsnes).unwrap_or_else(|error| fail(error));
        lsnes
            .step(crate::record::Pad(0))
            .unwrap_or_else(|error| fail(error))
            .wram
            .clone()
    };
    let plain = step(&base);
    let derived = step(&NativeSeed::read(out).unwrap_or_else(|error| fail(error)));
    for word in &seed.words {
        let (before, after) = (word.read(&plain), word.read(&derived));
        println!(
            "${:05X}: {before:#06x} -> {after:#06x} after one step (written {:#06x})",
            word.offset, word.value
        );
        if after != word.value {
            fail(format!("${:05X} did not keep its write", word.offset));
        }
    }
    println!(
        "derived {}: {} work-RAM bytes differ from the base after one step",
        out.display(),
        differing(&derived, &plain)
    );
}

/// Run the command line `arguments` (without the program name).
pub fn main(arguments: &[String]) -> i32 {
    match arguments.split_first() {
        Some((command, rest)) if command == "movie" => movie(rest),
        Some((command, rest)) if command == "corpus" => corpus(rest),
        Some((command, rest)) if command == "trace" => trace(rest),
        Some((command, rest)) if command == "measure" => measure(rest),
        Some((command, rest)) if command == "measure-movie" => measure_movie(rest),
        Some((command, rest)) if command == "derive-seed" => derive_seed(rest),
        Some((command, rest)) if command == "ladder-check" => return ladder_check(rest),
        Some((command, rest)) if command == "ladder-rung" => ladder_rung(rest),
        Some((command, rest)) if command == "cross" => cross(rest),
        _ => fail("usage: oracle movie ROM INPUTS [...]"),
    }
    0
}

/// `ladder-check ROM INPUTS DIR [--rungs A..B]`: resume each rung of the
/// movie's ladder whose frame lies in A..B (inclusive; all by default), run to
/// the next rung and compare the record of the step before it with the stored
/// one (`crate::ladder`). Exits 1 when any rung does not reproduce the next.
fn ladder_check(arguments: &[String]) -> i32 {
    let usage = "ladder-check ROM INPUTS DIR [--rungs A..B]";
    let [rom, inputs, directory, rest @ ..] = arguments else {
        fail(usage)
    };
    let rungs = match rest {
        [] => 0..=u64::MAX,
        [option, range] if option == "--rungs" => {
            let Some((Ok(low), Ok(high))) = range
                .split_once("..")
                .map(|(low, high)| (low.parse(), high.parse()))
            else {
                fail(usage)
            };
            low..=high
        }
        _ => fail(usage),
    };
    let rom = std::fs::read(rom).unwrap_or_else(|error| fail(format!("{rom}: {error}")));
    let pads = crate::inputs::read(Path::new(inputs)).unwrap_or_else(|error| fail(error));
    let ladder = crate::ladder::Ladder::open(Path::new(directory), Path::new(inputs))
        .unwrap_or_else(|error| fail(error));
    let frames: Vec<u64> = ladder.frames().collect();
    let (mut equal, mut different) = (0u32, 0u32);
    for pair in frames.windows(2) {
        let &[from, to] = pair else { continue };
        if !rungs.contains(&from) {
            continue;
        }
        let started = Instant::now();
        let verdict = check_rung(&rom, &pads, &ladder, from, to);
        let seconds = started.elapsed().as_secs_f64();
        match verdict {
            Ok(parts) if parts.is_empty() => {
                equal += 1;
                println!("rung {from} -> {to}: equal ({seconds:.0}s)");
            }
            Ok(parts) => {
                different += 1;
                let parts: Vec<String> = parts
                    .iter()
                    .map(|(part, bytes)| format!("{part} {bytes}"))
                    .collect();
                println!(
                    "rung {from} -> {to}: different, bytes {} ({seconds:.0}s)",
                    parts.join(", ")
                );
            }
            Err(error) => {
                different += 1;
                println!("rung {from} -> {to}: failed: {error}");
            }
        }
    }
    println!(
        "ladder-check: {equal} equal, {different} different or failed, of {}",
        equal + different
    );
    i32::from(different > 0)
}

/// Resume rung `from` and step to rung `to`: the parts in which the record of
/// step `to - 1` differs from the stored one, empty when it reproduces it.
fn check_rung(
    rom: &[u8],
    pads: &[crate::Pad],
    ladder: &crate::ladder::Ladder,
    from: u64,
    to: u64,
) -> Result<Vec<(&'static str, usize)>, String> {
    let read = |path: PathBuf| {
        std::fs::read(&path).map_err(|error| format!("{}: {error}", path.display()))
    };
    let stored = read(ladder.record_path(to))?;
    let sha = crate::ladder::hex(&crate::ladder::sha256(&stored));
    if ladder.record_sha256(to) != Some(sha.as_str()) {
        return Err(format!(
            "rung {to}'s record is not the one its manifest lists"
        ));
    }
    let state = read(ladder.state_path(from))?;
    let mut lsnes =
        crate::lsnes::Lsnes::open(rom, Start::State(state)).map_err(|error| error.to_string())?;
    let steps = usize::try_from(from)
        .ok()
        .zip(usize::try_from(to).ok())
        .and_then(|(from, to)| pads.get(from..to))
        .ok_or_else(|| format!("rungs {from}..{to} lie past the movie's end"))?;
    let last = steps.len().saturating_sub(1);
    for (step, &pad) in steps.iter().enumerate() {
        lsnes.memories = step == last;
        lsnes.step(pad).map_err(|error| error.to_string())?;
    }
    Ok(crate::ladder::differences(lsnes.raw(), &stored))
}

/// `ladder-rung INPUTS DIR FRAME`: the latest rung of the movie's ladder at or
/// before FRAME, as `FRAME PATH`, to resume with `movie --from PATH FRAME`.
fn ladder_rung(arguments: &[String]) {
    let [inputs, directory, frame] = arguments else {
        fail("ladder-rung INPUTS DIR FRAME")
    };
    let frame = frame
        .parse()
        .unwrap_or_else(|_| fail("ladder-rung INPUTS DIR FRAME"));
    let ladder = crate::ladder::Ladder::open(Path::new(directory), Path::new(inputs))
        .unwrap_or_else(|error| fail(error));
    match ladder.nearest(frame) {
        Some(rung) => println!("{rung} {}", ladder.state_path(rung).display()),
        None => fail(format!("no rung at or before frame {frame}")),
    }
}

/// Print the differing address ranges of two WRAM images, coalescing gaps under 16 bytes.
fn print_ranges(left: &[u8], right: &[u8]) {
    let mut ranges: Vec<(usize, usize, usize)> = Vec::new();
    for (address, (a, b)) in left.iter().zip(right).enumerate() {
        if a == b {
            continue;
        }
        match ranges.last_mut() {
            Some((_, end, count)) if address - *end < 16 => {
                *end = address;
                *count += 1;
            }
            _ => ranges.push((address, address, 1)),
        }
    }
    for (start, end, count) in ranges.iter().take(60) {
        let shown = (*end - *start + 1).min(20);
        println!(
            "  {:05X}-{:05X} {count} bytes: emulator {:02x?} capture {:02x?}",
            start,
            end,
            &left[*start..*start + shown],
            &right[*start..*start + shown]
        );
    }
    println!("  {} ranges", ranges.len());
}

/// An lsnes lag log (`tools/lsnes_lag_log.lua`): per movie frame, the frame
/// counter $05B6, the NMI counter $05B8, the lag flag, the game state $0998,
/// the room $079B and Samus's X and Y ($0AF6, $0AFA) after the frame.
fn read_lag_log(path: &Path) -> Vec<[i64; 7]> {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|error| fail(format!("{}: {error}", path.display())));
    text.lines()
        .skip(1)
        .map(|line| {
            let fields: Vec<i64> = line
                .split(',')
                .map(|field| field.parse().unwrap_or(-1))
                .collect();
            [
                fields[3], fields[5], fields[6], fields[7], fields[8], fields[9], fields[10],
            ]
        })
        .collect()
}

/// `corpus ROM LOCK DATA [--only ID]... [--limit N]`: run the lock's rows
/// whose private data is restored under DATA (one directory per bundle) and
/// compare each with its `faithful` entry.
fn corpus(arguments: &[String]) {
    let [rom, lock, data, rest @ ..] = arguments else {
        fail(
            "corpus ROM LOCK DATA [--only ID]... [--shard I/N] [--limit N] [--record FILE] [--oracle lakesnes|lsnes] [--native-seeds DIR] [--watch-words ADDR,...]",
        )
    };
    let rom = std::fs::read(rom).unwrap_or_else(|error| fail(format!("{rom}: {error}")));
    let lock =
        std::fs::read_to_string(lock).unwrap_or_else(|error| fail(format!("{lock}: {error}")));
    let lock = crate::lock::Lock::parse(&lock);
    let data = Path::new(data);
    let mut only = Vec::new();
    let mut limit = usize::MAX;
    let mut shard: (usize, usize) = (0, 1);
    let mut record: Option<std::fs::File> = None;
    let mut backend = String::new();
    let mut native_seeds: Option<PathBuf> = None;
    let mut watched: Vec<usize> = Vec::new();
    let mut rom = rom;
    let mut options = rest.iter();
    while let Some(option) = options.next() {
        match option.as_str() {
            "--only" => only.push(options.next().unwrap_or_else(|| fail("--only ID")).clone()),
            "--shard" => {
                let text = options.next().unwrap_or_else(|| fail("--shard I/N"));
                let (index, count) = text.split_once('/').unwrap_or_else(|| fail("--shard I/N"));
                shard = (
                    index.parse().unwrap_or_else(|_| fail("--shard I/N")),
                    count.parse().unwrap_or_else(|_| fail("--shard I/N")),
                );
            }
            "--diagnose-kept-patches" => {
                // Diagnosis only, never an oracle: the old stepper's kept ROM
                // patches applied to a copy of the cartridge, to test whether
                // a lock mismatch is theirs.
                let names = options
                    .next()
                    .unwrap_or_else(|| fail("--diagnose-kept-patches LIST"));
                for name in names.split(',') {
                    let patch = crate::diagnose::KEPT_PATCHES
                        .iter()
                        .find(|patch| patch.name == name)
                        .unwrap_or_else(|| fail(format!("no kept patch {name}")));
                    crate::diagnose::apply(&mut rom, patch);
                }
            }
            "--record" => {
                let path = options.next().unwrap_or_else(|| fail("--record FILE"));
                record = Some(
                    std::fs::File::create(path)
                        .unwrap_or_else(|error| fail(format!("{path}: {error}"))),
                );
            }
            "--oracle" => {
                backend = options
                    .next()
                    .cloned()
                    .unwrap_or_else(|| fail("--oracle NAME"));
            }
            "--native-seeds" => {
                native_seeds = Some(PathBuf::from(
                    options.next().unwrap_or_else(|| fail("--native-seeds DIR")),
                ));
            }
            "--watch-words" => {
                // Work-RAM words (hex) printed whenever their value changes,
                // to read a row's outcome (game state, energy) off the oracle.
                let list = options
                    .next()
                    .unwrap_or_else(|| fail("--watch-words ADDR,..."));
                for address in list.split(',') {
                    watched.push(
                        usize::from_str_radix(address.trim_start_matches('$'), 16)
                            .unwrap_or_else(|_| fail(format!("--watch-words: {address}"))),
                    );
                }
            }
            "--limit" => {
                limit = options
                    .next()
                    .and_then(|value| value.parse().ok())
                    .unwrap_or_else(|| fail("--limit N"));
            }
            other => fail(format!("unknown option {other}")),
        }
    }
    let names = crate::streams::NAMES;
    let mut matched = [0u32; 8];
    let mut compared = [0u32; 8];
    let (mut rows, mut failed, mut all_six) = (0u32, 0u32, 0u32);
    let (mut ticks, mut frames, mut seconds) = (0u64, 0u64, 0.0f64);
    let entries = lock
        .entries
        .iter()
        .filter(|entry| entry.oracle == "faithful")
        .filter(|entry| only.is_empty() || only.contains(&entry.id))
        .enumerate()
        .filter(|(position, _)| position % shard.1 == shard.0)
        .map(|(_, entry)| entry)
        .take(limit);
    for entry in entries {
        let Some(pin) = lock.pin(&entry.id) else {
            println!("{}: no pin", entry.id);
            continue;
        };
        let observing = crate::corpus::Observe {
            backend: &backend,
            native_seeds: native_seeds.as_deref(),
            ..crate::corpus::Observe::default()
        };
        let mut last: Vec<Option<u16>> = vec![None; watched.len()];
        let mut frame = 0u64;
        let mut watch = |record: &FrameRecord| {
            for (address, last) in watched.iter().zip(last.iter_mut()) {
                let value = word(&record.wram, *address);
                if *last != Some(value) {
                    println!("  {} frame {frame}: ${address:04x}={value:#06x}", entry.id);
                    *last = Some(value);
                }
            }
            frame += 1;
        };
        match crate::corpus::run_row_with(&rom, pin, entry, data, &observing, &mut watch) {
            Err(error) => {
                failed += 1;
                println!("{}: {error}", entry.id);
            }
            Ok(run) => {
                rows += 1;
                ticks += u64::from(run.ticks);
                frames += run.frames;
                seconds += run.seconds;
                if let Some(file) = record.as_mut() {
                    use std::io::Write as _;
                    let oracle = if backend == "lsnes" {
                        crate::lsnes::IDENTITY
                    } else {
                        crate::lakesnes::IDENTITY
                    };
                    let text = crate::corpus::lock_entry(entry, oracle, &run);
                    writeln!(file, "{text}").unwrap_or_else(|error| fail(error));
                }
                let verdicts = crate::corpus::compare(entry, &run);
                let mut line = String::new();
                for (stream, verdict) in verdicts.iter().enumerate() {
                    if let Some(same) = verdict {
                        compared[stream] += 1;
                        matched[stream] += u32::from(*same);
                        line.push_str(&format!(
                            " {}={}",
                            names[stream],
                            if *same { "ok" } else { "DIFF" }
                        ));
                    }
                }
                all_six += u32::from(verdicts.iter().flatten().all(|&same| same));
                println!(
                    "{}: {} ticks, {} frames, {} overrun, {} without NMI, {:.0} frames/s;{line}",
                    entry.id,
                    run.ticks,
                    run.frames,
                    run.overrun,
                    run.without_nmi,
                    run.frames as f64 / run.seconds
                );
            }
        }
    }
    println!(
        "rows run {rows}, failed or not run {failed}; all comparable streams equal on {all_six}"
    );
    for (stream, name) in names.iter().enumerate() {
        if compared[stream] > 0 {
            println!(
                "  {name}: {} of {} rows equal",
                matched[stream], compared[stream]
            );
        } else {
            println!("  {name}: not comparable (redefined by the oracle contract)");
        }
    }
    println!(
        "{ticks} ticks, {frames} frames in {seconds:.1}s: {:.0} ticks/s, {:.0} frames/s",
        ticks as f64 / seconds,
        frames as f64 / seconds
    );
}

/// `trace ROM LOCK DATA ID FROM TO`: one row's frames FROM-TO in detail.
fn trace(arguments: &[String]) {
    let [rom, lock, data, id, from, to] = arguments else {
        fail("trace ROM LOCK DATA ID FROM TO")
    };
    let rom = std::fs::read(rom).unwrap_or_else(|error| fail(format!("{rom}: {error}")));
    let lock =
        std::fs::read_to_string(lock).unwrap_or_else(|error| fail(format!("{lock}: {error}")));
    let lock = crate::lock::Lock::parse(&lock);
    let (Some(pin), Some(entry)) = (lock.pin(id), lock.entry(id, "faithful")) else {
        fail(format!("{id}: no pin or entry"))
    };
    let range = from.parse().unwrap_or(0)..=to.parse().unwrap_or(0);
    let mut frame = 0usize;
    let tiers = Tiers {
        apu_state: true,
        ..Tiers::default()
    };
    let result = crate::corpus::run_row_with(
        &rom,
        pin,
        entry,
        Path::new(data),
        &crate::corpus::Observe {
            tiers,
            ..Default::default()
        },
        &mut |record| {
            if range.contains(&frame) {
                let w = |address| word(&record.wram, address);
                let spc = record
                    .spc
                    .as_ref()
                    .map(|spc| (spc.pc, spc.cpu_to_apu, spc.apu_to_cpu));
                let apu: Vec<String> = record
                    .apu_log
                    .iter()
                    .take(12)
                    .map(|access| format!("{:?}{}={:02x}", access.kind, access.port, access.value))
                    .collect();
                println!(
                    "frame {frame}: cut {:?} cpu {:02x}:{:04x} state {:04x} room {:04x} spc {spc:02x?} apu[{}] {}",
                    record.cut,
                    record.cpu.k,
                    record.cpu.pc,
                    w(0x0998),
                    w(0x079b),
                    record.apu_log.len(),
                    apu.join(" ")
                );
            }
            frame += 1;
        },
    );
    if let Err(error) = result {
        println!("{id}: {error}");
    }
}

/// The tiers and watched addresses the measurements need.
fn measuring() -> Tiers {
    Tiers {
        picture: false,
        write_log: true,
        ..Tiers::default()
    }
}

/// `measure ROM LOCK DATA ID...`: the architecture lane's measurements over
/// lock rows (from their lsnes seeds), as Markdown.
fn measure(arguments: &[String]) {
    let [rom_path, lock, data, ids @ ..] = arguments else {
        fail("measure ROM LOCK DATA ID...")
    };
    let rom = std::fs::read(rom_path).unwrap_or_else(|error| fail(format!("{rom_path}: {error}")));
    let lock =
        std::fs::read_to_string(lock).unwrap_or_else(|error| fail(format!("{lock}: {error}")));
    let lock = crate::lock::Lock::parse(&lock);
    for id in ids {
        let (Some(pin), Some(entry)) = (lock.pin(id), lock.entry(id, "faithful")) else {
            println!("### {id}\n- no pin or entry");
            continue;
        };
        let mut measurements = crate::measure::Measurements::default();
        let result = crate::corpus::run_row_with(
            &rom,
            pin,
            entry,
            Path::new(data),
            &crate::corpus::Observe {
                tiers: measuring(),
                watch: &crate::measure::WATCHED,
                peek: crate::measure::PEEK,
                ..crate::corpus::Observe::default()
            },
            &mut |record| measurements.frame(record),
        );
        println!("### {id} (from its lsnes seed)");
        if let Err(error) = result {
            println!("- run ended: {error}");
        }
        print!("{}", measurements.report(&rom));
    }
}

/// `measure-movie ROM INPUTS FRAMES`: the measurements over a movie's first
/// FRAMES frames from power-on, as Markdown.
fn measure_movie(arguments: &[String]) {
    let [rom_path, inputs, frames] = arguments else {
        fail("measure-movie ROM INPUTS FRAMES")
    };
    let rom = std::fs::read(rom_path).unwrap_or_else(|error| fail(format!("{rom_path}: {error}")));
    let pads = crate::inputs::read(Path::new(inputs)).unwrap_or_else(|error| fail(error));
    let frames: usize = frames.parse().unwrap_or_else(|_| fail("FRAMES"));
    let mut oracle = LakeSnes::open(&rom, Start::PowerOn { wram_fill: 0x55 })
        .unwrap_or_else(|error| fail(error));
    oracle.set_tiers(measuring());
    oracle.set_watch(&crate::measure::WATCHED, crate::measure::PEEK);
    let mut measurements = crate::measure::Measurements::default();
    for &pad in pads.iter().take(frames) {
        measurements.frame(oracle.step(pad).unwrap_or_else(|error| fail(error)));
    }
    println!("### {inputs}, frames 0-{} (from power-on)", frames - 1);
    print!("{}", measurements.report(&rom));
}

/// `cross ROM INPUTS [--frames A..B] [--ranges] [--ranges-at A..B]`: step
/// LakeSnes (left) and lsnes (right) from power-on with the movie's pads
/// through step B (0-based, inclusive), and print for each step from A whether
/// their records are equal or which parts differ (`crate::cross`), then the
/// first differing step of each part. `--ranges` lists the differing address
/// ranges of each memory at its first differing step, `--ranges-at` at every
/// differing step in its window.
fn cross(arguments: &[String]) {
    use crate::cross::{Compare, Difference, Part, describe};
    let usage = "cross ROM INPUTS [--frames A..B] [--ranges] [--ranges-at A..B]";
    let [rom, inputs, rest @ ..] = arguments else {
        fail(usage)
    };
    let rom = std::fs::read(rom).unwrap_or_else(|error| fail(format!("{rom}: {error}")));
    let pads = crate::inputs::read(Path::new(inputs)).unwrap_or_else(|error| fail(error));
    let last = u64::try_from(pads.len().saturating_sub(1)).unwrap_or(u64::MAX);
    let steps = |text: Option<&String>| {
        let (from, to) = text
            .and_then(|text| text.split_once(".."))
            .unwrap_or_else(|| fail(usage));
        let (Ok(from), Ok(to)) = (from.parse::<u64>(), to.parse::<u64>()) else {
            fail(usage)
        };
        from..=to.min(last)
    };
    let mut range = 0..=last;
    let mut first_ranges = false;
    let mut ranges_at: Option<std::ops::RangeInclusive<u64>> = None;
    let mut options = rest.iter();
    while let Some(option) = options.next() {
        match option.as_str() {
            "--frames" => range = steps(options.next()),
            "--ranges" => first_ranges = true,
            "--ranges-at" => ranges_at = Some(steps(options.next())),
            _ => fail(usage),
        }
    }
    let start = Start::PowerOn { wram_fill: 0x55 };
    let mut left =
        crate::Backend::open("lakesnes", &rom, start.clone()).unwrap_or_else(|error| fail(error));
    left.set_tiers(Tiers {
        picture: false,
        apu_log: false,
        ..Tiers::default()
    });
    let mut right = crate::Backend::open("lsnes", &rom, start).unwrap_or_else(|error| fail(error));
    println!(
        "cross: left {} {}, right {} {}, steps {}..={}",
        left.identity().emulator,
        left.identity().version,
        right.identity().emulator,
        right.identity().version,
        range.start(),
        range.end()
    );
    let mut compare = Compare::default();
    let mut first: std::collections::BTreeMap<Part, (u64, Difference)> =
        std::collections::BTreeMap::new();
    let mut frames_differing: std::collections::BTreeMap<Part, u64> =
        std::collections::BTreeMap::new();
    let mut equal = 0u64;
    let started = Instant::now();
    for (step, &pad) in (0..=*range.end()).zip(&pads) {
        let a = left.step(pad).unwrap_or_else(|error| fail(error)).clone();
        let b = right.step(pad).unwrap_or_else(|error| fail(error));
        let differences = compare.frame(&a, b);
        if !range.contains(&step) {
            continue;
        }
        if differences.is_empty() {
            equal += 1;
            println!("{step} equal");
            continue;
        }
        let parts: Vec<String> = differences.iter().map(describe).collect();
        println!("{step} {}", parts.join("; "));
        for difference in differences {
            *frames_differing.entry(difference.part).or_default() += 1;
            let is_first = !first.contains_key(&difference.part);
            if ((first_ranges && is_first)
                || ranges_at
                    .as_ref()
                    .is_some_and(|window| window.contains(&step)))
                && let (Some(left_bytes), Some(right_bytes)) =
                    (difference.part.bytes(&a), difference.part.bytes(b))
            {
                let ranges = crate::cross::ranges(&left_bytes, &right_bytes);
                println!(
                    "  {} at {step}: {} ranges",
                    difference.part.name(),
                    ranges.len()
                );
                for &(start, end, count) in ranges.iter().take(40) {
                    let shown = start..(end + 1).min(start + 16);
                    println!(
                        "    {start:05x}-{end:05x} {count} bytes: left {:02x?} right {:02x?}",
                        left_bytes.get(shown.clone()).unwrap_or_default(),
                        right_bytes.get(shown).unwrap_or_default()
                    );
                }
            }
            first.entry(difference.part).or_insert((step, difference));
        }
    }
    let compared = range.end().saturating_sub(*range.start()) + 1;
    println!(
        "summary: {equal} of {compared} steps equal, in {:.1} s; video frame offset (right less left) {}",
        started.elapsed().as_secs_f64(),
        compare.frame_offset().unwrap_or_default()
    );
    for part in Part::ALL {
        match first.get(&part) {
            Some((step, difference)) => println!(
                "  {:<12} first differs at {step}, on {} steps: {}",
                part.name(),
                frames_differing.get(&part).copied().unwrap_or_default(),
                describe(difference)
            ),
            None => println!("  {:<12} equal throughout", part.name()),
        }
    }
}
