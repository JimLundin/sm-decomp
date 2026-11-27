//! The oracle harness's command line (`oracle`).
//!
//! `oracle movie ROM INPUTS [...]` plays a per-video-frame movie from power-on;
//!   `--lag-log` compares it frame by frame with an lsnes lag log and
//!   `--seeds` with lsnes room captures.
//! `oracle corpus ROM LOCK BUNDLE [...]` runs the lock's rows from their pinned pads and native seeds.
//! `oracle ladder-check ROM INPUTS DIR` checks that each rung of a movie's
//!   ladder (`movie --ladder N DIR`), resumed, reproduces the next.

mod csv;
mod movie_report;
mod requests;
#[cfg(test)]
mod tests;

use crate::lsnes::Lsnes;
use crate::{FrameRecord, NmiKind, Start, Streams};
use std::path::{Path, PathBuf};
use std::time::Instant;

fn fail(message: impl core::fmt::Display) -> ! {
    eprintln!("oracle: {message}");
    std::process::exit(2)
}

/// One row of an lsnes room-seed CSV.
#[derive(Debug, PartialEq, Eq)]
struct RoomSeed {
    index: u32,
    room: u16,
    movie_frame: u64,
}

fn room_seeds(path: &Path) -> Vec<RoomSeed> {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|error| fail(format!("{}: {error}", path.display())));
    csv::room_seeds(&text).unwrap_or_else(|error| fail(format!("{}: {error}", path.display())))
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

/// A save asked for before one step and taken at its boundary, written by
/// lsnes once the next step has run: the native seeds and the ladder rung it
/// serves.
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
/// frame; one saved state then serves them all. The save is requested before
/// the step (`save_due`) and handled after it.
fn save_after_step(
    lsnes: &mut Lsnes,
    lists: &[(Vec<RoomSeed>, PathBuf)],
    mut cutting: Option<&mut Cutting>,
    (step, frames): (u64, u64),
    (requested, pending): (Option<PathBuf>, &mut Option<PendingSave>),
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
    if let Some(state) = requested {
        *pending = Some(PendingSave { state, seeds, rung });
    }
}

/// Whether a native seed or a rung follows step `step`: its save must be
/// requested before the step.
fn save_due(
    lists: &[(Vec<RoomSeed>, PathBuf)],
    cutting: Option<&Cutting>,
    (step, frames): (u64, u64),
) -> bool {
    lists
        .iter()
        .any(|(list, _)| list.iter().any(|seed| seed.movie_frame == step + 1))
        || cutting.is_some_and(|cutting| cutting.rung_after(step, frames))
}

