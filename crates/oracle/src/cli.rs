//! The oracle harness's command line (`oracle`).
//!
//! `oracle movie ROM INPUTS [...]` plays a per-video-frame movie from power-on;
//!   `--lag-log` compares it frame by frame with an lsnes lag log and
//!   `--seeds` with lsnes room captures.
//! `oracle corpus ROM LOCK DATA [...]` runs the lock's rows and compares them.

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

fn movie(arguments: &[String]) {
    let [rom, inputs, rest @ ..] = arguments else {
        fail(
            "movie ROM INPUTS [--frames N] [--seeds CSV DIR] [--ranges] [--lag-log CSV] [--watch FROM EVERY] [--spc] [--no-picture]",
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
            "--no-picture" => picture = false,
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
    let mut oracle = LakeSnes::open(&rom, Start::PowerOn { wram_fill: 0x55 })
        .unwrap_or_else(|error| fail(error));
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
    for (frame, &pad) in pads.iter().enumerate().take(frames) {
        let record: &FrameRecord = oracle.step(pad);
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
        frames as f64 / seconds
    );
    if seeds.is_some() {
        println!("room seeds in the same room: {synced} of {checked}");
    }
}

/// Run the command line `arguments` (without the program name).
pub fn main(arguments: &[String]) -> i32 {
    match arguments.split_first() {
        Some((command, rest)) if command == "movie" => movie(rest),
        Some((command, rest)) if command == "corpus" => corpus(rest),
        Some((command, rest)) if command == "trace" => trace(rest),
        Some((command, rest)) if command == "measure" => measure(rest),
        Some((command, rest)) if command == "measure-movie" => measure_movie(rest),
        _ => fail("usage: oracle movie ROM INPUTS [...]"),
    }
    0
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
        fail("corpus ROM LOCK DATA [--only ID]... [--shard I/N] [--limit N] [--record FILE]")
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
            "--record" => {
                let path = options.next().unwrap_or_else(|| fail("--record FILE"));
                record = Some(
                    std::fs::File::create(path)
                        .unwrap_or_else(|error| fail(format!("{path}: {error}"))),
                );
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
        match crate::corpus::run_row(&rom, pin, entry, data) {
            Err(error) => {
                failed += 1;
                println!("{}: not run: {error}", entry.id);
            }
            Ok(run) => {
                rows += 1;
                ticks += u64::from(run.ticks);
                frames += run.frames;
                seconds += run.seconds;
                if let Some(file) = record.as_mut() {
                    use std::io::Write as _;
                    let text = crate::corpus::lock_entry(entry, crate::lakesnes::IDENTITY, &run);
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
    println!("rows run {rows}, not run {failed}; all comparable streams equal on {all_six}");
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
        measurements.frame(oracle.step(pad));
    }
    println!("### {inputs}, frames 0-{} (from power-on)", frames - 1);
    print!("{}", measurements.report(&rom));
}
