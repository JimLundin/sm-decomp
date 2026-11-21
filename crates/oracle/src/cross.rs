//! The cross-oracle report: two backends stepped with the same pads, their
//! records compared part by part each frame (ARCHITECTURE D22, D25).
//!
//! Only the parts both backends produce are compared: the cut, the frame
//! counter, the pad, the joypad reads, the CPU registers and the five
//! memories. A memory that differs is also compared with the other backend's
//! record one frame earlier and one frame later, which tells a timing
//! difference (one oracle a frame behind) from a content difference.

use std::{collections::BTreeMap, fmt::Write, ops::RangeInclusive, time::Duration};

use crate::record::{ApuAccessKind, CpuRegisters, Cut, FrameRecord, Picture, PpuRegisters};

/// A part of the record both backends produce.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Part {
    /// The cut's kind, interrupted PC and whether the handler polled.
    Cut,
    /// The emulator's video frame counter, against its offset at the first
    /// step (lsnes counts from 1, LakeSnes from 0).
    Frame,
    Pad,
    /// Joypad register reads in the interval.
    JoypadReads,
    Cpu,
    Wram,
    Vram,
    Cgram,
    Oam,
    Sram,
    /// The PPU registers at the cut, without the CPU I/O words lsnes lacks.
    Ppu,
    /// The CPU's APU port accesses in the interval, in order.
    ApuLog,
    /// The picture as it shows: 8-bit R, G, B per dot (`Picture::shown`).
    Picture,
}

impl Part {
    pub const ALL: [Self; 13] = [
        Self::Cut,
        Self::Frame,
        Self::Pad,
        Self::JoypadReads,
        Self::Cpu,
        Self::Wram,
        Self::Vram,
        Self::Cgram,
        Self::Oam,
        Self::Sram,
        Self::Ppu,
        Self::ApuLog,
        Self::Picture,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Cut => "cut",
            Self::Frame => "frame",
            Self::Pad => "pad",
            Self::JoypadReads => "joypad-reads",
            Self::Cpu => "cpu",
            Self::Wram => "wram",
            Self::Vram => "vram",
            Self::Cgram => "cgram",
            Self::Oam => "oam",
            Self::Sram => "sram",
            Self::Ppu => "ppu",
            Self::ApuLog => "apu-log",
            Self::Picture => "picture",
        }
    }

    /// The part's bytes, for the memories.
    pub fn bytes(self, record: &FrameRecord) -> Option<Vec<u8>> {
        let words = |words: &[u16]| words.iter().flat_map(|word| word.to_le_bytes()).collect();
        match self {
            Self::Wram => Some(record.wram.to_vec()),
            Self::Vram => Some(words(&record.vram)),
            Self::Cgram => Some(words(&record.cgram)),
            Self::Oam => Some(record.oam.to_vec()),
            Self::Sram => Some(record.sram.to_vec()),
            Self::Ppu => Some(ppu_bytes(&record.ppu)),
            Self::ApuLog => Some(
                record
                    .apu_log
                    .iter()
                    .filter_map(|access| match access.kind {
                        ApuAccessKind::CpuWrite => Some([0, access.port, access.value]),
                        ApuAccessKind::CpuRead => Some([1, access.port, access.value]),
                        ApuAccessKind::DmaWrite
                        | ApuAccessKind::DmaRead
                        | ApuAccessKind::SpcWrite => None,
                    })
                    .flatten()
                    .collect(),
            ),
            Self::Picture => record
                .picture
                .as_ref()
                .map(|picture| picture.shown().into_iter().flatten().collect()),
            Self::Cut | Self::Frame | Self::Pad | Self::JoypadReads | Self::Cpu => None,
        }
    }
}

/// How the other backend's neighbouring frame compares, for a memory that differs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shift {
    /// Left at this frame is closer to right one frame earlier, by more than
    /// half the differing bytes: left is a frame behind.
    LeftBehind,
    /// Right at this frame is that much closer to left one frame earlier.
    RightBehind,
    /// Neither neighbour is that much closer.
    Content,
}

/// One part that differs at a frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Difference {
    pub part: Part,
    /// Differing bytes for a memory; differing fields for the others.
    pub count: usize,
    /// Names of the differing fields within a part.
    pub fields: Vec<&'static str>,
    /// For a memory: differing bytes against each neighbour, and the verdict.
    pub shifted: Option<(usize, usize, Shift)>,
}

fn differing_bytes(left: &[u8], right: &[u8]) -> usize {
    left.iter().zip(right).filter(|(a, b)| a != b).count() + left.len().abs_diff(right.len())
}