fn movie(arguments: &[String]) -> Result<(), requests::RequestError> {
    let requests::MovieRequest {
        rom,
        inputs,
        frames,
        seeds,
        streams,
        ranges,
        lag_log,
        watch,
        trace,
        positions_only,
        native,
        memories,
        start,
        cutting,
        from_ladder,
    } = requests::MovieRequest::parse(arguments)?;
    let rom =
        std::fs::read(&rom).unwrap_or_else(|error| fail(format!("{}: {error}", rom.display())));
    let pads = crate::inputs::read(&inputs).unwrap_or_else(|error| fail(error));
    let frames = frames.unwrap_or(pads.len());
    let seeds = seeds.map(|(csv, directory)| (room_seeds(&csv), directory));
    let native: Vec<_> = native
        .into_iter()
        .map(|(csv, directory)| (room_seeds(&csv), directory))
        .collect();
    let lag_log = lag_log.map(|csv| read_lag_log(&csv));
    let mut lag_mismatches = 0;
    let (mut start, mut first) = match start {
        requests::MovieStart::PowerOn => (Start::PowerOn, 0),
        requests::MovieStart::Saved { state, frame } => (
            Start::State(
                std::fs::read(&state)
                    .unwrap_or_else(|error| fail(format!("{}: {error}", state.display()))),
            ),
            frame,
        ),
    };
    let mut cutting = cutting.map(|(every, directory)| {
        let ladder =
            crate::ladder::Ladder::open(&directory, &inputs).unwrap_or_else(|error| fail(error));
        Cutting { ladder, every }
    });
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
        let ladder =
            crate::ladder::Ladder::open(directory, &inputs).unwrap_or_else(|error| fail(error));
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
    // A trace peeks the door transition's function pointer at each watched PC.
    let mut oracle = match &trace {
        Some((_, _, pcs)) => Lsnes::open_measuring(&rom, start, pcs, 0x099c),
        None => Lsnes::open(&rom, start),
    }
    .unwrap_or_else(|error| fail(error));
    let mut pending_save: Option<PendingSave> = None;
    if let Some(streams) = streams {
        oracle
            .record_streams(streams)
            .unwrap_or_else(|error| fail(error));
    }
    let mut history: std::collections::VecDeque<Box<[u8]>> = std::collections::VecDeque::new();
    let mut next_seed = 0;
    let (mut lag, mut overrun, mut no_nmi, mut synced, mut checked) = (0u64, 0u64, 0u64, 0, 0);
    let started = Instant::now();
    let run = u64::try_from(frames.min(pads.len())).unwrap_or(u64::MAX);
    for (frame, &pad) in pads.iter().enumerate().take(frames).skip(first) {
        let step = u64::try_from(frame).unwrap_or(u64::MAX);
        let mut requested = None;
        // The record of a step a native seed or a rung follows always
        // carries the memories: a rung's is checked against the run from
        // the rung before, a seed's against its lsnes room capture.
        let save = save_due(&native, cutting.as_ref(), (step, run));
        oracle.memories = memories || save;
        if save {
            requested = Some(oracle.request_save().unwrap_or_else(|error| fail(error)));
        }
        if let Some(&next) = pads.get(frame + 1) {
            oracle.preview(next);
        }
        let record: &FrameRecord = oracle.step(pad).unwrap_or_else(|error| fail(error));
        let record = record.clone();
        let record = &record;
        if !native.is_empty() || cutting.is_some() {
            save_after_step(
                &mut oracle,
                &native,
                cutting.as_mut(),
                (step, run),
                (requested, &mut pending_save),
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
            && let Some(line) = movie_report::lag_difference(frame, record, row, positions_only)
        {
            lag_mismatches += 1;
            if lag_mismatches <= 40 {
                println!("{line}");
            }
        }
        if let Some((from, to, _)) = &trace
            && (*from..=*to).contains(&frame)
        {
            println!(
                "trace {frame}: state {:04x} door {:04x} wait {:?} cut {:?} hdmaen {:02x} dmap {:02x?}",
                word(&record.wram, 0x0998),
                word(&record.wram, 0x099c),
                record.wait_entry,
                record.cut,
                record.ppu.hdmaen,
                record
                    .ppu
                    .dma
                    .iter()
                    .map(|channel| channel[0])
                    .collect::<Vec<u8>>()
            );
            for hit in &record.hits {
                println!(
                    "  hit {:08x} {} {} {:02x?}",
                    hit.pc, hit.beam.v, hit.beam.h, hit.peek
                );
            }
        }
        if let Some((from, every)) = watch
            && frame >= from
            && frame.is_multiple_of(every)
        {
            let w = |address| word(&record.wram, address);
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
            let Ok(capture) = crate::seed::capture_wram(directory, seed.index) else {
                continue;
            };
            checked += 1;
            let (matched, report) =
                movie_report::seed_comparison(seed, frame, &history, &capture, ranges);
            synced += usize::from(matched);
            print!("{report}");
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
    Ok(())
}

/// Derive an lsnes native seed: the state of `BASE` with the words written
/// right after it loads (`NativeSeed`), saved as `OUT` and `OUT`'s `.words`.
/// Then prove that every word stuck at the load, which is what a run from the
/// seed relies on (`NativeSeed::apply`). The server reads memory only at a
/// step's cut, so the proof is one step from `OUT` as read back beside one
/// step from the plain base: a word kept its write when it reads the value
/// written, or when it reads other than the plain run (the write landed and
/// the game moved it on, as physics moves Samus's position). It was lost when
/// it reads the plain run's value and not the written one. The work-RAM bytes
/// differing from the plain run are reported. `tools/scenario_seeds.py
/// native-args` gives a scenario's arguments.
fn derive_seed(arguments: &[String]) {
    use crate::lsnes::NativeSeed;
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
        let (kept, report) = seed_write_report(*word, &plain, &derived);
        println!("{report}");
        if !kept {
            fail(format!("${:05X} did not keep its write", word.offset));
        }
    }
    println!(
        "derived {}: every word kept at load; {} work-RAM bytes differ from the base after one step",
        out.display(),
        differing(&derived, &plain)
    );
}

fn seed_write_report(word: crate::lsnes::WramWord, plain: &[u8], derived: &[u8]) -> (bool, String) {
    let (before, after) = (word.read(plain), word.read(derived));
    let kept = after == word.value || after != before;
    let status = if kept { "kept at load" } else { "LOST" };
    (kept, format!("${:05X}: {status}", word.offset))
}

/// Run the command line `arguments` (without the program name).
pub fn main(arguments: &[String]) -> i32 {
    match arguments.split_first() {
        Some((command, rest)) if command == "movie" => {
            movie(rest).unwrap_or_else(|error| fail(error))
        }
        Some((command, rest)) if command == "corpus" => {
            corpus(rest).unwrap_or_else(|error| fail(error))
        }
        Some((command, rest)) if command == "trace" => trace(rest),
        Some((command, rest)) if command == "measure" => measure(rest),
        Some((command, rest)) if command == "derive-seed" => derive_seed(rest),
        Some((command, rest)) if command == "ladder-check" => return ladder_check(rest),
        Some((command, rest)) if command == "ladder-rung" => ladder_rung(rest),
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
    let sha = crate::ladder::hex(&data::rom::sha256(&stored));
    if ladder.record_sha256(to) != Some(sha.as_str()) {
        return Err(format!(
            "rung {to}'s record is not the one its manifest lists"
        ));
    }
    let state = read(ladder.state_path(from))?;
    let mut lsnes = Lsnes::open(rom, Start::State(state)).map_err(|error| error.to_string())?;
    let steps = usize::try_from(from)
        .ok()
        .zip(usize::try_from(to).ok())
        .and_then(|(from, to)| pads.get(from..to))
        .ok_or_else(|| format!("rungs {from}..{to} lie past the movie's end"))?;
    let last = steps.len().saturating_sub(1);
    let after = usize::try_from(to).ok().and_then(|to| pads.get(to));
    for (step, &pad) in steps.iter().enumerate() {
        lsnes.memories = step == last;
        if let Some(&next) = steps.get(step + 1).or(after) {
            lsnes.preview(next);
        }
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

/// An lsnes lag log (`tools/lsnes_lag_log.lua`): per movie frame, the frame
/// counter $05B6, the NMI counter $05B8, the lag flag, the game state $0998,
/// the room $079B and Samus's X and Y ($0AF6, $0AFA) after the frame.
fn read_lag_log(path: &Path) -> Vec<[i64; 7]> {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|error| fail(format!("{}: {error}", path.display())));
    csv::lag_log(&text).unwrap_or_else(|error| fail(format!("{}: {error}", path.display())))
}

fn read_corpus_lock(path: &Path) -> crate::lock::Lock {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|error| fail(format!("{}: {error}", path.display())));
    let lock = crate::lock::Lock::parse(&text)
        .unwrap_or_else(|error| fail(format!("{}: {error}", path.display())));
    let oracle = crate::lsnes::IDENTITY;
    let header = &lock.header;
    if header.oracle.emulator != oracle.emulator
        || header.oracle.version != oracle.version
        || header.oracle.fixes != oracle.fixes
        || header.oracle.server != crate::lsnes::server_sha256()
    {
        fail(format!("{}: incompatible oracle", path.display()));
    }
    let name = |value: &str| {
        !value.is_empty()
            && value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    };
    for row in &lock.rows {
        if !name(&row.id.0)
            || matches!(&row.start, crate::lock::Start::Seed { capture, .. } if !name(capture))
        {
            fail(format!("{}: invalid row or seed name", path.display()));
        }
    }
    lock
}

fn corpus(arguments: &[String]) -> Result<(), requests::RequestError> {
    let requests::CorpusRequest {
        rom,
        lock,
        bundle,
        only,
        limit,
        shard,
        watched,
        watched_pcs,
        lean,
        cache,
    } = requests::CorpusRequest::parse(arguments)?;
    let rom =
        std::fs::read(&rom).unwrap_or_else(|error| fail(format!("{}: {error}", rom.display())));
    let lock = read_corpus_lock(&lock);
    for id in &only {
        if !lock.rows.iter().any(|row| row.id.0 == *id) {
            fail(format!("{id}: no such row in the lock"));
        }
    }
    let lsnes = cache.as_ref().map(|_| {
        crate::lsnes::executable_sha256()
            .unwrap_or_else(|error| fail(format!("the lsnes executable: {error}")))
    });
    let observing = crate::corpus::Observe {
        streams: cache.is_some().then_some(Streams::PpuAndApuLog),
        watch: &watched_pcs,
        lean,
        ..Default::default()
    };
    let (mut rows, mut failed) = (0u32, 0u32);
    let (mut ticks, mut frames, mut seconds) = (0u64, 0u64, 0.0f64);
    let entries = lock
        .rows
        .iter()
        .filter(|row| only.is_empty() || only.contains(&row.id.0))
        .enumerate()
        .filter(|(position, _)| shard.includes(*position))
        .map(|(_, row)| row)
        .take(limit);
    for row in entries {
        let id = &row.id.0;
        let sources = match crate::corpus::Sources::read(&bundle, row) {
            Ok(sources) => sources,
            Err(error) => {
                failed += 1;
                println!("{id}: {error}");
                continue;
            }
        };
        let mut last: Vec<Option<u16>> = vec![None; watched.len()];
        let mut frame = 0u64;
        let mut cached = cache.as_deref().map(|root| {
            crate::cache::Writer::create(root, id)
                .map(|writer| writer.watching(&watched_pcs))
                .unwrap_or_else(|error| fail(format!("{}: {error}", root.display())))
        });
        let mut watch = |record: &FrameRecord| {
            if let Some(writer) = cached.as_mut() {
                writer
                    .push(record)
                    .unwrap_or_else(|error| fail(format!("{id}: cache: {error}")));
            }
            for hit in &record.hits {
                println!(
                    "  {id} frame {frame}: pc ${:06x} at {},{}",
                    hit.pc, hit.beam.v, hit.beam.h
                );
            }
            for (address, last) in watched.iter().zip(last.iter_mut()) {
                let value = word(&record.wram, *address);
                if *last != Some(value) {
                    println!("  {id} frame {frame}: ${address:04x}={value:#06x}");
                    *last = Some(value);
                }
            }
            frame += 1;
        };
        let run = match crate::corpus::run_row_with(&rom, sources, &observing, &mut watch) {
            Ok(run) => run,
            Err(error) => {
                failed += 1;
                println!("{id}: {error}");
                continue;
            }
        };
        if let (Some(writer), Some(lsnes)) = (cached, lsnes) {
            let bytes = writer
                .finish(id, &crate::cache::Pins::of(row, lsnes))
                .unwrap_or_else(|error| fail(format!("{id}: cache: {error}")));
            println!("  {id} cache: {} frames, {bytes} bytes", run.frames);
        }
        rows += 1;
        ticks += u64::from(run.ticks);
        frames += run.frames;
        seconds += run.seconds;
        println!(
            "{id}: {} ticks, {} frames, {} overrun, {} without NMI, {} cut with the NMI running, {:.0} frames/s",
            run.ticks,
            run.frames,
            run.overrun,
            run.without_nmi,
            run.still_running,
            run.frames as f64 / run.seconds
        );
    }
    println!("rows run {rows}, failed or not run {failed}");
    if rows > 0 {
        println!(
            "{ticks} ticks, {frames} frames in {seconds:.1}s: {:.0} frames/s",
            frames as f64 / seconds
        );
    }
    if failed != 0 {
        return Err(requests::RequestError::Invalid(format!(
            "{failed} rows failed"
        )));
    }
    Ok(())
}

/// `trace ROM LOCK BUNDLE ID FROM TO`: one row's frames FROM-TO in detail.
fn trace(arguments: &[String]) {
    let [rom, lock, bundle, id, from, to] = arguments else {
        fail("trace ROM LOCK BUNDLE ID FROM TO")
    };
    let rom = std::fs::read(rom).unwrap_or_else(|error| fail(format!("{rom}: {error}")));
    let lock = read_corpus_lock(Path::new(lock));
    let row = lock
        .row(&crate::lock::RowId(id.clone()))
        .unwrap_or_else(|| fail(format!("{id}: no such row in the lock")));
    let sources =
        crate::corpus::Sources::read(Path::new(bundle), row).unwrap_or_else(|error| fail(error));
    let range = from.parse().unwrap_or(0)..=to.parse().unwrap_or(0);
    let mut frame = 0usize;
    let result = crate::corpus::run_row_with(
        &rom,
        sources,
        &crate::corpus::Observe {
            streams: Some(Streams::PpuAndApuLog),
            ..Default::default()
        },
        &mut |record| {
            if range.contains(&frame) {
                let w = |address| word(&record.wram, address);
                let apu: Vec<String> = record
                    .apu_log
                    .iter()
                    .take(12)
                    .map(|access| format!("{:?}{}={:02x}", access.kind, access.port, access.value))
                    .collect();
                println!(
                    "frame {frame}: cut {:?} cpu {:02x}:{:04x} state {:04x} room {:04x} apu[{}] {}",
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
        fail(format!("{id}: {error}"));
    }
}

/// `measure ROM LOCK BUNDLE ID...`: the architecture lane's measurements over
/// lock rows (from their lsnes seeds), as Markdown.
fn measure(arguments: &[String]) {
    let [rom_path, lock, bundle, ids @ ..] = arguments else {
        fail("measure ROM LOCK BUNDLE ID...")
    };
    let rom = std::fs::read(rom_path).unwrap_or_else(|error| fail(format!("{rom_path}: {error}")));
    let lock = read_corpus_lock(Path::new(lock));
    for id in ids {
        let row = lock
            .row(&crate::lock::RowId(id.clone()))
            .unwrap_or_else(|| fail(format!("{id}: no such row in the lock")));
        let sources = crate::corpus::Sources::read(Path::new(bundle), row)
            .unwrap_or_else(|error| fail(error));
        let mut measurements = crate::measure::Measurements::default();
        let result = crate::corpus::run_row_with(
            &rom,
            sources,
            &crate::corpus::Observe {
                streams: Some(Streams::PpuAndApuLog),
                watch: &crate::measure::WATCHED,
                peek: Some(crate::measure::PEEK),
                ..crate::corpus::Observe::default()
            },
            &mut |record| measurements.frame(record),
        );
        println!("### {id} (from its lsnes seed)");
        if let Err(error) = result {
            fail(format!("{id}: {error}"));
        }
        print!("{}", measurements.report(&rom));
    }
}
