//! Public diagnostics for movie comparisons.

use super::{RoomSeed, differing, word};
use crate::FrameRecord;
use std::{collections::VecDeque, fmt::Write};

pub(super) fn lag_difference(
    frame: usize,
    record: &FrameRecord,
    row: &[i64; 7],
    positions_only: bool,
) -> Option<String> {
    let w = |address| i64::from(word(&record.wram, address));
    let compared = if positions_only { 3 } else { 0 };
    let lagged = i64::from(record.nmi().is_none_or(|nmi| !nmi.polled));
    let fields = [
        ("frame counter", w(0x05b6)),
        ("NMI counter", w(0x05b8)),
        ("lag flag", lagged),
        ("game state", w(0x0998)),
        ("room", w(0x079b)),
        ("Samus X", w(0x0af6)),
        ("Samus Y", w(0x0afa)),
    ];
    let differences: Vec<_> = fields
        .into_iter()
        .zip(row)
        .skip(compared)
        .filter_map(|((name, actual), &expected)| (actual != expected).then_some(name))
        .collect();
    if differences.is_empty() {
        return None;
    }
    Some(format!(
        "frame {frame}: {} fields differ [{}]",
        differences.len(),
        differences.join(", ")
    ))
}

pub(super) fn seed_comparison(
    seed: &RoomSeed,
    frame: u64,
    history: &VecDeque<Box<[u8]>>,
    capture: &[u8],
    show_ranges: bool,
) -> (bool, String) {
    let counts: Vec<usize> = history
        .iter()
        .map(|wram| differing(wram, capture))
        .collect();
    let (best, offset) = counts
        .iter()
        .copied()
        .enumerate()
        .map(|(age, count)| (count, age))
        .min()
        .unwrap_or((usize::MAX, 0));
    let newest = history.len() - 1;
    let at = (frame - 1) as i64 - (newest - offset) as i64 - seed.movie_frame as i64;
    let wram = &history[offset];
    let matched = word(wram, 0x079b) == seed.room;
    let mut report = String::new();
    if show_ranges {
        writeln!(report, "  differing per frame, oldest first: {counts:?}").unwrap();
        let ranges = ranges(wram, capture);
        for &(start, end, count) in ranges.iter().take(60) {
            writeln!(report, "  {start:05X}-{end:05X} {count} bytes").unwrap();
        }
        writeln!(report, "  {} ranges", ranges.len()).unwrap();
    }
    writeln!(
        report,
        "seed {:4} frame {:6}: room {}; WRAM bytes differing {best} at frame offset {at:+}",
        seed.index,
        seed.movie_frame,
        if matched { "match" } else { "DIFFERS" }
    )
    .unwrap();
    (matched, report)
}

/// The differing byte ranges of two images, as (first, last, differing
/// bytes), with gaps under 16 bytes coalesced.
fn ranges(left: &[u8], right: &[u8]) -> Vec<(usize, usize, usize)> {
    let mut ranges: Vec<(usize, usize, usize)> = Vec::new();
    for (address, _) in left
        .iter()
        .zip(right)
        .enumerate()
        .filter(|(_, (a, b))| a != b)
    {
        match ranges.last_mut() {
            Some((_, end, count)) if address - *end < 16 => {
                *end = address;
                *count += 1;
            }
            _ => ranges.push((address, address, 1)),
        }
    }
    ranges
}