fn cut_fields(left: Cut, right: Cut) -> Vec<&'static str> {
    match (left, right) {
        (Cut::NmiReturn(left), Cut::NmiReturn(right)) => [
            ("kind", left.kind != right.kind),
            (
                "interrupted_pc",
                left.interrupted_pc != right.interrupted_pc,
            ),
            ("polled", left.polled != right.polled),
        ]
        .into_iter()
        .filter_map(|(name, differs)| differs.then_some(name))
        .collect(),
        (Cut::NmiStillRunning { .. }, Cut::NmiStillRunning { .. }) => vec!["interrupted_pc"],
        _ => vec!["kind"],
    }
}

/// The names of the CPU registers that differ.
fn cpu_fields(left: &CpuRegisters, right: &CpuRegisters) -> Vec<&'static str> {
    let fields: [(&str, u32, u32); 10] = [
        ("a", left.a.into(), right.a.into()),
        ("x", left.x.into(), right.x.into()),
        ("y", left.y.into(), right.y.into()),
        ("s", left.s.into(), right.s.into()),
        ("pc", left.pc.into(), right.pc.into()),
        ("d", left.d.into(), right.d.into()),
        ("k", left.k.into(), right.k.into()),
        ("db", left.db.into(), right.db.into()),
        ("p", left.p.into(), right.p.into()),
        ("e", left.emulation.into(), right.emulation.into()),
    ];
    fields
        .iter()
        .filter(|(_, a, b)| a != b)
        .map(|&(name, _, _)| name)
        .collect()
}

/// The records of the frame before, kept to test a one-frame shift.
#[derive(Default)]
pub struct Compare {
    previous: Option<(FrameRecord, FrameRecord)>,
    /// Right's video frame counter less left's, at the first step.
    frame_offset: Option<i64>,
    /// Compare the streams too: the PPU registers, the APU log and the picture.
    pub streams: bool,
}

impl Compare {
    /// Right's video frame counter less left's, once a step is compared.
    pub fn frame_offset(&self) -> Option<i64> {
        self.frame_offset
    }

    /// The parts in which `left` and `right` differ, in `Part::ALL` order.
    pub fn frame(&mut self, left: &FrameRecord, right: &FrameRecord) -> Vec<Difference> {
        let mut differences = Vec::new();
        let scalar = |part| Difference {
            part,
            count: 1,
            fields: Vec::new(),
            shifted: None,
        };
        if left.cut != right.cut {
            differences.push(Difference {
                fields: cut_fields(left.cut, right.cut),
                ..scalar(Part::Cut)
            });
        }
        let offset = i64::from(right.video_frame) - i64::from(left.video_frame);
        if *self.frame_offset.get_or_insert(offset) != offset {
            differences.push(scalar(Part::Frame));
        }
        if left.pad != right.pad {
            differences.push(scalar(Part::Pad));
        }
        if left.joypad_reads != right.joypad_reads {
            differences.push(scalar(Part::JoypadReads));
        }
        let registers = cpu_fields(&left.cpu, &right.cpu);
        if !registers.is_empty() {
            differences.push(Difference {
                part: Part::Cpu,
                count: registers.len(),
                fields: registers,
                shifted: None,
            });
        }
        for part in Part::ALL {
            if !self.streams && matches!(part, Part::Ppu | Part::ApuLog | Part::Picture) {
                continue;
            }
            let (Some(a), Some(b)) = (part.bytes(left), part.bytes(right)) else {
                continue;
            };
            let count = differing_bytes(&a, &b);
            if count == 0 {
                continue;
            }
            let shifted = self.previous.as_ref().map(|(left_before, right_before)| {
                let against = |record: &FrameRecord, bytes: &[u8]| {
                    part.bytes(record)
                        .map_or(usize::MAX, |other| differing_bytes(bytes, &other))
                };
                let left_behind = against(right_before, &a);
                let right_behind = against(left_before, &b);
                // A frame's work moves more bytes than the scratch and stack
                // bytes that stay different, so a lag shows as a shifted
                // count well under the unshifted one, not as zero.
                let closer = |shifted: usize| shifted.saturating_mul(2) < count;
                let verdict = if closer(left_behind) && left_behind <= right_behind {
                    Shift::LeftBehind
                } else if closer(right_behind) {
                    Shift::RightBehind
                } else {
                    Shift::Content
                };
                (left_behind, right_behind, verdict)
            });
            differences.push(Difference {
                part,
                count,
                fields: Vec::new(),
                shifted,
            });
        }
        self.previous = Some((left.clone(), right.clone()));
        differences
    }
}

