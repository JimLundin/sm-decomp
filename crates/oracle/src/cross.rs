//! The cross-oracle report: two backends stepped with the same pads, their
//! records compared part by part each frame (ARCHITECTURE D22, D25).
//!
//! Only the parts both backends produce are compared: the cut, the frame
//! counter, the pad, the joypad reads, the CPU registers and the five
//! memories. A memory that differs is also compared with the other backend's
//! record one frame earlier and one frame later, which tells a timing
//! difference (one oracle a frame behind) from a content difference.

use crate::record::{ApuAccessKind, CpuRegisters, Cut, FrameRecord, PpuRegisters};

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
    /// What differs, for the parts that are not memories: both values.
    pub detail: String,
    /// For a memory: differing bytes against each neighbour, and the verdict.
    pub shifted: Option<(usize, usize, Shift)>,
}

fn differing_bytes(left: &[u8], right: &[u8]) -> usize {
    left.iter().zip(right).filter(|(a, b)| a != b).count() + left.len().abs_diff(right.len())
}

fn cut_text(cut: &Cut) -> String {
    match cut {
        Cut::NmiReturn(nmi) => format!(
            "rti({:?} {:06x}{})",
            nmi.kind,
            nmi.interrupted_pc,
            if nmi.polled { " polled" } else { "" }
        ),
        Cut::VblankWithoutNmi => "no-nmi".to_owned(),
        Cut::NmiStillRunning { interrupted_pc } => format!("nmi-running({interrupted_pc:06x})"),
    }
}

/// The CPU fields that differ, as `name=left/right`.
fn cpu_fields(left: &CpuRegisters, right: &CpuRegisters) -> Vec<String> {
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
        .map(|(name, a, b)| format!("{name}={a:x}/{b:x}"))
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
        let small = |part, count: usize, detail: String| Difference {
            part,
            count,
            detail,
            shifted: None,
        };
        if left.cut != right.cut {
            differences.push(small(
                Part::Cut,
                1,
                format!("{}/{}", cut_text(&left.cut), cut_text(&right.cut)),
            ));
        }
        let offset = i64::from(right.video_frame) - i64::from(left.video_frame);
        if *self.frame_offset.get_or_insert(offset) != offset {
            differences.push(small(
                Part::Frame,
                1,
                format!("{}/{}", left.video_frame, right.video_frame),
            ));
        }
        if left.pad != right.pad {
            differences.push(small(
                Part::Pad,
                1,
                format!("{:04x}/{:04x}", left.pad.0, right.pad.0),
            ));
        }
        if left.joypad_reads != right.joypad_reads {
            differences.push(small(
                Part::JoypadReads,
                1,
                format!("{}/{}", left.joypad_reads, right.joypad_reads),
            ));
        }
        let cpu = cpu_fields(&left.cpu, &right.cpu);
        if !cpu.is_empty() {
            differences.push(small(Part::Cpu, cpu.len(), cpu.join(" ")));
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
                detail: String::new(),
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
    if !difference.detail.is_empty() {
        text.push_str(&format!(" [{}]", difference.detail));
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
mod tests {
    use super::*;

    fn record(fill: u8) -> FrameRecord {
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
        assert_eq!(differences[0].detail, "pc=0/8344");
    }
}
