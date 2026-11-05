//! The oracle harness's command line (`oracle`).
//!
//! `oracle movie ROM INPUTS [--frames N] [--seeds CSV DIR] [--no-picture]`
//!   plays a per-video-frame movie from power-on and, at each room seed of an
//!   lsnes capture, reports how far the emulator's WRAM is from the capture.
//! `oracle row ROM LOCK ID INPUTS [--seed DIR INDEX]` runs one lock row
//!   and compares its streams with the lock.

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
        fail("movie ROM INPUTS [--frames N] [--seeds CSV DIR] [--no-picture]")
    };
    let rom = std::fs::read(rom).unwrap_or_else(|error| fail(format!("{rom}: {error}")));
    let pads = crate::inputs::read(Path::new(inputs)).unwrap_or_else(|error| fail(error));
    let mut frames = pads.len();
    let mut seeds: Option<(Vec<RoomSeed>, PathBuf)> = None;
    let mut picture = true;
    let mut ranges = false;
    let mut lag_log: Option<Vec<[i64; 7]>> = None;
    let mut lag_mismatches = 0;
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
        ..Tiers::default()
    });
    let mut history: std::collections::VecDeque<Box<[u8]>> = std::collections::VecDeque::new();
    let mut next_seed = 0;
    let (mut lag, mut overrun, mut no_nmi, mut synced, mut checked) = (0u64, 0u64, 0u64, 0, 0);
    let started = Instant::now();
    for (frame, &pad) in pads.iter().enumerate().take(frames) {
        let record: &FrameRecord = oracle.step(pad);
        if std::env::var("OH_TRACE")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .is_some_and(|at| frame + 8 >= at && frame <= at + 2)
        {
            let writes = record
                .apu_log
                .iter()
                .filter(|a| a.kind == crate::ApuAccessKind::CpuWrite)
                .count();
            println!(
                "trace {frame}: wait {:?} cut {:?} apu reads {} writes {writes} joy {} pc {:02x}:{:04x}",
                record.wait_entry,
                record.cut,
                record.apu_log.len() - writes,
                record.joypad_reads,
                record.cpu.k,
                record.cpu.pc
            );
        }
        match record.nmi() {
            Some(nmi) => {
                lag += u64::from(nmi.kind == NmiKind::Lag);
                overrun += u64::from(nmi.overrun);
            }
            None => no_nmi += 1,
        }
        if let Some(log) = &lag_log
            && let Some(row) = log.get(frame)
        {
            let w = |address| i64::from(word(&record.wram, address));
            let lagged = i64::from(record.nmi().is_none_or(|nmi| nmi.kind == NmiKind::Lag));
            let mine = [
                w(0x05b6),
                w(0x05b8),
                lagged,
                w(0x0998),
                w(0x079b),
                w(0x0af6),
                w(0x0afa),
            ];
            let skip_counter = std::env::var_os("OH_SKIP_COUNTER").is_some();
            if (skip_counter && mine[1..] != row[1..]) || (!skip_counter && mine != *row) {
                lag_mismatches += 1;
                if lag_mismatches <= 40 {
                    println!(
                        "frame {frame}: lsnes {row:?} emulator {mine:?} wait entry {:?}",
                        record.wait_entry
                    );
                }
            }
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
        "frames {frames} in {seconds:.2}s: {:.0} frames/s; lag NMIs {lag}, overrun NMIs {overrun}, frames without NMI {no_nmi}",
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