/// The differing byte ranges of two images, as (first, last, differing
/// bytes), with gaps under 16 bytes coalesced.
pub fn ranges(left: &[u8], right: &[u8]) -> Vec<(usize, usize, usize)> {
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

/// The text of one difference: `wram 12 (vs right-1 0, vs left-1 30: left behind)`.
pub fn describe(difference: &Difference) -> String {
    let mut text = format!("{} {}", difference.part.name(), difference.count);
    if !difference.fields.is_empty() {
        text.push_str(&format!(" [{}]", difference.fields.join(" ")));
    }
    if let Some((left_behind, right_behind, verdict)) = difference.shifted {
        let verdict = match verdict {
            Shift::LeftBehind => "left a frame behind",
            Shift::RightBehind => "right a frame behind",
            Shift::Content => "content",
        };
        text.push_str(&format!(
            " (vs right-1 {left_behind}, vs left-1 {right_behind}: {verdict})"
        ));
    }
    text
}

pub(crate) fn describe_streams(
    step: u64,
    left: &FrameRecord,
    right: &FrameRecord,
    left_before: Option<&Picture>,
) -> Option<String> {
    let (lake, lsnes) = (left.picture.as_ref()?, right.picture.as_ref()?);
    // The displayed dots and their BGR555 source can differ below full brightness.
    let shown = lsnes.shown();
    let equal_dots = |picture: &Picture| {
        picture
            .shown()
            .iter()
            .zip(&shown)
            .filter(|(x, y)| x == y)
            .count()
    };
    let equal_words = lake
        .rows()
        .flatten()
        .zip(lsnes.rows().flatten())
        .filter(|(x, y)| x == y)
        .count();
    let cpu_accesses = left
        .apu_log
        .iter()
        .filter(|access| {
            matches!(
                access.kind,
                ApuAccessKind::CpuWrite | ApuAccessKind::CpuRead
            )
        })
        .count();
    Some(format!(
        "{step} picture {} of {} dots equal as shown ({equal_words} as BGR555), {} against left's previous; apu log {cpu_accesses}/{}",
        equal_dots(lake),
        lake.rows().flatten().count(),
        left_before.map_or(0, equal_dots),
        right.apu_log.len()
    ))
}

/// The text and accumulated counts for the public cross command.
pub(crate) struct Report {
    compare: Compare,
    first_ranges: bool,
    ranges_at: Option<RangeInclusive<u64>>,
    first: BTreeMap<Part, (u64, Difference)>,
    frames_differing: BTreeMap<Part, u64>,
    equal: u64,
}

impl Report {
    pub(crate) fn new(
        streams: bool,
        first_ranges: bool,
        ranges_at: Option<RangeInclusive<u64>>,
    ) -> Self {
        Self {
            compare: Compare {
                streams,
                ..Compare::default()
            },
            first_ranges,
            ranges_at,
            first: BTreeMap::new(),
            frames_differing: BTreeMap::new(),
            equal: 0,
        }
    }

    pub(crate) fn step(
        &mut self,
        step: u64,
        in_window: bool,
        left: &FrameRecord,
        right: &FrameRecord,
    ) -> Option<String> {
        let differences = self.compare.frame(left, right);
        if !in_window {
            return None;
        }
        if differences.is_empty() {
            self.equal += 1;
            return Some(format!("{step} equal\n"));
        }
        let parts: Vec<String> = differences.iter().map(describe).collect();
        let mut text = format!("{step} {}\n", parts.join("; "));
        for difference in differences {
            let part = difference.part;
            *self.frames_differing.entry(part).or_default() += 1;
            let is_first = !self.first.contains_key(&part);
            if ((self.first_ranges && is_first)
                || self
                    .ranges_at
                    .as_ref()
                    .is_some_and(|window| window.contains(&step)))
                && let (Some(left_bytes), Some(right_bytes)) = (part.bytes(left), part.bytes(right))
            {
                let ranges = ranges(&left_bytes, &right_bytes);
                writeln!(text, "  {} at {step}: {} ranges", part.name(), ranges.len()).unwrap();
                for (start, end, count) in ranges.into_iter().take(40) {
                    writeln!(text, "    {start:05x}-{end:05x} {count} bytes").unwrap();
                }
            }
            self.first.entry(part).or_insert((step, difference));
        }
        Some(text)
    }

    pub(crate) fn summary(&self, compared: u64, elapsed: Duration) -> String {
        let mut text = format!(
            "summary: {} of {compared} steps equal, in {:.1} s; video frame offset (right less left) {}\n",
            self.equal,
            elapsed.as_secs_f64(),
            self.compare.frame_offset().unwrap_or_default()
        );
        for part in Part::ALL {
            let name = part.name();
            match self.first.get(&part) {
                Some((step, difference)) => writeln!(
                    text,
                    "  {name:<12} first differs at {step}, on {} steps: {}",
                    self.frames_differing
                        .get(&part)
                        .copied()
                        .unwrap_or_default(),
                    describe(difference)
                )
                .unwrap(),
                None => writeln!(text, "  {name:<12} equal throughout").unwrap(),
            }
        }
        text
    }
}

/// The PPU registers both backends record: all but the CPU I/O words
/// (`nmitimen`, `htime`, `vtime`, `hdmaen`, `memsel`), which lsnes cannot read.
fn ppu_bytes(ppu: &PpuRegisters) -> Vec<u8> {
    let mut bytes = vec![ppu.inidisp, ppu.obsel, ppu.bgmode, ppu.mosaic];
    bytes.extend(ppu.bg_sc);
    bytes.extend([ppu.bg12nba, ppu.bg34nba]);
    bytes.extend(
        ppu.bg_scroll
            .iter()
            .flatten()
            .flat_map(|word| word.to_le_bytes()),
    );
    bytes.push(ppu.m7sel);
    bytes.extend(ppu.m7.iter().flat_map(|word| word.to_le_bytes()));
    bytes.extend(ppu.window_select);
    bytes.extend(ppu.window_positions);
    bytes.extend([ppu.wbglog, ppu.wobjlog]);
    bytes.extend(ppu.screens);
    bytes.extend([ppu.cgwsel, ppu.cgadsub]);
    bytes.extend(ppu.fixed_color);
    bytes.push(ppu.setini);
    bytes.extend(ppu.dma.iter().flatten());
    bytes
}
#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) fn record(fill: u8) -> FrameRecord {
        FrameRecord {
            index: 0,
            video_frame: 0,
            pad: crate::Pad(0),
            cut: Cut::VblankWithoutNmi,
            joypad_reads: 0,
            wait_entry: None,
            overscan: false,
            cpu: CpuRegisters::default(),
            wram: vec![fill; 16].into_boxed_slice(),
            vram: vec![0; 4].into_boxed_slice(),
            cgram: vec![0; 4].into_boxed_slice(),
            oam: vec![0; 4].into_boxed_slice(),
            sram: vec![0; 4].into_boxed_slice(),
            ppu: crate::PpuRegisters::default(),
            apu_log: Vec::new(),
            picture: None,
            spc: None,
            bank_instructions: None,
            writes: Vec::new(),
            hits: Vec::new(),
            legacy_ppu_digest: 0,
        }
    }

    #[test]
    fn equal_records_have_no_differences() {
        assert!(Compare::default().frame(&record(1), &record(1)).is_empty());
    }

    #[test]
    fn a_left_frame_behind_is_timing() {
        let mut compare = Compare::default();
        assert!(compare.frame(&record(1), &record(1)).is_empty());
        let differences = compare.frame(&record(1), &record(2));
        assert_eq!(differences.len(), 1);
        assert_eq!(differences[0].part, Part::Wram);
        assert_eq!(differences[0].count, 16);
        // Right moved on to 2 while left stayed at 1: right one frame earlier was 1.
        assert_eq!(differences[0].shifted, Some((0, 16, Shift::LeftBehind)));
    }

    #[test]
    fn a_shift_with_a_few_scratch_bytes_is_still_timing() {
        let mut compare = Compare::default();
        let mut stale = record(1);
        stale.wram[15] = 9;
        assert!(!compare.frame(&stale, &record(1)).is_empty());
        let mut behind = record(1);
        behind.wram[15] = 7;
        let differences = compare.frame(&record(2), &behind);
        // Right now holds what left held a frame earlier, but for one byte.
        assert_eq!(differences[0].shifted, Some((16, 1, Shift::RightBehind)));
    }

    #[test]
    fn a_constant_frame_offset_is_equal() {
        let mut compare = Compare::default();
        let mut right = record(1);
        right.video_frame = 1;
        assert!(compare.frame(&record(1), &right).is_empty());
        right.video_frame = 3;
        let differences = compare.frame(&record(1), &right);
        assert_eq!(differences[0].part, Part::Frame);
    }

    #[test]
    fn ranges_coalesce_short_gaps() {
        let left = [0u8; 64];
        let mut right = [0u8; 64];
        right[1] = 1;
        right[10] = 1;
        right[40] = 1;
        assert_eq!(ranges(&left, &right), vec![(1, 10, 2), (40, 40, 1)]);
    }

    #[test]
    fn a_cpu_difference_names_its_fields() {
        let mut right = record(1);
        right.cpu.pc = 0x8344;
        let differences = Compare::default().frame(&record(1), &right);
        assert_eq!(differences[0].part, Part::Cpu);
        assert_eq!(differences[0].fields, ["pc"]);
    }
}
